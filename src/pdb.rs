// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Parser for Pioneer DeviceSQL database exports (PDB).
//!
//! The Rekordbox DJ software uses writes PDB files to `/PIONEER/rekordbox/export.pdb`.
//!
//! Most of the file format has been reverse-engineered by Henry Betts, Fabian Lesniak and James
//! Elliott.
//!
//! - <https://github.com/Deep-Symmetry/crate-digger/blob/master/doc/Analysis.pdf>
//! - <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html>
//! - <https://github.com/henrybetts/Rekordbox-Decoding>
//! - <https://github.com/flesniak/python-prodj-link/tree/master/prodj/pdblib>

use nom::IResult;
use std::num::TryFromIntError;

/// The type of pages found inside a `Table`.
#[derive(Debug)]
pub enum PageType {
    /// Holds rows of track metadata, such as title, artist, genre, artwork ID, playing time, etc.
    Tracks,
    /// Holds rows of musical genres, for reference by tracks and searching.
    Genres,
    /// Holds rows of artists, for reference by tracks and searching.
    Artists,
    /// Holds rows of albums, for reference by tracks and searching.
    Albums,
    /// Holds rows of music labels, for reference by tracks and searching.
    Labels,
    /// Holds rows of musical keys, for reference by tracks, searching, and key matching.
    Keys,
    /// Holds rows of color labels, for reference  by tracks and searching.
    Colors,
    /// Holds rows that describe the hierarchical tree structure of available playlists and folders
    /// grouping them.
    PlaylistTree,
    /// Holds rows that links tracks to playlists, in the right order.
    PlaylistEntries,
    /// Holds rows of history playlists, i.e. playlists that are recorded every time the device is
    /// mounted by a player.
    HistoryPlaylists,
    /// Holds rows that links tracks to history playlists, in the right order.
    HistoryEntries,
    /// Holds rows pointing to album artwork images.
    Artwork,
    /// Holds information used by rekordbox to synchronize history playlists (not yet studied).
    History,
    /// Unknown Page type.
    Unknown(u32),
}

impl PageType {
    fn parse(input: &[u8]) -> IResult<&[u8], PageType> {
        let (input, page_type_id) = nom::number::complete::le_u32(input)?;

        let page_type = match page_type_id {
            0 => PageType::Tracks,
            1 => PageType::Genres,
            2 => PageType::Artists,
            3 => PageType::Albums,
            4 => PageType::Labels,
            5 => PageType::Keys,
            6 => PageType::Colors,
            7 => PageType::PlaylistTree,
            8 => PageType::PlaylistEntries,
            11 => PageType::HistoryPlaylists,
            12 => PageType::HistoryEntries,
            13 => PageType::Artwork,
            19 => PageType::History,
            x => PageType::Unknown(x),
        };

        Ok((input, page_type))
    }
}

/// Points to a table page and can be used to calculate the page's file offset by multiplying it
/// with the page size (found in the file header).
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct PageIndex(pub u32);

impl PageIndex {
    fn parse(input: &[u8]) -> IResult<&[u8], PageIndex> {
        let (input, index) = nom::number::complete::le_u32(input)?;
        Ok((input, PageIndex(index)))
    }
}

/// Tables are linked lists of pages containing rows of a single type, which are organized
/// into groups.
#[derive(Debug)]
pub struct Table {
    /// Identifies the type of rows that this table contains.
    pub page_type: PageType,
    /// Unknown field, maybe links to a chain of empty pages if the database is ever garbage
    /// collected (?).
    #[allow(dead_code)]
    empty_candidate: u32,
    /// Index of the first page that belongs to this table.
    ///
    /// *Note:* The first page apparently does not contain any rows. If the table is non-empty, the
    /// actual row data can be found in the pages after.
    pub first_page: PageIndex,
    /// Index of the last page that belongs to this table.
    pub last_page: PageIndex,
}

