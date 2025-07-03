// SPDX-License-Identifier: Apache-2.0

//! Placeholder main, look at bins and lib for now.

use anyhow::{Context, bail};
use clap::Parser;
use gix_glob::wildmatch::Mode;
use rawr::compare::CompareArgs;
use rawr::downstream::scan;
use rawr::downstream::scan::Downstream;
use rawr::downstream::scan::DownstreamScanArgs;
use rawr::lang::LanguageDefinition;
use rawr::lang::java::Java;
use rawr::upstream::matched::UpstreamMatch;
use rawr::upstream::{SourceRoot, Upstream, UpstreamScanArgs};
use rawr::{compare, db};
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Parser, Debug)]
enum Cmd {
    /// Enumerate items in the upstream codebase(s) as per their language configurations.
    UpstreamScan(UpstreamScanArgs),

    /// Enumerate watched items in the downstream codebase
    DownstreamWatches(DownstreamScanArgs),

    /// Compare the watched items to those in the upstream
    DownstreamCompare(CompareArgs),
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
        Cmd::DownstreamWatches(_args) => {
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
            let matches = downstream.scan().await?;
            info!("Found {} downstream watches", matches.len());
        }
        Cmd::DownstreamCompare(args) => {
            let conn = db::connect_rw(args.database)?;

            let upstream = Upstream {
                id: "generic-java".into(),
                name: "Java Test".into(),
                path: args.upstream_repo,
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
            let upstream_matches = upstream.scan(&args.upstream_revision).await?;
            info!("Found {} upstream matches", upstream_matches.len());
            let _affected = UpstreamMatch::insert_batch(&conn, &upstream_matches)?;
            // if let Err((_conn, err)) = conn.close() {
            //     bail!("Could not close initial database connection {err:?}");
            // }

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

            let downstream_watches = downstream.scan().await?;
            info!("Found {} downstream watches", downstream_watches.len());

            debug!("Compare against upstream");
            // compare::compare(downstream_watches, upstream_matches).await?;
        }
    }

    Ok(())
}
