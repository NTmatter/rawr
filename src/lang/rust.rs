// SPDX-License-Identifier: Apache-2.0

use crate::lang::{LanguageMatcher, Matcher};

pub struct Rust {}

impl LanguageMatcher for Rust {
    fn name() -> String {
        "Rust".to_string()
    }

    fn matchers() -> Vec<Matcher> {
        use crate::lang::MatchType::*;
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
