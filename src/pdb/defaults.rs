// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Default content of the tables of a device export.
//!
//! Rekordbox fills a handful of tables with the same content in every export, even in one that
//! has no tracks in it: the colors a track can be labelled with, the columns tracks can be
//! browsed by, and the menu those columns appear in. Players read these, so an export built from
//! scratch needs them.
//!
//! The content here was taken from an export written by Rekordbox, which
//! `tests/test_pdb_create.rs` compares against byte for byte.

use super::ext::{ParentId, TagId, TagOrCategory, TagOrCategoryStrings, Unknown7};
use super::io::Database;
use super::offset_array::OffsetArrayContainer;
use super::{Color, ColumnEntry, History, Menu, MenuVisibility, Subtype, Unknown18};
use crate::pdb::string::DeviceSQLString;
use crate::util::{ColorIndex, MaybeCalculated, RekordcrateResult};
use std::io::{Read, Seek};
use std::num::NonZero;

/// The colors a track can be labelled with, in the order Rekordbox writes them.
const COLORS: [(ColorIndex, &str); 8] = [
    (ColorIndex::Pink, "Pink"),
    (ColorIndex::Red, "Red"),
    (ColorIndex::Orange, "Orange"),
    (ColorIndex::Yellow, "Yellow"),
    (ColorIndex::Green, "Green"),
    (ColorIndex::Aqua, "Aqua"),
    (ColorIndex::Blue, "Blue"),
    (ColorIndex::Purple, "Purple"),
];

/// The columns that tracks can be browsed by, as `(id, unknown0, name)`.
///
/// The ids are consecutive, but the values of the unknown field are not, and the names are not in
/// alphabetical or menu order, so all three are listed as Rekordbox writes them.
const COLUMNS: [(u16, u16, &str); 27] = [
    (1, 128, "GENRE"),
    (2, 129, "ARTIST"),
    (3, 130, "ALBUM"),
    (4, 131, "TRACK"),
    (5, 133, "BPM"),
    (6, 134, "RATING"),
    (7, 135, "YEAR"),
    (8, 136, "REMIXER"),
    (9, 137, "LABEL"),
    (10, 138, "ORIGINAL ARTIST"),
    (11, 139, "KEY"),
    (12, 141, "CUE"),
    (13, 142, "COLOR"),
    (14, 146, "TIME"),
    (15, 147, "BITRATE"),
    (16, 148, "FILE NAME"),
    (17, 132, "PLAYLIST"),
    (18, 152, "HOT CUE BANK"),
    (19, 149, "HISTORY"),
    (20, 145, "SEARCH"),
    (21, 150, "COMMENTS"),
    (22, 140, "DATE ADDED"),
    (23, 151, "DJ PLAY COUNT"),
    (24, 144, "FOLDER"),
    (25, 161, "DEFAULT"),
    (26, 162, "ALPHABET"),
    (27, 170, "MATCHING"),
];

