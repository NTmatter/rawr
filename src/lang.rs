// SPDX-License-Identifier: Apache-2.0

//! Language matchers

use regex::Regex;
use serde::de;
use serde::de::Deserialize;
use serde::Deserializer;
use std::sync::OnceLock;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum SupportedLanguage {
    Rust,
    #[cfg(feature = "lang-bash")]
    Bash,
    // #[cfg(feature = "lang-c")]
    // C,
    // #[cfg(feature = "lang-cpp")]
    // Cpp,
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
            _ => {
                return Err(de::Error::unknown_variant(
                    "",
                    &["Match", "Named", "Kind", "String", "SubQuery"],
                ));
            }
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

pub struct Rust {}
impl LanguageMatcher for Rust {
    fn name() -> String {
        "Rust".to_string()
    }

    fn matchers() -> Vec<Matcher> {
        use MatchType::*;
        vec![
            Matcher {
                kind: "function".to_string(),
                query: "((function_item) @fi)".to_string(),
                identifier: Named("name".to_string()),
                contents: Match,
                notes: Some(
                    "Function, including visibility, name, parameters, return type, and body"
                        .to_string(),
                ),
            },
            Matcher {
                kind: "struct".to_string(),
                query: "((struct_item) @si)".to_string(),
                identifier: Named("name".to_string()),
                contents: Match,
                notes: None,
            },
            Matcher {
                kind: "const".to_string(),
                query: "((const_item) @ci)".to_string(),
                identifier: Named("name".to_string()),
                // Should be the entire match, or possibly just the type and value.
                contents: Named("value".to_string()),
                notes: None,
            },
            Matcher {
                kind: "enum".to_string(),
                query: "((enum_item) @ei)".to_string(),
                identifier: Named("name".to_string()),
                contents: Named("body".to_string()),
                notes: None,
            },
        ]
    }
}

pub struct Bash {}
impl LanguageMatcher for Bash {
    fn name() -> String {
        "Bash".to_string()
    }

    fn matchers() -> Vec<Matcher> {
        use MatchType::*;
        vec![
            Matcher {
                kind: "variable".to_string(),
                query: "((variable_assignment) @va)".to_string(),
                identifier: Named("name".to_string()),
                contents: Named("value".to_string()),
                notes: None,
            },
            Matcher {
                kind: "function".to_string(),
                query: "((function_definition) @fd)".to_string(),
                identifier: Named("name".to_string()),
                contents: Named("body".to_string()),
                notes: None,
            },
        ]
    }
}
