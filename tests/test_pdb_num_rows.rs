// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::BinRead;
use rekordcrate::pdb::{Header, PageType};
use std::io::Cursor;

fn assert_pdb_row_count(page_type: PageType, expected_row_count: usize) {
    let data = include_bytes!("../data/pdb/num_rows/export.pdb").as_slice();
    let mut reader = Cursor::new(data);
    let header = Header::read(&mut reader).expect("failed to parse header");

    let table = header
        .tables
        .iter()
        .find(|table| table.page_type == page_type)
        .expect("Failed to find table of given type");
    let pages = header
        .read_pages(
            &mut reader,
            binrw::Endian::NATIVE,
            (&table.first_page, &table.last_page),
        )
        .expect("failed to read pages");

    let actual_row_count: usize = pages
        .into_iter()
        .flat_map(|page| page.row_groups.into_iter())
        .map(|row_group| row_group.present_rows().count())
        .sum();
    assert_eq!(
        actual_row_count, expected_row_count,
        "wrong row count for page type {:?}",
        table.page_type
    );
}

#[test]
fn test_pdb_row_count_albums() {
    assert_pdb_row_count(PageType::Albums, 2226);
}

#[test]
fn test_pdb_row_count_artists() {
    assert_pdb_row_count(PageType::Artists, 2216);
}

#[test]
fn test_pdb_row_count_artwork() {
    assert_pdb_row_count(PageType::Artwork, 2178);
}

#[test]
fn test_pdb_row_count_colors() {
    assert_pdb_row_count(PageType::Colors, 8);
}

#[test]
fn test_pdb_row_count_genres() {
    assert_pdb_row_count(PageType::Genres, 315);
}

#[test]
fn test_pdb_row_count_historyplaylists() {
    assert_pdb_row_count(PageType::HistoryPlaylists, 1);
}

#[test]
fn test_pdb_row_count_historyentries() {
    assert_pdb_row_count(PageType::HistoryEntries, 73);
}

#[test]
fn test_pdb_row_count_keys() {
    assert_pdb_row_count(PageType::Keys, 67);
}

#[test]
fn test_pdb_row_count_labels() {
    assert_pdb_row_count(PageType::Labels, 688);
}

#[test]
fn test_pdb_row_count_playlisttree() {
    assert_pdb_row_count(PageType::PlaylistTree, 104);
}

#[test]
fn test_pdb_row_count_playlistentries() {
    assert_pdb_row_count(PageType::PlaylistEntries, 6637);
}

#[test]
fn test_pdb_row_count_columns() {
    assert_pdb_row_count(PageType::Columns, 27);
}

#[test]
fn test_pdb_row_count_tracks() {
    assert_pdb_row_count(PageType::Tracks, 3886);
}
