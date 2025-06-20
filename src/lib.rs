// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use clap::Args;
use rusqlite::Connection;
use std::path::PathBuf;
use url::Url;

pub mod downstream;
pub mod lang;
pub mod upstream;

#[derive(Args, Clone, Debug)]
pub struct DatabaseArgs {
    /// Connection URL for database.
    #[arg(long, default_value = "sqlite://./rawr.sqlite")]
    pub database: Url,
}

pub fn db_connection(db_path: PathBuf) -> anyhow::Result<Connection> {
    // TODO Disable Open with URI flag with `Connection::open_with_flags`
    let conn = Connection::open(db_path).context("Open or create database")?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .context("Enable foreign key support")?;

    conn.execute_batch(include_str!("rawr.sql"))
        .context("Create tables if needed")?;

    Ok(conn)
}

/// Core information about an upstream codebase.
#[derive(Debug, Eq, PartialEq)]
pub struct Codebase {
    pub name: String,
    pub relative_path: String,
    pub notes: Option<String>,
}

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
