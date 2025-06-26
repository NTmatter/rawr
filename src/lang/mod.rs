// SPDX-License-Identifier: Apache-2.0

//! Language matchers

use crate::upstream::matcher::Matcher;
use gix::bstr::BString;
use tree_sitter::{Language, QueryError};

#[cfg(feature = "lang-java")]
pub mod java;
// pub mod rust;

/// Outputs a language configuration
pub trait LanguageDefinition {
    /// Produce a `LanguageConfig` with relevant name and matchers.
    fn configuration(&self) -> anyhow::Result<Dialect, QueryError>;
}

/// Function indicating that `Matcher`s should process the given path `BString`.
pub type ShouldMatchFn = fn(&BString) -> bool;

/// Always match the provided path.
pub const ALWAYS_MATCH: ShouldMatchFn = |_path: &BString| -> bool { true };

/// Describes a language and matchers for its contents.
pub struct Dialect {
    pub name: String,

    // DESIGN Would it be better to keep the language function?
    /// Tree-Sitter parser language.
    pub language: Language,

    /// Optional function for vetoing path matches. Returns true if path should
    /// be matched, or false if it should be ignored. When None, the Dialect
    /// does not have an opinion, and will trust the upstream filters set in the
    /// `Upstream`'s `SourceRoot` configuration.
    pub should_match: Option<ShouldMatchFn>,
    pub matchers: Vec<Matcher>,
}
