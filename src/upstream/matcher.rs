// SPDX-License-Identifier: Apache-2.0

//! Functionality for matching upstream items.

use tree_sitter::{Query, QueryMatch};

/// Match a class of items in an upstream codebase
pub struct Matcher {
    /// Unique name for matched kind
    pub kind: &'static str,

    /// Tree-Sitter query for matching the full item body
    pub query: Query,

    /// Strategy for extracting items ident
    pub ident: Option<Extractor>,

    /// Optional human-friendly notes about this matcher
    pub notes: Option<&'static str>,
}

/// Strategy for extracting data from a larger match.
pub enum Extractor {
    /// Convert the entire match to a string
    WholeMatch,

    /// Supply a constant, filtered through a templating replacement.
    Constant(&'static str),

    /// Extract from the named field, specified in the grammar's node type.
    NamedField(&'static str, Box<Extractor>),

    /// Use the Nth child of the given type
    NthChild(usize, &'static str, Box<Extractor>),

    /// Execute an additional query to identify the content, and extract it with
    /// the given extractor.
    Subquery(Query, Box<Extractor>),
}

impl Extractor {
    pub fn extract(&self, matched: &QueryMatch, data: &[u8]) -> &[u8] {
        todo!()
    }
}
