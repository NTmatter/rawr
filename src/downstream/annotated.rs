// SPDX-License-Identifier: Apache-2.0

// SPDX-License-Identifier: Apache-2.0

//! Tools for matching and extracting information from RAWR Annotations.

/// Tree-Sitter query for `rawr` annotations.
///
/// RESEARCH Is it possible to extract the repeated ident/literal matches?
///
/// It might be necessary to build a state machine that starts a new object
/// upon finding an identifier, collecting literals into a map. Not too complex.
pub const RAWR_QUERY: &str = r#"(attribute
  (identifier) @name (#eq? @name "rawr")
  arguments: (token_tree
    ((identifier) @ident "="
     [(string_literal)(boolean_literal)(integer_literal)] @literal))+) @attr"#;

/// Query for outer attribute, capturing arguments for a follow-up search with
/// a simpler query and fixed bounds.
pub const RAWR_QUERY_ATTRIBUTE: &str = r#"(attribute
  (identifier) @name (#eq? @name "rawr")
  arguments: (token_tree) @args)"#;

/// Query for `identifier = literal` pairs inside arguments token tree.
pub const RAWR_QUERY_ARGS: &str =
    r#"((identifier) @ident "=" [(string_literal)(boolean_literal)(integer_literal)] @literal)"#;

// How will this be destructured? It might be necessary to do a two-part search
// to identify the relevant attribute then pull apart the token tree idents and
// literals in a second pass. I believe that Tree-Sitter allows for searching
// within matches, which should take care of a lot of bounds checking.

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

/// Original location of a Watch annotation. Unclear if this will be needed,
/// or how to extract the start point from Tree-Sitter.
///
/// Ultimately, this should contain the source File, Line, and Column
pub type WatchLocation = ();
