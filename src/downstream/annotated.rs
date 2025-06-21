// SPDX-License-Identifier: Apache-2.0

//! Tools for matching and extracting information from RAWR Annotations.

use crate::downstream::scan::Literal;
use std::collections::HashMap;

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
    pub upstream: String,

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

enum ParseWatchedError {
    MissingRequiredArg,
    IncorrectArgLiteral,
    ParseError,
}

impl TryFrom<HashMap<String, Literal>> for Watched {
    type Error = Vec<String>;

    fn try_from(value: HashMap<String, Literal>) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// Original location of a Watch annotation. Unclear if this will be needed,
/// or how to extract the start point from Tree-Sitter.
///
/// Ultimately, this should contain the source File, Line, and Column
pub type WatchLocation = ();
