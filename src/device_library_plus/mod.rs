// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Read-only access to Rekordbox Device Library Plus (`exportLibrary.db`) databases.

mod model;
mod schema;

use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use std::path::Path;

use crate::Result;

pub use model::*;

/// Default SQLCipher key used by Rekordbox Device Library Plus databases.
pub const SQLCIPHER_KEY: &str = "r8gddnr4k847830ar6cqzbkk0el6qytmb3trbbx805jm74vez64i5o8fnrqryqls";

fn establish_connection(path: &Path) -> Result<SqliteConnection> {
    let url = path.to_string_lossy();
    let mut conn = SqliteConnection::establish(&url)?;
    // SQLCipher requires the key to be configured before accessing the database.
    conn.batch_execute(&format!("PRAGMA key = '{SQLCIPHER_KEY}';"))?;
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
    /// Open an existing Device Library Plus database using Rekordbox's SQLCipher key.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            conn: establish_connection(path.as_ref())?,
        })
    }

    /// Return a mutable reference to the underlying database connection.
    pub fn connection_mut(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }
}
