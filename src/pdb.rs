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

use crate::util::{nom_input_error_with_kind, ColorIndex};
use nom::error::ErrorKind;
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
        let table_count = usize::try_from(num_tables)
            .map_err(|_| nom_input_error_with_kind(input, ErrorKind::TooLarge))?;

        let (input, next_unused_page) = PageIndex::parse(input)?;
        let (input, unknown) = nom::number::complete::le_u32(input)?;
        let (input, sequence) = nom::number::complete::le_u32(input)?;

        // Gap
        let (input, _) = nom::bytes::complete::tag(b"\0\0\0\0")(input)?;

        // Tables
        let (input, tables) = nom::multi::count(Table::parse, table_count)(input)?;

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
        let position = self
            .page_offset(page_index)
            .map_err(|_| nom_input_error_with_kind(input, ErrorKind::TooLarge))?;
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
    fn parse(page_data: &[u8], page_size: u32) -> IResult<&[u8], Page> {
        let page_end = usize::try_from(page_size)
            .map_err(|_| nom_input_error_with_kind(page_data, ErrorKind::TooLarge))?;

        // Signature (?)
        let (input, _) = nom::bytes::complete::tag(b"\0\0\0\0")(page_data)?;
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
        let row_groups_offset = usize::try_from(page_size).unwrap();
        self.row_group_counts()
            .map(usize::try_from)
            .map(Result::unwrap)
            .scan(row_groups_offset, |offset, num_rows_in_group| {
                *offset -= num_rows_in_group * 2 + 4;
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
        let row_offset = usize::try_from(row_offset)
            .map_err(|_| nom_input_error_with_kind(page_data, ErrorKind::TooLarge))?;
        let offset = Self::HEADER_SIZE + row_offset;
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
/// A string.
pub enum DeviceSQLString {
    /// A short ASCII encoded string (max 126 characters).
    ShortASCII(String),
    /// A long ASCII encoded string (max 65535 characters).
    LongASCII(String),
    /// A long UTF-16 little-endian encoded string (max 32767 characters).
    LongUTF16LE(String),
}

impl DeviceSQLString {
    fn parse(input: &[u8]) -> IResult<&[u8], DeviceSQLString> {
        let (_, length_and_kind) = nom::number::complete::u8(input)?;
        match length_and_kind {
            0x40 => Self::parse_long_ascii(input),
            0x90 => Self::parse_long_utf16le(input),
            _ => Self::parse_short_ascii(input),
        }
    }

    fn parse_short_ascii(input: &[u8]) -> IResult<&[u8], DeviceSQLString> {
        let (input, mangled_length) = nom::number::complete::u8(input)?;
        let length = ((mangled_length - 1) / 2) - 1;
        let (input, data) = nom::bytes::complete::take(length)(input)?;
        std::str::from_utf8(data).map_or_else(
            |_| Err(nom_input_error_with_kind(input, ErrorKind::Char)),
            |text| Ok((input, Self::ShortASCII(text.to_owned()))),
        )
    }

    fn parse_long_ascii(input: &[u8]) -> IResult<&[u8], DeviceSQLString> {
        let (input, _) = nom::bytes::complete::tag(b"\x40")(input)?;
        let (input, data) = nom::multi::length_value(
            nom::number::complete::le_u16,
            nom::bytes::complete::take(1usize),
        )(input)?;
        std::str::from_utf8(data).map_or_else(
            |_| Err(nom_input_error_with_kind(input, ErrorKind::Char)),
            |text| Ok((input, Self::LongASCII(text.to_owned()))),
        )
    }

    fn parse_long_utf16le(input: &[u8]) -> IResult<&[u8], DeviceSQLString> {
        let (input, _) = nom::bytes::complete::tag(b"\x90")(input)?;
        let (input, length) = nom::number::complete::le_u16(input)?;
        let (input, _) = nom::bytes::complete::tag(b"\x00")(input)?;

        // The length is in bytes, UTF-16 code points are always 2 bytes wide,
        // so a valid length must be even.
        debug_assert_eq!(length % 2, 0);

        let str_length = usize::from(length - 4) / 2;
        let (input, data) = nom::multi::count(nom::number::complete::le_u16, str_length)(input)?;

        String::from_utf16(&data).map_or_else(
            |_| Err(nom_input_error_with_kind(input, ErrorKind::Char)),
            |text| Ok((input, Self::LongUTF16LE(text))),
        )
    }
}

// The large enum size is unfortunate, but since users of this library will probably use iterators
// to consume the results on demand, we can live with this. The alternative of using a `Box` would
// require a heap allocation per row, which is arguably worse. Hence, the warning is disabled for
// this enum.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
/// A table rows contains the actual data
pub enum Row {
    /// Contains the album name, along with an ID of the corresponding artist.
    Album {
        /// Unknown field, usually `80 00`.
        unknown1: u16,
        /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
        index_shift: u16,
        /// Unknown field.
        unknown2: u32,
        /// ID of the artist row associated with this row.
        artist_id: u32,
        /// ID of this row.
        id: u32,
        /// Unknown field.
        unknown3: u32,
        /// Unknown field.
        unknown4: u8,
        /// Byte offset of the album name string, relative to the start of this row.
        name: DeviceSQLString,
    },
    /// Contains the artist name and ID.
    Artist {
        /// Determines if the `name` string is located at the 8-bit offset (0x60) or the 16-bit offset (0x64).
        subtype: u16,
        /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
        index_shift: u16,
        /// ID of this row.
        id: u32,
        /// Unknown field.
        unknown1: u8,
        /// Name of this artist.
        name: DeviceSQLString,
    },
    /// Contains the artwork path and ID.
    Artwork {
        /// ID of this row.
        id: u32,
        /// Path to the album art file.
        path: DeviceSQLString,
    },
    /// Contains numeric color ID
    Color {
        /// Unknown field.
        unknown1: u32,
        /// Unknown field.
        unknown2: u8,
        /// Numeric color ID
        color: ColorIndex,
        /// Unknown field.
        unknown3: u8,
        /// User-defined name of the color.
        name: DeviceSQLString,
    },
    /// Represents a musical genre.
    Genre {
        /// ID of this row.
        id: u32,
        /// Name of the genre.
        name: DeviceSQLString,
    },
    /// Represents a history playlist.
    HistoryPlaylist {
        /// ID of this row.
        id: u32,
        /// Name of the playlist.
        name: DeviceSQLString,
    },
    /// Represents a history playlist.
    HistoryEntry {
        /// ID of the track played at this position in the playlist.
        track_id: u32,
        /// ID of the history playlist.
        playlist_id: u32,
        /// Position within the playlist.
        entry_index: u32,
    },
    /// Represents a musical key.
    Key {
        /// ID of this row.
        id: u32,
        /// Apparently a second copy of the row ID.
        id2: u32,
        /// Name of the key.
        name: DeviceSQLString,
    },
    /// Represents a record label.
    Label {
        /// ID of this row.
        id: u32,
        /// Name of the record label.
        name: DeviceSQLString,
    },
    /// Represents a node in the playlist tree (either a folder or a playlist).
    PlaylistTreeNode {
        /// ID of parent row of this row (which means that the parent is a folder).
        parent_id: u32,
        /// Unknown field.
        unknown: u32,
        /// ID of this row.
        id: u32,
        /// Sort order indicastor.
        sort_order: u32,
        /// Indicates if the node is a folder. Non-zero if it's a leaf node, i.e. a playlist.
        node_is_folder: u32,
        /// Name of this node, as shown when navigating the menu.
        name: DeviceSQLString,
    },
    /// Represents a track entry in a playlist.
    PlaylistEntry {
        /// Position within the playlist.
        entry_index: u32,
        /// ID of the track played at this position in the playlist.
        track_id: u32,
        /// ID of the history playlist.
        playlist_id: u32,
    },
    /// Contains the album name, along with an ID of the corresponding artist.
    Track {
        /// Unknown field, usually `24 00`.
        unknown1: u16,
        /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
        index_shift: u16,
        /// Unknown field, called `bitmask` by [@flesniak](https://github.com/flesniak).
        bitmask: u32,
        /// Sample Rate in Hz.
        sample_rate: u32,
        /// Composer of this track as artist row ID (non-zero if set).
        composer_id: u32,
        /// File size in bytes.
        file_size: u32,
        /// Unknown field (maybe another ID?)
        unknown2: u32,
        /// Unknown field ("always 19048?" according to [@flesniak](https://github.com/flesniak))
        unknown3: u16,
        /// Unknown field ("always 30967?" according to [@flesniak](https://github.com/flesniak))
        unknown4: u16,
        /// Artwork row ID for the cover art (non-zero if set),
        artwork_id: u32,
        /// Key row ID for the cover art (non-zero if set).
        key_id: u32,
        /// Artist row ID of the original performer (non-zero if set).
        orig_artist_id: u32,
        /// Label row ID of the original performer (non-zero if set).
        label_id: u32,
        /// Artist row ID of the remixer (non-zero if set).
        remixer_id: u32,
        /// Bitrate of the track.
        bitrate: u32,
        /// Track number of the track.
        track_number: u32,
        /// Track tempo in centi-BPM (= 1/100 BPM).
        tempo: u32,
        /// Genre row ID for this track (non-zero if set).
        genre_id: u32,
        /// Album row ID for this track (non-zero if set).
        album_id: u32,
        /// Artist row ID for this track (non-zero if set).
        artist_id: u32,
        /// Row ID of this track (non-zero if set).
        id: u32,
        /// Disc number of this track (non-zero if set).
        disc_number: u16,
        /// Number of times this track was played.
        play_count: u16,
        /// Year this track was released.
        year: u16,
        /// Bits per sample of the track aduio file.
        sample_depth: u16,
        /// Playback duration of this track in seconds (at normal speed).
        duration: u16,
        /// Unknown field, apparently always "29".
        unknown5: u16,
        /// Color row ID for this track (non-zero if set).
        color: ColorIndex,
        /// User rating of this track (0 to 5 starts).
        rating: u8,
        /// Unknown field, apparently always "1".
        unknown6: u16,
        /// Unknown field (alternating "2" and "3"?).
        unknown7: u16,
        /// International Standard Recording Code (ISRC), in mangled format.
        isrc: DeviceSQLString,
        /// Unknown string field.
        unknown_string1: DeviceSQLString,
        /// Unknown string field.
        unknown_string2: DeviceSQLString,
        /// Unknown string field.
        unknown_string3: DeviceSQLString,
        /// Unknown string field.
        unknown_string4: DeviceSQLString,
        /// Unknown string field (named by [@flesniak](https://github.com/flesniak)).
        message: DeviceSQLString,
        /// Probably describes whether the track is public on kuvo.com (?). Value is either "ON" or empty string.
        kuvo_public: DeviceSQLString,
        /// Determines if hotcues should be autoloaded. Value is either "ON" or empty string.
        autoload_hotcues: DeviceSQLString,
        /// Unknown string field.
        unknown_string5: DeviceSQLString,
        /// Unknown string field (usually empty).
        unknown_string6: DeviceSQLString,
        /// Date when the track was added to the Rekordbox collection.
        date_added: DeviceSQLString,
        /// Date when the track was released.
        release_date: DeviceSQLString,
        /// Name of the remix (if any).
        mix_name: DeviceSQLString,
        /// Unknown string field (usually empty).
        unknown_string7: DeviceSQLString,
        /// File path of the track analysis file.
        analyze_path: DeviceSQLString,
        /// Date when the track analysis was performed.
        analyze_date: DeviceSQLString,
        /// Track comment.
        comment: DeviceSQLString,
        /// Track title.
        title: DeviceSQLString,
        /// Unknown string field (usually empty).
        unknown_string8: DeviceSQLString,
        /// Name of the file.
        filename: DeviceSQLString,
        /// Path of the file.
        file_path: DeviceSQLString,
    },
    /// The row format (and also its size) is unknown, which means it can't be parsed.
    Unknown,
}

impl Row {
    fn parse<'a>(input: &'a [u8], page_type: &PageType) -> IResult<&'a [u8], Row> {
        match page_type {
            PageType::Albums => Row::parse_album(input),
            PageType::Artists => Row::parse_artist(input),
            PageType::Artwork => Row::parse_artwork(input),
            PageType::Colors => Row::parse_color(input),
            PageType::PlaylistTree => Row::parse_playlist_tree_node(input),
            PageType::PlaylistEntries => Row::parse_playlist_entry(input),
            PageType::Genres => Row::parse_genre(input),
            PageType::HistoryPlaylists => Row::parse_history_playlist(input),
            PageType::HistoryEntries => Row::parse_history_entry(input),
            PageType::Keys => Row::parse_key(input),
            PageType::Labels => Row::parse_label(input),
            PageType::Tracks => Row::parse_track(input),
            _ => Ok((input, Row::Unknown)),
        }
    }

    fn parse_string_offset<'a>(
        input: &'a [u8],
        row_data: &'a [u8],
    ) -> IResult<&'a [u8], DeviceSQLString> {
        let (input, offset) = nom::number::complete::le_u16(input)?;
        let (_, text) = DeviceSQLString::parse(&row_data[usize::from(offset)..])?;
        Ok((input, text))
    }

    fn parse_album(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, unknown1) = nom::number::complete::le_u16(row_data)?;
        let (input, index_shift) = nom::number::complete::le_u16(input)?;
        let (input, unknown2) = nom::number::complete::le_u32(input)?;
        let (input, artist_id) = nom::number::complete::le_u32(input)?;
        let (input, id) = nom::number::complete::le_u32(input)?;
        let (input, unknown3) = nom::number::complete::le_u32(input)?;
        let (input, unknown4) = nom::number::complete::u8(input)?;
        let (input, offset) = nom::number::complete::u8(input)?;
        let (_, name) = DeviceSQLString::parse(&row_data[usize::from(offset)..])?;

        Ok((
            input,
            Row::Album {
                unknown1,
                index_shift,
                unknown2,
                artist_id,
                id,
                unknown3,
                unknown4,
                name,
            },
        ))
    }

    fn parse_artist(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, subtype) = nom::number::complete::le_u16(row_data)?;
        let (input, index_shift) = nom::number::complete::le_u16(input)?;
        let (input, id) = nom::number::complete::le_u32(input)?;
        let (input, unknown1) = nom::number::complete::u8(input)?;
        let (input, ofs_name_near) = nom::number::complete::u8(input)?;
        let (input, name) = if subtype == 0x60 {
            let (_, text) = DeviceSQLString::parse(&row_data[usize::from(ofs_name_near)..])?;
            (input, text)
        } else {
            let (input, ofs_name_far) = nom::number::complete::le_u16(row_data)?;
            let (_, text) = DeviceSQLString::parse(&row_data[usize::from(ofs_name_far)..])?;
            (input, text)
        };

        Ok((
            input,
            Row::Artist {
                subtype,
                index_shift,
                id,
                unknown1,
                name,
            },
        ))
    }

    fn parse_artwork(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, id) = nom::number::complete::le_u32(row_data)?;
        let (input, path) = DeviceSQLString::parse(input)?;

        Ok((input, Row::Artwork { id, path }))
    }

    fn parse_color(input: &[u8]) -> IResult<&[u8], Row> {
        let (input, unknown1) = nom::number::complete::le_u32(input)?;
        let (input, unknown2) = nom::number::complete::u8(input)?;
        let (input, color) = ColorIndex::parse_u16(input)?;
        let (input, unknown3) = nom::number::complete::u8(input)?;
        let (input, name) = DeviceSQLString::parse(input)?;
        Ok((
            input,
            Row::Color {
                unknown1,
                unknown2,
                color,
                unknown3,
                name,
            },
        ))
    }

    fn parse_genre(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, id) = nom::number::complete::le_u32(row_data)?;
        let (input, name) = DeviceSQLString::parse(input)?;

        Ok((input, Row::Genre { id, name }))
    }

    fn parse_history_playlist(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, id) = nom::number::complete::le_u32(row_data)?;
        let (input, name) = DeviceSQLString::parse(input)?;

        Ok((input, Row::HistoryPlaylist { id, name }))
    }

    fn parse_history_entry(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, track_id) = nom::number::complete::le_u32(row_data)?;
        let (input, playlist_id) = nom::number::complete::le_u32(input)?;
        let (input, entry_index) = nom::number::complete::le_u32(input)?;

        Ok((
            input,
            Row::HistoryEntry {
                track_id,
                playlist_id,
                entry_index,
            },
        ))
    }

    fn parse_key(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, id) = nom::number::complete::le_u32(row_data)?;
        let (input, id2) = nom::number::complete::le_u32(input)?;
        let (input, name) = DeviceSQLString::parse(input)?;

        Ok((input, Row::Key { id, id2, name }))
    }

    fn parse_label(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, id) = nom::number::complete::le_u32(row_data)?;
        let (input, name) = DeviceSQLString::parse(input)?;

        Ok((input, Row::Label { id, name }))
    }

    fn parse_playlist_tree_node(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, parent_id) = nom::number::complete::le_u32(row_data)?;
        let (input, unknown) = nom::number::complete::le_u32(input)?;
        let (input, sort_order) = nom::number::complete::le_u32(input)?;
        let (input, id) = nom::number::complete::le_u32(input)?;
        let (input, node_is_folder) = nom::number::complete::le_u32(input)?;
        let (input, name) = DeviceSQLString::parse(input)?;

        Ok((
            input,
            Row::PlaylistTreeNode {
                parent_id,
                unknown,
                sort_order,
                id,
                node_is_folder,
                name,
            },
        ))
    }

    fn parse_playlist_entry(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, entry_index) = nom::number::complete::le_u32(row_data)?;
        let (input, track_id) = nom::number::complete::le_u32(input)?;
        let (input, playlist_id) = nom::number::complete::le_u32(input)?;

        Ok((
            input,
            Row::PlaylistEntry {
                entry_index,
                track_id,
                playlist_id,
            },
        ))
    }

    fn parse_track(row_data: &[u8]) -> IResult<&[u8], Row> {
        let (input, unknown1) = nom::number::complete::le_u16(row_data)?;
        let (input, index_shift) = nom::number::complete::le_u16(input)?;
        let (input, bitmask) = nom::number::complete::le_u32(input)?;
        let (input, sample_rate) = nom::number::complete::le_u32(input)?;
        let (input, composer_id) = nom::number::complete::le_u32(input)?;
        let (input, file_size) = nom::number::complete::le_u32(input)?;
        let (input, unknown2) = nom::number::complete::le_u32(input)?;
        let (input, unknown3) = nom::number::complete::le_u16(input)?;
        let (input, unknown4) = nom::number::complete::le_u16(input)?;
        let (input, artwork_id) = nom::number::complete::le_u32(input)?;
        let (input, key_id) = nom::number::complete::le_u32(input)?;
        let (input, orig_artist_id) = nom::number::complete::le_u32(input)?;
        let (input, label_id) = nom::number::complete::le_u32(input)?;
        let (input, remixer_id) = nom::number::complete::le_u32(input)?;
        let (input, bitrate) = nom::number::complete::le_u32(input)?;
        let (input, track_number) = nom::number::complete::le_u32(input)?;
        let (input, tempo) = nom::number::complete::le_u32(input)?;
        let (input, genre_id) = nom::number::complete::le_u32(input)?;
        let (input, album_id) = nom::number::complete::le_u32(input)?;
        let (input, artist_id) = nom::number::complete::le_u32(input)?;
        let (input, id) = nom::number::complete::le_u32(input)?;
        let (input, disc_number) = nom::number::complete::le_u16(input)?;
        let (input, play_count) = nom::number::complete::le_u16(input)?;
        let (input, year) = nom::number::complete::le_u16(input)?;
        let (input, sample_depth) = nom::number::complete::le_u16(input)?;
        let (input, duration) = nom::number::complete::le_u16(input)?;
        let (input, unknown5) = nom::number::complete::le_u16(input)?;
        let (input, color) = ColorIndex::parse_u8(input)?;
        let (input, rating) = nom::number::complete::u8(input)?;
        let (input, unknown6) = nom::number::complete::le_u16(input)?;
        let (input, unknown7) = nom::number::complete::le_u16(input)?;

        // String offsets
        let (input, isrc) = Self::parse_string_offset(input, row_data)?;
        let (input, unknown_string1) = Self::parse_string_offset(input, row_data)?;
        let (input, unknown_string2) = Self::parse_string_offset(input, row_data)?;
        let (input, unknown_string3) = Self::parse_string_offset(input, row_data)?;
        let (input, unknown_string4) = Self::parse_string_offset(input, row_data)?;
        let (input, message) = Self::parse_string_offset(input, row_data)?;
        let (input, kuvo_public) = Self::parse_string_offset(input, row_data)?;
        let (input, autoload_hotcues) = Self::parse_string_offset(input, row_data)?;
        let (input, unknown_string5) = Self::parse_string_offset(input, row_data)?;
        let (input, unknown_string6) = Self::parse_string_offset(input, row_data)?;
        let (input, date_added) = Self::parse_string_offset(input, row_data)?;
        let (input, release_date) = Self::parse_string_offset(input, row_data)?;
        let (input, mix_name) = Self::parse_string_offset(input, row_data)?;
        let (input, unknown_string7) = Self::parse_string_offset(input, row_data)?;
        let (input, analyze_path) = Self::parse_string_offset(input, row_data)?;
        let (input, analyze_date) = Self::parse_string_offset(input, row_data)?;
        let (input, comment) = Self::parse_string_offset(input, row_data)?;
        let (input, title) = Self::parse_string_offset(input, row_data)?;
        let (input, unknown_string8) = Self::parse_string_offset(input, row_data)?;
        let (input, filename) = Self::parse_string_offset(input, row_data)?;
        let (input, file_path) = Self::parse_string_offset(input, row_data)?;

        Ok((
            input,
            Row::Track {
                unknown1,
                index_shift,
                bitmask,
                sample_rate,
                composer_id,
                file_size,
                unknown2,
                unknown3,
                unknown4,
                artwork_id,
                key_id,
                orig_artist_id,
                label_id,
                remixer_id,
                bitrate,
                track_number,
                tempo,
                genre_id,
                album_id,
                artist_id,
                id,
                disc_number,
                play_count,
                year,
                sample_depth,
                duration,
                unknown5,
                color,
                rating,
                unknown6,
                unknown7,
                isrc,
                unknown_string1,
                unknown_string2,
                unknown_string3,
                unknown_string4,
                message,
                kuvo_public,
                autoload_hotcues,
                unknown_string5,
                unknown_string6,
                date_added,
                release_date,
                mix_name,
                unknown_string7,
                analyze_path,
                analyze_date,
                comment,
                title,
                unknown_string8,
                filename,
                file_path,
            },
        ))
    }
}
