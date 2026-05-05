// SPDX-License-Identifier: Apache-2.0

//! Use sem to parse AST entities from a source file
use anyhow::bail;
use sem_core::parser::plugins::create_default_registry;
use typed_path::Utf8UnixPathBuf;

/// The entity to be greeted. Unlikely to change.
const GREETED: &str = "world";

fn main() -> anyhow::Result<()> {
    println!("Hello, {GREETED}!");
    let source = include_str!("hello_sem.rs");
    do_parse(Utf8UnixPathBuf::from("hello_sem.rs"), source)?;

    let source = include_str!("../README.md");
    do_parse(Utf8UnixPathBuf::from("../README.md"), source)?;

    Ok(())
}

/// Parse a file and output discovered entities.
fn do_parse(relative_path: Utf8UnixPathBuf, contents: impl AsRef<str>) -> anyhow::Result<()> {
    println!("Parsing file: {}", relative_path.as_str());

    if !relative_path.is_relative() {
        println!("Git path should be relative. Ignoring.")
    }

    let registry = create_default_registry();
    let Some(plugin) = registry.get_plugin_with_content(relative_path.as_str(), contents.as_ref())
    else {
        bail!(
            "Failed to parse plugin from content at path: {}",
            relative_path.as_str()
        );
    };

    if plugin.id() == "fallback" {
        bail!(
            "Fallback plugin detected. Ignoring content at path: {}",
            relative_path.as_str()
        );
    }

    let entities = plugin.extract_entities(contents.as_ref(), relative_path.as_str());
    println!("Found {} entities", entities.len());

    for entity in entities {
        let entity_json = serde_json::to_string_pretty(&entity)
            .unwrap_or("(Entity JSON serialization failed)".to_string());
        println!("{entity_json}");
    }

    Ok(())
}
