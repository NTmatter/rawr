// SPDX-License-Identifier: Apache-2.0

//! Built-in Tree-Sitter queries for matching elements in Java files.
//!
//! Assumes that files are written in UTF-8 (the current default), with the
//! acknowledgement that the JDK supports a [wide range](https://docs.oracle.com/en/java/javase/22/intl/supported-encodings.html)
//! of encodings.
//!
//! See tree-sitter-java's [node-types.json](https://github.com/tree-sitter/tree-sitter-java/blob/master/src/node-types.json]

#![allow(unused)]

use anyhow::Context;
use tree_sitter::{Language, Query, QueryError};

#[test]
fn validate_queries() -> anyhow::Result<()> {
    let queries = queries().context("All queries must be valid")?;

    Ok(())
}

pub fn queries() -> Result<Vec<Query>, QueryError> {
    let java: Language = tree_sitter_java::LANGUAGE.into();

    let queries = [WHOLE_FILE, CLASS_DECLARATION]
        .into_iter()
        .map(|query| Query::new(&java, query))
        .collect::<Result<Vec<Query>, QueryError>>()?;

    Ok(queries)
}

const WHOLE_FILE: &str = "(program)";
const CLASS_DECLARATION: &str = "(class_declaration
  name: (identifier) @name
  body: (class_body) @contents)";

#[test]
fn test_java_parse() -> anyhow::Result<()> {
    Ok(())
}
