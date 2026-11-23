// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{BinWrite, Endian};
use fallible_iterator::FallibleIterator;
use rekordcrate::pdb::bitfields::{PackedRowCounts, PageFlags};
use rekordcrate::pdb::io::Database;
use rekordcrate::pdb::*;
use rekordcrate::util::TableIndex;
use std::{collections::BTreeMap, io::Cursor, path::PathBuf};

// Set REKORDCRATE_TEST_DUMP_PATH to dump modified databases to that directory for inspection.

fn get_table_row_count<RowT: RowVariant>(
    db: &mut Database<impl std::io::Read + std::io::Seek>,
) -> usize {
    db.iter_rows::<RowT>()
        .expect("Failed to load rows")
        .count()
        .expect("Failed to count rows")
}

fn assert_pdb_modify_verify(
    test_name: &str,
    modify: impl FnOnce(&mut Database<Cursor<&mut [u8]>>),
    verify: impl FnOnce(&mut Database<Cursor<&[u8]>>),
) {
    let mut data = Vec::from(include_bytes!("../data/pdb/num_rows/export.pdb"));
    let io = Cursor::new(data.as_mut_slice());
    println!("Opening database for modification");
    let mut db = Database::open(io, DatabaseType::Plain).expect("Failed to open database");

    println!("Modifying database");
    modify(&mut db);
    println!("Closing database");
    db.close().expect("failed to close database");

    if let Some(save_dir) = std::env::var("REKORDCRATE_TEST_DUMP_PATH")
        .ok()
        .map(PathBuf::from)
    {
        let save_subdir = save_dir.join("test_pdb_write").join(test_name);
        std::fs::create_dir_all(&save_subdir).expect("failed to create dump directory");
        let save_path = save_subdir.join("export.pdb");
        println!("Dumping database for introspection: {:?}", save_path);
        std::fs::write(save_path, &data).expect("failed to dump modified test database");
    }

    let io = Cursor::new(data.as_slice());
    println!("Opening database for verification");
    let mut db =
        Database::open_non_persistent(io, DatabaseType::Plain).expect("Failed to open database");

    println!("Verifying database");
    verify(&mut db);
}

#[test]
fn test_data_page_write_reaches_page_size_with_vec_writer() {
    const PAGE_SIZE: u32 = 4096;
    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(1).unwrap(),
            page_type: PageType::Plain(PlainPageType::Tracks),
            next_page: PageIndex::try_from(2).unwrap(),
            unknown1: 0,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::default(),
            page_flags: PageFlags::new_data_page(),
            free_size: (PAGE_SIZE - PageHeader::BINARY_SIZE - DataPageHeader::BINARY_SIZE) as u16,
            used_size: 0,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 0,
                unknown_not_num_rows_large: 0,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups: vec![],
            rows: BTreeMap::new(),
        }),
    };

    let mut writer = Cursor::new(Vec::new());
    page.write_options(&mut writer, Endian::Little, (PAGE_SIZE,))
        .expect("failed to serialize page");

    assert_eq!(
        writer.into_inner().len(),
        PAGE_SIZE as usize,
        "an empty final data page must still occupy a full page"
    );
}

#[test]
fn test_pdb_page_chain_metadata() {
    assert_eq!(
        PageIndex::try_from(0x03FF_FFFF).unwrap(),
        PageIndex::SENTINEL
    );
    assert!(PageIndex::try_from(0x0400_0000).is_err());

    let data = include_bytes!("../data/pdb/num_rows/export.pdb");
    let mut db = Database::open_non_persistent(Cursor::new(data.as_slice()), DatabaseType::Plain)
        .expect("failed to open database");
    let tables = db.get_header().tables.clone();

    for (table_index, table) in tables.iter().enumerate() {
        let mut pages = db
            .iter_pages_for_table(TableIndex::from(table_index))
            .expect("failed to get page iterator");
        let (first_index, first_next, inner_next) = {
            let first = pages
                .next()
                .expect("page iterator error")
                .expect("table chain must have a first page");
            let index_content = first
                .content
                .as_index()
                .expect("the first page must be a free-space/index page");
            (
                first.header.page_index,
                first.header.next_page,
                index_content.header.next_page,
            )
        };

        assert_eq!(first_index, table.first_page);
        assert_eq!(
            inner_next,
            if table.first_page == table.last_page {
                PageIndex::SENTINEL
            } else {
                first_next
            },
            "table {table_index}: inner free-space pointer has the wrong empty/non-empty meaning"
        );

        let mut last_index = first_index;
        let mut last_next = first_next;
        while let Some(page) = pages.next().expect("page iterator error") {
            assert!(
                page.content.as_data().is_some(),
                "table {table_index}: pages after the free-space page must contain rows"
            );
            last_index = page.header.page_index;
            last_next = page.header.next_page;
        }

        assert_eq!(last_index, table.last_page);
        assert_eq!(
            last_next,
            PageIndex::try_from(table.empty_candidate)
                .expect("empty_candidate must be a valid page index"),
            "table {table_index}: the chain's final page must point to empty_candidate"
        );
    }
}

#[test]
fn test_pdb_no_loaded_pages() {
    assert_pdb_modify_verify(
        "no_loaded_pages",
        |_| {},
        |db| {
            assert_eq!(get_table_row_count::<Album>(db), 2226);
            assert_eq!(get_table_row_count::<Artist>(db), 2216);
            assert_eq!(get_table_row_count::<Artwork>(db), 2178);
            assert_eq!(get_table_row_count::<Color>(db), 8);
            assert_eq!(get_table_row_count::<Genre>(db), 315);
            assert_eq!(get_table_row_count::<HistoryPlaylist>(db), 1);
            assert_eq!(get_table_row_count::<HistoryEntry>(db), 73);
            assert_eq!(get_table_row_count::<Key>(db), 67);
            assert_eq!(get_table_row_count::<Label>(db), 688);
            assert_eq!(get_table_row_count::<PlaylistTreeNode>(db), 104);
            assert_eq!(get_table_row_count::<PlaylistEntry>(db), 7440);
            assert_eq!(get_table_row_count::<ColumnEntry>(db), 27);
            assert_eq!(get_table_row_count::<Track>(db), 3886);
        },
    );
}

#[test]
fn test_pdb_unchanged_table() {
    assert_pdb_modify_verify(
        "unchanged_table",
        |db| {
            db.iter_rows::<Track>()
                .expect("failed to load tracks table")
                .for_each(|_| {
                    // No modifications.
                    Ok(())
                })
                .expect("failed to iterate over tracks");
        },
        |db| {
            assert_eq!(get_table_row_count::<Track>(db), 3886);
        },
    );
}

#[test]
fn test_pdb_modify_tracks() {
    assert_pdb_modify_verify(
        "modify_tracks",
        |db| {
            db.iter_rows::<Track>()
                .expect("failed to load tracks table")
                .for_each(|track| {
                    // Set the rating of all tracks to 5 stars.
                    track.rating = 5;
                    Ok(())
                })
                .expect("failed to iterate over tracks");
        },
        |db| {
            assert_eq!(get_table_row_count::<Track>(db), 3886);
            db.iter_rows::<Track>()
                .expect("failed to load tracks table")
                .for_each(|track| {
                    assert_eq!(track.rating, 5, "track rating was not modified correctly");
                    Ok(())
                })
                .expect("failed to iterate over tracks");
        },
    );
}
