// SPDX-License-Identifier: Apache-2.0

//! Placeholder main, look at bins and lib for now.

use anyhow::{Context, bail};
use clap::Parser;
use gix_glob::wildmatch::Mode;
use rawr::downstream;
use rawr::downstream::scan;
use rawr::downstream::scan::Downstream;
use rawr::downstream::scan::DownstreamScanArgs;
use rawr::lang::LanguageDefinition;
use rawr::lang::java::Java;
use rawr::upstream::{SourceRoot, Upstream, UpstreamScanArgs};
use std::sync::Arc;

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
                    dialect: Arc::new(Java {}.configuration()?),
                    notes: None,
                    includes: vec![(
                        gix_glob::parse("**/*.java").context("Glob must be valid")?,
                        Mode::NO_MATCH_SLASH_LITERAL,
                    )],
                    excludes: vec![],
                }],
                notes: Some("This should come from a config file.".into()),
            };
            upstream.scan(&args.revision).await?;
        }
        Cmd::DownstreamWatches(args) => {
            // XXX Use a hard-coded downstream scan for source and tests
            let downstream = Downstream {
                name: "self".into(),
                roots: vec![
                    scan::SourceRoot {
                        id: "tests".to_string(),
                        path: "tests".into(),
                        includes: vec![(
                            gix_glob::parse("**/*.rs").context("Glob must be valid")?,
                            Mode::NO_MATCH_SLASH_LITERAL,
                        )],
                        excludes: vec![],
                    },
                    scan::SourceRoot {
                        id: "lib".to_string(),
                        path: "src".into(),
                        includes: vec![(
                            gix_glob::parse("**/*.rs").context("Glob must be valid")?,
                            Mode::NO_MATCH_SLASH_LITERAL,
                        )],
                        excludes: vec![],
                    },
                ],
            };
            downstream::scan::scan(args).await?;
        }
        Cmd::DownstreamCompare => {}
    }

    Ok(())
}
