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

pub mod string;

use crate::pdb::string::DeviceSQLString;
use crate::util::ColorIndex;
use binrw::{binread, binrw, io::SeekFrom, BinRead, BinResult, FilePtr16, ReadOptions};
use std::io::{Read, Seek};

/// Do not read anything, but the return the current stream position of `reader`.
fn current_offset<R: Read + Seek>(reader: &mut R, _: &ReadOptions, _: ()) -> BinResult<u64> {
    reader.stream_position().map_err(binrw::Error::Io)
}

/// The type of pages found inside a `Table`.
#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[brw(little)]
pub enum PageType {
    /// Holds rows of track metadata, such as title, artist, genre, artwork ID, playing time, etc.
    #[brw(magic = 0u32)]
    Tracks,
    /// Holds rows of musical genres, for reference by tracks and searching.
    #[brw(magic = 1u32)]
    Genres,
    /// Holds rows of artists, for reference by tracks and searching.
    #[brw(magic = 2u32)]
    Artists,
    /// Holds rows of albums, for reference by tracks and searching.
    #[brw(magic = 3u32)]
    Albums,
    /// Holds rows of music labels, for reference by tracks and searching.
    #[brw(magic = 4u32)]
    Labels,
    /// Holds rows of musical keys, for reference by tracks, searching, and key matching.
    #[brw(magic = 5u32)]
    Keys,
    /// Holds rows of color labels, for reference  by tracks and searching.
    #[brw(magic = 6u32)]
    Colors,
    /// Holds rows that describe the hierarchical tree structure of available playlists and folders
    /// grouping them.
    #[brw(magic = 7u32)]
    PlaylistTree,
    /// Holds rows that links tracks to playlists, in the right order.
    #[brw(magic = 8u32)]
    PlaylistEntries,
    /// Holds rows of history playlists, i.e. playlists that are recorded every time the device is
    /// mounted by a player.
    #[brw(magic = 11u32)]
    HistoryPlaylists,
    /// Holds rows that links tracks to history playlists, in the right order.
    #[brw(magic = 12u32)]
    HistoryEntries,
    /// Holds rows pointing to album artwork images.
    #[brw(magic = 13u32)]
    Artwork,
    /// Holds information used by rekordbox to synchronize history playlists (not yet studied).
    #[brw(magic = 19u32)]
    History,
    /// Unknown Page type.
    Unknown(u32),
}

/// Points to a table page and can be used to calculate the page's file offset by multiplying it
/// with the page size (found in the file header).
#[binrw]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[brw(little)]
pub struct PageIndex(u32);

impl PageIndex {
    /// Calculate the absolute file offset of the page in the PDB file for the given `page_size`.
    #[must_use]
    pub fn offset(&self, page_size: u32) -> u64 {
        u64::from(self.0) * u64::from(page_size)
    }
}

/// Tables are linked lists of pages containing rows of a single type, which are organized
/// into groups.
#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[brw(little)]
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

/// The PDB header structure, including the list of tables.
#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[brw(little)]
pub struct Header {
    /// Unknown purpose, perhaps an unoriginal signature, seems to always have the value 0.
    #[br(temp, assert(unknown1 == 0))]
    #[bw(calc = 0u32)]
    unknown1: u32,
    /// Size of a single page in bytes.
    ///
    /// The byte offset of a page can be calculated by multiplying a page index with this value.
    pub page_size: u32,
    /// Number of tables.
    #[br(temp)]
    #[bw(calc = tables.len().try_into().expect("too many tables"))]
    num_tables: u32,
    /// Unknown field, not used as any `empty_candidate`, points past end of file.
    #[allow(dead_code)]
    next_unused_page: PageIndex,
    /// Unknown field.
    #[allow(dead_code)]
    unknown: u32,
    /// Always incremented by at least one, sometimes by two or three.
    pub sequence: u32,
    /// The gap seems to be always zero.
    #[br(temp, assert(gap == 0))]
    #[bw(calc = 0u32)]
    gap: u32,
    /// Each table is a linked list of pages containing rows of a particular type.
    #[br(count = num_tables)]
    pub tables: Vec<Table>,
}

impl Header {
    /// Returns pages for the given Table.
    pub fn read_pages<R: Read + Seek>(
        &self,
        reader: &mut R,
        ro: &ReadOptions,
        args: (&PageIndex, &PageIndex),
    ) -> BinResult<Vec<Page>> {
        let (first_page, last_page) = args;

        let mut pages = vec![];
        let mut page_index = first_page.clone();
        loop {
            let page_offset = SeekFrom::Start(page_index.offset(self.page_size));
            reader.seek(page_offset).map_err(binrw::Error::Io)?;
            let page = Page::read_options(reader, ro, (self.page_size,))?;
            let is_last_page = &page.page_index == last_page;
            page_index = page.next_page.clone();
            pages.push(page);

            if is_last_page {
                break;
            }
        }
        Ok(pages)
    }
}

