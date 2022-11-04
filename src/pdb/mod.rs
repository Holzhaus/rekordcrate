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
use binrw::{
    binread, binrw,
    io::{Read, Seek, SeekFrom, Write},
    BinRead, BinResult, BinWrite, Endian, FilePtr16, FilePtr8, ReadOptions, WriteOptions,
};
use bitflags::bitflags;

/// Do not read anything, but the return the current stream position of `reader`.
fn current_offset<R: Read + Seek>(reader: &mut R, _: &ReadOptions, _: ()) -> BinResult<u64> {
    reader.stream_position().map_err(binrw::Error::Io)
}

/// The type of pages found inside a `Table`.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    /// Contains the metadata categories by which Tracks can be browsed by.
    #[brw(magic = 16u32)]
    Columns,
    /// Holds information used by rekordbox to synchronize history playlists (not yet studied).
    #[brw(magic = 19u32)]
    History,
    /// Unknown Page type.
    Unknown(u32),
}

/// Points to a table page and can be used to calculate the page's file offset by multiplying it
/// with the page size (found in the file header).
#[binrw]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
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
#[derive(Debug, PartialEq, Eq, Clone)]
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
#[derive(Debug, PartialEq, Eq, Clone)]
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

bitflags! {
    /// Flags of a table page.
    #[derive(BinRead, BinWrite)]
    struct PageFlags: u8 {
        /// Determines whether the page is a data page.
        const DATA = 0x40;
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
#[derive(Debug, PartialEq)]
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
    page_flags: PageFlags,
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
    /// Number of rows in this page.
    ///
    /// **Note:** This is a virtual field and not actually read from the file.
    #[br(temp)]
    #[br(calc = if num_rows_large > num_rows_small.into() && num_rows_large != 0x1fff { num_rows_large } else { num_rows_small.into() })]
    num_rows: u16,
    /// Number of rows groups in this page.
    ///
    /// **Note:** This is a virtual field and not actually read from the file.
    #[br(temp)]
    // TODO: Use `num_rows.div_ceil(RowGroup::MAX_ROW_COUNT)` here when it becomes available
    // (currently nightly-only, see https://github.com/rust-lang/rust/issues/88581).
    #[br(calc = if num_rows > 0 { (num_rows - 1) / RowGroup::MAX_ROW_COUNT + 1 } else { 0 })]
    num_row_groups: u16,
    /// The offset at which row groups for this page are located.
    ///
    /// **Note:** This is a virtual field and not actually read from the file.
    #[br(temp)]
    #[br(calc = SeekFrom::Current(i64::from(page_size) - i64::from(Self::HEADER_SIZE) - i64::from(num_rows) * 2 - i64::from(num_row_groups) * 4))]
    row_groups_offset: SeekFrom,
    /// The offset at which the row data for this page are located.
    ///
    /// **Note:** This is a virtual field and not actually read from the file.
    #[br(temp)]
    #[br(calc = page_index.offset(page_size) + u64::from(Self::HEADER_SIZE))]
    page_heap_offset: u64,
    /// Row groups belonging to this page.
    #[br(seek_before(row_groups_offset), restore_position)]
    #[br(parse_with = Self::parse_row_groups, args(page_type, page_heap_offset, num_rows, num_row_groups, page_flags))]
    pub row_groups: Vec<RowGroup>,
}

impl Page {
    /// Size of the page header in bytes.
    pub const HEADER_SIZE: u32 = 0x28;

    /// Parse the row groups at the end of the page.
    fn parse_row_groups<R: Read + Seek>(
        reader: &mut R,
        ro: &ReadOptions,
        args: (PageType, u64, u16, u16, PageFlags),
    ) -> BinResult<Vec<RowGroup>> {
        let (page_type, page_heap_offset, num_rows, num_row_groups, page_flags) = args;
        if num_row_groups == 0 || !page_flags.contains(PageFlags::DATA) {
            return Ok(vec![]);
        }

        let mut row_groups = Vec::with_capacity(num_row_groups.into());

        // Calculate number of rows in last row group
        let mut num_rows_in_last_row_group = num_rows % RowGroup::MAX_ROW_COUNT;
        if num_rows_in_last_row_group == 0 {
            num_rows_in_last_row_group = RowGroup::MAX_ROW_COUNT;
        }

        // Read last row group
        let row_group = RowGroup::read_options(
            reader,
            ro,
            (page_type, page_heap_offset, num_rows_in_last_row_group),
        )?;
        row_groups.push(row_group);

        // Read remaining row groups
        for _ in 1..num_row_groups {
            let row_group = RowGroup::read_options(
                reader,
                ro,
                (page_type, page_heap_offset, RowGroup::MAX_ROW_COUNT),
            )?;
            row_groups.insert(0, row_group);
        }

        Ok(row_groups)
    }

    #[must_use]
    /// Returns `true` if the page actually contains row data.
    pub fn has_data(&self) -> bool {
        self.page_flags.contains(PageFlags::DATA)
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

/// A group of row indices, which are built backwards from the end of the page. Holds up to sixteen
/// row offsets, along with a bit mask that indicates whether each row is actually present in the
/// table.
#[binread]
#[derive(Debug, PartialEq)]
#[brw(little)]
#[br(import(page_type: PageType, page_heap_offset: u64, num_rows: u16))]
pub struct RowGroup {
    /// An offset which points to a row in the table, whose actual presence is controlled by one of the
    /// bits in `row_present_flags`. This instance allows the row itself to be lazily loaded, unless it
    /// is not present, in which case there is no content to be loaded.
    #[br(offset = page_heap_offset, args { count: num_rows.into(), inner: (page_type,) })]
    rows: Vec<FilePtr16<Row>>,
    row_presence_flags: u16,
    /// Unknown field, probably padding.
    ///
    /// Apparently this is not always zero, so it might also be something different.
    unknown: u16,
}

impl RowGroup {
    const MAX_ROW_COUNT: u16 = 16;

    /// Return the ordered list of row offsets that are actually present.
    pub fn present_rows(&self) -> impl Iterator<Item = &Row> {
        self.rows
            .iter()
            .rev()
            .enumerate()
            .filter_map(|(i, row_offset)| {
                if (self.row_presence_flags & (1 << i)) != 0 {
                    row_offset.value.as_ref()
                } else {
                    None
                }
            })
    }
}

/// Identifies a track.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[br(little)]
pub struct TrackId(pub u32);

/// Identifies an artwork item.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[br(little)]
pub struct ArtworkId(pub u32);

/// Identifies an album.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[br(little)]
pub struct AlbumId(pub u32);

/// Identifies an artist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[br(little)]
pub struct ArtistId(pub u32);

/// Identifies a genre.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[br(little)]
pub struct GenreId(pub u32);

/// Identifies a key.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[br(little)]
pub struct KeyId(pub u32);

/// Identifies a label.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[br(little)]
pub struct LabelId(pub u32);

/// Identifies a playlist tree node.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[br(little)]
pub struct PlaylistTreeNodeId(pub u32);

/// Identifies a history playlist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[br(little)]
pub struct HistoryPlaylistId(pub u32);

/// Contains the album name, along with an ID of the corresponding artist.
#[binread]
#[derive(Debug, PartialEq, Eq, Clone)]
#[br(little)]
pub struct Album {
    /// Position of start of this row (needed of offset calculations).
    ///
    /// **Note:** This is a virtual field and not actually read from the file.
    #[br(temp, parse_with = current_offset)]
    #[bw(ignore)]
    base_offset: u64,
    /// Unknown field, usually `80 00`.
    unknown1: u16,
    /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
    index_shift: u16,
    /// Unknown field.
    unknown2: u32,
    /// ID of the artist row associated with this row.
    artist_id: ArtistId,
    /// ID of this row.
    id: AlbumId,
    /// Unknown field.
    unknown3: u32,
    /// Unknown field.
    unknown4: u8,
    /// Album name String
    #[br(offset = base_offset, parse_with = FilePtr8::parse)]
    name: DeviceSQLString,
}

/// Contains the artist name and ID.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Artist {
    /// Determines if the `name` string is located at the 8-bit offset (0x60) or the 16-bit offset (0x64).
    subtype: u16,
    /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
    index_shift: u16,
    /// ID of this row.
    id: ArtistId,
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
    #[br(seek_before = Artist::calculate_name_seek(ofs_name_near, &ofs_name_far))]
    #[bw(seek_before = Artist::calculate_name_seek(*ofs_name_near, ofs_name_far))]
    #[brw(restore_position)]
    name: DeviceSQLString,
}

impl Artist {
    fn calculate_name_seek(ofs_near: u8, ofs_far: &Option<u16>) -> SeekFrom {
        let offset: u16 = ofs_far.map_or_else(|| ofs_near.into(), |v| v - 2) - 10;
        SeekFrom::Current(offset.into())
    }
}

/// Contains the artwork path and ID.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Artwork {
    /// ID of this row.
    id: ArtworkId,
    /// Path to the album art file.
    path: DeviceSQLString,
}

/// Contains numeric color ID
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Color {
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
}

/// Represents a musical genre.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Genre {
    /// ID of this row.
    id: GenreId,
    /// Name of the genre.
    name: DeviceSQLString,
}

