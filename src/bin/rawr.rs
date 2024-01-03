#![allow(dead_code)]
use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read};
use tree_sitter;
use tree_sitter::{Parser, Query, QueryCursor};
use tree_sitter_rust;
use tree_sitter_traversal as tst;
use tree_sitter_traversal::Order;

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "At least one path required",
        ));
    }

    // Hard-code as Rust
    let mut default_codebase_default_lang = HashMap::new();
    default_codebase_default_lang.insert(String::from("/tests"), tree_sitter_rust::language());
    let default_codebase = Codebase {
        paths: default_codebase_default_lang,
    };

    let mut codebases = HashMap::<Option<String>, Codebase>::new();
    codebases.insert(None, default_codebase);

    // TODO Iterate over all paths in all codebases.

    // see: https://github.com/tree-sitter/tree-sitter/tree/master/lib/binding_rust
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_rust::language())
        .expect("Use Rust parser");
    let source_file = args.get(1).unwrap();
    let mut source_code = File::open(source_file).expect("Read test file");
    let file_length = source_code.metadata().expect("Get file metadata").len() as usize;
    let mut buf: Vec<u8> = Vec::with_capacity(file_length);
    source_code.read_to_end(&mut buf).expect("Read file");

    // Parse and walk tree
    let tree = parser.parse(&mut buf, None).expect("Parse test file");
    let cur = tst::traverse(tree.walk(), Order::Pre);
    for node in cur {
        println!("Node of type {}", node.kind());

        match node.kind() {
            "source_file" => println!("Source File"),
            "attribute_item" => println!("Attribute"),
            _ => {}
        };
    }

    // How do we filter this with a query?
    // see: https://tree-sitter.github.io/tree-sitter/using-parsers#query-syntax

    // see https://deepsource.com/blog/lightweight-linting
    println!("--- Matches ---");
    // let query_string = "(function_item name: (identifier) @fn)";
    // let query_string = "(attribute_item)";

    // Search for all rawr attributes.
    let query_string = "
    (attribute_item
      (attribute (identifier) @id) @att
      (#eq? @id \"rawr\"))";
    let query = Query::new(tree_sitter_rust::language(), &query_string).expect("Create query");
    let mut query_cursor = QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), buf.as_slice());
    matches.for_each(|m| {
        println!(
            "Capture {} has {} matches",
            m.pattern_index,
            m.captures.len()
        );
        m.captures.iter().for_each(|capture| {
            println!(
                "  {:?} {:?} {:?}",
                capture,
                capture.node.range(),
                String::from_utf8_lossy(&buf[capture.node.start_byte()..capture.node.end_byte()])
            )
        });
    });

    Ok(())
}

/// Common options for annotations
pub struct Rawr {
    /// Optional name of codebase that the upstream resides in.
    codebase: Option<String>,
    /// Path to original file, relative to codebase root
    path: Option<String>,
    /// Tree-Sitter query identifying the upstream implementation.
    /// Mutually exclusive to class/function.
    query: Option<String>,
    class: Option<String>,
    /// Function or class method.
    /// TODO How do we handle function overloading?
    function: Option<String>,
    /// Git revision (treeish)
    revision: Option<String>,
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
