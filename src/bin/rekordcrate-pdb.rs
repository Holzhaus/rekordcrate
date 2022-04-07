// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{BinRead, ReadOptions};
use rekordcrate::pdb::{Header, Page, Row, RowOffset};
use std::io::{Seek, SeekFrom};

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let mut reader = std::fs::File::open(&path).expect("failed to open file");
    let header = Header::read(&mut reader).expect("failed to parse pdb file");

    println!("{:#?}", header);

    for (i, table) in header.tables.iter().enumerate() {
        println!("Table {}: {:?}", i, table.page_type);
        for page in header
            .read_pages(
                &mut reader,
                &ReadOptions::default(),
                (&table.first_page, &table.last_page),
            )
            .unwrap()
            .into_iter()
        {
            let page_offset = header.page_offset(&page.page_index);
            println!("  {:?}", page);
            page.row_groups.iter().for_each(|row_group| {
                println!("    {:?}", row_group);
                for &RowOffset(row_offset) in row_group.present_rows() {
                    let abs_offset: u64 = page_offset
                        + u64::try_from(Page::HEADER_SIZE).unwrap()
                        + u64::from(row_offset);
                    reader
                        .seek(SeekFrom::Start(abs_offset))
                        .expect("failed to seek to row offset");
                    let row = Row::read_options(
                        &mut reader,
                        &ReadOptions::default(),
                        (page.page_type.clone(),),
                    )
                    .expect("failed to parse row");
                    println!("      {:?}", row);
                }
            })
        }
    }
}
