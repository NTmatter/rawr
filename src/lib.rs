// SPDX-License-Identifier: Apache-2.0

use rusqlite::{named_params, Connection, Row, Statement};

pub mod lang;

#[derive(Debug, Eq, PartialEq)]
pub struct Codebase {
    pub name: String,
    pub relative_path: String,
    pub notes: Option<String>,
}

/// Automatically-matched item of interest. These are generally persisted to the
/// database for tracking movement.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Interesting {
    // Location containing match.
    pub codebase: String,
    pub revision: String,
    pub path: String,
    /// Offset from start of file, in bytes.
    pub start_byte: usize,
    /// Length of match, in bytes.
    pub length: usize,

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

impl Interesting {
    /// Builds a prepared statement for bulk inserting into database
    pub fn insert_query(db: &Connection) -> anyhow::Result<Statement> {
        // language=sqlite
        let prepared_query = db.prepare(
            r#"INSERT OR IGNORE INTO upstream
(codebase, revision, path, start_byte, length, identifier, kind, hash_algorithm, salt, hash,
 hash_stripped, notes)
VALUES
(:codebase, :revision, :path, :start_byte, :length, :identifier, :kind, :hash_algorithm, :salt, :hash,
:hash_stripped, :notes)"#,
        )?;

        Ok(prepared_query)
    }

    /// Insert into database via prepared statemet
    pub fn insert_prepared(&self, stmt: &mut Statement) -> anyhow::Result<usize> {
        let count = stmt.execute(named_params! {
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
}

impl TryFrom<Row<'_>> for Interesting {
    type Error = anyhow::Error;

    fn try_from(value: Row<'_>) -> Result<Self, Self::Error> {
        let item = Self {
            codebase: value.get("codebase")?,
            revision: value.get("revision")?,
            path: value.get("path")?,
            start_byte: value.get("start_byte")?,
            length: value.get("length")?,
            kind: value.get("kind")?,
            identifier: value.get("identifier")?,
            hash_algorithm: value.get("hash_algorithm")?,
            salt: value.get("salt")?,
            hash: value.get("hash")?,
            hash_stripped: value.get("hash_stripped")?,
            notes: value.get("notes")?,
        };

        Ok(item)
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
