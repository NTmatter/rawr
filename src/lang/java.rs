// SPDX-License-Identifier: Apache-2.0

//! Built-in Tree-Sitter queries for matching elements in Java files.
//!
//! Assumes that files are written in UTF-8 (the current default), with the
//! acknowledgement that the JDK supports a [wide range](https://docs.oracle.com/en/java/javase/22/intl/supported-encodings.html)
//! of encodings.
//!
//! See tree-sitter-java's [node-types.json](https://github.com/tree-sitter/tree-sitter-java/blob/master/src/node-types.json]

#![allow(unused)]

use crate::lang::{ALWAYS_MATCH, Dialect, LanguageDefinition};
use crate::upstream::matcher::Extractor::*;
use crate::upstream::matcher::{Extractor, Matcher};
use Extractor::*;
use anyhow::Context;
use gix::bstr::{BString, ByteSlice};
use std::path::Path;
use std::sync::Arc;
use tree_sitter::{Language, Query, QueryError};
use tree_sitter_language::LanguageFn;

pub struct Java {}

impl LanguageDefinition for Java {
    fn configuration(&self) -> anyhow::Result<Dialect, QueryError> {
        let java: Language = tree_sitter_java::LANGUAGE.into();
        Ok(Dialect {
            name: "Java".into(),
            language: tree_sitter_java::LANGUAGE.into(),
            should_match: Some(ALWAYS_MATCH),
            matchers: vec![
                Matcher {
                    kind: "whole-file",
                    query: Query::new(&java, "((program) @body)")?,
                    // Replace with file name for easier reference.
                    // Also avoids storing entire contents in database.
                    ident: Some(Constant("{filename}")),
                    notes: None,
                },
                Matcher {
                    kind: "class",
                    query: Query::new(&java, "((class_declaration) @body)")?,
                    ident: Some(Subquery(
                        Query::new(&java, "(class_declaration name: (identifier) @ident)")?,
                        Box::new(WholeMatch),
                    )),
                    notes: None,
                },
                // This doesn't work for identical methods in different classes. A
                // full in-file path is required.
                // PERF: Responsible for 30s of a 60s runtime on a single core.
                Matcher {
                    kind: "method",
                    query: Query::new(&java, "((method_declaration) @body)")?,
                    // Build ident from modifiers and arguments.
                    ident: Some(Subquery(
                        Query::new(
                            &java,
                            // Workaround for multiline idents. Less idiomatic, but still readable.
                            r#"
(method_declaration ((modifiers
    ([
        (annotation)
        (marker_annotation)
        "public"
        "protected"
        "private"
        "static"])* @mods)
    . type: (_) @ty
    . name: (identifier) @name
    . parameters: (formal_parameters "(" @ob ([(formal_parameter) (spread_parameter) ","]*) @params ")" @cb)
    (#strip! @params "\\s{2,}")
    ))
    "#,
                        )?,
                        Box::new(JoinNamed(" ".into())),
                    )),
                    notes: None,
                },
            ],
        })
    }
}

// Ensure that all matchers load
#[test]
fn validate_matchers() -> anyhow::Result<()> {
    let dialect = Java {}
        .configuration()
        .context("Should create successfully")?;
    for matcher in dialect.matchers {
        matcher
            .validate()
            .map_err(|errs| anyhow::Error::msg(errs.join("\n")))
            .context("Matcher validation")?;
    }

    Ok(())
}
