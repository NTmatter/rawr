// SPDX-License-Identifier: Apache-2.0

//! Learn to use gitoxide. Refer to the Gix examples as a start.
//! Try to use Gix to read the tree at a particular revision and parse a file with TreeSitter.
//! Ultimately, I'll need to look for changes to tracked items along a series of revisions.
use gix::{self, bstr::BString, object::Kind, traverse::tree::Recorder};
use tree_sitter::Parser;

const TREEISH: &str = "main";
fn main() -> anyhow::Result<()> {
    // Find repo for current work copy
    let repo = gix::discover(".").expect("Discover repository for current directory");

    // Parse revision, find object and get tree
    let rev = repo.rev_parse_single(TREEISH)?;
    println!("Got revision: {}", rev);

    // What is "peel to tree" exactly?
    let tree = rev.object()?.peel_to_tree()?;

    // Build list of entries in the tree.
    let mut recorder = Recorder::default();

    // Visit entire tree and collect results into recorder.
    tree.traverse().breadthfirst(&mut recorder)?;

    // The original only prints directories.
    let entries = recorder
        .records
        .into_iter()
        //.filter(|entry| entry.mode.is_no_tree())
        //.filter(|entry| entry.mode.is_tree())
        .collect::<Vec<_>>();

    println!("Found {} entries.", entries.len());

    for entry in entries {
        println!(
            "{:06o} {:4} {}    {}",
            *entry.mode,
            entry.mode.as_str(),
            entry.oid,
            entry.filepath
        );
    }

    // Incrementally traverse down to a file
    let path = BString::from("tests");
    let entity_ref = tree.find_entry(path).unwrap();
    let tests_dir = entity_ref.object()?.peel_to_tree()?;
    println!("Tests directory: {:?}", tests_dir);

    let path = BString::from("upstream.sh");
    let entity_ref = tests_dir.find_entry(path).unwrap();
    let upstream_sh = entity_ref.object()?.peel_to_kind(Kind::Blob)?;
    // entity_ref.object().unwrap().into_blob().take_data();
    println!("Upstream Script: {:?}", upstream_sh);

    // An object can be directly retrieved.
    let mut buf = Vec::<u8>::new();
    let path = std::path::Path::new("tests/upstream.sh");
    let tests_upstream = tree.lookup_entry_by_path(path, &mut buf)?.unwrap();
    println!("Upstream Script: {:?}", tests_upstream);

    // We can get the raw bytes,
    let binding = tests_upstream.object()?.into_blob().take_data();
    let file_data = binding.as_slice();

    // The raw bytes can be passed directly to Tree-Sitter.
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_bash::language())?;
    let _tree = parser.parse(file_data, None).unwrap();

    println!("Successfully parsed tree");

    Ok(())
}
