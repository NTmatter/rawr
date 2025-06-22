// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use crate::lang::LanguageConfig;
use crate::lang::java::Java;
use crate::upstream::matched::UpstreamMatch;
use std::path::PathBuf;
use url::Url;
use walkdir::WalkDir;

pub mod matched;
pub mod matcher;

pub type UpstreamId = String;

pub struct Upstream {
    /// Unique ID for upstream
    pub id: UpstreamId,

    /// Human-friendly name of upstream
    pub name: String,

    /// Relative path from the current directory to upstream root
    pub path: PathBuf,

    /// Link to the repository for display
    pub repo: Option<Url>,

    /// List of source directories within this upstream
    pub roots: Vec<SourceRoot>,

    /// Optional human-friendly notes for this upstream
    pub notes: Option<String>,
}

impl Upstream {
    /// Collect all matched items for the given upstream configuration
    pub async fn scan(&self) -> anyhow::Result<Vec<UpstreamMatch>> {
        let mut matched_items = Vec::new();
        for root in &self.roots {
            let mut matches = root.scan(self).await?;
            matched_items.append(&mut matches);
        }
        Ok(matched_items)
    }
}

pub struct SourceRoot {
    /// Relative path from Upstream to Source
    pub path: PathBuf,

    /// Language used within this source root
    pub lang: Box<dyn LanguageConfig>,

    /// Optional human-friendly notes for this language
    pub notes: Option<String>,
    // TODO Includes and excludes
}

impl SourceRoot {
    pub async fn scan(&self, upstream: &Upstream) -> anyhow::Result<Vec<UpstreamMatch>> {
        let mut matched_items = Vec::new();

        let root = upstream.path.join(&self.path);
        println!("Scanning {:?}", &root);

        WalkDir::new(root)
            .sort_by_file_name()
            .into_iter()
            .for_each(|entry| println!("{:?}", entry.unwrap().path()));

        let files = Vec::<PathBuf>::new();
        // Iterate over files
        for file in files {
            for matcher in self.lang.matchers()? {
                // TODO Apply matcher and add to results
            }
        }

        Ok(matched_items)
    }
}

#[tokio::test]
async fn test_scan() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    println!("{:?}", cwd);

    let upstream = Upstream {
        id: "test".to_string(),
        name: "Test".to_string(),
        path: PathBuf::from("./"),
        repo: None,
        roots: vec![SourceRoot {
            path: PathBuf::from("tests"),
            lang: Box::new(Java {}),
            notes: None,
        }],
        notes: None,
    };

    let results = upstream.scan().await?;
    println!("Found {} results", results.len());
    dbg!(&results);

    Ok(())
}
