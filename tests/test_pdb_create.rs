// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Tests for databases created from scratch.
//!
//! The databases that Rekordbox writes for an export without any content are the reference for
//! what a created database should look like, so these tests compare against them page by page.
//! Everything that cannot be compared that way, because Rekordbox put content in it, is covered
//! by comparing against a fixture of our own output instead.

use fallible_iterator::FallibleIterator;
use rekordcrate::pdb::io::Database;
use rekordcrate::pdb::*;
use rekordcrate::util::TableIndex;
use std::io::Cursor;
use std::path::Path;

/// Set `REKORDCRATE_UPDATE_FIXTURES=1` to rewrite the fixtures in `data/pdb/created` instead of
/// comparing against them.
const UPDATE_FIXTURES_VAR: &str = "REKORDCRATE_UPDATE_FIXTURES";

struct Fixture {
    db_type: DatabaseType,
    /// A database created from scratch by rekordcrate.
    created: &'static [u8],
    /// The database of the same type that Rekordbox writes for an export without any content.
    reference: &'static [u8],
    path: &'static str,
}

const PLAIN: Fixture = Fixture {
    db_type: DatabaseType::Plain,
    created: include_bytes!("../data/pdb/created/export.pdb"),
    reference: include_bytes!("../data/complete_export/empty/PIONEER/rekordbox/export.pdb"),
    path: "data/pdb/created/export.pdb",
};

const EXT: Fixture = Fixture {
    db_type: DatabaseType::Ext,
    created: include_bytes!("../data/pdb/created/exportExt.pdb"),
    reference: include_bytes!("../data/complete_export/empty/PIONEER/rekordbox/exportExt.pdb"),
    path: "data/pdb/created/exportExt.pdb",
};

/// Creates a database from scratch and returns the bytes it was written as.
fn create(db_type: DatabaseType) -> Vec<u8> {
    let mut db =
        Database::create(Cursor::new(Vec::new()), db_type).expect("failed to create database");
    db.flush().expect("failed to flush database");
    db.into_inner().into_inner()
}

/// Returns the byte range of a page, so that pages can be compared without parsing them.
fn page_range(page_index: PageIndex, page_size: u32) -> std::ops::Range<usize> {
    let start = page_index.offset(page_size) as usize;
    start..start + page_size as usize
}

/// A created database must still be byte-identical to the fixture of our output.
///
/// Unlike the comparison against Rekordbox, this covers the whole database, including the
/// header and the pages that Rekordbox fills with content.
fn assert_matches_fixture(fixture: &Fixture) {
    let data = create(fixture.db_type);

    if std::env::var(UPDATE_FIXTURES_VAR).is_ok() {
        std::fs::write(Path::new(fixture.path), &data).expect("failed to update fixture");
        return;
    }

    if data == fixture.created {
        return;
    }

    // Point at the first page that differs, which is far more useful than the length of the
    // database when something changes.
    let page_size = Database::<Cursor<Vec<u8>>>::DEFAULT_PAGE_SIZE as usize;
    let first_difference = data
        .chunks(page_size)
        .zip(fixture.created.chunks(page_size))
        .position(|(page, expected)| page != expected);
    panic!(
        "created database does not match {}: {} bytes instead of {}, first differing page: {:?}\n\
         Regenerate the fixture with {}=1 if the change was intended.",
        fixture.path,
        data.len(),
        fixture.created.len(),
        first_difference,
        UPDATE_FIXTURES_VAR,
    );
}

/// The pages of a created database must match those that Rekordbox writes for a database it has
/// not put any content into.
fn assert_matches_rekordbox(fixture: &Fixture) {
    let data = create(fixture.db_type);
    let reference = Database::open_non_persistent(Cursor::new(fixture.reference), fixture.db_type)
        .expect("failed to open reference database");
    let reference_header = reference.get_header().clone();

    let db = Database::open_non_persistent(Cursor::new(data.as_slice()), fixture.db_type)
        .expect("failed to open created database");
    let header = db.get_header().clone();

    assert_eq!(header.page_size, reference_header.page_size);
    assert_eq!(header.num_tables, reference_header.num_tables);
    assert_eq!(header.unknown, reference_header.unknown);
    assert_eq!(header.tables.len(), reference_header.tables.len());

    let mut compared = 0;
    for (table, reference_table) in header.tables.iter().zip(&reference_header.tables) {
        assert_eq!(table.page_type, reference_table.page_type);
        assert_eq!(table.first_page, reference_table.first_page);

        // Rekordbox fills a handful of tables with default content, and a table that has claimed
        // its reserved page has a different page reserved in its place, so only the tables it
        // left empty are comparable.
        if reference_table.first_page != reference_table.last_page {
            continue;
        }
        assert_eq!(table.last_page, reference_table.last_page);

        let range = page_range(table.first_page, header.page_size);
        assert_eq!(
            data[range.clone()],
            fixture.reference[range],
            "index page of table {:?} does not match Rekordbox",
            table.page_type
        );
        compared += 1;
    }
    assert!(
        compared > 0,
        "no tables were comparable, so nothing was checked"
    );
}

/// A created database must be readable, with a table per page type and no rows in any of them.
fn assert_readable_and_empty(fixture: &Fixture) {
    let data = create(fixture.db_type);
    let mut db = Database::open_non_persistent(Cursor::new(data.as_slice()), fixture.db_type)
        .expect("failed to open created database");

    let tables = db.get_header().tables.clone();
    for (i, table) in tables.iter().enumerate() {
        // Pages of a table whose row format is unknown cannot be parsed.
        if matches!(table.page_type, PageType::Unknown(_)) {
            continue;
        }

        let pages: Vec<PageIndex> = db
            .iter_pages_for_table(TableIndex::from(i))
            .expect("failed to read pages")
            .map(|page| {
                assert_eq!(
                    page.content.as_data().map(|data| data.rows.len()),
                    None,
                    "table {:?} has a data page",
                    table.page_type
                );
                Ok(page.header.page_index)
            })
            .collect()
            .expect("failed to load page");
        assert_eq!(
            pages,
            vec![table.first_page],
            "table {:?} should consist of its index page alone",
            table.page_type
        );
    }
}

#[test]
fn test_create_plain_matches_fixture() {
    assert_matches_fixture(&PLAIN);
}

#[test]
fn test_create_ext_matches_fixture() {
    assert_matches_fixture(&EXT);
}

#[test]
fn test_create_plain_matches_rekordbox() {
    assert_matches_rekordbox(&PLAIN);
}

#[test]
fn test_create_ext_matches_rekordbox() {
    assert_matches_rekordbox(&EXT);
}

#[test]
fn test_create_plain_readable_and_empty() {
    assert_readable_and_empty(&PLAIN);
}

#[test]
fn test_create_ext_readable_and_empty() {
    assert_readable_and_empty(&EXT);
}
