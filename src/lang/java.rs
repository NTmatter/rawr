// SPDX-License-Identifier: Apache-2.0

//! Built-in Tree-Sitter queries for matching elements in Java files.
//!
//! Assumes that files are written in UTF-8 (the current default), with the
//! acknowledgement that the JDK supports a [wide range](https://docs.oracle.com/en/java/javase/22/intl/supported-encodings.html)
//! of encodings.
//!
//! See tree-sitter-java's [node-types.json](https://github.com/tree-sitter/tree-sitter-java/blob/master/src/node-types.json]

#![allow(unused)]

use crate::lang::LanguageConfig;
use crate::upstream::matcher::Extractor::*;
use crate::upstream::matcher::{Extractor, Matcher};
use Extractor::*;
use anyhow::Context;
use gix::bstr::{BString, ByteSlice};
use std::path::Path;
use tree_sitter::{Language, Query, QueryError};

pub struct Java {}
impl LanguageConfig for Java {
    fn name(&self) -> String {
        "Java".to_string()
    }

    fn language(&self) -> Language {
        tree_sitter_java::LANGUAGE.into()
    }

    fn should_parse(&self, path: &BString) -> bool {
        path.to_string().ends_with(".java")
    }

    fn matchers(&self) -> anyhow::Result<Vec<Matcher>, QueryError> {
        let java: Language = self.language();
        let matchers = vec![
            Matcher {
                kind: "whole-file",
                query: Query::new(&java, "((program) @outer)")?,
                ident: Some(Constant("{filename}")),
                notes: None,
            },
            Matcher {
                kind: "class",
                query: Query::new(&java, "((class_declaration) @outer)")?,
                ident: None,
                notes: None,
            },
            // This doesn't work for identical methods in different classes. A
            // full in-file path is required.
            // PERF: Responsible for 30s of a 60s runtime on a single core.
            Matcher {
                kind: "method",
                query: Query::new(&java, "((method_declaration) @outer)")?,
                // Build ident from modifiers and arguments.
                ident: Some(Subquery(
                    Query::new(
                        &java,
                        "((modifiers)* @mods
                      . type: (_) @ty
                      . name: (identifier) @name
                      . parameters: (formal_parameters) @params)",
                    )?,
                    Box::new(WholeMatch),
                )),
                notes: None,
            },
        ];

        Ok(matchers)
    }
}

// Ensure that all matchers load
#[test]
fn validate_matchers() {
    Java {}.matchers().unwrap();
}
