// SPDX-License-Identifier: Apache-2.0

//! Language matchers

use crate::upstream::matcher::Matcher;
use std::path::Path;
use tree_sitter::{Language, QueryError};

#[cfg(feature = "lang-java")]
pub mod java;
// pub mod rust;

// DESIGN Can this be read from a TOML?
/// Central
pub trait LanguageConfig {
    /// Name for matcher
    fn name(&self) -> String;

    /// Output underlying Tree Sitter language.
    fn language(&self) -> Language;

    /// Determine if file should be parsed by this matcher, typically based on
    /// file extension.
    ///
    /// DESIGN This should be covered by the Upstream roots' includes/excludes
    fn should_parse(&self, path: &Path) -> bool;

    /// Generate a list of recognized items
    fn matchers(&self) -> anyhow::Result<Vec<Matcher>, QueryError>;
}
