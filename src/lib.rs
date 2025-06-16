// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use rusqlite::{named_params, Connection, OptionalExtension};
use std::path::PathBuf;

pub mod lang;

pub fn db_connection(db_path: PathBuf) -> anyhow::Result<Connection> {
    // TODO Disable Open with URI
    let conn = Connection::open(db_path).context("Open or create database")?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .context("Enable foreign key support")?;

    conn.execute_batch(include_str!("rawr.sql"))
        .context("Create tables if needed")?;

    Ok(conn)
}

/// Core information about an upstream codebase.
#[derive(Debug, Eq, PartialEq)]
pub struct Codebase {
    pub name: String,
    pub relative_path: String,
    pub notes: Option<String>,
}

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

/// Points at an UpstreamMatch in the database.
///
/// Built from annotations on the downstream codebase and used to search for
/// changes in the upstream codebase.
///
/// Corresponds to the (not yet defined) fields of the RAWR annotation.
/// Look up `(codebase, revision, path, kind, identifier)` tuple in database to
/// find current information, including salt, then compute local checksum for
/// comparison.
// Pain point: Finding the item that an annotation is connected to. This might
// not be a problem, as we're only looking at the referenced item in the current
// and new revision.
#[derive(Debug, Eq, PartialEq)]
pub struct Watched {
    /// Identifier for upstream codebase
    pub codebase: String,

    /// Last-seen revision within upstream repository
    pub revision: String,

    /// Path to file within upstream codebase's repository
    pub path: Option<String>,

    /// Type of matched item, specific to the Tree-Sitter grammar.
    pub kind: Option<String>,

    /// Identifier for named item
    pub identifier: Option<String>,

    /// User-facing implementation action to take.
    ///
    /// Special-case for case-insensitive `IGNORE`, in default workflow.
    ///
    /// DESIGN Should this be an enum? What other states could be useful?
    pub action: Option<String>,

    /// Human-friendly notes on the item in question.
    pub notes: Option<String>,

    /// Optional checksum to avoid recomputation during lookup.
    pub checksum: Option<String>,
}

/// Represent the type of change to an item in a given revision
#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Change {
    /// Item has been created
    Add,
    /// Item has been deleted
    Delete,
    /// Item contents have changed
    Modify,
    /// Whitespace changes only
    Whitespace,
}
