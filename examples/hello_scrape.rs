// SPDX-License-Identifier: Apache-2.0

//! Prototype implementation of codebase scrape functionality.
//! - Enumerate all heads and accessible revisions in repository
//! - Parse items of interest from all revisions

use anyhow::Context;
use clap::Parser as ClapParser;
use gix::bstr::BString;
use gix::traverse::tree::Recorder;
use gix::{Blob, Id, ObjectId, Repository};
use rawr::db_connection;
#[cfg(feature = "lang-bash")]
use rawr::lang::bash::Bash;
use rawr::lang::rust::Rust;
use rawr::lang::{LanguageConfig, MatchType, Matcher, SupportedLanguage};
use rawr::upstream::matched::UpstreamMatch;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use streaming_iterator::StreamingIterator;
use tracing::{debug, error, info, trace, warn};
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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
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
    language_matchers.insert(SupportedLanguage::Rust, Rust::matchers());
    #[cfg(feature = "lang-bash")]
    language_matchers.insert(SupportedLanguage::Bash, Bash::matchers());

    let repo = gix::discover(repo_path).context("Repository exists at provided path")?;
    debug!("Repo uses hash type {}", repo.object_hash());

    let db = db_connection(db_path)?;

    // TODO Consider a concurrent hashmap
    let mut cache: HashMap<MemoKey, Vec<UpstreamMatch>> = HashMap::new();

    // TODO Iterate over all heads
    let head = heads
        .first()
        .context("At least one head must be specified")?
        .as_str();
    process_head(repo, &db, &mut cache, head)?;

    let _ = db.close();

    Ok(())
}

fn process_head(
    repo: Repository,
    db: &Connection,
    cache: &mut HashMap<MemoKey, Vec<UpstreamMatch>>,
    head: &str,
) -> anyhow::Result<()> {
    let rev = repo
        .rev_parse_single(head)
        .context("Repo must contain specified revision")?;
    let mut ancestors = rev
        .ancestors()
        .all()
        .context("Walk all ancestor revisions")?;
    ancestors.try_for_each(|info| {
        let info = info?;
        let revision = info.id();
        debug!("Processing revision: {}", info.id());

        let Ok(commit) = info.object() else {
            warn!("Not a commit. Skipping.");
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
                // NOTE Retrieved objects have a baked-in revision.
                let memo_key = MemoKey {
                    path: entry.filepath.clone(),
                    object_id: entry.oid,
                };

                // DESIGN Use the entries API, but how to handle control flow?
                // Reuse cached parses for already-processed objects
                let cached = cache.get(&memo_key);
                let results = match cached {
                    None => {
                        let obj = repo.find_object(entry.oid).context("Find file blob")?;

                        // Temp: Prove that we can get access to the file data.
                        let blob = obj.try_into_blob().context("Convert object to Blob")?;

                        let mut results = find_matches_in_blob(&entry.filepath, info.id(), &blob)
                            .unwrap_or(Vec::new());

                        // Checksum whole file instead of relying on a matcher.
                        let file_match = match_whole_file(&entry.filepath, info.id(), &blob);
                        results.push(file_match);

                        cache.insert(memo_key.clone(), results);
                        cache.get(&memo_key).unwrap()
                    }
                    Some(results) => results,
                };

                if !results.is_empty() {
                    debug!(
                        "\t\t{} {} {} results",
                        entry.filepath,
                        entry.oid,
                        results.len()
                    );
                    // Fixup potentially cached revisions
                    for result in results {
                        let result = UpstreamMatch {
                            revision: revision.to_string(),
                            ..result.clone()
                        };

                        trace!(
                            "\t\t\t{}: {} ({} {}) @ {}",
                            result.kind,
                            result.identifier,
                            result.hash,
                            result.length,
                            result.revision,
                        );

                        let _count = result.insert(db)?;
                    }
                }

                Result::<(), anyhow::Error>::Ok(())
            })?;

        Result::<(), anyhow::Error>::Ok(())
    })?;

    Ok(())
}

