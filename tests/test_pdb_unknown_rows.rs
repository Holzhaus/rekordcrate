// Copyright (c) 2026 Robin McCorkell <robin@mccorkell.me.uk>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{BinRead, BinWrite, Endian};
use fallible_iterator::FallibleIterator;
use rekordcrate::pdb::bitfields::{Unknown7Flags0, Unknown7Flags1, Unknown7Flags2};
use rekordcrate::pdb::io::Database;
use rekordcrate::pdb::*;
use std::io::Cursor;

#[test]
fn parses_plain_table_18_rows() {
    let data = include_bytes!("../data/complete_export/demo_tracks/PIONEER/rekordbox/export.pdb");
    let mut reader = Cursor::new(data.as_slice());
    let mut db = Database::open_non_persistent(&mut reader, DatabaseType::Plain).unwrap();

    let rows: Vec<_> = db
        .iter_rows::<Unknown18Row>()
        .unwrap()
        .map(|row| Ok(*row))
        .collect()
        .unwrap();

    assert_eq!(
        rows,
        vec![
            Unknown18Row::new(1, 6, 0x0001, 0),
            Unknown18Row::new(21, 7, 0x0001, 0),
            Unknown18Row::new(14, 8, 0x0001, 0),
            Unknown18Row::new(8, 9, 0x0001, 0),
            Unknown18Row::new(9, 10, 0x0001, 0),
            Unknown18Row::new(10, 11, 0x0001, 0),
            Unknown18Row::new(15, 13, 0x0001, 0),
            Unknown18Row::new(13, 15, 0x0001, 0),
            Unknown18Row::new(23, 16, 0x0001, 0),
            Unknown18Row::new(22, 17, 0x0001, 0),
            Unknown18Row::new(25, 0, 0x0100, 0),
            Unknown18Row::new(26, 1, 0x0200, 0),
            Unknown18Row::new(2, 2, 0x0300, 0),
            Unknown18Row::new(3, 3, 0x0400, 0),
            Unknown18Row::new(5, 4, 0x0500, 0),
            Unknown18Row::new(6, 5, 0x0600, 0),
            Unknown18Row::new(11, 12, 0x0700, 0),
        ]
    );
}

#[test]
fn parses_export_ext_table_7_row() {
    let data =
        include_bytes!("../data/complete_export/demo_tracks/PIONEER/rekordbox/exportExt.pdb");
    let mut reader = Cursor::new(data.as_slice());
    let mut db = Database::open_non_persistent(&mut reader, DatabaseType::Ext).unwrap();

    let mut rows = db.iter_rows::<Unknown7Row>().unwrap();
    let row = *rows.next().unwrap().unwrap();
    assert!(rows.next().unwrap().is_none());

    assert_eq!(row.reserved0, 0x0700);
    assert_eq!(row.data_page, 0);
    assert_eq!(row.table_id, 0);
    assert_eq!(row.empty_candidate_page, 0);
    assert_eq!(row.state, 0);
    assert_eq!(row.reserved1, 0);
    assert_eq!(row.flags0.into_bytes(), [0x81, 0xfa, 0xe7, 0x05]);
    assert_eq!(row.flags1.into_bytes(), [0x03, 0x22, 0x23, 0x24]);
    assert_eq!(row.version, 0x03032625);
    assert_eq!(row.reserved2, 0x00030303);
    assert_eq!(row.flags2.into_bytes(), [0; 4]);
    assert_eq!(
        [row.reserved3, row.reserved4, row.reserved5, row.reserved6],
        [0; 4]
    );
}

#[test]
fn serializes_typed_opaque_rows() {
    let row18 = Unknown18Row::new(21, 7, 1, 0);
    let mut bytes = Cursor::new(Vec::new());
    row18.write_options(&mut bytes, Endian::Little, ()).unwrap();
    assert_eq!(bytes.get_ref(), &[21, 0, 7, 0, 1, 0, 0, 0]);
    let parsed18 =
        Unknown18Row::read_options(&mut Cursor::new(bytes.into_inner()), Endian::Little, ())
            .unwrap();
    assert_eq!(parsed18, row18);

    let row7 = Unknown7Row::new(
        0,
        16,
        7,
        19,
        1,
        0,
        Unknown7Flags0::canonical(),
        Unknown7Flags1::canonical(),
        1,
        0,
        Unknown7Flags2::canonical(),
        0,
        0,
        0,
        0,
    );
    let mut bytes = Cursor::new(Vec::new());
    row7.write_options(&mut bytes, Endian::Little, ()).unwrap();
    assert_eq!(bytes.get_ref().len(), 60);
    let parsed7 =
        Unknown7Row::read_options(&mut Cursor::new(bytes.into_inner()), Endian::Little, ())
            .unwrap();
    assert_eq!(parsed7, row7);

    for (row, page_type) in [
        (Row::Unknown18(row18), PageType::Unknown(18)),
        (Row::Unknown7(row7), PageType::Unknown(7)),
    ] {
        let mut bytes = Cursor::new(Vec::new());
        row.write_options(&mut bytes, Endian::Little, ()).unwrap();
        let parsed = Row::read_options(
            &mut Cursor::new(bytes.into_inner()),
            Endian::Little,
            (page_type,),
        )
        .unwrap();
        assert_eq!(parsed, row);
    }
}