impl Table {
    fn parse(input: &[u8]) -> IResult<&[u8], Table> {
        let (input, page_type) = PageType::parse(input)?;
        let (input, empty_candidate) = nom::number::complete::le_u32(input)?;
        let (input, first_page) = PageIndex::parse(input)?;
        let (input, last_page) = PageIndex::parse(input)?;

        Ok((
            input,
            Table {
                page_type,
                empty_candidate,
                first_page,
                last_page,
            },
        ))
    }

    /// An iterator that yields all page indices belonging to this table.
    pub fn page_indices(&self) -> impl Iterator<Item = PageIndex> {
        let PageIndex(first_page_index) = self.first_page;
        let PageIndex(last_page_index) = self.last_page;
        (first_page_index..=last_page_index)
            .into_iter()
            .map(PageIndex)
    }
}

#[derive(Debug)]
/// The PDB header structure, including the list of tables.
pub struct Header {
    /// Size of a single page in bytes.
    ///
    /// The byte offset of a page can be calculated by multiplying a page index with this value.
    pub page_size: u32,
    /// Unknown field, not used as any `empty_candidate`, points past end of file.
    #[allow(dead_code)]
    next_unused_page: PageIndex,
    /// Unknown field.
    #[allow(dead_code)]
    unknown: u32,
    /// Always incremented by at least one, sometimes by two or three.
    pub sequence: u32,
    /// Each table is a linked list of pages containing rows of a particular type.
    pub tables: Vec<Table>,
}

impl Header {
    /// Parse the header of a PDB file.
    pub fn parse(input: &[u8]) -> IResult<&[u8], Header> {
        // Unknown purpose, perhaps an unoriginal signature, seems to always have the value 0.
        let (input, _) = nom::bytes::complete::tag(b"\0\0\0\0")(input)?;

        let (input, page_size) = nom::number::complete::le_u32(input)?;
        let (input, num_tables) = nom::number::complete::le_u32(input)?;
        let (input, next_unused_page) = PageIndex::parse(input)?;
        let (input, unknown) = nom::number::complete::le_u32(input)?;
        let (input, sequence) = nom::number::complete::le_u32(input)?;

        // Gap
        let (input, _) = nom::bytes::complete::tag(b"\0\0\0\0")(input)?;

        // Tables
        let (input, tables) =
            nom::multi::count(Table::parse, num_tables.try_into().unwrap())(input)?;

        Ok((
            input,
            Header {
                page_size,
                next_unused_page,
                sequence,
                unknown,
                tables,
            },
        ))
    }

    /// Returns the offset for the given `page_index`, relative to the start of the PDB file.
    pub fn page_offset(&self, PageIndex(page_index): &PageIndex) -> Result<usize, TryFromIntError> {
        (page_index * self.page_size).try_into()
    }

    /// Parses and returns a page from the original data slice.
    pub fn page<'a>(&self, input: &'a [u8], page_index: &PageIndex) -> IResult<&'a [u8], Page> {
        let position = self.page_offset(page_index).unwrap();
        let (data, page) = Page::parse(&input[position..], self.page_size)?;
        Ok((data, page))
    }
}

