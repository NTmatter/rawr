// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use clap::Args;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use tracing::debug;
use url::Url;

#[derive(Args, Clone, Debug)]
pub struct DatabaseArgs {
    /// Connection URL for database.
    #[arg(long, default_value = "rawr.sqlite")]
    pub database: PathBuf,
}

pub fn connect_rw(args: DatabaseArgs) -> anyhow::Result<Connection> {
    let DatabaseArgs { database: db_path } = args;
    dbg!(&db_path);
    debug!(path = %db_path.display(), "Open database");

    // Default flags as per https://docs.rs/rusqlite/latest/rusqlite/struct.OpenFlags.html
    // SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI | SQLITE_OPEN_NO_MUTEX
    // Disable OPEN_URI for now.
    let open_flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;

    // let conn =
    //     Connection::open_with_flags(&db_path, open_flags).context("Open or create database")?;
    let conn = Connection::open(&db_path).context("Open or create database")?;

    // Ensure that foreign key support is enabled, as it may be required later on.
    conn.pragma_update(None, "foreign_keys", "ON")
        .context("Enable foreign key support")?;

    // Execute setup script on each connection.
    conn.execute_batch(include_str!("rawr.sql"))
        .context("Create tables if needed")?;

    Ok(conn)
}

pub fn connect_ro(args: DatabaseArgs) -> anyhow::Result<Connection> {
    let DatabaseArgs { database: db_path } = args;
    debug!(path = %db_path.display(), "Open database");

    // Default flags as per https://docs.rs/rusqlite/latest/rusqlite/struct.OpenFlags.html
    // SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI | SQLITE_OPEN_NO_MUTEX
    // Disable OPEN_URI for now.
    let open_flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;

    let conn =
        Connection::open_with_flags(&db_path, open_flags).context("Open or create database")?;

    // Ensure that foreign key support is enabled, as it may be required later on.
    conn.pragma_update(None, "foreign_keys", "ON")
        .context("Enable foreign key support")?;

    Ok(conn)
}
