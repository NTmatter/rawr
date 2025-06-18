// SPDX-License-Identifier: Apache-2.0

//! Placeholder main, look at bins and lib for now.

use clap::{Parser, Subcommand};
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

fn main() -> anyhow::Result<()> {
    println!("RAWR - Reimplement and Watch Revisions!");
    Ok(())
}