#[derive(Debug)]
/// A table page.
///
/// Each page consists of a header that contains information about the type, number of rows, etc.,
/// followed by the data section that holds the row data. Each row needs to be located using an
/// offset found in the page footer at the end of the page.
pub struct Page {
    /// Index of the page.
    ///
    /// Should match the index used for lookup and can be used to verify that the correct page was loaded.
    pub page_index: PageIndex,
    /// Type of information that the rows of this page contain.
    ///
    /// Should match the page type of the table that this page belongs to.
    pub page_type: PageType,
    /// Index of the next page with the same page type.
    ///
    /// If this page is the last one of that type, the page index stored in the field will point
    /// past the end of the file.
    pub next_page: PageIndex,
    /// Unknown field.
    #[allow(dead_code)]
    unknown1: u32,
    /// Unknown field.
    #[allow(dead_code)]
    unknown2: u32,
    /// Number of rows in this table (8-bit version).
    ///
    /// Used if `num_rows_large` not greater than this value and not equal to `0x1FFF`, which means
    /// that the number of rows fits into a single byte.
    pub num_rows_small: u8,
    /// Unknown field.
    ///
    /// According to [@flesniak](https://github.com/flesniak):
    /// > a bitmask (first track: 32)
    #[allow(dead_code)]
    unknown3: u8,
    /// Unknown field.
    ///
    /// According to [@flesniak](https://github.com/flesniak):
    /// > often 0, sometimes larger, esp. for pages with high real_entry_count (e.g. 12 for 101 entries)
    #[allow(dead_code)]
    unknown4: u8,
    /// Page flags.
    ///
    /// According to [@flesniak](https://github.com/flesniak):
    /// > strange pages: 0x44, 0x64; otherwise seen: 0x24, 0x34
    pub page_flags: u8,
    /// Free space in bytes in the data section of the page (excluding the row offsets in the page footer).
    pub free_size: u16,
    /// Used space in bytes in the data section of the page.
    pub used_size: u16,
    /// Unknown field.
    ///
    /// According to [@flesniak](https://github.com/flesniak):
    /// > (0->1: 2)
    #[allow(dead_code)]
    unknown5: u16,
    /// Number of rows in this table (16-bit version).
    ///
    /// Used when the number of rows does not fit into a single byte. In that case,`num_rows_large`
    /// is greater than `num_rows_small`, but is not equal to `0x1FFF`.
    pub num_rows_large: u16,
    /// Unknown field.
    #[allow(dead_code)]
    unknown6: u16,
    /// Unknown field.
    ///
    /// According to [@flesniak](https://github.com/flesniak):
    /// > always 0, except 1 for history pages, num entries for strange pages?"
    #[allow(dead_code)]
    unknown7: u16,
    /// Unknown field.
    ///
    /// In contrast to the other fields, this is part of the footer, at the last two bytes of the
    /// page.
    #[allow(dead_code)]
    unknown8: u16,
}

impl Page {
    const HEADER_SIZE: usize = 0x28;

    /// Parses a page of a PDB file.
    fn parse(input: &[u8], page_size: u32) -> IResult<&[u8], Page> {
        let page_data = input;
        // Signature (?)
        let (input, _) = nom::bytes::complete::tag(b"\0\0\0\0")(input)?;
        let (input, page_index) = PageIndex::parse(input)?;
        let (input, page_type) = PageType::parse(input)?;
        let (input, next_page) = PageIndex::parse(input)?;
        let (input, unknown1) = nom::number::complete::le_u32(input)?;
        let (input, unknown2) = nom::number::complete::le_u32(input)?;
        let (input, num_rows_small) = nom::number::complete::u8(input)?;
        let (input, unknown3) = nom::number::complete::u8(input)?;
        let (input, unknown4) = nom::number::complete::u8(input)?;
        let (input, page_flags) = nom::number::complete::u8(input)?;
        let (input, free_size) = nom::number::complete::le_u16(input)?;
        let (input, used_size) = nom::number::complete::le_u16(input)?;
        let (input, unknown5) = nom::number::complete::le_u16(input)?;
        let (input, num_rows_large) = nom::number::complete::le_u16(input)?;
        let (input, unknown6) = nom::number::complete::le_u16(input)?;
        let (input, unknown7) = nom::number::complete::le_u16(input)?;

        let page_end = usize::try_from(page_size).unwrap();
        let (_, unknown8) = nom::number::complete::le_u16(&page_data[..page_end - 2])?;

        let page = Page {
            page_index,
            page_type,
            next_page,
            unknown1,
            unknown2,
            num_rows_small,
            unknown3,
            unknown4,
            page_flags,
            free_size,
            used_size,
            unknown5,
            num_rows_large,
            unknown6,
            unknown7,
            unknown8,
        };

        Ok((input, page))
    }

    #[must_use]
    /// Returns `true` if the page actually contains row data.
    pub fn has_data(&self) -> bool {
        (self.page_flags & 0x40) == 0
    }

    #[must_use]
    /// Number of rows on this page.
    ///
    /// Note that this number includes rows that have been flagged as missing by the row group.
    pub fn num_rows(&self) -> u16 {
        if self.num_rows_large > self.num_rows_small.into() && self.num_rows_large != 0x1fff {
            self.num_rows_large
        } else {
            self.num_rows_small.into()
        }
    }