/// A table page.
///
/// Each page consists of a header that contains information about the type, number of rows, etc.,
/// followed by the data section that holds the row data. Each row needs to be located using an
/// offset found in the page footer at the end of the page.
///
/// **Note: The `Page` struct is currently not writable, because row offsets are not taken into
/// account and rows are not serialized correctly yet.**
#[binread]
#[derive(Debug, PartialEq, Clone)]
#[br(little, magic = 0u32)]
#[br(import(page_size: u32))]
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

    #[br(temp)]
    #[br(calc = if num_rows_large > num_rows_small.into() && num_rows_large != 0x1fff { num_rows_large } else { num_rows_small.into() })]
    num_rows: u16,

    #[br(temp)]
    // TODO: Use `num_rows.div_ceil(RowGroup::MAX_ROW_COUNT)` here when it becomes available
    // (currently nightly-only, see https://github.com/rust-lang/rust/issues/88581).
    #[br(calc = if num_rows > 0 { (num_rows - 1) / RowGroup::MAX_ROW_COUNT + 1 } else { 0 })]
    num_row_groups: u16,

    #[br(temp)]
    #[br(calc = SeekFrom::Current(i64::from(page_size) - i64::try_from(Self::HEADER_SIZE).unwrap() - i64::from(num_rows) * 2 - i64::from(num_row_groups) * 4))]
    row_groups_offset: SeekFrom,

    /// Row groups belonging to this page.
    #[br(seek_before(row_groups_offset), restore_position)]
    #[br(parse_with = Self::parse_row_groups, args(num_rows, num_row_groups))]
    pub row_groups: Vec<RowGroup>,
}

impl Page {
    /// Size of the page header in bytes.
    pub const HEADER_SIZE: usize = 0x28;

    /// Parse the row groups at the end of the page.
    fn parse_row_groups<R: Read + Seek>(
        reader: &mut R,
        ro: &ReadOptions,
        args: (u16, u16),
    ) -> BinResult<Vec<RowGroup>> {
        let (num_rows, num_row_groups) = args;
        if num_row_groups == 0 {
            return Ok(vec![]);
        }

        let mut row_groups = Vec::with_capacity(num_row_groups.into());

        // Calculate number of rows in last row group
        let mut num_rows_in_last_row_group = num_rows % RowGroup::MAX_ROW_COUNT;
        if num_rows_in_last_row_group == 0 {
            num_rows_in_last_row_group = RowGroup::MAX_ROW_COUNT;
        }

        // Read last row group
        let row_group = RowGroup::read_options(reader, ro, (num_rows_in_last_row_group,))?;
        row_groups.push(row_group);

        // Read remaining row groups
        for _ in 1..num_row_groups {
            let row_group = RowGroup::read_options(reader, ro, (RowGroup::MAX_ROW_COUNT,))?;
            row_groups.insert(0, row_group);
        }

        Ok(row_groups)
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
        // TODO: Use `num_rows.div_ceil(RowGroup::MAX_ROW_COUNT)` here when it becomes available
        // (currently nightly-only, see https://github.com/rust-lang/rust/issues/88581).
        if num_rows > 0 {
            (num_rows - 1) / RowGroup::MAX_ROW_COUNT + 1
        } else {
            0
        }
    }
}

/// An offset which points to a row in the table, whose actual presence is controlled by one of the
/// bits in `row_present_flags`. This instance allows the row itself to be lazily loaded, unless it
/// is not present, in which case there is no content to be loaded.
#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[brw(little)]
pub struct RowOffset(pub u16);

/// A group of row indices, which are built backwards from the end of the page. Holds up to sixteen
/// row offsets, along with a bit mask that indicates whether each row is actually present in the
/// table.
#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[brw(little)]
#[br(import(num_rows: u16))]
pub struct RowGroup {
    #[br(count = num_rows)]
    rows: Vec<RowOffset>,
    row_presence_flags: u16,
    /// Unknown field, probably padding.
    #[br(temp)]
    #[bw(calc = 0)]
    padding: u16,
}

impl RowGroup {
    const MAX_ROW_COUNT: u16 = 16;

    /// Return the ordered list of row offsets that are actually present.
    pub fn present_rows(&self) -> impl Iterator<Item = &RowOffset> {
        self.rows
            .iter()
            .rev()
            .enumerate()
            .filter_map(|(i, offset)| {
                if (self.row_presence_flags & (1 << i)) != 0 {
                    Some(offset)
                } else {
                    None
                }
            })
    }
}

