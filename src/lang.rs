// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum SupportedLanguage {
    Rust,
    Bash,
    C,
    Cpp,
}

/// Extract information with a named match in the Tree-Sitter grammar, or use a
/// new query to extract the node.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum MatchType {
    /// Reuse the entire match
    Match,
    /// A named type from the grammar
    Kind(usize, String),
    /// Nth Named child to extract as text.
    Named(String),
    /// Tree-Sitter query and nth-match from which to extract text.
    Query(usize, String),
    /// Use a fixed string in place of a match.
    Format(String),
}

/// Assumes that the interesting parts are actually named in the Tree-Sitter
/// grammar.
#[derive(Debug, Eq, PartialEq)]
pub struct Matcher {
    /// Friendly name for matches
    pub kind: String,
    /// Tree-Sitter query to match items of this type
    // TODO Convert over to MatchType?
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

/// Build list of items that should be matched for Rust.
pub fn matchers_rust() -> Vec<Matcher> {
    use MatchType::*;
    vec![
        Matcher {
            kind: "file".to_string(),
            query: "((source_file) @f)".to_string(),
            identifier: Format("${file_name}".to_string()),
            contents: Match,
            notes: Some("Exact contents of entire file".to_string()),
        },
        Matcher {
            kind: "function".to_string(),
            query: "((function_item) @fi)".to_string(),
            identifier: Named("name".to_string()),
            contents: Match,
            notes: Some(
                "Function, including visibility, name, parameters, return type, and body "
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

/// Build list of items that should be matched for Bash
pub fn matchers_bash() -> Vec<Matcher> {
    use MatchType::*;
    vec![
        Matcher {
            kind: "Variable".to_string(),
            query: "(variable_assignment)".to_string(),
            identifier: Named("name".to_string()),
            contents: Named("value".to_string()),
            notes: None,
        },
        Matcher {
            kind: "Function".to_string(),
            query: "(function_definition)".to_string(),
            identifier: Named("name".to_string()),
            contents: Named("body".to_string()),
            notes: None,
        },
    ]
}
