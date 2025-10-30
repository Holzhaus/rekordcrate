// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use fallible_iterator::FallibleIterator;
use rekordcrate::pdb::io::Database;
use rekordcrate::pdb::*;
use std::{io::Cursor, path::PathBuf};

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
        .map(|s| PathBuf::from(s))
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
