// SPDX-License-Identifier: Apache-2.0

//! Representation and functionality for items that have been matched in
//! upstream repositories.

use crate::upstream::UpstreamId;
use anyhow::bail;
use rusqlite::{Connection, named_params};
use std::path::PathBuf;
use tracing::debug;
use tree_sitter::Range;

/// Hash of matched data
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Hash {
    Sha256([u8; 32]),
}

/// Item of interest in the upstream codebase.
///
/// Uniquely identified by the codebase, revision, path, kind, and identifier.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct UpstreamMatch {
    /// Identifier of upstream codebase.
    pub upstream: UpstreamId,

    /// Revision of upstream codebase
    pub revision: String,

    /// Relative path to file within the upstream codebase.
    pub path: PathBuf,

    /// Location of item within file, as byte offset and line/character
    pub range: Range,

    /// Name of the Tree-Sitter grammar.
    pub lang: String,

    /// Type of matched object, defined in the Tree-Sitter grammar.
    pub kind: String,

    /// Identifier for item, usually its name.
    pub identifier: String,

    /// Name of algorithm used to hash body of the element.
    pub hash_algorithm: String,

    /// Hash of matched data for faster lookup. Original data can be retrieved
    /// from the repository.
    pub hash: Vec<u8>,

    /// Hash of matched data, lossy-converted to utf8 and whitespace removed.
    /// Optional, to allow for binary data.
    pub hash_stripped: Option<Vec<u8>>,

    /// Human-friendly notes attached to the matched object.
    ///
    /// Given the automated sourcing of these matches, notes are unlikely.
    pub notes: Option<String>,
}

// INSERT INTO upstream ( ... ) VALUES ( ... ) ON CONFLICT IGNORE;

impl UpstreamMatch {
    pub fn insert(&self, conn: &Connection) -> anyhow::Result<bool> {
        // DESIGN Should this be INSERT OR IGNORE/REPLACE/ROLLBACK for error handling?
        // Roll back the transaction when duplicates are encountered.
        let mut statement = conn.prepare_cached(
            r#"
INSERT OR ROLLBACK INTO upstream
    (upstream, revision, path,
     lang, kind, identifier, hash, hash_stripped,
     start_byte, end_byte, start_line, start_column, end_line, end_column,
     notes)
VALUES
    (:upstream, :revision, :path,
     :lang, :kind, :identifier, :hash, :hash_stripped,
     :start_byte, :end_byte, :start_line, :start_column, :end_line, :end_column,
     :notes)"#,
        )?;

        let count = statement.execute(named_params! {
            ":upstream": &self.upstream,
            ":revision": &self.revision,
            ":path": &self.path.to_string_lossy(),
            ":lang": &self.lang,
            ":kind": &self.kind,
            ":identifier": &self.identifier,
            ":hash": &self.hash,
            ":hash_stripped": &self.hash_stripped,
            ":start_byte": &self.range.start_byte,
            ":end_byte": &self.range.end_byte,
            ":start_line": &self.range.start_point.row,
            ":start_column": &self.range.start_point.column,
            ":end_line": &self.range.end_point.row,
            ":end_column": &self.range.end_point.column,
            ":notes": &self.notes,
        })?;

        Ok(count > 0)
    }

    pub fn insert_batch(conn: &Connection, items: &[Self]) -> anyhow::Result<usize> {
        let _ = conn.execute("BEGIN TRANSACTION", [])?;
        debug!("Inserting {} upstream match rows", items.len());

        let mut affected: usize = 0;
        for item in items {
            if let Err(err) = item.insert(conn) {
                bail!("Failed to insert {item:?}")
            } else {
                affected += 1;
            }
        }

        let _ = conn.execute("COMMIT TRANSACTION", [])?;
        debug!("Done. Affected {affected} rows.");

        Ok(affected)
    }
}
