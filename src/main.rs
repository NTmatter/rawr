// SPDX-License-Identifier: Apache-2.0

//! Placeholder main, look at bins and lib for now.

use anyhow::{Context, bail};
use clap::Parser;
use gix_glob::wildmatch::Mode;
use rawr::downstream;
use rawr::downstream::scan::DownstreamScanArgs;
use rawr::lang::LanguageDefinition;
use rawr::lang::java::Java;
use rawr::upstream::{SourceRoot, Upstream, UpstreamScanArgs};

#[derive(Parser, Debug)]
enum Cmd {
    /// Enumerate items in the upstream codebase(s) as per their language configurations.
    UpstreamScan(UpstreamScanArgs),

    /// Enumerate watched items in the downstream codebase
    DownstreamWatches(DownstreamScanArgs),

    /// Compare the watched items to those in the upstream
    DownstreamCompare,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("RAWR - Reimplement and Watch Revisions!");

    // Read .env file if it exists
    match dotenv::dotenv() {
        Ok(_) => {}
        Err(dotenv::Error::Io(_)) => {}
        Err(err) => bail!(err),
    }

    tracing_subscriber::fmt::init();

    let command = Cmd::parse();
    match command {
        // XXX Use a mostly hard-coded Java scanner for early testing
        Cmd::UpstreamScan(args) => {
            let upstream = Upstream {
                id: "generic-java".into(),
                name: "Java Test".into(),
                path: args.repo_path,
                repo: None,
                roots: vec![SourceRoot {
                    id: "java".into(),
                    name: "Java".into(),
                    dialect: Java {}.configuration()?,
                    notes: None,
                    includes: vec![(
                        gix_glob::parse("src/**/*.java").context("Glob must be valid")?,
                        Mode::NO_MATCH_SLASH_LITERAL,
                    )],
                    excludes: vec![],
                }],
                notes: Some("This should come from a config file.".into()),
            };
            upstream.scan(&args.revision).await?;
        }
        Cmd::DownstreamWatches(args) => {
            downstream::scan::scan(args).await?;
        }
        Cmd::DownstreamCompare => {}
    }

    Ok(())
}
