// SPDX-License-Identifier: Apache-2.0

//! Prototype implementation of codebase scrape functionality.
//! - Enumerate all heads and accessible revisions in repository
//! - Parse items of interest from all revisions

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser as ClapParser;
use gix::hashtable::HashSet;
use gix::traverse::tree::recorder::Entry;
use gix::traverse::tree::Recorder;
use gix::{Id, ObjectId, Repository};
use tracing::{debug, info};

use rawr::Interesting;

#[derive(ClapParser, Debug)]
struct Args {
    /// Path to database file
    #[arg(long, default_value = "rawr-scrape.sqlite3")]
    db_path: PathBuf,

    #[arg(long, default_value = "main")]
    heads: Vec<String>,

    /// Path to Git Repository
    #[arg(required = true)]
    repo_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Args {
        db_path,
        heads,
        repo_path,
    } = Args::try_parse()?;

    tracing_subscriber::fmt::init();

    info!("Scraping repo {repo_path:?} into db {db_path:?}");

    let repo = gix::discover(repo_path).context("Repository exists at provided path")?;
    debug!("Repo uses hash type {}", repo.object_hash());

    // TODO Iterate over heads
    let head = heads
        .first()
        .context("At least one head must be specified")?
        .as_str();
    let rev = repo
        .rev_parse_single(head)
        .context("Repo must contain specified revision")?;
    let mut ancestors = rev
        .ancestors()
        .all()
        .context("Walk all ancestor revisions")?;
    ancestors.try_for_each(|info| {
        let info = info?;
        println!("Got Ancestor: {}", info.id());

        let Ok(commit) = info.object() else {
            println!("Not a commit. Skipping.");
            return Result::<(), anyhow::Error>::Ok(());
        };

        // println!("\tMessage: {}", commit.message_raw_sloppy());

        // Iterate over files in revision
        let mut recorder = Recorder::default();
        commit
            .tree()
            .context("Get tree from commit")?
            .traverse()
            .breadthfirst(&mut recorder)
            .context("Build breadth-first searcher")?;
        recorder
            .records
            .iter()
            .filter(|entry| entry.mode.is_blob())
            .try_for_each(|entry| {
                // Get basic information about entry and retrieve underlying blob.

                // Is OID a sha1? If so, this is useful for memoization on parsing files.
                // file path + oid seems sufficient. Might need a custom key that supports Hash
                let obj = repo.find_object(entry.oid).context("Find file blob")?;

                // TODO If the entry corresponds to a new (path, oid), parse the file based on its
                //   extension.

                // Temp: Prove that we can get access to the file data.
                let blob = obj.try_into_blob().context("Convert object to Blob")?;

                println!(
                    "\t\t{} {} {} bytes",
                    entry.filepath,
                    entry.oid,
                    blob.data.len(),
                );

                Result::<(), anyhow::Error>::Ok(())
            })?;

        Result::<(), anyhow::Error>::Ok(())
    })?;

    Ok(())
}
