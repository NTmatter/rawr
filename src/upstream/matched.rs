// SPDX-License-Identifier: Apache-2.0

//! Representation and functionality for items that have been matched in
//! upstream repositories.

use crate::upstream::UpstreamId;
use std::path::PathBuf;
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
    pub file: PathBuf,

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
