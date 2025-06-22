// SPDX-License-Identifier: Apache-2.0

//! Tools for matching and extracting information from RAWR Annotations.

use crate::downstream::scan::Literal;
use std::collections::HashMap;
use thiserror::Error;

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
    /// Identifier for upstream codebase. Defaults to the first upstream in the list.
    pub upstream: Option<String>,

    /// Last-seen revision within upstream repository.
    ///
    /// This can be anything that git recognizes as a revision, including tag
    /// and branch names.
    pub revision: String,

    /// Relative path to file within upstream codebase's repository
    pub file: String,

    /// Type of matched item, specific to the Tree-Sitter grammar.
    pub kind: String,

    // DESIGN Can this capture nested structure? X::y() vs A::y() vs F::G::y()
    /// Identifier for named items
    pub identifier: Option<String>,

    /// Free-form field for optional implementation status.
    ///
    /// The default workflow uses `DONE` and `TODO`
    pub state: Option<String>,

    /// Free-form field for optional implementation planning.
    pub action: Option<String>,

    /// Free-form field for optional implementation notes.
    pub notes: Option<String>,

    /// Ignore this item in the upstream.
    pub ignore: Option<bool>,
}

#[derive(Debug, Error)]
pub enum ParseWatchedError {
    #[error("Missing required argument: {field}")]
    MissingRequiredArg { field: String },
    #[error("Incorrect type for argument {field}. Expected {expected_kind}.")]
    IncorrectArgType {
        field: String,
        expected_kind: String,
    },
}

// DESIGN Parse with syn or use derive_builder crate.
impl TryFrom<HashMap<String, Literal>> for Watched {
    type Error = Vec<ParseWatchedError>;

    fn try_from(value: HashMap<String, Literal>) -> Result<Self, Self::Error> {
        use ParseWatchedError::*;

        let mut errors = Vec::new();

        // Upstream - Optional String
        let key = "upstream";
        let upstream = match value.get(key) {
            Some(Literal::String(s)) => Some(s).cloned(),
            Some(_) => {
                errors.push(IncorrectArgType {
                    field: key.to_string(),
                    expected_kind: "String".to_string(),
                });
                None
            }
            None => None,
        };

        // Revision - Required String
        let key = "rev";
        let revision = match value.get(key) {
            Some(Literal::String(s)) => Some(s).cloned(),
            Some(_) => {
                errors.push(IncorrectArgType {
                    field: key.to_string(),
                    expected_kind: "String".to_string(),
                });
                None
            }
            None => {
                errors.push(MissingRequiredArg {
                    field: key.to_string(),
                });
                None
            }
        };

        // File - Required String
        let key = "file";
        let file = match value.get(key) {
            Some(Literal::String(s)) => Some(s).cloned(),
            Some(_) => {
                errors.push(IncorrectArgType {
                    field: key.to_string(),
                    expected_kind: "String".to_string(),
                });
                None
            }
            None => {
                errors.push(MissingRequiredArg {
                    field: key.to_string(),
                });
                None
            }
        };

        // Kind - Required String
        let key = "kind";
        let kind = match value.get(key) {
            Some(Literal::String(s)) => Some(s).cloned(),
            Some(_) => {
                errors.push(IncorrectArgType {
                    field: key.to_string(),
                    expected_kind: "String".to_string(),
                });
                None
            }
            None => {
                errors.push(MissingRequiredArg {
                    field: key.to_string(),
                });
                None
            }
        };

        // Identifier - Required String
        let key = "ident";
        let identifier = match value.get(key) {
            Some(Literal::String(s)) => Some(s).cloned(),
            Some(_) => {
                errors.push(IncorrectArgType {
                    field: key.to_string(),
                    expected_kind: "String".to_string(),
                });
                None
            }
            None => None,
        };

        // State - Optional String
        let key = "state";
        let state = match value.get(key) {
            Some(Literal::String(s)) => Some(s).cloned(),
            Some(_) => {
                errors.push(IncorrectArgType {
                    field: key.to_string(),
                    expected_kind: "String".to_string(),
                });
                None
            }
            None => None,
        };

        // Action - Optional String
        let key = "action";
        let action = match value.get(key) {
            Some(Literal::String(s)) => Some(s).cloned(),
            Some(_) => {
                errors.push(IncorrectArgType {
                    field: key.to_string(),
                    expected_kind: "String".to_string(),
                });
                None
            }
            None => None,
        };

        // Notes - Optional String
        let key = "notes";
        let notes = match value.get(key) {
            Some(Literal::String(s)) => Some(s).cloned(),
            Some(_) => {
                errors.push(IncorrectArgType {
                    field: key.to_string(),
                    expected_kind: "String".to_string(),
                });
                None
            }
            None => None,
        };

        // Ignore - Optional Boolean
        let key = "ignore";
        let ignore = match value.get(key) {
            Some(Literal::Boolean(b)) => Some(b).cloned(),
            Some(_) => {
                errors.push(IncorrectArgType {
                    field: key.to_string(),
                    expected_kind: "bool".to_string(),
                });
                None
            }
            None => None,
        };

        // Return error if there are any missing or incorrect fields
        if !errors.is_empty() {
            return Err(errors);
        }

        // Safely unpack required fields. A builder pattern would be nicer here.
        let Some(revision) = revision else {
            return Err(vec![MissingRequiredArg {
                field: "rev".to_string(),
            }]);
        };
        let Some(file) = file else {
            return Err(vec![MissingRequiredArg {
                field: "file".to_string(),
            }]);
        };
        let Some(kind) = kind else {
            return Err(vec![MissingRequiredArg {
                field: "kind".to_string(),
            }]);
        };

        // Return struct
        Ok(Self {
            upstream,
            revision,
            file,
            kind,
            identifier,
            state,
            action,
            notes,
            ignore,
        })
    }
}

/// Original location of a Watch annotation. Unclear if this will be needed,
/// or how to extract the start point from Tree-Sitter.
///
/// Ultimately, this should contain the source File, Line, and Column
pub type WatchLocation = ();