/// Represents a history playlist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct HistoryPlaylist {
    /// ID of this row.
    id: HistoryPlaylistId,
    /// Name of the playlist.
    name: DeviceSQLString,
}

/// Represents a history playlist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct HistoryEntry {
    /// ID of the track played at this position in the playlist.
    track_id: TrackId,
    /// ID of the history playlist.
    playlist_id: HistoryPlaylistId,
    /// Position within the playlist.
    entry_index: u32,
}

/// Represents a musical key.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Key {
    /// ID of this row.
    id: KeyId,
    /// Apparently a second copy of the row ID.
    id2: u32,
    /// Name of the key.
    name: DeviceSQLString,
}

/// Represents a record label.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Label {
    /// ID of this row.
    id: LabelId,
    /// Name of the record label.
    name: DeviceSQLString,
}

/// Represents a node in the playlist tree (either a folder or a playlist).
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct PlaylistTreeNode {
    /// ID of parent row of this row (which means that the parent is a folder).
    pub parent_id: PlaylistTreeNodeId,
    /// Unknown field.
    unknown: u32,
    /// Sort order indicastor.
    sort_order: u32,
    /// ID of this row.
    pub id: PlaylistTreeNodeId,
    /// Indicates if the node is a folder. Non-zero if it's a leaf node, i.e. a playlist.
    node_is_folder: u32,
    /// Name of this node, as shown when navigating the menu.
    pub name: DeviceSQLString,
}

