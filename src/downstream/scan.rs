// SPDX-License-Identifier: Apache-2.0

//! Search for file and extract information from any annotations

use crate::downstream::annotated::{WatchLocation, Watched};
use crate::DatabaseArgs;
use clap::Args;
use jwalk::WalkDir;
use std::path::PathBuf;

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

    let walk_dir = WalkDir::new(project_root).sort(true).process_read_dir(
        |depth, path, read_dir_state, children| {
            children.retain(|dir_entry_result| {
                dir_entry_result
                    .as_ref()
                    .map(|dir_entry| {
                        dir_entry
                            .file_name
                            .to_str()
                            .map(|s| s.ends_with(".rs"))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            });
        },
    );

    todo!()
}

#[test]
fn example_jwalk() {
    use jwalk::WalkDirGeneric;
    use std::cmp::Ordering;

    let walk_dir = WalkDirGeneric::<((usize), (bool))>::new("foo").process_read_dir(
        |depth, path, read_dir_state, children| {
            // 1. Custom sort
            children.sort_by(|a, b| match (a, b) {
                (Ok(a), Ok(b)) => a.file_name.cmp(&b.file_name),
                (Ok(_), Err(_)) => Ordering::Less,
                (Err(_), Ok(_)) => Ordering::Greater,
                (Err(_), Err(_)) => Ordering::Equal,
            });
            // 2. Custom filter
            children.retain(|dir_entry_result| {
                dir_entry_result
                    .as_ref()
                    .map(|dir_entry| {
                        dir_entry
                            .file_name
                            .to_str()
                            .map(|s| s.starts_with('.'))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            });
            // 3. Custom skip
            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    if dir_entry.depth == 2 {
                        dir_entry.read_children_path = None;
                    }
                }
            });
            // 4. Custom state
            // read_dir_state is the usize from the first item in the tuple.
            *read_dir_state += 1;
            children.first_mut().map(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    // Unpacking the dir_entry for the first child,
                    dir_entry.client_state = true;
                }
            });
        },
    );
}
