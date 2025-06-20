// SPDX-License-Identifier: Apache-2.0

//! Placeholder main, look at bins and lib for now.

use anyhow::bail;
use clap::Parser;
use rawr::downstream;
use rawr::downstream::scan::ScanArgs;

#[derive(Parser, Debug)]
enum Cmd {
    /// Enumerate items in the upstream codebase(s) as per their language configurations.
    UpstreamScan,

    /// Enumerate watched items in the downstream codebase
    DownstreamWatches(ScanArgs),

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
        Cmd::UpstreamScan => {}
        Cmd::DownstreamWatches(args) => {
            downstream::scan::scan(args).await?;
        }
        Cmd::DownstreamCompare => {}
    }

    Ok(())
}
