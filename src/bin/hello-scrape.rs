// SPDX-License-Identifier: Apache-2.0

//! Prototype implementation of codebase scrape functionality.
//! - Enumerate all heads and accessible revisions in repository
//! - Parse items of interest from all revisions

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser as ClapParser;
use gix::{Id, ObjectId, Repository};
use gix::hashtable::HashSet;
use gix::traverse::tree::recorder::Entry;
use tracing::info;

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
    let Args { db_path, heads, repo_path } = Args::try_parse()?;

    tracing_subscriber::fmt::init();

    info!("Scraping repo {repo_path:?} into db {db_path:?}");

    let repo = gix::discover(repo_path).context("Repository exists at provided path")?;
    // TODO Iterate over heads
    let head = heads.first().context("At least one head must be specified")?.as_str();
    let rev = repo.rev_parse_single(head).context("Repo must contain specified revision")?;
    let mut ancestors = rev.ancestors().all().context("Walk all ancestor revisions")?;
    ancestors.try_for_each(|info| {
        let info = info?;
        println!("Got Ancestor: {}", info.id());

        let Ok(commit) = info.object() else {
            println!("Not a commit. Skipping.");
            return Result::<(), anyhow::Error>::Ok(());
        };

        println!("\tMessage: {}", commit.message_raw_sloppy());

        Result::<(), anyhow::Error>::Ok(())
    })?;

    Ok(())
}
