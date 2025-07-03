// SPDX-License-Identifier: Apache-2.0
extern crate rawr_macro;

pub mod compare;
pub mod db;
pub mod downstream;
pub mod lang;
pub mod upstream;

// Re-export macros for library users.
pub use rawr_macro::{Rawr, rawr, rawr_fn};
use tree_sitter::{Point, QueryMatch, Range};

/// Represent the type of change to an item in a given revision
#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Change {
    /// Item has been created
    Add,
    /// Item has been deleted
    Delete,
    /// Item contents have changed
    Modify,
    /// Whitespace changes only
    Whitespace,
}

pub(crate) fn matched_outer_range(matched: &QueryMatch) -> Range {
    // Build outer range for match.
    let mut range = Range {
        start_byte: usize::MAX,
        end_byte: usize::MIN,
        start_point: Point::default(),
        end_point: Point::default(),
    };
    for cap in matched.captures {
        // Find the lowest start point
        if cap.node.start_byte() <= range.start_byte {
            range.start_byte = cap.node.start_byte();
            range.start_point = cap.node.start_position();
        }
        // Find the highest endpoint
        if cap.node.end_byte() >= range.end_byte {
            range.end_byte = cap.node.end_byte();
            range.end_point = cap.node.end_position();
        }
    }
    range
}
