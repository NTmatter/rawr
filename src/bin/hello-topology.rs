//! Look for 'changes since revision' for a symbol of interest.
//! Traverse the graph from the last accepted revision through to the
//! target head, looking for changes in the hash and stripped hash.

use anyhow::{bail, Context};
use clap::Parser;
use gix::bstr::BStr;
use gix::revision::walk::Info;
use rawr::{db_connection, Interesting};
use std::path::PathBuf;
use tracing::{trace, warn};

#[derive(Debug, Default, Clone, Parser)]
struct Args {
    /// Path to database
    #[arg(long, default_value = "rawr-scrape.sqlite3")]
    db_path: PathBuf,

    #[arg(long, default_value = "(self)")]
    codebase: String,

    /// Path to repository
    ///
    /// DESIGN: Can the repo be looked up from the target file?
    repo_path: PathBuf,

    /// Relative path to file in repository
    file: PathBuf,

    /// Kind of symbol to look for
    kind: String,

    /// Name of symbol to look up
    symbol: String,

    /// Revision in which implementation was last reviewed
    approved_rev: String,

    /// Revision to work towards
    to_rev: String,
}

fn main() -> anyhow::Result<()> {
    // Item of interest
    let Args {
        db_path,
        codebase,
        repo_path,
        file,
        kind,
        symbol,
        approved_rev,
        to_rev,
    } = Args::parse();

    tracing_subscriber::fmt::init();

    // Fetch initial (mut hash, mut hash_stripped) from database.
    let db = db_connection(db_path)?;
    let items = Interesting::get_watched_item_at_revision(
        &db,
        &codebase,
        &approved_rev,
        &file,
        &kind,
        &symbol,
    )?;

    if items.len() > 1 {
        warn!("Got multiple results for {kind} {symbol} in {file:?}@{approved_rev}");
    }
    let Some(interesting) = items.first() else {
        bail!("Could not find initial entry for {kind} {symbol} in {file:?}@{approved_rev}");
    };

    // Build list of commits between approved and target revision
    let repo = gix::discover(repo_path).context("Repository must exist at provided path")?;

    let from_rev = repo
        .rev_parse_single(BStr::new(approved_rev.as_str()))
        .with_context(|| format!("Revision {approved_rev} must exist"))?
        .object()
        .with_context(|| format!("Revision {approved_rev} must be an object"))?
        .id;
    let to_rev = repo
        .rev_parse_single(BStr::new(to_rev.as_str()))
        .with_context(|| format!("Revision {to_rev} must exist"))?
        .object()
        .with_context(|| format!("Revision {to_rev} must be an object"))?
        .id;

    let mut revs = Vec::new();
    repo.rev_walk(vec![to_rev])
        .with_pruned(vec![from_rev])
        .all()
        .context("Build list of new revisions")?
        .try_for_each(|revision| {
            revs.push(revision?);
            Ok::<(), anyhow::Error>(())
        })?;
    revs.reverse();

    let mut changes: Vec<Info> = Vec::new();

    let mut hash = interesting.hash.clone();
    let mut hash_stripped = interesting.hash_stripped.clone();

    // Walk revision history to find changed hashes
    for rev in revs {
        println!("{}: {:?}", rev.id, rev.commit_time);

        let interesting = Interesting::get_watched_item_at_revision(
            &db,
            &codebase,
            &rev.id.to_string(),
            &file,
            &kind,
            &symbol,
        )?;
        let Some(Interesting {
            hash: new_hash,
            hash_stripped: new_hash_stripped,
            ..
        }) = interesting.first()
        else {
            // Revision not found, or item deleted.
            trace!(
                "No database result for {} {} in {} @ {}",
                kind,
                symbol,
                file.display(),
                rev.id.to_string()
            );
            continue;
        };

        if hash.ne(new_hash) {
            if hash_stripped.ne(new_hash_stripped) {
                // Content updated
                trace!(
                    "Updated content for {} {} in {} @ {}",
                    kind,
                    symbol,
                    file.display(),
                    rev.id.to_string()
                );
            } else {
                // Whitespace change only
                trace!(
                    "Whitespace change for {} {} in {} @ {}",
                    kind,
                    symbol,
                    file.display(),
                    rev.id.to_string()
                );
            }
            hash = new_hash.clone();
            hash_stripped = new_hash_stripped.clone();
            changes.push(rev);
        } else {
            // No change
        }
    }

    // Traverse from approved_in_rev to head
    // foreach rev {
    //   fetch new hashes from database
    //   compare and notify if changed.
    //     hash change vs whitespace-only change.
    //   set hashes and look for more changes
    //  }

    Ok(())
}
