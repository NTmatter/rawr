// SPDX-License-Identifier: Apache-2.0

//! Functionality for representing, scanning, and interrogating the downstream
//! (reimplementing) codebase.

#![allow(unused, reason = "Early development")]

use crate::DatabaseArgs;
use crate::downstream::annotated::Watched;
use crate::upstream::matched::UpstreamMatch;
use annotated::WatchLocation;
use clap::{Args, Subcommand};
use std::path::PathBuf;

pub mod annotated;
pub mod scan;

#[derive(Debug)]
pub enum Literal {
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

#[derive(Args, Debug, Clone)]
pub struct CompareArgs {
    #[command(flatten)]
    pub database: DatabaseArgs,

    #[arg(default_value = "./")]
    pub path: PathBuf,
}

pub struct CompareResult {
    /// Watched items that have not changed upstream.
    pub unchanged: Vec<(Watched, WatchLocation, UpstreamMatch)>,

    /// Watched items whose contents have changed.
    pub modified: Vec<(Watched, WatchLocation, UpstreamMatch)>,

    /// Items that have not yet been watched.
    pub new: Vec<UpstreamMatch>,

    /// Items that have been explicitly ignored.
    ignored: Vec<(Watched, WatchLocation, UpstreamMatch)>,

    /// Watched items that have no match. This may be due to deletion, ident
    /// change (eg moving or renaming), or
    pub unmatched: Vec<(Watched, WatchLocation)>,
}

pub async fn compare(args: CompareArgs) -> anyhow::Result<CompareResult> {
    let CompareArgs { database, path } = args;
    todo!()
}
