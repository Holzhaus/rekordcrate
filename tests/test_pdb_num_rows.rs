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
use std::io::Cursor;

fn assert_pdb_row_count<RowT: RowVariant>(expected_row_count: usize) {
    let data = include_bytes!("../data/pdb/num_rows/export.pdb").as_slice();
    let mut reader = Cursor::new(data);
    let mut db = Database::open_non_persistent(&mut reader, DatabaseType::Plain)
        .expect("Failed to open database");

    let actual_row_count: usize = db
        .iter_rows::<RowT>()
        .expect("Failed to load rows")
        .count()
        .expect("Failed to count rows");

    assert_eq!(
        actual_row_count,
        expected_row_count,
        "wrong row count for page type {:?}",
        RowT::PAGE_TYPE
    );
}

#[test]
fn test_pdb_row_count_albums() {
    assert_pdb_row_count::<Album>(2226);
}

#[test]
fn test_pdb_row_count_artists() {
    assert_pdb_row_count::<Artist>(2216);
}

#[test]
fn test_pdb_row_count_artwork() {
    assert_pdb_row_count::<Artwork>(2178);
}

#[test]
fn test_pdb_row_count_colors() {
    assert_pdb_row_count::<Color>(8);
}

#[test]
fn test_pdb_row_count_genres() {
    assert_pdb_row_count::<Genre>(315);
}

#[test]
fn test_pdb_row_count_historyplaylists() {
    assert_pdb_row_count::<HistoryPlaylist>(1);
}

#[test]
fn test_pdb_row_count_historyentries() {
    assert_pdb_row_count::<HistoryEntry>(73);
}

#[test]
fn test_pdb_row_count_keys() {
    assert_pdb_row_count::<Key>(67);
}

#[test]
fn test_pdb_row_count_labels() {
    assert_pdb_row_count::<Label>(688);
}

#[test]
fn test_pdb_row_count_playlisttree() {
    assert_pdb_row_count::<PlaylistTreeNode>(104);
}

#[test]
fn test_pdb_row_count_playlistentries() {
    assert_pdb_row_count::<PlaylistEntry>(7440);
}

#[test]
fn test_pdb_row_count_columns() {
    assert_pdb_row_count::<ColumnEntry>(27);
}

#[test]
fn test_pdb_row_count_tracks() {
    assert_pdb_row_count::<Track>(3886);
}
