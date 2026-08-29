// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Default seed rows for Rekordbox Device Library Plus databases.

#![allow(missing_docs)]

use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use diesel::QueryResult;

use super::{
    BinaryFlag, Category, CategoryId, Color, ColorId, Key, KeyId, MenuItem, MenuItemId, Property,
    Sort, SortId, TableRecord,
};

/// Rekordbox's current Device Library Plus schema version string.
pub const DEFAULT_DB_VERSION: &str = "10000";

/// The `myTagMasterDBID` value seen in both minimal and full sample exports.
pub const DEFAULT_MY_TAG_MASTER_DBID: i64 = 2_402_936_626;

/// Insert the static lookup rows present in a freshly exported blank database.
///
/// This mirrors `data/incremental/000/exportLibrary.db` and intentionally does
/// not insert any `key` rows, because Rekordbox leaves that table empty until
/// keyed content is written.
pub fn insert_initial_content(conn: &mut SqliteConnection) -> QueryResult<()> {
    conn.transaction(|conn| {
        for color in default_colors() {
            color.insert(conn)?;
        }
        for menu_item in default_menu_items() {
            menu_item.insert(conn)?;
        }
        for category in default_categories() {
            category.insert(conn)?;
        }
        for sort in default_sorts() {
            sort.insert(conn)?;
        }
        Ok(())
    })
}

/// Insert the canonical Rekordbox key lookup rows used by populated exports.
pub fn insert_key_lookup_rows(conn: &mut SqliteConnection) -> QueryResult<()> {
    conn.transaction(|conn| {
        for key in default_keys() {
            key.insert(conn)?;
        }
        Ok(())
    })
}

/// Build the default `property` row used by a newly exported blank database.
pub fn default_property(created_date: impl Into<String>) -> Property {
    Property::new(
        "",
        DEFAULT_DB_VERSION,
        0,
        created_date,
        DEFAULT_MY_TAG_MASTER_DBID,
    )
}

fn default_keys() -> Vec<Key> {
    [
        (1, "Dm"),
        (2, "Abm"),
        (3, "Cm"),
        (4, "Bbm"),
        (5, "Fm"),
        (6, "Dbm"),
        (7, "Bm"),
        (8, "Gm"),
        (9, "F#m"),
        (10, "Em"),
        (11, "Am"),
        (12, "D"),
        (13, "Ebm"),
        (14, "Bb"),
        (15, "G"),
        (16, "Db"),
        (17, "A"),
        (18, "B"),
        (19, "C"),
        (20, "F"),
        (21, "Ab"),
        (22, "Eb"),
        (23, "F#"),
        (24, "E"),
        (25, "5A"),
        (26, "6A"),
        (27, "3B"),
        (28, "8B"),
        (29, "9A"),
        (30, "7B"),
        (31, "9B"),
        (32, "4A"),
        (33, "3A"),
        (34, "6B"),
        (35, "10A"),
        (36, "8A"),
        (37, "7A"),
        (38, "11A"),
        (39, "12A"),
        (40, "C♯m"),
        (41, "F♯m"),
        (42, "11B"),
        (43, "D♭"),
        (44, "B♭m"),
    ]
    .into_iter()
    .map(|(id, name)| Key::new(KeyId(id), name))
    .collect()
}

fn default_colors() -> Vec<Color> {
    [
        (1, "Pink"),
        (2, "Red"),
        (3, "Orange"),
        (4, "Yellow"),
        (5, "Green"),
        (6, "Aqua"),
        (7, "Blue"),
        (8, "Purple"),
    ]
    .into_iter()
    .map(|(id, name)| Color::new(ColorId(id), name))
    .collect()
}

