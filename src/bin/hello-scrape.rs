// SPDX-License-Identifier: Apache-2.0

//! Prototype implementation of codebase scrape functionality.
//! - Enumerate all heads and accessible revisions in repository
//! - Parse items of interest from all revisions

use anyhow::Context;
use clap::Parser as ClapParser;
use gix::bstr::BString;
use gix::traverse::tree::Recorder;
use gix::{Blob, Id, ObjectId};
use rawr::lang::{MatchType, Matcher, SupportedLanguage};
use rawr::Interesting;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use tree_sitter::{Language, Parser, Query, QueryCursor, QueryMatch};

#[derive(ClapParser, Debug)]
struct Args {
    /// Path to database file
    #[arg(long, default_value = "rawr-scrape.sqlite3")]
    db_path: PathBuf,

    #[arg(long, default_value = "main")]
    heads: Vec<String>,

    /// Path to Git Repository
    #[arg(required = true)]
    repo_path: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
struct MemoKey {
    path: BString,
    object_id: ObjectId,
}

fn main() -> anyhow::Result<()> {
    let Args {
        db_path,
        heads,
        repo_path,
    } = Args::try_parse()?;

    tracing_subscriber::fmt::init();

    info!("Scraping repo {repo_path:?} into db {db_path:?}");

    let mut language_matchers = HashMap::<SupportedLanguage, Vec<Matcher>>::new();
    language_matchers.insert(SupportedLanguage::Rust, rawr::lang::matchers_rust());
    language_matchers.insert(SupportedLanguage::Bash, rawr::lang::matchers_bash());

    // TODO Use concurrent hashmap instead of RWLock.
    // let cache = RwLock::new(HashMap::<MemoKey, Vec<Interesting>>::new());

    let repo = gix::discover(repo_path).context("Repository exists at provided path")?;
    debug!("Repo uses hash type {}", repo.object_hash());

    // TODO Iterate over heads
    let head = heads
        .first()
        .context("At least one head must be specified")?
        .as_str();
    let rev = repo
        .rev_parse_single(head)
        .context("Repo must contain specified revision")?;
    let mut ancestors = rev
        .ancestors()
        .all()
        .context("Walk all ancestor revisions")?;
    ancestors.try_for_each(|info| {
        let info = info?;
        println!("Got Ancestor: {}", info.id());

        let Ok(commit) = info.object() else {
            println!("Not a commit. Skipping.");
            return Result::<(), anyhow::Error>::Ok(());
        };

        // Iterate over files in revision
        let mut recorder = Recorder::default();
        commit
            .tree()
            .context("Get tree from commit")?
            .traverse()
            .breadthfirst(&mut recorder)
            .context("Build breadth-first searcher")?;
        recorder
            .records
            .iter()
            .filter(|entry| entry.mode.is_blob())
            .try_for_each(|entry| {
                // Get basic information about entry and retrieve underlying blob.

                // Is OID a sha1? If so, this is useful for memoization on parsing files.
                // file path + oid seems sufficient. Might need a custom key that supports Hash
                let obj = repo.find_object(entry.oid).context("Find file blob")?;

                // TODO If the entry corresponds to a new (path, oid), parse the file based on its
                //   extension.

                // Temp: Prove that we can get access to the file data.
                let blob = obj.try_into_blob().context("Convert object to Blob")?;

                let results = find_matches_in_blob(&entry.filepath, &rev, &blob).unwrap_or(None);

                match results {
                    Some(ref results) => println!(
                        "\t\t{} {} {} bytes, {} results",
                        entry.filepath,
                        entry.oid,
                        blob.data.len(),
                        results.len(),
                    ),
                    None => println!(
                        "\t\t{} {} {} bytes",
                        entry.filepath,
                        entry.oid,
                        blob.data.len()
                    ),
                };

                Result::<(), anyhow::Error>::Ok(())
            })?;

        Result::<(), anyhow::Error>::Ok(())
    })?;

    Ok(())
}

/// Extract interesting features from file.
fn find_matches_in_blob(
    path: &BString,
    rev: &Id,
    blob: &Blob,
) -> anyhow::Result<Option<Vec<Interesting>>> {
    let path = path.to_string();
    let path = Path::new(&path);

    // Primitive language detection, should eventually be abstracted out and configured with the
    // project.
    let lang = path.extension().and_then(|ext| match ext.to_str() {
        Some("rs") => Some(SupportedLanguage::Rust),
        Some("sh") => Some(SupportedLanguage::Bash),
        _ => None,
    });

    // Only parse known languages for now.
    let Some(lang) = lang else {
        return Ok(None);
    };

    let (language, matchers) = match lang {
        SupportedLanguage::Rust => (tree_sitter_rust::language(), rawr::lang::matchers_rust()),
        SupportedLanguage::Bash => (tree_sitter_bash::language(), rawr::lang::matchers_bash()),
    };

    // Parse file
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .expect("Create language parser");

    let tree = parser
        .parse(blob.data.as_slice(), None)
        .expect("Parse file");

    // Find matches
    let mut interesting_matches = Vec::<Interesting>::new();
    for matcher in &matchers {
        // Find matches and extract information
        let query = match Query::new(&language, matcher.query.as_str()) {
            Ok(query) => query,
            Err(e) => {
                eprintln!("Skipping unparseable query {}", matcher.query);
                eprintln!("{}", e);
                continue;
            }
        };

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), blob.data.as_slice());
        let processed = matches.filter_map(|matched| {
            process_match(
                &"(self)".to_string(),
                &rev.to_string(),
                path,
                &language,
                blob.data.as_slice(),
                matcher,
                &matched,
            )
        });
        interesting_matches.extend(processed);
    }

    // These should probably be concatenated for efficiency, but settle for repeated searches. O(matches * files)
    // todo!("Open file, parse, and build list of all matches");
    Ok(Some(interesting_matches))
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
    let root_match = matched.captures.first()?;

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
            let _query = Query::new(language, query_string).expect("Parse identifier query");
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
            let _query = Query::new(language, query_string.as_str()).expect("Parse matcher query");
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

    // Consider salting the hash. This will prevent simple lookup.
    // let salt: Option<u64> = Some(rand::random());
    let salt: Option<u64> = None;
    if let Some(salt) = salt {
        hasher.update(salt.to_be_bytes());
    }

    hasher.update(contents);

    let hash = format!("{:02x}", Sha256::digest(contents));

    let start_byte = root_match.node.start_byte();
    let length = root_match.node.end_byte() - root_match.node.start_byte();

    Some(Interesting {
        codebase: codebase.to_string(),
        revision: revision.to_string(),
        path: file_path.to_string(),
        start_byte,
        length,
        kind: matcher.kind.to_string(),
        identifier: identifier.to_string(),
        hash_algorithm,
        salt,
        hash,
        notes: None,
    })
}