impl PlaylistTreeNode {
    /// Indicates whether the node is a folder or a playlist.
    #[must_use]
    pub fn is_folder(&self) -> bool {
        self.node_is_folder > 0
    }
}

/// Represents a track entry in a playlist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct PlaylistEntry {
    /// Position within the playlist.
    entry_index: u32,
    /// ID of the track played at this position in the playlist.
    track_id: TrackId,
    /// ID of the playlist.
    playlist_id: PlaylistTreeNodeId,
}

/// Contains the kinds of Metadata Categories tracks can be browsed by
/// on CDJs.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct ColumnEntry {
    // Possibly the primary key, though I don't know if that would
    // make sense as I don't think there are references to these
    // rows anywhere else. This could be a stable ID to identify
    // a category by in hardware (instead of by name).
    id: u16,
    // Maybe a bitfield containing infos on sort order and which
    // columns are displayed.
    unknown0: u16,
    /// TODO Contained string is prefixed by the "interlinear annotation"
    /// characters "\u{fffa}" and postfixed with "\u{fffb}" for some reason?!
    /// Contained strings are actually `DeviceSQLString::LongBody` even though
    /// they only contain ascii (apart from their unicode annotations)
    // TODO since there are only finite many categories, it would make sense
    // to encode those as an enum as part of the high-level api.
    pub column_name: DeviceSQLString,
}

