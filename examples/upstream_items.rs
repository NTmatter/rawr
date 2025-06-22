// SPDX-License-Identifier: Apache-2.0

//! Represent matchers for upstream items, find matches in a file, and extract
//! item names, content, and context.

#![allow(unused)]

use std::path::PathBuf;
use tree_sitter::{Node, Query, Range};

fn main() -> anyhow::Result<()> {
    Ok(())
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Hash {
    // format: sha256:hex-bytes, use hex::encode/decode
    Sha256([u8; 32]),
}

pub type UpstreamId = String;
pub type NodeType = String;
struct MatchedUpstreamItem {
    /// Identifier of upstream codebase
    upstream: String,

    /// Revision of upstream codebase
    revision: String,

    /// Path to file, relative to root of upstream codebase
    file: PathBuf,

    /// Name of the Tree-Sitter grammar
    lang: String,

    /// Name of node type, defined in grammar.
    kind: NodeType,

    /// Identifier for node, usually its name.
    ident: String,

    /// Path to item. Leave empty for now.
    ///
    /// What makes these unique in the face of duplicate code? A combination of
    /// `(offset, Hash, kind)` should be unique in the case of overlapping empty
    /// types.
    ancestors: Vec<PrimaryKey>,

    /// SHA-256 hash of entire match
    hash: Hash,

    /// Location of match as bytes and row/column coordinates
    range: Range,
}

// DESIGN Should the language be stored alongside NodeType? Collision is unlikely due to offset.
/// Uniquely identifies an item.
///
/// Offsets are used to distinguish duplicate code within a file, and the
/// type name is used to further disambiguate identical matches.
///
type PrimaryKey = (UpstreamId, usize, Hash, NodeType);
impl MatchedUpstreamItem {
    fn primary_key(&self) -> PrimaryKey {
        (
            self.upstream.clone(),
            self.range.start_byte,
            self.hash,
            self.kind.clone(),
        )
    }
}

// DESIGN is it possible to build a path by traversing upwards through ancestors?
/// Describe how to match an upstream item and extract relevant data.
struct UpstreamItemMatcher {
    kind: String,
    /// Tree-Sitter query
    query: Query,
    ident: Option<ExtractWith>,
    notes: Option<String>,
}

// TODO Extract named child (from grammar) or named node (from match)
enum ExtractWith {
    /// Convert the entire match to a string
    WholeMatch,

    /// Supply a constant, filtered through a templating replacement.
    Constant(String),

    /// Extract from the named field, specified in the grammar's node type.
    NamedField(String, Box<ExtractWith>),

    /// Use the Nth child of the given type
    NthChild(usize, String, Box<ExtractWith>),

    /// Execute an additional query to identify the content, and extract it with
    /// the given extractor.
    Subquery(Query, Box<ExtractWith>),
}

#[test]
fn foo() -> anyhow::Result<()> {
    let java = tree_sitter_java::LANGUAGE.into();

    use ExtractWith::*;
    let class_matcher = UpstreamItemMatcher {
        kind: "class".into(),
        query: Query::new(&java, "(class_declaration)")?,

        // Identify classes by name. This might not be unique for nested classes.
        ident: Some(NamedField("name".into(), Box::new(WholeMatch))),

        notes: None,
    };

    let method_ident: &str = "((modifiers)* @mods
  . type: (_) @ty
  . name: (identifier) @name
  . parameters: (formal_parameters) @params)";

    let function_matcher = UpstreamItemMatcher {
        kind: "method_declaration".into(),
        query: Query::new(&java, "(method_declaration)")?,
        ident: Some(Subquery(
            Query::new(&java, method_ident)?,
            Box::new(WholeMatch),
        )),
        notes: None,
    };

    Ok(())
}

// DESIGN Is the extraction machinery really necessary? Yes, to combine type+name+parameters.
/// Extract raw data from a query.
fn extract<'data>(
    extractor: &ExtractWith,
    upstream: &str,
    file: PathBuf,
    kind: &str,
    root: &Node,
    data: &'data [u8],
) -> anyhow::Result<Option<MatchedUpstreamItem>> {
    // Execute query and extract match

    //
    use ExtractWith::*;
    let matched: Option<&'data [u8]> = match extractor {
        WholeMatch => todo!(),
        Constant(value) => todo!(),
        // Recursion required
        NamedField(name, extractor) => todo!(),
        NthChild(index, kind, extractor) => todo!(),
        Subquery(query, extractor) => todo!(),
    };

    todo!()
}
