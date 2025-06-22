// SPDX-License-Identifier: Apache-2.0

//! Represent matchers for upstream items, find matches in a file, and extract
//! item names, content, and context.

use std::path::PathBuf;
use tree_sitter::{Node, Query};

fn main() -> anyhow::Result<()> {
    Ok(())
}

struct MatchedUpstreamItem {
    upstream: String,
    file: PathBuf,
    kind: String,
    ident: String,
    hash: String,
}

/// Describe how to match an upstream item and extract relevant data.
struct UpstreamItemMatcher {
    kind: String,
    /// Tree-Sitter query
    query: Query,
    ident: Option<ExtractWith>,
    body: Option<ExtractWith>,
}

enum ExtractWith {
    /// Convert the entire match to a string
    WholeMatch,

    /// Supply a constant, filtered through a templating replacement.
    Constant(String),

    /// Use the contents of a named child
    NamedChild(String, Box<ExtractWith>),

    /// Use the Nth child of the given type
    NthChild(usize, String, Box<ExtractWith>),

    /// Execute an additional query to identify the content, and extract it with
    /// the given extractor.
    Subquery(Query, Box<ExtractWith>),
}

fn extract(
    upstream: &str,
    kind: &str,
    with: &ExtractWith,
    root: &Node,
    data: &[u8],
) -> anyhow::Result<Option<MatchedUpstreamItem>> {
    match with {
        ExtractWith::WholeMatch => {}
        ExtractWith::Constant(value) => {}
        ExtractWith::NamedChild(name, extractor) => {}
        ExtractWith::NthChild(index, kind, extractor) => {}
        ExtractWith::Subquery(query, extractor) => {
            // Recursion required
        }
    }

    todo!()
}