/// Contains the album name, along with an ID of the corresponding artist.
#[binread]
#[derive(Debug, PartialEq, Eq, Clone)]
#[br(little)]
pub struct Track {
    /// Position of start of this row (needed of offset calculations).
    ///
    /// **Note:** This is a virtual field and not actually read from the file.
    #[br(temp, parse_with = current_offset)]
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
    composer_id: ArtistId,
    /// File size in bytes.
    file_size: u32,
    /// Unknown field (maybe another ID?)
    unknown2: u32,
    /// Unknown field ("always 19048?" according to [@flesniak](https://github.com/flesniak))
    unknown3: u16,
    /// Unknown field ("always 30967?" according to [@flesniak](https://github.com/flesniak))
    unknown4: u16,
    /// Artwork row ID for the cover art (non-zero if set),
    artwork_id: ArtworkId,
    /// Key row ID for the cover art (non-zero if set).
    key_id: KeyId,
    /// Artist row ID of the original performer (non-zero if set).
    orig_artist_id: ArtistId,
    /// Label row ID of the original performer (non-zero if set).
    label_id: LabelId,
    /// Artist row ID of the remixer (non-zero if set).
    remixer_id: ArtistId,
    /// Bitrate of the track.
    bitrate: u32,
    /// Track number of the track.
    track_number: u32,
    /// Track tempo in centi-BPM (= 1/100 BPM).
    tempo: u32,
    /// Genre row ID for this track (non-zero if set).
    genre_id: GenreId,
    /// Album row ID for this track (non-zero if set).
    album_id: AlbumId,
    /// Artist row ID for this track (non-zero if set).
    artist_id: ArtistId,
    /// Row ID of this track (non-zero if set).
    id: TrackId,
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
}

// #[bw(little)] on #[binread] types does
// not seem to work so we manually define the endianness here.
impl binrw::meta::WriteEndian for Track {
    const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::Endian(Endian::Little);
}

impl BinWrite for Track {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        debug_assert!(options.endian() == Endian::Little);

        let base_position = writer.stream_position()?;
        self.unknown1.write_options(writer, options, ())?;
        self.index_shift.write_options(writer, options, ())?;
        self.bitmask.write_options(writer, options, ())?;
        self.sample_rate.write_options(writer, options, ())?;
        self.composer_id.write_options(writer, options, ())?;
        self.file_size.write_options(writer, options, ())?;
        self.unknown2.write_options(writer, options, ())?;
        self.unknown3.write_options(writer, options, ())?;
        self.unknown4.write_options(writer, options, ())?;
        self.artwork_id.write_options(writer, options, ())?;
        self.key_id.write_options(writer, options, ())?;
        self.orig_artist_id.write_options(writer, options, ())?;
        self.label_id.write_options(writer, options, ())?;
        self.remixer_id.write_options(writer, options, ())?;
        self.bitrate.write_options(writer, options, ())?;
        self.track_number.write_options(writer, options, ())?;
        self.tempo.write_options(writer, options, ())?;
        self.genre_id.write_options(writer, options, ())?;
        self.album_id.write_options(writer, options, ())?;
        self.artist_id.write_options(writer, options, ())?;
        self.id.write_options(writer, options, ())?;
        self.disc_number.write_options(writer, options, ())?;
        self.play_count.write_options(writer, options, ())?;
        self.year.write_options(writer, options, ())?;
        self.sample_depth.write_options(writer, options, ())?;
        self.duration.write_options(writer, options, ())?;
        self.unknown5.write_options(writer, options, ())?;
        self.color.write_options(writer, options, ())?;
        self.rating.write_options(writer, options, ())?;
        self.unknown6.write_options(writer, options, ())?;
        self.unknown7.write_options(writer, options, ())?;

        let start_of_string_section = writer.stream_position()?;
        debug_assert_eq!(start_of_string_section - base_position, 0x5e);

