// SPDX-License-Identifier: Apache-2.0

//! Search for file and extract information from any annotations
// DESIGN find and match with language machinery, parse matches with syn.

use crate::DatabaseArgs;
use crate::downstream::annotated::{WatchLocation, Watched};
use crate::downstream::{Literal, annotated};
use anyhow::{Context, bail};
use clap::Args;
use gix::bstr::BStr;
use gix_glob::Pattern;
use gix_glob::wildmatch::Mode;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use streaming_iterator::StreamingIterator;
use syn::parse::Parse;
use syn::{LitBool, LitFloat, LitInt, LitStr};
use thiserror::__private::AsDisplay;
use tokio::task::JoinSet;
use tracing::{debug, error, info, trace, warn};
use tree_sitter::{Language, Parser, Query, QueryCapture, QueryCursor};
use walkdir::{DirEntry, WalkDir};

/// Tree-Sitter query for rawr attributes. Only the outermost structure is matched,
/// while the internal arguments are matched by `RAWR_ATTRIBUTE_ARGS_QUERY` in
/// a second processing step..
pub const RAWR_ATTRIBUTE_QUERY: &str = r#"(attribute
  (identifier) @name (#eq? @name "rawr")
  arguments: (token_tree) @args)
(macro_invocation
  macro: (identifier) @name (#eq? @name "rawr_fn" )
  (token_tree) @args)
"#;

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
pub struct DownstreamScanArgs {
    #[command(flatten)]
    pub database: DatabaseArgs,

    /// Path to code root
    #[arg(default_value = "./")]
    pub downstream_root: PathBuf,
}

pub struct Downstream {
    pub name: String,
    pub roots: Vec<SourceRoot>,
}

impl Downstream {
    pub async fn scan(&self) -> anyhow::Result<Vec<Watched>> {
        debug!(name = self.name, "Scanning downstream");
        let mut results = Vec::new();
        for root in &self.roots {
            let mut root_results = root.scan().await?;
            results.append(&mut root_results);
        }
        info!("Found {} downstream watches", results.len());
        Ok(results)
    }
}

pub struct SourceRoot {
    pub id: String,
    pub path: PathBuf,
    pub includes: Vec<(Pattern, Mode)>,
    pub excludes: Vec<(Pattern, Mode)>,
}

impl SourceRoot {
    pub async fn scan(&self) -> anyhow::Result<Vec<Watched>> {
        debug!(path = %self.path.display(), "Scanning downstream root");
        // Pre-check roots
        if !self.path.exists() {
            bail!("Scan root does not exist: {}", self.path.display())
        }

        if !(self.path.is_file() || self.path.is_dir()) {
            bail!(
                "Scan root is not a file or directory: {}",
                self.path.display()
            )
        }

        // Enumerate and filter files
        let all_rust_files = enumerate_rust_files(&self.path).await?;
        let unfiltered_file_count = all_rust_files.len();

        let files: Vec<PathBuf> = all_rust_files
            .into_iter()
            .filter(|path| {
                let path = BStr::new(path.as_os_str().as_encoded_bytes());
                if !self
                    .includes
                    .iter()
                    .any(|(pattern, mode)| pattern.matches(path, *mode))
                {
                    return false;
                }

                if self
                    .excludes
                    .iter()
                    .any(|(pattern, mode)| pattern.matches(path, *mode))
                {
                    return false;
                }

                true
            })
            .collect();
        debug!(
            "Processing {}/{} rust files",
            files.len(),
            unfiltered_file_count
        );

        let mut join_set = JoinSet::new();
        for path in files {
            join_set.spawn(async move { extract_annotations(&path).await });
        }

        let watches = join_set
            .join_all()
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<Vec<Watched>>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<Watched>>();

        Ok(watches)
    }
}

/// Find all rust files in the provided path.
async fn enumerate_rust_files(root: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let rust_files = WalkDir::new(root)
        .into_iter()
        .collect::<Result<Vec<DirEntry>, walkdir::Error>>()?
        .iter()
        .filter_map(|entry| {
            // TODO Use a Downstream configuration to handle include/exclude globbing.
            if entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "rs")
            {
                Some(entry.path().to_path_buf())
            } else {
                None
            }
        })
        .collect::<Vec<PathBuf>>();

    Ok(rust_files)
}

/// Find and return annotations in file
async fn extract_annotations(path: &PathBuf) -> anyhow::Result<Vec<Watched>> {
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
    // TODO Refactor - Extract attribute parser function

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
            let Some(identifier) = pair_match.captures.first() else {
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
