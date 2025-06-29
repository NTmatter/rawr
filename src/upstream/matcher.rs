// SPDX-License-Identifier: Apache-2.0

//! Functionality for matching upstream items.

use anyhow::{Context, bail};
use gix::bstr::ByteSlice;
use sha2::digest::{Output, Update};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor, QueryMatch};

/// Match a class of items in an upstream codebase
pub struct Matcher {
    /// Unique name for matched kind
    pub kind: &'static str,

    /// Tree-Sitter query for matching the full item body. This should have a single
    /// top-level match, which will be used as a root for the ident extractor.
    pub query: Query,

    // TODO Ident post-processor?
    /// Strategy for extracting items ident
    pub ident: Option<Extractor>,

    /// Optional human-friendly notes about this matcher
    pub notes: Option<&'static str>,
}

impl Matcher {
    pub fn validate(&self) -> anyhow::Result<(), Vec<&'static str>> {
        let mut issues = Vec::new();

        if self.query.pattern_count() != 1 {
            issues.push("Body Query should have a single pattern");
        }
        if !self.query.is_pattern_rooted(0) {
            issues.push("Body Query must have a single root node");
        }

        if self.query.capture_names().len() != 1 {
            issues.push("Body Query must have a single capture named '@outer'")
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

/// Strategy for extracting data from a larger match.
pub enum Extractor {
    /// Convert the entire match to a string using the outer bounds of all
    /// sub-matches.
    WholeMatch,

    /// Convert all matches to strings, normalize spaces, and join them with the
    /// given delimiter.
    JoinNamed(&'static str),

    /// Supply a constant, filtered through a templating replacement.
    Constant(&'static str),

    /// Extract from the named field, specified in the grammar's node type.
    NamedMatch(&'static str, Box<Extractor>),

    /// Use the given match index
    NumberedMatch(usize, Box<Extractor>),

    /// Execute an additional query to identify the content, and extract it with
    /// the given extractor.
    Subquery(Query, Box<Extractor>),
}

pub enum ExtractionError {
    /// No matches were present in input
    NoMatches,

    // TODO Pre-check against Query
    /// The query did not contain a match
    NamedMatchNotFound,

    // TODO Pre-check against Query
    /// The numbered match was not found in the list
    NumberedMatchNotFound,

    /// Matched range was outside range of data slice
    MatchBytesOutOfBounds,
}

impl Extractor {
    /// Returns the data covered by a Matcher using the provided matching strategy.
    /// Use the `checksum` function if the checksum is the only required
    pub fn extract<'data>(&self, outer: &QueryMatch, data: &'data [u8]) -> anyhow::Result<Vec<u8>> {
        match self {
            Extractor::WholeMatch => Self::extract_whole_match(outer, data).map(Vec::from),
            Extractor::JoinNamed(delimiter) => Self::extract_joined_match(outer, delimiter, data),
            // DESIGN How to pass down the environment for substitution? eg, Filename/Path
            Extractor::Constant(s) => Ok(s.as_bytes().to_vec()),
            Extractor::NamedMatch(_, _) => todo!(),
            Extractor::NumberedMatch(_, _) => todo!(),
            Extractor::Subquery(subquery, extractor) => {
                Self::extract_subquery(outer, subquery, extractor, data)
            }
        }
    }

    /// Checksum
    pub fn checksum<'data, D>(
        &self,
        outer: &QueryMatch,
        data: &'data [u8],
    ) -> anyhow::Result<Output<D>>
    where
        D: Digest,
    {
        match self {
            Extractor::WholeMatch => Self::checksum_whole_match::<D>(outer, data),
            Extractor::JoinNamed(delimiter) => {
                Self::checksum_joined_match::<D>(outer, delimiter, data)
            }
            Extractor::Constant(_) => todo!(),
            Extractor::NamedMatch(_, _) => todo!(),
            Extractor::NumberedMatch(_, _) => todo!(),
            Extractor::Subquery(_, _) => todo!(),
        }
    }

    pub fn extract_whole_match<'data>(
        outer: &QueryMatch,
        data: &'data [u8],
    ) -> anyhow::Result<&'data [u8]> {
        // Find outer range of captures, which might be out of order
        let start_byte = outer.captures.iter().fold(usize::MAX, |acc, cap| {
            usize::min(acc, cap.node.start_byte())
        });
        let end_byte = outer
            .captures
            .iter()
            .fold(usize::MIN, |acc, cap| usize::max(acc, cap.node.end_byte()));

        data.get(start_byte..end_byte)
            .context("Extracted data range must lie inside source data slice")
    }

    pub fn checksum_whole_match<'data, D>(
        outer: &QueryMatch,
        data: &'data [u8],
    ) -> anyhow::Result<Output<D>>
    where
        D: Digest,
    {
        let body = Self::extract_whole_match(outer, data)?;
        let body_checksum = D::digest(&body);

        Ok(body_checksum)
    }

    pub fn extract_joined_match<'data>(
        outer: &QueryMatch,
        delimiter: &str,
        data: &'data [u8],
    ) -> anyhow::Result<Vec<u8>> {
        if outer.captures.is_empty() {
            bail!("No captures to match");
        }

        let ranges = outer
            .captures
            .iter()
            .map(|cap| {
                data.get(cap.node.byte_range())
                    .context("Extracted data range must lie inside source data slice")
            })
            .collect::<Result<Vec<&[u8]>, anyhow::Error>>()?
            .join(delimiter.as_bytes());

        Ok(ranges)
    }

    pub fn checksum_joined_match<'data, D>(
        outer: &QueryMatch,
        delimiter: &str,
        data: &'data [u8],
    ) -> anyhow::Result<Output<D>>
    where
        D: Digest,
    {
        if outer.captures.is_empty() {
            bail!("No matching captures found");
        }

        // Incrementally build hash from segments, injecting delimiter between items
        // to avoid copying matches around.
        let mut hasher = D::new();
        for (idx, cap) in outer.captures.iter().enumerate() {
            if idx > 0 {
                Digest::update(&mut hasher, delimiter.as_bytes());
            }
            let data = data
                .get(cap.node.byte_range())
                .context("Extracted data range must lie inside source data slice")?;
            Digest::update(&mut hasher, data);
        }

        Ok(hasher.finalize())
    }

    // DESIGN Should subquery only act on the first node in the match?
    pub fn extract_subquery(
        outer: &QueryMatch,
        subquery: &Query,
        extractor: &Extractor,
        data: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        let root_node = outer
            .captures
            .first()
            .map(|capture| capture.node)
            .context("No captures in outer match")?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(subquery, root_node, data);
        let Some(matched) = matches.next() else {
            let ctx = Self::extract_whole_match(outer, data)
                .map(|bytes| bytes.to_str_lossy())
                .context("Failed to extract match")?;
            bail!("No matches found by subquery");
        };

        Self::extract(extractor, matched, data)
    }
}