        // Skip offsets, because we want to write the actual strings first.
        let mut string_offsets = [0u16; 21];
        writer.seek(SeekFrom::Current(0x2a))?;
        for (i, string) in [
            &self.isrc,
            &self.unknown_string1,
            &self.unknown_string2,
            &self.unknown_string3,
            &self.unknown_string4,
            &self.message,
            &self.kuvo_public,
            &self.autoload_hotcues,
            &self.unknown_string5,
            &self.unknown_string6,
            &self.date_added,
            &self.release_date,
            &self.mix_name,
            &self.unknown_string7,
            &self.analyze_path,
            &self.analyze_date,
            &self.comment,
            &self.title,
            &self.unknown_string8,
            &self.filename,
            &self.file_path,
        ]
        .into_iter()
        .enumerate()
        {
            let current_position = writer.stream_position()?;
            let offset: u16 = current_position
                .checked_sub(base_position)
                .and_then(|v| u16::try_from(v).ok())
                .ok_or_else(|| binrw::Error::AssertFail {
                    pos: current_position,
                    message: "Wraparound while calculating row offset".to_string(),
                })?;
            string_offsets[i] = offset;
            string.write_options(writer, options, ())?;
        }

        let end_of_row = writer.stream_position()?;
        writer.seek(SeekFrom::Start(start_of_string_section))?;
        string_offsets.write_options(writer, options, ())?;
        writer.seek(SeekFrom::Start(end_of_row))?;

        Ok(())
    }
}

