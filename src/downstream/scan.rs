// SPDX-License-Identifier: Apache-2.0

//! Search for file and extract information from any annotations

use crate::downstream::annotated;
use crate::downstream::annotated::{WatchLocation, Watched};
use crate::DatabaseArgs;
use anyhow::{bail, Context};
use clap::Args;
use jwalk::{DirEntry, WalkDir};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use streaming_iterator::StreamingIterator;
use syn::parse::Parse;
use syn::{LitBool, LitFloat, LitInt, LitStr};
use thiserror::__private::AsDisplay;
use tracing::{error, info, trace, warn};
use tree_sitter::{Language, Parser, Query, QueryCapture, QueryCursor};

/// Tree-Sitter query for rawr attributes. Only the outermost structure is matched,
/// while the internal arguments are matched by `RAWR_ATTRIBUTE_ARGS_QUERY` in
/// a second processing step..
pub const RAWR_ATTRIBUTE_QUERY: &str = r#"(attribute
  (identifier) @name (#eq? @name "rawr")
  arguments: (token_tree) @args)"#;

/// Tree-Sitter query for `identifier = literal` pairs nested inside
/// the arguments token tree. Only String, Boolean, Integer, and Float
/// literals are supported.
pub const RAWR_ATTRIBUTE_ARGS_QUERY: &str = r#"
((identifier) @ident
. "=" .
[
  (string_literal)
  (boolean_literal)
  (integer_literal)
  (float_literal)
] @literal)"#;

#[derive(Args, Debug, Clone)]
pub struct ScanArgs {
    #[command(flatten)]
    pub database: DatabaseArgs,

    /// Path to code root
    #[arg(default_value = "./")]
    pub project_root: PathBuf,
}

/// Find Rust files and parse them to identify annotations and their watched items.
pub async fn scan(args: ScanArgs) -> anyhow::Result<Vec<Watched>> {
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

    let mut watches = Vec::new();
    for path in paths {
        let mut file_watches = extract_annotations(path).await?;
        watches.append(&mut file_watches);
    }

    Ok(watches)
}

/// Find all rust files in the provided path.
// DESIGN This uses a Rayon threadpool. Should this be async?
fn enumerate_rust_files(root: PathBuf) -> anyhow::Result<Vec<DirEntry<((), ())>>> {
    // TODO Replace jwalk with walkdir.
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

#[derive(Debug)]
pub(crate) enum Literal {
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

/// Find and return annotations in file
async fn extract_annotations(path: PathBuf) -> anyhow::Result<Vec<Watched>> {
    let rust: Language = tree_sitter_rust::LANGUAGE.into();
    let attribute_query =
        Query::new(&rust, RAWR_ATTRIBUTE_QUERY).context("Compile annotation query")?;
    let args_query =
        Query::new(&rust, RAWR_ATTRIBUTE_ARGS_QUERY).context("Compile arguments query")?;

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
        .context("Parse file as Rust source")?;

    // Search for annotations
    let mut query_cursor = QueryCursor::new();
    let mut matched_attributes =
        query_cursor.matches(&attribute_query, tree.root_node(), source_bytes.as_slice());

    // Process each annotation's arguments.
    // TODO Extract attribute parser function

    let mut watches = Vec::new();
    while let Some(attribute_match) = matched_attributes.next() {
        let Some(args) = attribute_match.captures.get(1) else {
            trace!("Empty annotation. Skipping.");
            continue;
        };

        let mut args_cursor = QueryCursor::new();
        let mut arg_matches = args_cursor.matches(&args_query, args.node, source_bytes.as_slice());

        let mut args: HashMap<String, Literal> = HashMap::new();
        while let Some(pair_match) = arg_matches.next() {
            // Extract identifier name, if present
            let Some(identifier) = pair_match.captures.get(0) else {
                continue;
            };
            if identifier.node.kind() != "identifier" {
                trace!("Expected an identifier node. Skipping pair.");
                continue;
            }
            let start_byte = identifier.node.start_byte();
            let end_byte = identifier.node.end_byte();
            let identifier = source_bytes
                .get(start_byte..end_byte)
                .context("Get slice from downstream source file")?;
            let identifier = String::from_utf8(identifier.into())
                .context("Rust attribute variable's identifier must be valid UTF-8")?;

            // Parse literal value
            let Some(literal) = pair_match.captures.get(1) else {
                continue;
            };
            let literal_kind = literal.node.kind();
            let start_byte = literal.node.start_byte();
            let end_byte = literal.node.end_byte();
            let literal = source_bytes
                .get(start_byte..end_byte)
                .context("Get slice from downstream source file")?;
            let literal_string = String::from_utf8(literal.into())
                .context("Rust attribute's literal must be valid UTF-8")?;

            // Tree-Sitter types are listed after `"type": "_literal"` in
            // https://github.com/tree-sitter/tree-sitter-rust/blob/master/src/node-types.json#L259
            let literal: Literal = match literal_kind {
                "string_literal" => {
                    let s = syn::parse_str::<LitStr>(&literal_string)?.value();
                    Literal::String(s)
                }
                "boolean_literal" => {
                    let b = syn::parse_str::<LitBool>(&literal_string)?.value;
                    Literal::Boolean(b)
                }
                "integer_literal" => {
                    let i = syn::parse_str::<LitInt>(&literal_string)?.base10_parse::<i64>()?;
                    Literal::Integer(i)
                }
                "float_literal" => {
                    let f = syn::parse_str::<LitFloat>(&literal_string)?.base10_parse::<f64>()?;
                    Literal::Float(f)
                }
                kind => {
                    warn!(identifier, kind, "Skipping identifier unknown literal type");
                    continue;
                }
            };

            args.insert(identifier, literal);
        }

        // TODO Capture file and position in errors.
        let watched = Watched::try_from(args)
            .map_err(|errs| {
                errs.iter()
                    .map(|err| err.as_display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .map_err(anyhow::Error::msg)?;

        watches.push(watched);
    }

    Ok(watches)
}
