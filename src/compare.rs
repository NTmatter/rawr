// SPDX-License-Identifier: Apache-2.0

#![allow(unused, reason = "Early development")]

use crate::db::DatabaseArgs;
use crate::downstream::annotated::Watched;
use crate::upstream::Upstream;
use crate::upstream::matched::UpstreamMatch;
use clap::Args;
use std::path::PathBuf;
use tree_sitter::Range;

#[derive(Args, Clone, Debug)]
pub struct CompareArgs {
    #[command(flatten)]
    pub database: DatabaseArgs,

    /// Path to upstream Git Repository
    pub upstream_repo: PathBuf,

    /// Git branch or hash to scan
    pub upstream_revision: String,
}

pub struct PrimaryKey {
    upstream_id: String,
    revision: String,
    file: PathBuf,
    kind: String,
    identifier: String,
    range: Option<Range>,
}

impl PrimaryKey {
    pub fn for_watched(watched: &Watched) -> PrimaryKey {
        todo!()
    }

    pub fn for_upstream(watched: &Upstream) -> PrimaryKey {
        todo!()
    }
}

pub struct UpstreamMatchRow {
    upstream_id: String,
    revision: String,
    file: PathBuf,
    kind: String,
    identifier: String,
    range: Option<Range>,
    checksum: String,
}

pub async fn compare(downstream: Vec<Watched>, upstream: Vec<UpstreamMatch>) -> anyhow::Result<()> {
    todo!()
}
