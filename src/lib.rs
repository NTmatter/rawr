// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use rusqlite::{named_params, Connection, OptionalExtension};
use std::collections::HashSet;
use std::path::PathBuf;

pub mod lang;

#[derive(Debug, Eq, PartialEq)]
pub struct Codebase {
    pub name: String,
    pub relative_path: String,
    pub notes: Option<String>,
}

/// Automatically-matched item of interest. These are generally persisted to the
/// database for tracking movement.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Interesting {
    // Location containing match.
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
    pub hash_algorithm: String,
    pub salt: Option<u64>,
    /// Hash of matched data. Matched data is not stored, as it can be retrieved
    /// from the repository.
    pub hash: String,

    /// Hash of matched data with spaces stripped. Optional, in case of binary data.
    pub hash_stripped: Option<String>,

    pub notes: Option<String>,
}

pub fn db_connection(db_path: PathBuf) -> anyhow::Result<Connection> {
    // TODO Disable Open with URI
    let conn = Connection::open(db_path).context("Open or create database")?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .context("Enable foreign key support")?;

    conn.execute_batch(include_str!("rawr.sql"))
        .context("Create tables if needed")?;

    Ok(conn)
}

impl Interesting {
    /// Insert into database via prepared statemet
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

/// Corresponds to the fields of the RAWR annotation.
/// Look up (codebase, revision, path, kind, identifier) tuple in database to
/// find salt, then compute local checksum for comparison.
// Pain point: Finding the item that an annotation is connected to. This might
// not be a problem, as we're only looking at the referenced item in the current
// and new revision.
#[derive(Debug, Eq, PartialEq)]
pub struct Watched {
    pub codebase: String,
    pub revision: String,

    pub path: Option<String>,
    pub kind: Option<String>,
    pub identifier: Option<String>,

    pub notes: Option<String>,
    // TODO Optional checksum to avoid lookup?
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Change {
    Added,
    Deleted,
    Moved,
    Modified,
    WhitespaceOnly,
}

pub type Changes = HashSet<Change>;