fn default_menu_items() -> Vec<MenuItem> {
    [
        (1, 128, "￺GENRE￻"),
        (2, 129, "￺ARTIST￻"),
        (3, 130, "￺ALBUM￻"),
        (4, 131, "￺TRACK￻"),
        (5, 133, "￺BPM￻"),
        (6, 134, "￺RATING￻"),
        (7, 135, "￺YEAR￻"),
        (8, 136, "￺REMIXER￻"),
        (9, 137, "￺LABEL￻"),
        (10, 138, "￺ORIGINAL ARTIST￻"),
        (11, 139, "￺KEY￻"),
        (12, 141, "￺CUE￻"),
        (13, 142, "￺COLOR￻"),
        (14, 146, "￺TIME￻"),
        (15, 147, "￺BITRATE￻"),
        (16, 148, "￺FILE NAME￻"),
        (17, 132, "￺PLAYLIST￻"),
        (18, 152, "￺HOT CUE BANK￻"),
        (19, 149, "￺HISTORY￻"),
        (20, 145, "￺SEARCH￻"),
        (21, 150, "￺COMMENTS￻"),
        (22, 140, "￺DATE ADDED￻"),
        (23, 151, "￺DJ PLAY COUNT￻"),
        (24, 144, "￺FOLDER￻"),
        (25, 161, "￺DEFAULT￻"),
        (26, 162, "￺ALPHABET￻"),
        (27, 170, "￺MATCHING￻"),
    ]
    .into_iter()
    .map(|(id, kind, name)| MenuItem::new(MenuItemId(id), kind, name))
    .collect()
}

fn default_categories() -> Vec<Category> {
    [
        (1, 1, 0, 0),
        (2, 2, 1, 1),
        (3, 3, 2, 1),
        (4, 4, 3, 1),
        (5, 17, 5, 1),
        (6, 5, 0, 0),
        (7, 6, 0, 0),
        (8, 7, 0, 0),
        (9, 8, 0, 0),
        (10, 9, 0, 0),
        (11, 10, 0, 0),
        (12, 11, 4, 1),
        (15, 13, 0, 0),
        (17, 24, 9, 1),
        (18, 20, 7, 1),
        (19, 14, 0, 0),
        (20, 15, 0, 0),
        (21, 16, 0, 0),
        (22, 19, 6, 1),
        (23, 18, 0, 0),
        (26, 27, 8, 1),
        (27, 22, 10, 1),
    ]
    .into_iter()
    .map(|(category_id, menu_item_id, sequence_no, is_visible)| {
        Category::new(
            CategoryId(category_id),
            MenuItemId(menu_item_id),
            sequence_no,
            BinaryFlag::from(is_visible),
        )
    })
    .collect()
}

fn default_sorts() -> Vec<Sort> {
    [
        (0, 25, 1, 1, 0),
        (1, 26, 2, 1, 0),
        (2, 2, 3, 1, 0),
        (3, 3, 4, 1, 0),
        (4, 5, 5, 1, 0),
        (5, 6, 6, 1, 0),
        (6, 1, 0, 0, 0),
        (7, 21, 0, 0, 0),
        (8, 14, 0, 0, 0),
        (9, 8, 0, 0, 0),
        (10, 9, 0, 0, 0),
        (11, 10, 0, 0, 0),
        (12, 11, 7, 1, 0),
        (13, 15, 0, 0, 0),
        (15, 13, 0, 0, 0),
        (16, 23, 0, 0, 0),
        (17, 22, 0, 0, 0),
    ]
    .into_iter()
    .map(
        |(sort_id, menu_item_id, sequence_no, is_visible, is_selected_as_sub_column)| {
            Sort::new(
                SortId(sort_id),
                MenuItemId(menu_item_id),
                sequence_no,
                BinaryFlag::from(is_visible),
                BinaryFlag::from(is_selected_as_sub_column),
            )
        },
    )
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_library_plus::{Color, Database, Key, MenuItem, Property, Sort};

    #[test]
    fn inserts_minimal_initial_content_without_keys() -> crate::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("exportLibrary.db");

        let mut db = Database::create(&path)?;
        insert_initial_content(db.connection_mut())?;

        assert!(Key::all(db.connection_mut())?.is_empty());
        assert_eq!(Color::all(db.connection_mut())?.len(), 8);
        assert_eq!(MenuItem::all(db.connection_mut())?.len(), 27);
        assert_eq!(Category::all(db.connection_mut())?.len(), 22);
        assert_eq!(Sort::all(db.connection_mut())?.len(), 17);

        let property = default_property("2025-10-29");
        property.insert(db.connection_mut())?;
        assert_eq!(Property::first(db.connection_mut())?, Some(property));

        Ok(())
    }

    #[test]
    fn inserts_key_lookup_rows_on_demand() -> crate::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("exportLibrary.db");

        let mut db = Database::create(&path)?;
        insert_key_lookup_rows(db.connection_mut())?;

        assert_eq!(Key::all(db.connection_mut())?.len(), 44);

        Ok(())
    }
}
