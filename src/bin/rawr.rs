#![allow(dead_code)]
use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read};
use tree_sitter;
use tree_sitter::{Parser, Query, QueryCursor, Tree};
use tree_sitter_bash;
use tree_sitter_rust;
use tree_sitter_traversal as tst;
use tree_sitter_traversal::Order;

/// Tree-Sitter query for RAWR annotations attached to various declarations
// FIXME Only accepts last few rawr attributes. Consider post-filter?
// Event-based filter makes more sense. This is sufficient for capturing basic rust annotations and their targets.
const RAWR_ANNOTATION_QUERY: &str = "
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

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = args().collect();
    if args.len() < 3 {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "Usage: rawr rust_file bash_file",
        ));
    }
    let implementation_file = args.get(1).unwrap();
    let upstream_file = args.get(2).unwrap();

    parse_annotations(implementation_file);
    parse_bash(upstream_file);
    Ok(())
}

fn parse_bash(source_file: &String) {
    println!("--- Bash ---");
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_bash::language())
        .expect("Create Bash parser");

    let mut source_file = File::open(source_file).expect("Open upstream file");
    let mut source_bytes = Vec::new();
    source_file
        .read_to_end(&mut source_bytes)
        .expect("Read upstream file");

    let tree = parser
        .parse(&source_bytes.as_slice(), None)
        .expect("Parse upstream file");

    let cur = tst::traverse_tree(&tree, Order::Pre);
    for node in cur {
        println!("Node: {:?} named: {}", node, node.is_named());
    }

    // Find variable FOO
    let query = "(variable_assignment (variable_name) @var \"=\" (_) @body (#eq? @var \"FOO\"))";
    print_matches(query, &source_bytes, &tree);

    let query = "
    (([(function_definition) (variable_assignment)]) @def)";

    print_matches(query, &source_bytes, &tree);
}

fn print_matches(query_string: &str, source_bytes: &Vec<u8>, tree: &Tree) {
    let query = Query::new(tree.language(), query_string).expect("Create query");
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
                _ => {}
            };
        });
    });
}

fn parse_annotations(source_file: &String) {
    // TODO Iterate over all paths in all codebases.

    // see: https://github.com/tree-sitter/tree-sitter/tree/master/lib/binding_rust
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_rust::language())
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

    let cur = tst::traverse_tree(&tree, Order::Pre);
    for node in cur {
        println!("Node of type {} named: {}", node.kind(), node.is_named());
    }

    // see https://deepsource.com/blog/lightweight-linting
    println!("--- Matches ---");
    // let query_string = "(function_item name: (identifier) @fn)";
    // let query_string = "(attribute_item)";

    print_matches(RAWR_ANNOTATION_QUERY, &source_bytes, &tree);
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
