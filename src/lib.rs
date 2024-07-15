// SPDX-License-Identifier: Apache-2.0

pub mod lang;

#[derive(Debug, Eq, PartialEq)]
pub struct Codebase {
    pub name: String,
    pub relative_path: String,
    pub notes: Option<String>,
}

/// Automatically-matched item of interest. These are generally persisted to the
/// database for tracking movement.
#[derive(Debug, Eq, PartialEq)]
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

    pub notes: Option<String>,
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
