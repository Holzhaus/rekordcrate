// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{BinRead, ReadOptions};
use rekordcrate::pdb::{Header, Page};
use std::io::{Seek, SeekFrom};

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let data = std::fs::read(&path).expect("failed to read file");
    let mut reader = std::fs::File::open(&path).expect("failed to open file");
    let header = Header::read(&mut reader).expect("failed to parse pdb file");

    println!("{:#?}", header);

    for (i, table) in header.tables.iter().enumerate() {
        println!("Table {}: {:?}", i, table.page_type);
        for page_index in table.page_indices(&header, &data) {
            let page_offset = header.page_offset(&page_index);
            reader
                .seek(SeekFrom::Start(page_offset))
                .expect("failed to seek to page offset");
            let page =
                Page::read_options(&mut reader, &ReadOptions::default(), (header.page_size,))
                    .expect("failed to parse page");
            println!("  {:?}", page);
            assert_eq!(page.page_index, page_index);
            let page_data = &data[page_offset.try_into().unwrap()..];
            page.row_groups.iter().for_each(|row_group| {
                println!("    {:?}", row_group);
                for row_offset in row_group.present_rows() {
                    let (_, row) = page.row(page_data, row_offset).unwrap();
                    println!("      {:?}", row);
                }
            })
        }
    }
}
