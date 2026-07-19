// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Default rows for colors, columns, and menus used in Pioneer database exports.
//! This is what Rekordbox inserts when you create a new export without configuring anything.

use super::io::Database;
use super::string::DeviceSQLString;
use super::{Color, ColumnEntry, Menu, MenuVisibility, PlainRow, Row};
use crate::util::ColorIndex;
use std::io::{Read, Seek, Write};

const DEFAULT_COLORS: &[(u8, ColorIndex, &str)] = &[
    (1, ColorIndex::Pink, "Pink"),
    (2, ColorIndex::Red, "Red"),
    (3, ColorIndex::Orange, "Orange"),
    (4, ColorIndex::Yellow, "Yellow"),
    (5, ColorIndex::Green, "Green"),
    (6, ColorIndex::Aqua, "Aqua"),
    (7, ColorIndex::Blue, "Blue"),
    (8, ColorIndex::Purple, "Purple"),
];

const DEFAULT_COLUMNS: &[(u16, u16, &str)] = &[
    (1, 128, "\u{fffa}GENRE\u{fffb}"),
    (2, 129, "\u{fffa}ARTIST\u{fffb}"),
    (3, 130, "\u{fffa}ALBUM\u{fffb}"),
    (4, 131, "\u{fffa}TRACK\u{fffb}"),
    (5, 133, "\u{fffa}BPM\u{fffb}"),
    (6, 134, "\u{fffa}RATING\u{fffb}"),
    (7, 135, "\u{fffa}YEAR\u{fffb}"),
    (8, 136, "\u{fffa}REMIXER\u{fffb}"),
    (9, 137, "\u{fffa}LABEL\u{fffb}"),
    (10, 138, "\u{fffa}ORIGINAL ARTIST\u{fffb}"),
    (11, 139, "\u{fffa}KEY\u{fffb}"),
    (12, 141, "\u{fffa}CUE\u{fffb}"),
    (13, 142, "\u{fffa}COLOR\u{fffb}"),
    (14, 146, "\u{fffa}TIME\u{fffb}"),
    (15, 147, "\u{fffa}BITRATE\u{fffb}"),
    (16, 148, "\u{fffa}FILE NAME\u{fffb}"),
    (17, 132, "\u{fffa}PLAYLIST\u{fffb}"),
    (18, 152, "\u{fffa}HOT CUE BANK\u{fffb}"),
    (19, 149, "\u{fffa}HISTORY\u{fffb}"),
    (20, 145, "\u{fffa}SEARCH\u{fffb}"),
    (21, 150, "\u{fffa}COMMENTS\u{fffb}"),
    (22, 140, "\u{fffa}DATE ADDED\u{fffb}"),
    (23, 151, "\u{fffa}DJ PLAY COUNT\u{fffb}"),
    (24, 144, "\u{fffa}FOLDER\u{fffb}"),
    (25, 161, "\u{fffa}DEFAULT\u{fffb}"),
    (26, 162, "\u{fffa}ALPHABET\u{fffb}"),
    (27, 170, "\u{fffa}MATCHING\u{fffb}"),
];

const DEFAULT_MENUS: &[(u16, u16, u8, MenuVisibility, u16)] = &[
    (1, 1, 99, MenuVisibility::Hidden, 0),
    (5, 6, 5, MenuVisibility::Hidden, 0),
    (6, 7, 99, MenuVisibility::Hidden, 0),
    (7, 8, 99, MenuVisibility::Hidden, 0),
    (8, 9, 99, MenuVisibility::Hidden, 0),
    (9, 10, 99, MenuVisibility::Hidden, 0),
    (10, 11, 99, MenuVisibility::Hidden, 0),
    (13, 15, 99, MenuVisibility::Hidden, 0),
    (14, 19, 4, MenuVisibility::Hidden, 0),
    (15, 20, 6, MenuVisibility::Hidden, 0),
    (16, 21, 99, MenuVisibility::Hidden, 0),
    (18, 23, 99, MenuVisibility::Hidden, 0),
    (2, 2, 2, MenuVisibility::Visible, 1),
    (3, 3, 3, MenuVisibility::Visible, 2),
    (4, 4, 1, MenuVisibility::Visible, 3),
    (11, 12, 99, MenuVisibility::Visible, 4),
    (17, 5, 99, MenuVisibility::Visible, 5),
    (19, 22, 99, MenuVisibility::Visible, 6),
    (20, 18, 99, MenuVisibility::Visible, 7),
    (27, 26, 99, MenuVisibility::Unknown(2), 8),
    (24, 17, 99, MenuVisibility::Visible, 9),
    (22, 27, 99, MenuVisibility::Visible, 10),
];

/// Insert the default color rows into the database.
pub fn insert_default_colors<RW: Read + Write + Seek>(db: &mut Database<RW>) -> crate::Result<()> {
    for &(id, ref color, name) in DEFAULT_COLORS {
        db.add_row(Row::Plain(PlainRow::Color(Color {
            unknown1: 0,
            unknown2: id,
            color: color.clone(),
            unknown3: 0,
            name: DeviceSQLString::new(name)?,
        })))?;
    }
    Ok(())
}

/// Insert the default column rows into the database.
pub fn insert_default_columns<RW: Read + Write + Seek>(db: &mut Database<RW>) -> crate::Result<()> {
    for &(id, unknown0, name) in DEFAULT_COLUMNS {
        db.add_row(Row::Plain(PlainRow::ColumnEntry(ColumnEntry {
            id,
            unknown0,
            column_name: DeviceSQLString::new(name)?,
        })))?;
    }
    Ok(())
}

/// Insert the default menu rows into the database.
pub fn insert_default_menus<RW: Read + Write + Seek>(db: &mut Database<RW>) -> crate::Result<()> {
    for &(category_id, content_pointer, unknown, ref visibility, sort_order) in DEFAULT_MENUS {
        db.add_row(Row::Plain(PlainRow::Menu(Menu {
            category_id,
            content_pointer,
            unknown,
            visibility: *visibility,
            sort_order,
        })))?;
    }
    Ok(())
}
