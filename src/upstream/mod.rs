// SPDX-License-Identifier: Apache-2.0

use rusqlite::{named_params, Connection, OptionalExtension};
use std::path::PathBuf;

/// Item of interest in the upstream codebase.
///
/// Uniquely identified by the codebase, revision, path, kind, and identifier.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct UpstreamMatch {
    /// Name of codebase, or default if not specified.
    pub codebase: String,
    pub revision: String,
    /// Relative path to file
    pub path: String,
    /// Offset from start of file, in bytes.
    pub start_byte: u64,
    /// Length of match, in bytes.
    pub length: u64,

    // Type and identifier
    /// Type of matched object
    pub kind: String,
    /// Identifier for object
    pub identifier: String,

    // Hash details
    /// Name of algorithm used to hash body of the element.
    pub hash_algorithm: String,

    /// Optional salt for hash to mitigate rainbow attacks.
    pub salt: Option<u64>,

    /// Hash of matched data for faster lookup. Original data can be retrieved
    /// from the repository.
    pub hash: String,

    /// Hash of matched data with spaces stripped. Optional, in case of binary data.
    pub hash_stripped: Option<String>,

    /// Human-friendly notes attached to the matched object.
    ///
    /// Given the automated sourcing of these matches, notes are unlikely.
    pub notes: Option<String>,
}

impl UpstreamMatch {
    /// Insert into database via prepared statement
    pub fn insert(&self, db: &Connection) -> anyhow::Result<usize> {
        // language=sqlite
        let mut statement = db.prepare_cached(
            r#"INSERT OR IGNORE INTO upstream
(codebase, revision, path, start_byte, length, identifier, kind, hash_algorithm, salt, hash,
 hash_stripped, notes)
VALUES
(:codebase, :revision, :path, :start_byte, :length, :identifier, :kind, :hash_algorithm, :salt, :hash,
:hash_stripped, :notes)"#,
        )?;

        let count = statement.execute(named_params! {
            ":codebase": self.codebase,
            ":revision": self.revision,
            ":path": self.path,
            ":start_byte": self.start_byte,
            ":length": self.length,
            ":identifier": self.identifier,
            ":kind": self.kind,
            ":hash_algorithm": self.hash_algorithm,
            ":salt": self.salt,
            ":hash": self.hash,
            ":hash_stripped": self.hash_stripped,
            ":notes": self.notes,
        })?;

        Ok(count)
    }

    /// Retrieve metadata for an item at a particular revision.
    ///
    /// Does not allow handling duplicate definitions.
    ///
    /// DESIGN Improve handling of duplicate items (eg re-definition) within a file.
    /// TODO Take an optional hash and/or offset.
    pub fn get_watched_item_at_revision(
        db: &Connection,
        codebase: &str,
        revision: &str,
        path: &PathBuf,
        kind: &str,
        identifier: &str,
    ) -> anyhow::Result<Vec<Self>> {
        // language=sqlite
        let mut statement = db.prepare_cached(
            "SELECT *
FROM upstream
WHERE codebase = :codebase
  AND revision = :revision
  AND path = :path
  AND kind = :kind
  AND identifier = :identifier",
        )?;

        let mut results = Vec::new();
        statement
            .query_map(
                named_params! {
                    ":codebase": codebase,
                    ":revision": revision,
                    ":path": path.to_string_lossy(),
                    ":kind": kind,
                    ":identifier": identifier,
                },
                |row| {
                    Ok(Self {
                        codebase: row.get("codebase")?,
                        revision: row.get("revision")?,
                        path: row.get("path")?,
                        start_byte: row.get("start_byte")?,
                        length: row.get("length")?,
                        kind: row.get("kind")?,
                        identifier: row.get("identifier")?,
                        hash_algorithm: row.get("hash_algorithm")?,
                        salt: row.get("salt")?,
                        hash: row.get("hash")?,
                        hash_stripped: row.get("hash_stripped")?,
                        notes: row.get("notes")?,
                    })
                },
            )?
            .try_for_each(|result| {
                let result = result?;
                results.push(result);
                Ok::<(), anyhow::Error>(())
            })?;

        Ok(results)
    }

    pub fn get_one_from_db(
        db: &Connection,
        codebase: &str,
        revision: &str,
        path: PathBuf,
        kind: &str,
        identifier: &str,
        salt: Option<u64>,
        hash_algorithm: &str,
        hash: &str,
    ) -> anyhow::Result<Option<Self>> {
        // language=sqlite
        let mut statement = db.prepare_cached(
            "SELECT *
FROM upstream
WHERE codebase = :codebase
  AND revision = :revision
  AND path = :path
  AND kind = :kind
  AND identifier = :identifier
  AND hash_algorithm = :hash_algorithm
  AND salt = :salt
  AND hash = :hash
  LIMIT 1",
        )?;

        let res = statement
            .query_row(
                named_params! {
                    ":codebase": codebase,
                    ":revision": revision,
                    ":path": path.to_string_lossy(),
                    ":kind": kind,
                    ":identifier": identifier,
                    ":salt": salt,
                    ":hash_algorithm": hash_algorithm,
                    ":hash": hash,
                },
                |row| {
                    Ok(Self {
                        codebase: row.get("codebase")?,
                        revision: row.get("revision")?,
                        path: row.get("path")?,
                        start_byte: row.get("start_byte")?,
                        length: row.get("length")?,
                        kind: row.get("kind")?,
                        identifier: row.get("identifier")?,
                        hash_algorithm: row.get("hash_algorithm")?,
                        salt: row.get("salt")?,
                        hash: row.get("hash")?,
                        hash_stripped: row.get("hash_stripped")?,
                        notes: row.get("notes")?,
                    })
                },
            )
            .optional()?;

        Ok(res)
    }
}