/// A table row contains the actual data.
#[binread]
#[derive(Debug, PartialEq, Clone)]
#[br(little)]
#[br(import(page_type: PageType))]
// The large enum size is unfortunate, but since users of this library will probably use iterators
// to consume the results on demand, we can live with this. The alternative of using a `Box` would
// require a heap allocation per row, which is arguably worse. Hence, the warning is disabled for
// this enum.
#[allow(clippy::large_enum_variant)]
pub enum Row {
    /// Contains the album name, along with an ID of the corresponding artist.
    #[br(pre_assert(page_type == PageType::Albums))]
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
    #[br(pre_assert(page_type == PageType::Artists))]
    Artist {
        /// Determines if the `name` string is located at the 8-bit offset (0x60) or the 16-bit offset (0x64).
        subtype: u16,
        /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
        index_shift: u16,
        /// ID of this row.
        id: u32,
        /// Unknown field.
        unknown1: u8,
        /// One-byte name offset used if `subtype` is `0x60`.
        ofs_name_near: u8,
        /// Two-byte name offset used if `subtype` is `0x64`.
        ///
        /// In that case, the value of `ofs_name_near` is ignored
        #[br(if(subtype == 0x64))]
        ofs_name_far: Option<u16>,
        /// Name of this artist.
        name: DeviceSQLString,
    },
    /// Contains the artwork path and ID.
    #[br(pre_assert(page_type == PageType::Artwork))]
    Artwork {
        /// ID of this row.
        id: u32,
        /// Path to the album art file.
        path: DeviceSQLString,
    },
    /// Contains numeric color ID
    #[br(pre_assert(page_type == PageType::Colors))]
    Color {
        /// Unknown field.
        unknown1: u32,
        /// Unknown field.
        unknown2: u8,
        /// Numeric color ID
        color: ColorIndex,
        /// Unknown field.
        unknown3: u16,
        /// User-defined name of the color.
        name: DeviceSQLString,
    },
    /// Represents a musical genre.
    #[br(pre_assert(page_type == PageType::Genres))]
    Genre {
        /// ID of this row.
        id: u32,
        /// Name of the genre.
        name: DeviceSQLString,
    },
    /// Represents a history playlist.
    #[br(pre_assert(page_type == PageType::HistoryPlaylists))]
    HistoryPlaylist {
        /// ID of this row.
        id: u32,
        /// Name of the playlist.
        name: DeviceSQLString,
    },
    /// Represents a history playlist.
    #[br(pre_assert(page_type == PageType::HistoryEntries))]
    HistoryEntry {
        /// ID of the track played at this position in the playlist.
        track_id: u32,
        /// ID of the history playlist.
        playlist_id: u32,
        /// Position within the playlist.
        entry_index: u32,
    },
    /// Represents a musical key.
    #[br(pre_assert(page_type == PageType::Keys))]
    Key {
        /// ID of this row.
        id: u32,
        /// Apparently a second copy of the row ID.
        id2: u32,
        /// Name of the key.
        name: DeviceSQLString,
    },
    /// Represents a record label.
    #[br(pre_assert(page_type == PageType::Labels))]
    Label {
        /// ID of this row.
        id: u32,
        /// Name of the record label.
        name: DeviceSQLString,
    },
    /// Represents a node in the playlist tree (either a folder or a playlist).
    #[br(pre_assert(page_type == PageType::PlaylistTree))]
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
    #[br(pre_assert(page_type == PageType::PlaylistEntries))]
    PlaylistEntry {
        /// Position within the playlist.
        entry_index: u32,
        /// ID of the track played at this position in the playlist.
        track_id: u32,
        /// ID of the history playlist.
        playlist_id: u32,
    },
    /// Contains the album name, along with an ID of the corresponding artist.
    #[br(pre_assert(page_type == PageType::Tracks))]
    Track {
        #[br(temp, parse_with = current_offset)]
        #[bw(ignore)]
        base_offset: u64,
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
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        isrc: DeviceSQLString,
        /// Unknown string field.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        unknown_string1: DeviceSQLString,
        /// Unknown string field.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        unknown_string2: DeviceSQLString,
        /// Unknown string field.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        unknown_string3: DeviceSQLString,
        /// Unknown string field.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        unknown_string4: DeviceSQLString,
        /// Unknown string field (named by [@flesniak](https://github.com/flesniak)).
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        message: DeviceSQLString,
        /// Probably describes whether the track is public on kuvo.com (?). Value is either "ON" or empty string.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        kuvo_public: DeviceSQLString,
        /// Determines if hotcues should be autoloaded. Value is either "ON" or empty string.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        autoload_hotcues: DeviceSQLString,
        /// Unknown string field.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        unknown_string5: DeviceSQLString,
        /// Unknown string field (usually empty).
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        unknown_string6: DeviceSQLString,
        /// Date when the track was added to the Rekordbox collection.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        date_added: DeviceSQLString,
        /// Date when the track was released.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        release_date: DeviceSQLString,
        /// Name of the remix (if any).
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        mix_name: DeviceSQLString,
        /// Unknown string field (usually empty).
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        unknown_string7: DeviceSQLString,
        /// File path of the track analysis file.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        analyze_path: DeviceSQLString,
        /// Date when the track analysis was performed.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        analyze_date: DeviceSQLString,
        /// Track comment.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        comment: DeviceSQLString,
        /// Track title.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        title: DeviceSQLString,
        /// Unknown string field (usually empty).
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        unknown_string8: DeviceSQLString,
        /// Name of the file.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        filename: DeviceSQLString,
        /// Path of the file.
        #[br(offset = base_offset, parse_with = FilePtr16::parse)]
        file_path: DeviceSQLString,
    },
    /// The row format (and also its size) is unknown, which means it can't be parsed.
    #[br(pre_assert(matches!(page_type, PageType::History | PageType::Unknown(_))))]
    Unknown,
}
