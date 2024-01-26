// SPDX-License-Identifier: Apache-2.0

//! Use Tree-Sitter to find items of interest for a particular language. Rust
//! and C will be prototyped here.
//!
#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::{anyhow, bail};
use gix::attrs::Name;
use std::any::Any;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::Read;
use std::path;
use std::path::Path;

use tree_sitter::{Language, Parser, Query, QueryCursor, QueryMatch};
use tree_sitter_bash;
use tree_sitter_c;
use tree_sitter_cpp;
use tree_sitter_rust;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum SupportedLanguage {
    Rust,
    Bash,
    C,
    Cpp,
}

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
    // Build matchers for supported languages
    let mut language_matchers = HashMap::<SupportedLanguage, Vec<Matcher>>::new();
    language_matchers.insert(SupportedLanguage::Rust, matchers_rust());
    language_matchers.insert(SupportedLanguage::Bash, matchers_bash());

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        bail!("File names must be specified");
    }

    // Process known filetypes
    args.into_iter().skip(1).for_each(|arg| {
        let path = Path::new(&arg);

        let Some(file_extension) = path.extension() else {
            return;
        };

        let lang = match file_extension.to_str() {
            Some("rs") => SupportedLanguage::Rust,
            Some("sh") => SupportedLanguage::Bash,
            _ => return,
        };

        let Ok(matches) = find_matches_in_file(path, lang) else {
            return;
        };

        println!("Found {} matches in file.", matches.len());
    });

    Ok(())
}

fn find_matches_in_file(path: &Path, lang: SupportedLanguage) -> anyhow::Result<Vec<Interesting>> {
    println!("Searching for matches in {}", path.display());

    let (language, matchers) = match lang {
        SupportedLanguage::Rust => (tree_sitter_rust::language(), matchers_rust()),
        SupportedLanguage::Bash => (tree_sitter_bash::language(), matchers_bash()),
        SupportedLanguage::C => todo!(),
        SupportedLanguage::Cpp => todo!(),
    };

    // Open and read file
    let mut file = std::fs::File::open(path)?;
    let mut source_bytes = Vec::new();
    file.read_to_end(&mut source_bytes)?;

    // Parse file
    let mut parser = Parser::new();
    parser
        .set_language(language)
        .expect("Create language parser");

    let tree = parser
        .parse(&source_bytes.as_slice(), None)
        .expect("Parse file");

    // Find matches
    let interesting_matches = Vec::<Interesting>::new();
    for matcher in matchers {
        // Find matches and extract information
        let Ok(query) = Query::new(language, matcher.query.as_str()) else {
            println!("Skipping unparseable query {}", matcher.query);
            continue;
        };
        let mut cursor = QueryCursor::new();

        let matches = cursor.matches(&query, tree.root_node(), source_bytes.as_slice());
        matches.for_each(|matched| {
            process_match(&language, &matcher, &matched);
        });
    }

    // These should probably be concatenated for efficiency, but settle for repeated searches. O(matches * files)
    // todo!("Open file, parse, and build list of all matches");
    Ok(interesting_matches)
}

fn process_match(
    language: &Language,
    matcher: &Matcher,
    matched: &QueryMatch,
) -> Option<Interesting> {
    let Some(root_match) = matched.captures.get(0) else {
        return None;
    };

    let cursor = QueryCursor::new();

    // Identifier
    match &matcher.identifier {
        MatchType::Named(child_name) => {
            root_match.node.child_by_field_name(child_name);
        }
        MatchType::Query(query_string, match_id) => {
            let query =
                Query::new(*language, query_string.as_str()).expect("Parse identifier query");
        }
    }

    // Body

    todo!()
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

/// Build list of items that should be matched for Bash
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