/// Extract interesting features from file. Automatically guesses file type from
/// extension, and chooses the corresponding extractor.
///
/// TODO Extract language detection and matcher selection. Use file-format or infer crate.
fn find_matches_in_blob(
    path: &BString,
    rev: Id,
    blob: &Blob,
) -> anyhow::Result<Vec<UpstreamMatch>> {
    let path = path.to_string();
    let path = Path::new(&path);

    // Primitive language detection, should eventually be abstracted out and configured with the
    // project.
    let lang = path.extension().and_then(|ext| match ext.to_str() {
        Some("rs") => Some(SupportedLanguage::Rust),
        #[cfg(feature = "lang-bash")]
        Some("sh") => Some(SupportedLanguage::Bash),
        _ => None,
    });

    // Only parse known languages for now.
    let Some(lang) = lang else {
        return Ok(Vec::new());
    };

    let (language, matchers) = match lang {
        SupportedLanguage::Rust => (tree_sitter_rust::LANGUAGE, Rust::matchers()),
        #[cfg(feature = "lang-bash")]
        SupportedLanguage::Bash => (tree_sitter_bash::LANGUAGE, Bash::matchers()),
    };

    let language: Language = language.into();

    // Parse file
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .expect("Create language parser");

    let tree = parser
        .parse(blob.data.as_slice(), None)
        .expect("Parse file");

    // Find matches
    let mut interesting_matches = Vec::<UpstreamMatch>::new();
    for matcher in &matchers {
        // Find matches and extract information
        let query = match Query::new(&language, matcher.query.as_str()) {
            Ok(query) => query,
            Err(e) => {
                error!("Skipping unparseable query {}", matcher.query);
                error!("{}", e);
                continue;
            }
        };

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), blob.data.as_slice());
        matches.for_each(|matched| {
            if let Some(m) = process_match(
                &"(self)".to_string(),
                &rev,
                path,
                &language,
                blob.data.as_slice(),
                matcher,
                &matched,
            ) {
                interesting_matches.push(m);
            }
        });
    }

    // These should probably be concatenated for efficiency, but settle for repeated searches. O(matchers * files)
    Ok(interesting_matches)
}

/// Compute SHA256 Hash of a byte array and its best-effort
/// whitespace-agnostic hash.
fn blob_hashes(contents: &[u8]) -> (String, String, Option<String>) {
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
    let hash = hasher.finalize();

    let hash = format!("{:02x}", hash);

    // Strip whitespace and generate hash if text is valid utf8.
    let hash_stripped = String::from_utf8(contents.to_vec())
        .ok()
        .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect::<String>())
        .map(|s| {
            let mut hasher = Sha256::new();
            if let Some(salt) = salt {
                hasher.update(salt.to_be_bytes());
            }
            hasher.update(s);
            let result = hasher.finalize();
            format!("{:02x}", result)
        });

    (hash_algorithm, hash, hash_stripped)
}

fn match_whole_file(path: &BString, rev: Id, blob: &Blob) -> UpstreamMatch {
    let (hash_algorithm, hash, hash_stripped) = blob_hashes(&blob.data);

    UpstreamMatch {
        upstream: "(self)".to_string(),
        revision: rev.to_string(),
        path: path.to_string(),
        start_byte: 0,
        length: blob.data.len() as u64,
        kind: "file".to_string(),
        identifier: path.to_string(),
        hash_algorithm,
        salt: None,
        hash,
        hash_stripped,
        notes: None,
    }
}

fn process_match(
    codebase: &String,
    revision: &Id,
    path: &Path,
    language: &Language,
    source_bytes: &[u8],
    matcher: &Matcher,
    matched: &QueryMatch,
) -> Option<UpstreamMatch> {
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
        error!("Failed to match contents");
        return None;
    };

    // Salted hash of contents, in case of sensitive data.
    let (hash_algorithm, hash, hash_stripped) = blob_hashes(contents);
    let start_byte = root_match.node.start_byte() as u64;
    let length = (root_match.node.end_byte() - root_match.node.start_byte()) as u64;

    Some(UpstreamMatch {
        upstream: codebase.to_string(),
        revision: revision.to_string(),
        path: file_path.to_string(),
        start_byte,
        length,
        kind: matcher.kind.to_string(),
        identifier: identifier.to_string(),
        hash_algorithm,
        salt: None,
        hash,
        hash_stripped,
        notes: None,
    })
}
