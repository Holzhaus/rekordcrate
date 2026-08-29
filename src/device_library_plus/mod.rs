// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Reader and writer for Rekordbox Device Library Plus (`exportLibrary.db`) databases.

pub mod defaults;
mod model;
mod schema;

use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use std::fs;
use std::path::Path;

use crate::Result;

pub use model::*;

const SCHEMA_SQL: &str = include_str!("schema.sql");

/// Default SQLCipher key used by Rekordbox Device Library Plus databases.
pub const SQLCIPHER_KEY: &str = "r8gddnr4k847830ar6cqzbkk0el6qytmb3trbbx805jm74vez64i5o8fnrqryqls";

fn configure_connection(conn: &mut SqliteConnection, key: &str) -> Result<()> {
    let escaped_key = key.replace('\'', "''");
    conn.batch_execute(&format!(
        "PRAGMA key = '{escaped_key}';\
         PRAGMA foreign_keys = ON;\
         PRAGMA journal_mode = WAL;\
         PRAGMA busy_timeout = 5000;\
         PRAGMA synchronous = NORMAL;"
    ))?;
    Ok(())
}

fn establish_connection(path: &Path, key: &str) -> Result<SqliteConnection> {
    let url = path.to_string_lossy();
    let mut conn = SqliteConnection::establish(&url)?;
    configure_connection(&mut conn, key)?;
    Ok(conn)
}

/// An opened Device Library Plus database.
pub struct Database {
    conn: SqliteConnection,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish_non_exhaustive()
    }
}

impl Database {
    /// Open an existing Device Library Plus database using the default Rekordbox SQLCipher key.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_key(path, SQLCIPHER_KEY)
    }

    /// Open an existing Device Library Plus database using an explicit SQLCipher key.
    pub fn open_with_key<P: AsRef<Path>>(path: P, key: &str) -> Result<Self> {
        Ok(Self {
            conn: establish_connection(path.as_ref(), key)?,
        })
    }

    /// Create a new Device Library Plus database with the mirrored schema only.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::create_with_key(path, SQLCIPHER_KEY)
    }

    /// Create a new Device Library Plus database with the mirrored schema only using an explicit key.
    pub fn create_with_key<P: AsRef<Path>>(path: P, key: &str) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut conn = establish_connection(path, key)?;
        conn.batch_execute(SCHEMA_SQL)?;
        Ok(Self { conn })
    }

    /// Create a new Device Library Plus database with the schema and default seed rows.
    pub fn create_with_defaults<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut db = Self::create(path)?;
        defaults::insert_initial_content(&mut db.conn)?;
        Ok(db)
    }

    /// Return a mutable reference to the underlying database connection.
    pub fn connection_mut(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }
}
