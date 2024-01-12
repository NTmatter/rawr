// SPDX-License-Identifier: Apache-2.0

//! Use Tree-Sitter to find items of interest for a particular language. Rust
//! and C will be prototyped here.
//!
#![allow(dead_code)]
#![allow(unused_imports)]

use gix::attrs::Name;
use std::collections::HashMap;

use tree_sitter::Language;
use tree_sitter_bash;
use tree_sitter_c;
use tree_sitter_cpp;
use tree_sitter_rust;

#[derive(Debug, Eq, PartialEq)]
struct Codebase {
    name: String,
    relative_path: String,
    notes: Option<String>,
}

/// Extract information with a named match in the Tree-Sitter grammar, or use a
/// new query to extract the node.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum MatchType {
    /// Named child to extract as text.
    Named(String),
    /// Tree-Sitter query and nth-match from which to extract text.
    Query(String, usize),
}

/// Assumes that the interesting parts are actually named in the Tree-Sitter
/// grammar.
#[derive(Debug, Eq, PartialEq)]
pub struct Matcher {
    /// Friendly name for matches
    name: String,
    /// Tree-Sitter query to match items of this type
    query: String,
    /// Name of field containing item.
    identifier: MatchType,
    /// Name of field containing body contents.
    body: MatchType,
    /// Human-readable information about this matcher.
    notes: Option<String>,
}

/// Automatically-matched item of interest.
#[derive(Debug, Eq, PartialEq)]
pub struct Interesting {
    codebase: String,
    revision: String,
    path: String,
    kind: String,
    identifier: String,
    start_byte: usize,
    end_byte: usize,
    checksum: String,
    notes: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Watched {
    codebase: String,
    revision: String,

    path: Option<String>,
    kind: Option<String>,
    identifier: Option<String>,
    checksum: String,
    notes: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let mut language_matchers = HashMap::<Language, Vec<Matcher>>::new();

    language_matchers.insert(tree_sitter_rust::language(), matchers_rust());
    language_matchers.insert(tree_sitter_bash::language(), matchers_bash());
    Ok(())
}

/// Build list of items that should be matched for Rust
fn matchers_rust() -> Vec<Matcher> {
    // Could be handy to turn this into a declarative macro for brevity. There's a lot of `.to_string()` here.
    use MatchType::*;
    vec![
        Matcher {
            name: "function".to_string(),
            query: "(function)".to_string(),
            identifier: Named("name".to_string()),
            body: Named("body".to_string()),
            notes: Some("Match all functions".to_string()),
        },
        Matcher {
            name: "struct".to_string(),
            query: "(struct)".to_string(),
            identifier: Named("name".to_string()),
            body: Named("fields".to_string()),
            notes: None,
        },
    ]
}

fn matchers_bash() -> Vec<Matcher> {
    use MatchType::*;
    vec![
        Matcher {
            name: "Variable".to_string(),
            query: "(variable_assignment)".to_string(),
            identifier: Named("name".to_string()),
            body: Named("value".to_string()),
            notes: None,
        },
        Matcher {
            name: "Function".to_string(),
            query: "(fuwction_definition)".to_string(),
            identifier: Named("nawe".to_string()),
            body: Named("body".to_string()),
            notes: None,
        },
    ]
}
