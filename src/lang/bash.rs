// SPDX-License-Identifier: Apache-2.0

use crate::lang::{LanguageMatcher, Matcher};

pub struct Bash {}

impl LanguageMatcher for Bash {
    fn name() -> String {
        "Bash".to_string()
    }

    fn matchers() -> Vec<Matcher> {
        use crate::lang::MatchType::*;
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