/// A table row contains the actual data.
#[binread]
#[derive(Debug, PartialEq, Eq, Clone)]
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
    Album(Album),
    /// Contains the artist name and ID.
    #[br(pre_assert(page_type == PageType::Artists))]
    Artist(Artist),
    /// Contains the artwork path and ID.
    #[br(pre_assert(page_type == PageType::Artwork))]
    Artwork(Artwork),
    /// Contains numeric color ID
    #[br(pre_assert(page_type == PageType::Colors))]
    Color(Color),
    /// Represents a musical genre.
    #[br(pre_assert(page_type == PageType::Genres))]
    Genre(Genre),
    /// Represents a history playlist.
    #[br(pre_assert(page_type == PageType::HistoryPlaylists))]
    HistoryPlaylist(HistoryPlaylist),
    /// Represents a history playlist.
    #[br(pre_assert(page_type == PageType::HistoryEntries))]
    HistoryEntry(HistoryEntry),
    /// Represents a musical key.
    #[br(pre_assert(page_type == PageType::Keys))]
    Key(Key),
    /// Represents a record label.
    #[br(pre_assert(page_type == PageType::Labels))]
    Label(Label),
    /// Represents a node in the playlist tree (either a folder or a playlist).
    #[br(pre_assert(page_type == PageType::PlaylistTree))]
    PlaylistTreeNode(PlaylistTreeNode),
    /// Represents a track entry in a playlist.
    #[br(pre_assert(page_type == PageType::PlaylistEntries))]
    PlaylistEntry(PlaylistEntry),
    /// Contains the metadata categories by which Tracks can be browsed by.
    #[br(pre_assert(page_type == PageType::Columns))]
    ColumnEntry(ColumnEntry),
    /// Contains the album name, along with an ID of the corresponding artist.
    #[br(pre_assert(page_type == PageType::Tracks))]
    Track(Track),
    /// The row format (and also its size) is unknown, which means it can't be parsed.
    #[br(pre_assert(matches!(page_type, PageType::History | PageType::Unknown(_))))]
    Unknown,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::testing::test_roundtrip;

    #[test]
    fn empty_header() {
        let header = Header {
            page_size: 4096,
            next_unused_page: PageIndex(1),
            unknown: 0,
            sequence: 1,
            tables: vec![],
        };
        test_roundtrip(
            &[
                0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
            ],
            header,
        );
    }

    #[test]
    fn demo_tracks_header() {
        let header = Header {
            page_size: 4096,
            next_unused_page: PageIndex(51),
            unknown: 5,
            sequence: 34,
            tables: [
                Table {
                    page_type: PageType::Tracks,
                    empty_candidate: 47,
                    first_page: PageIndex(1),
                    last_page: PageIndex(2),
                },
                Table {
                    page_type: PageType::Genres,
                    empty_candidate: 4,
                    first_page: PageIndex(3),
                    last_page: PageIndex(3),
                },
                Table {
                    page_type: PageType::Artists,
                    empty_candidate: 49,
                    first_page: PageIndex(5),
                    last_page: PageIndex(6),
                },
                Table {
                    page_type: PageType::Albums,
                    empty_candidate: 8,
                    first_page: PageIndex(7),
                    last_page: PageIndex(7),
                },
                Table {
                    page_type: PageType::Labels,
                    empty_candidate: 50,
                    first_page: PageIndex(9),
                    last_page: PageIndex(10),
                },
                Table {
                    page_type: PageType::Keys,
                    empty_candidate: 46,
                    first_page: PageIndex(11),
                    last_page: PageIndex(12),
                },
                Table {
                    page_type: PageType::Colors,
                    empty_candidate: 42,
                    first_page: PageIndex(13),
                    last_page: PageIndex(14),
                },
                Table {
                    page_type: PageType::PlaylistTree,
                    empty_candidate: 16,
                    first_page: PageIndex(15),
                    last_page: PageIndex(15),
                },
                Table {
                    page_type: PageType::PlaylistEntries,
                    empty_candidate: 18,
                    first_page: PageIndex(17),
                    last_page: PageIndex(17),
                },
                Table {
                    page_type: PageType::Unknown(9),
                    empty_candidate: 20,
                    first_page: PageIndex(19),
                    last_page: PageIndex(19),
                },
                Table {
                    page_type: PageType::Unknown(10),
                    empty_candidate: 22,
                    first_page: PageIndex(21),
                    last_page: PageIndex(21),
                },
                Table {
                    page_type: PageType::HistoryPlaylists,
                    empty_candidate: 24,
                    first_page: PageIndex(23),
                    last_page: PageIndex(23),
                },
                Table {
                    page_type: PageType::HistoryEntries,
                    empty_candidate: 26,
                    first_page: PageIndex(25),
                    last_page: PageIndex(25),
                },
                Table {
                    page_type: PageType::Artwork,
                    empty_candidate: 28,
                    first_page: PageIndex(27),
                    last_page: PageIndex(27),
                },
                Table {
                    page_type: PageType::Unknown(14),
                    empty_candidate: 30,
                    first_page: PageIndex(29),
                    last_page: PageIndex(29),
                },
                Table {
                    page_type: PageType::Unknown(15),
                    empty_candidate: 32,
                    first_page: PageIndex(31),
                    last_page: PageIndex(31),
                },
                Table {
                    page_type: PageType::Columns,
                    empty_candidate: 43,
                    first_page: PageIndex(33),
                    last_page: PageIndex(34),
                },
                Table {
                    page_type: PageType::Unknown(17),
                    empty_candidate: 44,
                    first_page: PageIndex(35),
                    last_page: PageIndex(36),
                },
                Table {
                    page_type: PageType::Unknown(18),
                    empty_candidate: 45,
                    first_page: PageIndex(37),
                    last_page: PageIndex(38),
                },
                Table {
                    page_type: PageType::History,
                    empty_candidate: 48,
                    first_page: PageIndex(39),
                    last_page: PageIndex(41),
                },
            ]
            .to_vec(),
        };

        test_roundtrip(
            &[
                0, 0, 0, 0, 0, 16, 0, 0, 20, 0, 0, 0, 51, 0, 0, 0, 5, 0, 0, 0, 34, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 47, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 3,
                0, 0, 0, 3, 0, 0, 0, 2, 0, 0, 0, 49, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 3, 0, 0, 0,
                8, 0, 0, 0, 7, 0, 0, 0, 7, 0, 0, 0, 4, 0, 0, 0, 50, 0, 0, 0, 9, 0, 0, 0, 10, 0, 0,
                0, 5, 0, 0, 0, 46, 0, 0, 0, 11, 0, 0, 0, 12, 0, 0, 0, 6, 0, 0, 0, 42, 0, 0, 0, 13,
                0, 0, 0, 14, 0, 0, 0, 7, 0, 0, 0, 16, 0, 0, 0, 15, 0, 0, 0, 15, 0, 0, 0, 8, 0, 0,
                0, 18, 0, 0, 0, 17, 0, 0, 0, 17, 0, 0, 0, 9, 0, 0, 0, 20, 0, 0, 0, 19, 0, 0, 0, 19,
                0, 0, 0, 10, 0, 0, 0, 22, 0, 0, 0, 21, 0, 0, 0, 21, 0, 0, 0, 11, 0, 0, 0, 24, 0, 0,
                0, 23, 0, 0, 0, 23, 0, 0, 0, 12, 0, 0, 0, 26, 0, 0, 0, 25, 0, 0, 0, 25, 0, 0, 0,
                13, 0, 0, 0, 28, 0, 0, 0, 27, 0, 0, 0, 27, 0, 0, 0, 14, 0, 0, 0, 30, 0, 0, 0, 29,
                0, 0, 0, 29, 0, 0, 0, 15, 0, 0, 0, 32, 0, 0, 0, 31, 0, 0, 0, 31, 0, 0, 0, 16, 0, 0,
                0, 43, 0, 0, 0, 33, 0, 0, 0, 34, 0, 0, 0, 17, 0, 0, 0, 44, 0, 0, 0, 35, 0, 0, 0,
                36, 0, 0, 0, 18, 0, 0, 0, 45, 0, 0, 0, 37, 0, 0, 0, 38, 0, 0, 0, 19, 0, 0, 0, 48,
                0, 0, 0, 39, 0, 0, 0, 41, 0, 0, 0,
            ],
            header,
        );
    }

    #[test]
    fn track_row() {
        let row = Track {
            unknown1: 36,
            index_shift: 160,
            bitmask: 788224,
            sample_rate: 44100,
            composer_id: ArtistId(0),
            file_size: 6899624,
            unknown2: 214020570,
            unknown3: 64128,
            unknown4: 1511,
            artwork_id: ArtworkId(0),
            key_id: KeyId(5),
            orig_artist_id: ArtistId(0),
            label_id: LabelId(1),
            remixer_id: ArtistId(0),
            bitrate: 320,
            track_number: 0,
            tempo: 12800,
            genre_id: GenreId(0),
            album_id: AlbumId(0),
            artist_id: ArtistId(1),
            id: TrackId(1),
            disc_number: 0,
            play_count: 0,
            year: 0,
            sample_depth: 16,
            duration: 172,
            unknown5: 41,
            color: ColorIndex::None,
            rating: 0,
            unknown6: 1,
            unknown7: 3,
            isrc: DeviceSQLString::new_isrc("".to_string()).unwrap(),
            unknown_string1: DeviceSQLString::empty(),
            unknown_string2: DeviceSQLString::new("3".to_string()).unwrap(),
            unknown_string3: DeviceSQLString::new("3".to_string()).unwrap(),
            unknown_string4: DeviceSQLString::empty(),
            message: DeviceSQLString::empty(),
            kuvo_public: DeviceSQLString::empty(),
            autoload_hotcues: DeviceSQLString::new("ON".to_string()).unwrap(),
            unknown_string5: DeviceSQLString::empty(),
            unknown_string6: DeviceSQLString::empty(),
            date_added: DeviceSQLString::new("2018-05-25".to_string()).unwrap(),
            release_date: DeviceSQLString::empty(),
            mix_name: DeviceSQLString::empty(),
            unknown_string7: DeviceSQLString::empty(),
            analyze_path: DeviceSQLString::new(
                "/PIONEER/USBANLZ/P016/0000875E/ANLZ0000.DAT".to_string(),
            )
            .unwrap(),
            analyze_date: DeviceSQLString::new("2022-02-02".to_string()).unwrap(),
            comment: DeviceSQLString::new("Tracks by www.loopmasters.com".to_string()).unwrap(),
            title: DeviceSQLString::new("Demo Track 1".to_string()).unwrap(),
            unknown_string8: DeviceSQLString::empty(),
            filename: DeviceSQLString::new("Demo Track 1.mp3".to_string()).unwrap(),
            file_path: DeviceSQLString::new(
                "/Contents/Loopmasters/UnknownAlbum/Demo Track 1.mp3".to_string(),
            )
            .unwrap(),
        };
        test_roundtrip(
            &[
                36, 0, 160, 0, 0, 7, 12, 0, 68, 172, 0, 0, 0, 0, 0, 0, 168, 71, 105, 0, 218, 177,
                193, 12, 128, 250, 231, 5, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
                0, 64, 1, 0, 0, 0, 0, 0, 0, 0, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 172, 0, 41, 0, 0, 0, 1, 0, 3, 0, 136, 0, 137, 0,
                138, 0, 140, 0, 142, 0, 143, 0, 144, 0, 145, 0, 148, 0, 149, 0, 150, 0, 161, 0,
                162, 0, 163, 0, 164, 0, 208, 0, 219, 0, 249, 0, 6, 1, 7, 1, 24, 1, 3, 3, 5, 51, 5,
                51, 3, 3, 3, 7, 79, 78, 3, 3, 23, 50, 48, 49, 56, 45, 48, 53, 45, 50, 53, 3, 3, 3,
                89, 47, 80, 73, 79, 78, 69, 69, 82, 47, 85, 83, 66, 65, 78, 76, 90, 47, 80, 48, 49,
                54, 47, 48, 48, 48, 48, 56, 55, 53, 69, 47, 65, 78, 76, 90, 48, 48, 48, 48, 46, 68,
                65, 84, 23, 50, 48, 50, 50, 45, 48, 50, 45, 48, 50, 61, 84, 114, 97, 99, 107, 115,
                32, 98, 121, 32, 119, 119, 119, 46, 108, 111, 111, 112, 109, 97, 115, 116, 101,
                114, 115, 46, 99, 111, 109, 27, 68, 101, 109, 111, 32, 84, 114, 97, 99, 107, 32,
                49, 3, 35, 68, 101, 109, 111, 32, 84, 114, 97, 99, 107, 32, 49, 46, 109, 112, 51,
                105, 47, 67, 111, 110, 116, 101, 110, 116, 115, 47, 76, 111, 111, 112, 109, 97,
                115, 116, 101, 114, 115, 47, 85, 110, 107, 110, 111, 119, 110, 65, 108, 98, 117,
                109, 47, 68, 101, 109, 111, 32, 84, 114, 97, 99, 107, 32, 49, 46, 109, 112, 51,
            ],
            row,
        );
    }

    #[test]
    fn artist_row() {
        let row = Artist {
            subtype: 96,
            index_shift: 32,
            id: ArtistId(1),
            unknown1: 3,
            ofs_name_near: 10,
            ofs_name_far: None,
            name: DeviceSQLString::new("Loopmasters".to_string()).unwrap(),
        };
        test_roundtrip(
            &[
                96, 0, 32, 0, 1, 0, 0, 0, 3, 10, 25, 76, 111, 111, 112, 109, 97, 115, 116, 101,
                114, 115,
            ],
            row,
        );
    }

    #[test]
    fn label_row() {
        let row = Label {
            id: LabelId(1),
            name: DeviceSQLString::new("Loopmasters".to_string()).unwrap(),
        };
        test_roundtrip(
            &[
                1, 0, 0, 0, 25, 76, 111, 111, 112, 109, 97, 115, 116, 101, 114, 115,
            ],
            row,
        );
    }

    #[test]
    fn key_row() {
        let row = Key {
            id: KeyId(1),
            id2: 1,
            name: DeviceSQLString::new("Dm".to_string()).unwrap(),
        };
        test_roundtrip(&[1, 0, 0, 0, 1, 0, 0, 0, 7, 68, 109], row);
    }

    #[test]
    fn color_row() {
        let row = Color {
            unknown1: 0,
            unknown2: 1,
            color: ColorIndex::Pink,
            unknown3: 0,
            name: DeviceSQLString::new("Pink".to_string()).unwrap(),
        };
        test_roundtrip(&[0, 0, 0, 0, 1, 1, 0, 0, 11, 80, 105, 110, 107], row);
    }
    #[test]
    fn column_entry() {
        let row = ColumnEntry {
            id: 1,
            unknown0: 128,
            column_name: DeviceSQLString::new("\u{fffa}GENRE\u{fffb}".into()).unwrap(),
        };
        let bin = &[
            0x01, 0x00, 0x80, 0x00, 0x90, 0x12, 0x00, 0x00, 0xfa, 0xff, 0x47, 0x00, 0x45, 0x00,
            0x4e, 0x00, 0x52, 0x00, 0x45, 0x00, 0xfb, 0xff,
        ];
        test_roundtrip(bin, row);
    }
}