    #[must_use]
    /// Number of row groups.
    ///
    /// All row groups except the last one consist of 16 rows (but that number includes rows that
    /// have been flagged as missing by the row group.
    pub fn num_row_groups(&self) -> u16 {
        let num_rows = self.num_rows();
        if num_rows > 0 {
            (num_rows - 1) / RowGroup::MAX_ROW_COUNT + 1
        } else {
            0
        }
    }

    /// The number of row groups that are present in the index. Each group can hold up to sixteen
    /// rows. All but the final one will hold sixteen rows.
    fn row_group_counts(&self) -> impl Iterator<Item = u16> + '_ {
        let num_groups = if self.has_data() {
            self.num_row_groups()
        } else {
            0u16
        };
        (0..num_groups).map(move |i| {
            if (i + 1) == num_groups {
                let num = self.num_rows() % RowGroup::MAX_ROW_COUNT;
                if num == 0 {
                    RowGroup::MAX_ROW_COUNT
                } else {
                    num
                }
            } else {
                RowGroup::MAX_ROW_COUNT
            }
        })
    }

    /// The rows groups found in this page.
    pub fn row_groups<'a>(
        &'a self,
        page_data: &'a [u8],
        page_size: u32,
    ) -> impl Iterator<Item = RowGroup> + 'a {
        let row_groups_offset = usize::try_from(page_size).unwrap() - 2;
        self.row_group_counts()
            .map(usize::try_from)
            .map(Result::unwrap)
            .scan(row_groups_offset, |offset, num_rows_in_group| {
                *offset -= num_rows_in_group * 2 + 2;
                Some((*offset, num_rows_in_group))
            })
            .map(|(offset, num_rows_in_group)| {
                let (_, row_group) =
                    RowGroup::parse(&page_data[offset..], num_rows_in_group).unwrap();
                row_group
            })
    }

    /// Get the page row from the `page_data` slice.
    pub fn row<'a>(
        &self,
        page_data: &'a [u8],
        &RowOffset(row_offset): &RowOffset,
    ) -> IResult<&'a [u8], Row> {
        let offset: usize = Self::HEADER_SIZE + usize::try_from(row_offset).unwrap();
        let (input, row) = Row::parse(&page_data[offset..], &self.page_type)?;
        Ok((input, row))
    }
}

#[derive(Debug)]
/// An offset which points to a row in the table, whose actual presence is controlled by one of the
/// bits in `row_present_flags`. This instance allows the row itself to be lazily loaded, unless it
/// is not present, in which case there is no content to be loaded.
pub struct RowOffset(u16);

#[derive(Debug)]
/// A group of row indices, which are built backwards from the end of the page. Holds up to sixteen
/// row offsets, along with a bit mask that indicates whether each row is actually present in the
/// table.
pub struct RowGroup(pub Vec<RowOffset>);

impl RowGroup {
    const MAX_ROW_COUNT: u16 = 16;

    fn parse(input: &[u8], num_rows: usize) -> IResult<&[u8], RowGroup> {
        let (input, rows) = nom::multi::count(nom::number::complete::le_u16, num_rows)(input)?;
        let (input, row_presence_flags) = nom::number::complete::le_u16(input)?;

        let rows_filtered = rows
            .into_iter()
            .rev()
            .enumerate()
            .filter_map(|(i, index)| {
                if (row_presence_flags & (1 << i)) != 0 {
                    Some(RowOffset(index))
                } else {
                    None
                }
            })
            .collect();

        Ok((input, RowGroup(rows_filtered)))
    }
}

#[derive(Debug)]
/// A table rows contains the actual data
pub enum Row {
    /// The row format (and also its size) is unknown, which means it can't be parsed.
    Unknown,
}

impl Row {
    fn parse<'a>(input: &'a [u8], _page_type: &PageType) -> IResult<&'a [u8], Row> {
        Ok((input, Row::Unknown))
    }
}
