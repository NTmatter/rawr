// SPDX-License-Identifier: Apache-2.0

//! Search for file and extract information from any annotations

use crate::downstream::annotated;
use crate::downstream::annotated::{WatchLocation, Watched};
use crate::DatabaseArgs;
use anyhow::{bail, Context};
use clap::Args;
use jwalk::{DirEntry, WalkDir};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use streaming_iterator::StreamingIterator;
use tracing::info;
use tree_sitter::{Language, Parser, Query, QueryCursor};

#[derive(Args, Debug, Clone)]
pub struct ScanArgs {
    #[command(flatten)]
    pub database: DatabaseArgs,

    /// Path to code root
    #[arg(default_value = "./")]
    pub project_root: PathBuf,
}

/// Find Rust files and parse them to identify annotations and their watched items.
pub async fn scan(args: ScanArgs) -> anyhow::Result<Vec<(Watched, WatchLocation)>> {
    let ScanArgs {
        database,
        project_root,
    } = args;

    let readable_root = project_root.display().to_string();
    if !project_root.exists() {
        bail!("Scan root does not exist: {readable_root}")
    }
    if !(project_root.is_file() || project_root.is_dir()) {
        bail!("Scan root is not a file or directory: {readable_root}")
    }

    let files = enumerate_rust_files(project_root)?;
    info!("Found {} files to parse in {readable_root}", files.len());
    let paths = files
        .into_iter()
        .map(|dir_entry| dir_entry.path())
        .collect::<Vec<_>>();

    for path in paths {
        extract_annotations(path).await?;
    }

    Ok(Vec::new())
}

/// Find all rust files in the provided path.
// DESIGN This uses a Rayon threadpool. Should this be async?
fn enumerate_rust_files(root: PathBuf) -> anyhow::Result<Vec<DirEntry<((), ())>>> {
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

/// Find and return annotations in file
async fn extract_annotations(path: PathBuf) -> anyhow::Result<Vec<Watched>> {
    let rust: Language = tree_sitter_rust::LANGUAGE.into();
    let attribute_query =
        Query::new(&rust, annotated::RAWR_ATTRIBUTE_QUERY).context("Compile annotation query")?;
    let args_query = Query::new(&rust, annotated::RAWR_ATTRIBUTE_ARGS_QUERY)
        .context("Compile arguments query")?;

    let mut parser = Parser::new();
    parser
        .set_language(&rust)
        .context("Use Tree-Sitter Rust parser")?;

    // Parse file contents.
    let readable_path = path.display().to_string();
    let source_bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("Read downstream source code file at {readable_path}"))?;
    let tree = parser
        .parse(source_bytes.as_slice(), None)
        .context("Parse file as Rust")?;

    // Search for annotations

    let mut query_cursor = QueryCursor::new();
    let mut matches =
        query_cursor.matches(&attribute_query, tree.root_node(), source_bytes.as_slice());

    // Process arguments in each annotation match.
    while let Some(matched) = matches.next() {
        let Some(args) = matched.captures.get(1) else {
            // Empty annotation, missing arguments
            continue;
        };

        let mut args_cursor = QueryCursor::new();
        let arg_matches = args_cursor.matches(&args_query, args.node, source_bytes.as_slice());

        dbg!(arg_matches.count());
    }

    Ok(Vec::new())
}
