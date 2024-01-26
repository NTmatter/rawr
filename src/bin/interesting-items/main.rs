// SPDX-License-Identifier: Apache-2.0

//! Use Tree-Sitter to find items of interest for a particular language. Rust
//! and C will be prototyped here.
//!
#![allow(dead_code)]
#![allow(unused_imports)]

mod rawr_lib;

use anyhow::bail;
use sha2::{Digest, Sha256};
use std::any::Any;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use tree_sitter::{Language, Parser, Query, QueryCursor, QueryMatch};
use tree_sitter_bash;
use tree_sitter_c;
use tree_sitter_cpp;
use tree_sitter_rust;
use rawr_lib::{Interesting, Matcher, MatchType, SupportedLanguage};

fn main() -> anyhow::Result<()> {
    // Build matchers for supported languages
    let mut language_matchers = HashMap::<SupportedLanguage, Vec<Matcher>>::new();
    language_matchers.insert(SupportedLanguage::Rust, rawr_lib::matchers_rust());
    language_matchers.insert(SupportedLanguage::Bash, rawr_lib::matchers_bash());

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
        SupportedLanguage::Rust => (tree_sitter_rust::language(), rawr_lib::matchers_rust()),
        SupportedLanguage::Bash => (tree_sitter_bash::language(), rawr_lib::matchers_bash()),
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
    for matcher in &matchers {
        println!("Matching {}", matcher.kind);
        // Find matches and extract information
        let query = match Query::new(language, matcher.query.as_str()) {
            Ok(query) => query,
            Err(e) => {
                eprintln!("Skipping unparseable query {}", matcher.query);
                eprintln!("{}", e);
                continue;
            }
        };

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_bytes.as_slice());
        matches.for_each(|matched| {
            println!("Got Match");
            process_match(&language, &source_bytes, &matcher, &matched);
        });
    }

    // These should probably be concatenated for efficiency, but settle for repeated searches. O(matches * files)
    // todo!("Open file, parse, and build list of all matches");
    Ok(interesting_matches)
}

fn process_match(
    language: &Language,
    sources: &[u8],
    matcher: &Matcher,
    matched: &QueryMatch,
) -> Option<Interesting> {
    let Some(root_match) = matched.captures.get(0) else {
        return None;
    };

    // Identifier
    // FIXME Need to hand back a string, which could possibly be a constant value like the filename or empty string.
    let Some(identifier_match) = (match &matcher.identifier {
        MatchType::Match => Some(root_match.node),
        MatchType::Kind(_) => {
            todo!("Build query for subtype")
        }
        MatchType::Named(child_name) => root_match.node.child_by_field_name(child_name),
        MatchType::Query(query_string, _match_id) => {
            let _query =
                Query::new(*language, query_string.as_str()).expect("Parse identifier query");
            let mut _cursor = QueryCursor::new();
            todo!("Return results of sub-query")
        }
    }) else {
        println!("Failed to match identifier");
        return None;
    };

    let identifier = &sources[identifier_match.start_byte()..identifier_match.end_byte()];
    let identifier = String::from_utf8_lossy(identifier);
    println!("Found identifier named {}", identifier);

    // Contents
    let Some(contents_match) = (match &matcher.contents {
        MatchType::Match => Some(root_match.node),
        MatchType::Kind(kind) => {
            let query_string = format!("(({}) @kind)", kind);
            let _query = Query::new(*language, query_string.as_str()).expect("Query for kind");
            todo!("Build query for subtype")
        }
        MatchType::Named(child_name) => root_match.node.child_by_field_name(child_name),
        MatchType::Query(query_string, _match_id) => {
            let _query = Query::new(*language, query_string.as_str()).expect("Parse matcher query");
            let mut _cursor = QueryCursor::new();
            todo!("Return results of sub-query")
        }
    }) else {
        println!("Failed to match contents");
        return None;
    };

    let contents = &sources[contents_match.start_byte()..contents_match.end_byte()];

    // Salted hash of contents, in case of sensitive data.
    let mut hasher = Sha256::new();

    let salt: u64 = rand::random();
    hasher.update(salt.to_be_bytes());
    hasher.update(contents);
    let hash = format!("sha256:{:x}:{:02x}", salt, Sha256::digest(contents));
    dbg!(hash);

    // TODO Construct result

    None
}
