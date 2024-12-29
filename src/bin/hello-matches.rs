// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use streaming_iterator::StreamingIterator;

use clap::Parser as ClapParser;
use tree_sitter::{self, Parser, Query, QueryCursor, Tree};
use tree_sitter_bash;
use tree_sitter_rust;

/// Tree-Sitter query for RAWR annotations attached to various declarations
// FIXME Only accepts last few rawr attributes. Consider post-filter?
// Event-based filter makes more sense. This is sufficient for capturing basic rust annotations and their targets.
const FULL_ANNOTATIONS_QUERY: &str = "
    ((attribute_item
      (attribute
        (identifier) @rawr
        (#eq? @rawr \"rawr\")
        arguments: (token_tree
          ((identifier) @id \"=\" (_literal) @lit \",\"?)+)))+ @ai
      ; Ignore comments
      . [(line_comment) (block_comment)]*
      .
      ; Match most declarations. Consider matching (_) as the annotation can likely go anywhere.
      [(struct_item) (function_item) (const_item) (enum_item) (enum_variant) (let_declaration)] @item)";

/// Search for `rawr` annotations in Rust sources
const ANNOTATION_QUERY: &str = "
((attribute (identifier) @rawr) @ai
  (#eq? @rawr \"rawr\"))
";

/// Match key-value pairs in attribute arguments
/// TODO Test replacement of iterator
const ANNOTATION_ATTRIBUTE_QUERY: &str = "
(arguments: (token_tree ((identifier) @key . \"=\" . (_literal) @val)* @pair))
";

#[derive(ClapParser, Debug)]
struct Args {
    #[arg(required = true)]
    rust_file: PathBuf,

    #[arg(required = true)]
    bash_file: PathBuf,
}

fn main() -> Result<(), io::Error> {
    let Args {
        rust_file: implementation_file,
        bash_file: upstream_file,
    } = Args::parse();

    parse_annotations(implementation_file);
    parse_bash(upstream_file);
    Ok(())
}

fn parse_bash(source_file: PathBuf) {
    println!("--- Bash ---");
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_bash::LANGUAGE.into())
        .expect("Create Bash parser");

    let mut source_file = File::open(source_file).expect("Open upstream file");
    let mut source_bytes = Vec::new();
    source_file
        .read_to_end(&mut source_bytes)
        .expect("Read upstream file");

    let tree = parser
        .parse(&source_bytes.as_slice(), None)
        .expect("Parse upstream file");

    // Find variable FOO
    let query = "(variable_assignment (variable_name) @var \"=\" (_) @body (#eq? @var \"FOO\"))";
    print_matches(query, &source_bytes, &tree);

    let query = "
    (([(function_definition) (variable_assignment)]) @def)";

    print_matches(query, &source_bytes, &tree);
}

fn print_matches(query_string: &str, source_bytes: &Vec<u8>, tree: &Tree) {
    let query = Query::new(&tree.language(), query_string).expect("Create query");
    let mut query_cursor = QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), source_bytes.as_slice());
    matches.for_each(|m| {
        println!("Match {}: {:?}", m.pattern_index, m);

        m.captures.iter().for_each(|cap| {
            let node = cap.node;
            println!(
                "\t{}: {:?}, {} named children",
                cap.index,
                cap,
                node.named_child_count()
            );
            println!(
                "\t\t{:?} {:?}",
                String::from_utf8_lossy(&source_bytes[node.start_byte()..node.end_byte()]),
                node.to_sexp(),
            );

            // Grammars with named children are easier to pick apart.
            match node.kind() {
                "function_definition" => {
                    if let Some(name) = node.child_by_field_name("name") {
                        if let Some(body) = node.child_by_field_name("body") {
                            println!(
                                "\t\t{} -> {:?}",
                                String::from_utf8_lossy(
                                    &source_bytes[name.start_byte()..name.end_byte()]
                                ),
                                String::from_utf8_lossy(
                                    &source_bytes[body.start_byte()..body.end_byte()]
                                )
                            )
                        }
                    }
                }
                "variable_assignment" => {
                    if let Some(name) = node.child_by_field_name("name") {
                        if let Some(value) = node.child_by_field_name("value") {
                            println!(
                                "\t\t{} = {:?} -- {}",
                                String::from_utf8_lossy(
                                    &source_bytes[name.start_byte()..name.end_byte()]
                                ),
                                String::from_utf8_lossy(
                                    &source_bytes[value.start_byte()..value.end_byte()]
                                ),
                                node.to_sexp()
                            );
                        }
                    }
                }
                "attribute" => {
                    if let Some(args) = node.child_by_field_name("arguments") {
                        // Named children should form key-value pairs.
                        let mut tree_cursor = args.walk();
                        let mut children = args.named_children(&mut tree_cursor).into_iter();

                        while let Some(key) = children.next() {
                            if let Some(val) = children.next() {
                                println!(
                                    "\t\t\tArgument: {} = ({}) {}",
                                    String::from_utf8_lossy(
                                        &source_bytes[key.start_byte()..key.end_byte()]
                                    ),
                                    val.kind(),
                                    String::from_utf8_lossy(
                                        &source_bytes[val.start_byte()..val.end_byte()]
                                    )
                                )
                            }
                        }
                    }
                }
                _ => {}
            };
        });
    });
}

fn parse_annotations(source_file: PathBuf) {
    // TODO Iterate over all paths in all codebases.

    // see: https://github.com/tree-sitter/tree-sitter/tree/master/lib/binding_rust
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Create Rust parser");
    let mut source_file = File::open(source_file).expect("Open test file");
    let mut source_bytes = Vec::new();
    source_file
        .read_to_end(&mut source_bytes)
        .expect("Read test file");

    // Parse and walk tree
    let tree = parser
        .parse(&source_bytes.as_slice(), None)
        .expect("Parse test file");

    // see https://deepsource.com/blog/lightweight-linting
    println!("--- Matches ---");

    print_matches(ANNOTATION_QUERY, &source_bytes, &tree);
}

/// Common options for annotations
#[derive(Eq, PartialEq)]
pub struct Rawr {
    /// Optional name of codebase that the upstream resides in.
    codebase: Option<String>,
    /// Git revision (treeish), required
    rev: String,
    /// Path to original file, relative to codebase root
    path: Option<String>,
    /// Tree-Sitter query identifying the upstream implementation.
    /// Mutually exclusive to class/function/symbol.
    query: Option<String>,
    class: Option<String>,
    /// Function or class method.
    /// TODO How do we handle function overloading?
    function: Option<String>,
    /// Standalone variables and declarations
    // Renamee to Variable?
    symbol: Option<String>,
    /// Free-form notes regarding the implementation.
    notes: Option<String>,
    /// Free-form implementation status. Special case for NO, NONE, WIP, DONE, BROKEN, UPDATE.
    implemented: Option<String>,
    /// Hash of implementation body, without whitespace or comments.
    hash: Option<String>,
    /// Hash of implementation body, comments stripped, and normalized whitespace.
    hash_ws: Option<String>,
    /// Hash of full implementation body.
    hash_raw: Option<String>,
}

pub struct Codebase {
    /// Mapping of paths to parser configurations.
    paths: HashMap<String, tree_sitter::Language>,
}
