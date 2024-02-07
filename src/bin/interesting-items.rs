// SPDX-License-Identifier: Apache-2.0

//! Use Tree-Sitter to find items of interest for a particular language. Rust
//! and C will be prototyped here.
//!
#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::bail;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use rawr::lang::{MatchType, Matcher, SupportedLanguage};
use rawr::Interesting;
use tree_sitter::{Language, Parser, Query, QueryCursor, QueryMatch};
use tree_sitter_bash;
use tree_sitter_c;
use tree_sitter_cpp;
use tree_sitter_rust;

fn main() -> anyhow::Result<()> {
    // Build matchers for supported languages
    let mut language_matchers = HashMap::<SupportedLanguage, Vec<Matcher>>::new();
    language_matchers.insert(SupportedLanguage::Rust, rawr::lang::matchers_rust());
    language_matchers.insert(SupportedLanguage::Bash, rawr::lang::matchers_bash());

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
        SupportedLanguage::Rust => (tree_sitter_rust::language(), rawr::lang::matchers_rust()),
        SupportedLanguage::Bash => (tree_sitter_bash::language(), rawr::lang::matchers_bash()),
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
    let mut interesting_matches = Vec::<Interesting>::new();
    for matcher in &matchers {
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
        let processed = matches.filter_map(|matched| {
            process_match(
                &"(self)".to_string(),
                &"(unversioned)".to_string(),
                &path,
                &language,
                &source_bytes,
                &matcher,
                &matched,
            )
        });
        interesting_matches.extend(processed);
    }

    // These should probably be concatenated for efficiency, but settle for repeated searches. O(matches * files)
    // todo!("Open file, parse, and build list of all matches");
    Ok(interesting_matches)
}

fn process_match(
    codebase: &String,
    revision: &String,
    path: &Path,
    language: &Language,
    source_bytes: &[u8],
    matcher: &Matcher,
    matched: &QueryMatch,
) -> Option<Interesting> {
    let Some(root_match) = matched.captures.get(0) else {
        return None;
    };

    let file_path = path.to_string_lossy();

    // Identifier: Extract a string
    // FIXME Need to hand back a string, which could possibly be a constant value like the filename or empty string.
    let identifier_text = match &matcher.identifier {
        MatchType::Match => {
            let range = root_match.node.start_byte()..root_match.node.end_byte();
            let text = String::from_utf8_lossy(&source_bytes[range]);
            Some(text)
        }
        MatchType::Kind(_kind, _index) => {
            // Iterate over children to find one of the right kind.
            todo!("Build query for subtype")
        }
        MatchType::Named(child_name) => {
            let child = root_match.node.child_by_field_name(child_name);
            if let Some(node) = child {
                let range = node.start_byte()..node.end_byte();
                let text = String::from_utf8_lossy(&source_bytes[range]);
                Some(text)
            } else {
                None
            }
        }
        MatchType::SubQuery(_match_id, query_string) => {
            let _query = Query::new(*language, query_string).expect("Parse identifier query");
            let mut _cursor = QueryCursor::new();
            todo!("Return results of sub-query")
        }
        MatchType::String(text) => {
            Some(Cow::from(text.replace("${file_name}", file_path.as_ref())))
        }
    };

    let Some(identifier) = identifier_text else {
        println!("Failed to match identifier");
        return None;
    };

    // TODO Get matched bytes, then convert to string for identifiers?
    // TODO Try to capture start and length
    // DESIGN Rewrite all arms to fill a buf.
    // Contents
    let mut buf = Vec::<u8>::new();
    let body_bytes = match &matcher.contents {
        MatchType::Match => {
            let range = root_match.node.start_byte()..root_match.node.end_byte();
            let bytes = &source_bytes[range];
            Some(bytes)
        }
        MatchType::Kind(_index, _kind) => {
            // Iterate over all children for anything matching type, and pick index.
            todo!("Build query for subtype")
        }
        MatchType::Named(child_name) => {
            let child_node = root_match.node.child_by_field_name(child_name);
            if let Some(node) = child_node {
                let range = node.start_byte()..node.end_byte();
                let bytes = &source_bytes[range];
                Some(bytes)
            } else {
                None
            }
        }
        MatchType::SubQuery(_match_id, query_string) => {
            let _query = Query::new(*language, query_string.as_str()).expect("Parse matcher query");
            let mut _cursor = QueryCursor::new();
            todo!("Return results of sub-query")
        }
        MatchType::String(text) => {
            let replaced = text.replace("${file_name}", file_path.as_ref());
            let bytes = replaced.as_bytes();
            buf.copy_from_slice(bytes);
            Some(buf.as_slice())
        }
    };

    let Some(contents) = body_bytes else {
        println!("Failed to match contents");
        return None;
    };

    // Salted hash of contents, in case of sensitive data.
    let hash_algorithm = "sha256".to_string();
    let mut hasher = Sha256::new();

    let salt: u64 = rand::random();
    hasher.update(salt.to_be_bytes());
    hasher.update(contents);

    let hash = format!("{:02x}", Sha256::digest(contents));

    Some(Interesting {
        codebase: codebase.to_string(),
        revision: revision.to_string(),
        path: file_path.to_string(),
        start_byte: None,
        length: None,
        kind: matcher.kind.to_string(),
        identifier: identifier.to_string(),
        hash_algorithm,
        salt,
        hash,
        notes: None,
    })
}
