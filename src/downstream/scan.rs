// SPDX-License-Identifier: Apache-2.0

//! Search for file and extract information from any annotations

use crate::downstream::annotated::{WatchLocation, Watched};
use crate::DatabaseArgs;
use clap::Args;
use jwalk::{DirEntry, WalkDir};
use std::path::PathBuf;
use tracing::info;

#[derive(Args, Debug, Clone)]
pub struct ScanArgs {
    #[command(flatten)]
    pub database: DatabaseArgs,

    /// Path to code root
    #[arg(default_value = "./")]
    pub project_root: PathBuf,
}

/// Find Rust files and parse them to identify annotations and their watched items.
// DESIGN Is the bottleneck in the parse, or the scan? Should these eventually be praralellized?
pub async fn scan(args: ScanArgs) -> anyhow::Result<Vec<(Watched, WatchLocation)>> {
    let ScanArgs {
        database,
        project_root,
    } = args;

    let files = enumerate_rust_files(project_root).await?;
    info!("Found {} files to parse", files.len());

    Ok(Vec::new())
}

/// Find all rust files in the provided path.
async fn enumerate_rust_files(root: PathBuf) -> anyhow::Result<Vec<DirEntry<((), ())>>> {
    // Filter for directories and rust files
    let walk_dir =
        WalkDir::new(root)
            .sort(true)
            .process_read_dir(|depth, path, read_dir_state, children| {
                // Early filter for directory traversal and rust files.
                // This could be filtered post-enumeration, but leave it in for more flexibility
                // if additional criteria arise.
                children.retain(|dir_entry_result| {
                    dir_entry_result
                        .as_ref()
                        .map(|dir_entry| {
                            dir_entry.file_type().is_dir()
                                || dir_entry.file_type.is_file()
                                    && dir_entry.file_name.to_string_lossy().ends_with(".rs")
                        })
                        .unwrap_or(false)
                });
            });

    // Filter to rust files only
    let rust_files = walk_dir
        .into_iter()
        .flatten()
        .filter(|dir_entry| {
            dir_entry.file_type().is_file()
                && dir_entry.file_name.to_string_lossy().ends_with(".rs")
        })
        .collect();

    Ok(rust_files)
}
