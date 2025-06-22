// SPDX-License-Identifier: Apache-2.0

//! Language matchers

#[cfg(feature = "lang-bash")]
pub mod bash;
#[cfg(feature = "lang-java")]
pub mod java;
pub mod rust;

use regex::Regex;
use serde::de::{self, Deserialize};
use serde::Deserializer;
use std::sync::OnceLock;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum SupportedLanguage {
    Rust,
    #[cfg(feature = "lang-bash")]
    Bash,
    #[cfg(feature = "lang-c")]
    C,
    #[cfg(feature = "lang-cpp")]
    Cpp,
}

/// Extract information with a named match in the Tree-Sitter grammar, or use a
/// new query to extract the node.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum MatchType {
    /// Reuse the entire match
    Match,
    /// Named child to extract as text.
    Named(String),
    /// The nth child of the grammar's given type.
    Kind(usize, String),
    /// Use a formatted string in place of a match. The only supported
    /// substitution is `${file_name}`, however this will likely be switched to
    /// a templating system.
    String(String),
    /// Tree-Sitter query and nth-match from which to extract text.
    SubQuery(usize, String),
}

/// Deserialize a string containing a MatchType variant.
impl<'de> Deserialize<'de> for MatchType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as whole string
        let s = String::deserialize(deserializer)?;

        // The Match type doesn't take any options. Return if it is specified.
        if s == "Match" {
            return Ok(MatchType::Match);
        }

        static VARIANT_REGEX: OnceLock<Regex> = OnceLock::new();
        let variant_regex = VARIANT_REGEX.get_or_init(|| {
            Regex::new(r"^(?P<variant>[[:alnum:]]+)(?P<bracketed_args>\((?P<args>.+?)\))?$")
                .unwrap()
        });

        let Some(matches) = variant_regex.captures(&s) else {
            return Err(de::Error::custom(
                // TODO Usage example
                "Invalid format. Expected a variant of MatchType.",
            ));
        };

        let Some(variant) = matches.name("variant") else {
            return Err(de::Error::unknown_variant(
                "",
                ["Match", "Named", "Kind", "String", "SubQuery"].as_ref(),
            ));
        };

        match variant.as_str() {
            "Match" => unreachable!("Match was handled early in the function"),
            "Named" => todo!(),    // String
            "String" => todo!(),   // String
            "Kind" => todo!(),     // usize, String
            "SubQuery" => todo!(), // usize, String
            _ => Err(de::Error::unknown_variant(
                "",
                &["Match", "Named", "Kind", "String", "SubQuery"],
            )),
        }
    }
}

pub trait LanguageMatcher {
    fn name() -> String;
    fn matchers() -> Vec<Matcher>;
}

/// Assumes that the interesting parts are actually named in the Tree-Sitter
/// grammar.
#[derive(Debug, Eq, PartialEq)]
pub struct Matcher {
    /// Friendly name for matches
    pub kind: String,
    /// Tree-Sitter query to match items of this type
    // DESIGN Convert to MatchType?
    pub query: String,
    /// Name of field containing item.
    pub identifier: MatchType,
    /// Name of field containing body contents.
    pub contents: MatchType,
    /// Human-readable information about this matcher.
    pub notes: Option<String>,
}

pub enum Query {
    TreeSitter(String),
    Constant,
}