/// The menu entries, as `(category_id, content_pointer, unknown, visibility, sort_order)`.
///
/// The hidden entries come first, in the order Rekordbox writes them, followed by the visible
/// ones in the order they appear in the menu.
const MENU: [(u16, u16, u8, MenuVisibility, u16); 22] = [
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

/// The entries of the second, not understood set of menu entries, in the same form as [`MENU`].
///
/// These are identical in every export written by Rekordbox that we have seen, though a database
/// that has been used holds a different set, so they are not a constant of the format.
const UNKNOWN18: [(u16, u16, MenuVisibility, u8, u16); 17] = [
    (1, 6, MenuVisibility::Hidden, 0, 0),
    (21, 7, MenuVisibility::Hidden, 0, 0),
    (14, 8, MenuVisibility::Hidden, 0, 0),
    (8, 9, MenuVisibility::Hidden, 0, 0),
    (9, 10, MenuVisibility::Hidden, 0, 0),
    (10, 11, MenuVisibility::Hidden, 0, 0),
    (15, 13, MenuVisibility::Hidden, 0, 0),
    (13, 15, MenuVisibility::Hidden, 0, 0),
    (23, 16, MenuVisibility::Hidden, 0, 0),
    (22, 17, MenuVisibility::Hidden, 0, 0),
    (25, 0, MenuVisibility::Visible, 1, 0),
    (26, 1, MenuVisibility::Visible, 2, 0),
    (2, 2, MenuVisibility::Visible, 3, 0),
    (3, 3, MenuVisibility::Visible, 4, 0),
    (5, 4, MenuVisibility::Visible, 5, 0),
    (6, 5, MenuVisibility::Visible, 6, 0),
    (11, 12, MenuVisibility::Visible, 7, 0),
];

/// The tag categories and the tags in them that Rekordbox puts in every export, as
/// `(parent, position, id, is_category, name)`.
///
/// A row with no parent is a category and the rows following it are the tags in it. The ids of
/// the tags look arbitrary, but they are the same in every export written by Rekordbox, so they
/// are part of the tag set rather than made up per export.
const TAGS: [(Option<u32>, u32, u32, bool, &str); 28] = [
    (None, 0, 1, true, "Genre"),
    (Some(1), 0, 2179428034, false, "Acid House"),
    (Some(1), 1, 362712954, false, "Deep House"),
    (Some(1), 2, 4258818180, false, "Techno"),
    (Some(1), 3, 3211305494, false, "Nu Disco"),
    (Some(1), 4, 1837410776, false, "Electro House"),
    (Some(1), 5, 2309293472, false, "Bass Music"),
    (Some(1), 6, 2247489588, false, "Trap"),
    (None, 1, 2, true, "Components"),
    (Some(2), 0, 4044416570, false, "Synth"),
    (Some(2), 1, 168531128, false, "Vocal"),
    (Some(2), 2, 656614702, false, "Beat"),
    (Some(2), 3, 973268804, false, "Sub Bass"),
    (Some(2), 4, 725093373, false, "Percussion"),
    (Some(2), 5, 2739544171, false, "Piano"),
    (Some(2), 6, 2572886564, false, "Dark"),
    (Some(2), 7, 3537040097, false, "Upper"),
    (None, 2, 3, true, "Situation"),
    (Some(3), 0, 3464440893, false, "Main Floor"),
    (Some(3), 1, 3767726207, false, "Second Floor"),
    (Some(3), 2, 1932755035, false, "Lounge"),
    (Some(3), 3, 2178301443, false, "Mid Night"),
    (Some(3), 4, 612436637, false, "Morning"),
    (Some(3), 5, 3188249233, false, "Build up"),
    (Some(3), 6, 1743695786, false, "Peak Time"),
    (Some(3), 7, 946077, false, "Build down"),
    (None, 3, 4, true, "Untitled Column"),
    (Some(4), 0, 701802348, false, "My Comment"),
];

/// The single row of the not understood ext table, which is the same in every known export.
const UNKNOWN7: [u8; Unknown7::BINARY_SIZE as usize] = [
    0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81, 0xfa, 0xe7, 0x05, 0x03, 0x22, 0x23, 0x24,
    0x25, 0x26, 0x03, 0x03, 0x03, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Subtype of the history row that Rekordbox writes.
const HISTORY_SUBTYPE: u16 = 640;

/// Version string of the history row that Rekordbox writes.
const HISTORY_VERSION: &str = "1000";

/// Unused page heap space that Rekordbox leaves after a history or tag row.
///
/// The rule behind it is not known, so the amount is taken from the exports Rekordbox writes,
/// where every row of those two tables is given exactly this much space beyond what it needs.
const ROW_SLACK_BYTES: u16 = 8;

/// Adds the colors that a track can be labelled with to the colors table.
pub fn insert_default_colors<IO: Read + Seek>(db: &mut Database<IO>) -> RekordcrateResult<()> {
    for (id, (color, name)) in COLORS.into_iter().enumerate() {
        // The ids are 1-based and match the numeric color.
        let id = u8::try_from(id + 1).expect("color id fits in a byte");
        db.add_row(Color {
            unknown1: 0,
            unknown2: id,
            color,
            unknown3: 0,
            name: DeviceSQLString::new(name)?,
        })?;
    }
    Ok(())
}

/// Adds the columns that tracks can be browsed by to the columns table.
pub fn insert_default_columns<IO: Read + Seek>(db: &mut Database<IO>) -> RekordcrateResult<()> {
    for (id, unknown0, name) in COLUMNS {
        db.add_row(ColumnEntry {
            id,
            unknown0,
            // Rekordbox wraps the names of these columns in the "interlinear annotation"
            // characters, which also makes them non-ASCII and hence stored as UTF-16.
            column_name: DeviceSQLString::new(&format!("\u{fffa}{name}\u{fffb}"))?,
        })?;
    }
    Ok(())
}

/// Adds the menu entries that the columns are browsed through to the menu table.
pub fn insert_default_menu<IO: Read + Seek>(db: &mut Database<IO>) -> RekordcrateResult<()> {
    for (category_id, content_pointer, unknown, visibility, sort_order) in MENU {
        db.add_row(Menu {
            category_id,
            content_pointer,
            unknown,
            visibility,
            sort_order,
        })?;
    }
    Ok(())
}

/// Adds the entries of the second, not understood set of menu entries.
pub fn insert_default_unknown18<IO: Read + Seek>(db: &mut Database<IO>) -> RekordcrateResult<()> {
    for (category_id, content_pointer, visibility, sort_order, unknown) in UNKNOWN18 {
        db.add_row(Unknown18 {
            category_id,
            content_pointer,
            visibility,
            sort_order,
            unknown,
        })?;
    }
    Ok(())
}

/// Adds a row recording an export to the history table.
///
/// `date` is the date the export was written, which Rekordbox stores in `YYYY-MM-DD` form, and
/// `num_tracks` is the number of tracks the export holds.
pub fn insert_history<IO: Read + Seek>(
    db: &mut Database<IO>,
    date: &str,
    num_tracks: u32,
) -> RekordcrateResult<()> {
    db.add_row_with_slack(
        History {
            subtype: Subtype(HISTORY_SUBTYPE),
            index_shift: 0,
            num_tracks,
            date: DeviceSQLString::new(date)?,
            version: DeviceSQLString::new(HISTORY_VERSION)?,
            label: DeviceSQLString::empty(),
        },
        ROW_SLACK_BYTES,
    )?;
    Ok(())
}

/// Adds the content that Rekordbox writes into every `export.pdb`, whether it holds tracks or
/// not.
///
/// See [`insert_history`] for the meaning of `date` and `num_tracks`.
pub fn insert_defaults<IO: Read + Seek>(
    db: &mut Database<IO>,
    date: &str,
    num_tracks: u32,
) -> RekordcrateResult<()> {
    // Rekordbox fills the tables in this order, which decides which page each table claims and
    // therefore the bytes of the pages.
    insert_history(db, date, num_tracks)?;
    insert_default_colors(db)?;
    insert_default_columns(db)?;
    insert_default_menu(db)?;
    insert_default_unknown18(db)?;
    Ok(())
}

/// Adds a tag, or a category holding tags, with the id it is given.
///
/// An export holds the tag set of the library it came from, keeping the ids of that library, so
/// a tag is written with the id it already has.
pub fn insert_tag<IO: Read + Seek>(
    db: &mut Database<IO>,
    id: u32,
    parent_id: u32,
    position: u32,
    is_category: bool,
    name: &str,
) -> RekordcrateResult<()> {
    db.add_row_with_slack(
        TagOrCategory {
            subtype: Subtype(TagOrCategory::SUBTYPE),
            // Recorded when the row is stored, from the place it takes in its page.
            index_shift: 0,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(NonZero::new(parent_id)),
            position,
            id: TagId(id),
            raw_is_category: if is_category {
                TagOrCategory::IS_CATEGORY
            } else {
                0
            },
            offsets: OffsetArrayContainer {
                offsets: MaybeCalculated::Calculated,
                inner: TagOrCategoryStrings {
                    name: DeviceSQLString::new(name)?,
                    unknown: DeviceSQLString::empty(),
                },
            },
        },
        ROW_SLACK_BYTES,
    )?;
    Ok(())
}

/// Adds the row of the ext table whose purpose is not understood.
pub fn insert_unknown7<IO: Read + Seek>(db: &mut Database<IO>) -> RekordcrateResult<()> {
    db.add_row(Unknown7 { bytes: UNKNOWN7 })?;
    Ok(())
}

/// Adds the tag categories and tags that tracks can be labelled with.
pub fn insert_default_tags<IO: Read + Seek>(db: &mut Database<IO>) -> RekordcrateResult<()> {
    for (parent, position, id, is_category, name) in TAGS {
        db.add_row_with_slack(
            TagOrCategory {
                subtype: Subtype(TagOrCategory::SUBTYPE),
                // Recorded when the row is stored, from the place it takes in its page.
                index_shift: 0,
                unknown1: 0,
                unknown2: 0,
                parent_id: ParentId(parent.and_then(NonZero::new)),
                position,
                id: TagId(id),
                raw_is_category: if is_category {
                    TagOrCategory::IS_CATEGORY
                } else {
                    0
                },
                offsets: OffsetArrayContainer {
                    offsets: MaybeCalculated::Calculated,
                    inner: TagOrCategoryStrings {
                        name: DeviceSQLString::new(name)?,
                        unknown: DeviceSQLString::empty(),
                    },
                },
            },
            ROW_SLACK_BYTES,
        )?;
    }
    Ok(())
}

/// Adds the content that Rekordbox writes into every `exportExt.pdb`.
pub fn insert_ext_defaults<IO: Read + Seek>(db: &mut Database<IO>) -> RekordcrateResult<()> {
    // Rekordbox fills the tables in this order, which decides which page each table claims.
    insert_unknown7(db)?;
    insert_default_tags(db)?;
    Ok(())
}
