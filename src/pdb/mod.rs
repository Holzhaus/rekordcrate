// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
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

pub mod bitfields;
pub mod ext;
pub mod offset_array;
pub mod string;

use bitfields::PackedRowCounts;
use offset_array::OffsetArrayContainer;

#[cfg(test)]
mod test;

use std::collections::BTreeMap;
use std::fmt;

use crate::pdb::ext::{ExtPageType, ExtRow};
use crate::pdb::offset_array::{OffsetArray, OffsetSize};
use crate::pdb::string::DeviceSQLString;
use crate::util::{parse_at_offsets, write_at_offsets, ColorIndex, FileType};
use binrw::{
    binread, binrw,
    io::{Read, Seek, SeekFrom, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use thiserror::Error;

/// An error that can occur when parsing a PDB file.
#[derive(Debug, Error)]
pub enum PdbError {
    /// An invalid value was passed when creating a `PageIndex`.
    #[error("Invalid page index value: {0:#X}")]
    InvalidPageIndex(u32),
    /// Invalid flags were passed when creating an `IndexEntry`.
    #[error("Invalid index flags (expected max 3 bits): {0:#b}")]
    InvalidIndexFlags(u8),
    /// A row was added to a full `RowGroup`.
    #[error("Cannot add row to a full row group (max 16 rows)")]
    RowGroupFull,
}

/// The type of the database were looking at.
/// This influences the meaning of the the pagetypes found in tables.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum DatabaseType {
    #[default] // use plain by default for use of migration
    /// Standard export.pdb files.
    Plain,
    /// Extended exportExt.pdb files.
    Ext,
}

/// The type of pages found inside a `Table`.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[brw(little)]
#[br(import(db_type: DatabaseType))]
pub enum PageType {
    #[br(pre_assert(db_type == DatabaseType::Plain))]
    /// Pagetypes present in `export.pdb` files.
    Plain(PlainPageType),
    #[br(pre_assert(db_type == DatabaseType::Ext))]
    /// Pagetypes present in `exportExt.pdb` files.
    Ext(ExtPageType),
    /// Unknown page type.
    Unknown(u32),
}

/// The type of pages found inside a `Table` of export.pdb files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[brw(little)]
pub enum PlainPageType {
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
    /// Manages the active menus on the CDJ.
    #[brw(magic = 17u32)]
    Menu,
    /// Holds information used by rekordbox to synchronize history playlists (not yet studied).
    #[brw(magic = 19u32)]
    History,
}

/// Points to a table page and can be used to calculate the page's file offset by multiplying it
/// with the page size (found in the file header).
#[binrw]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
#[brw(little)]
pub struct PageIndex(u32);

impl TryFrom<u32> for PageIndex {
    type Error = PdbError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value < 0x03FF_FFFF {
            Ok(Self(value))
        } else {
            Err(PdbError::InvalidPageIndex(value))
        }
    }
}

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
#[brw(import(db_type: DatabaseType))]
pub struct Table {
    /// Identifies the type of rows that this table contains.
    #[br(args(db_type))]
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
#[brw(import(db_type: DatabaseType))]
pub struct Header {
    // Unknown purpose, perhaps an unoriginal signature, seems to always have the value 0.
    #[brw(magic = 0u32)]
    /// Size of a single page in bytes.
    ///
    /// The byte offset of a page can be calculated by multiplying a page index with this value.
    pub page_size: u32,
    /// Number of tables.
    pub num_tables: u32,
    /// Unknown field, not used as any `empty_candidate`, points past end of file.
    pub next_unused_page: PageIndex,
    /// Unknown field.
    pub unknown: u32,
    /// Always incremented by at least one, sometimes by two or three.
    pub sequence: u32,
    // The gap seems to be always zero.
    #[brw(magic = 0u32)]
    /// Each table is a linked list of pages containing rows of a particular type.
    #[br(count = num_tables, args {inner: (db_type,)})]
    #[bw(args(db_type))]
    pub tables: Vec<Table>,
}

impl Header {
    /// Returns pages for the given Table.
    pub fn read_pages<R: Read + Seek>(
        &self,
        reader: &mut R,
        _: Endian,
        args: (&PageIndex, &PageIndex, DatabaseType),
    ) -> BinResult<Vec<Page>> {
        let endian = Endian::Little;
        let (first_page, last_page, db_type) = args;

        let mut pages = vec![];
        let mut page_index = first_page.clone();
        loop {
            let page_offset = SeekFrom::Start(page_index.offset(self.page_size));
            reader.seek(page_offset).map_err(binrw::Error::Io)?;
            let page = Page::read_options(reader, endian, (self.page_size, db_type))?;
            let is_last_page = &page.header.page_index == last_page;
            page_index = page.header.next_page.clone();
            pages.push(page);

            if is_last_page {
                break;
            }
        }
        Ok(pages)
    }
}

/// Bit flags describing various page properties.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PageFlags(u8);

impl PageFlags {
    /// Check whether the page contains data rows.
    #[must_use]
    pub fn page_has_data(&self) -> bool {
        (self.0 & 0x40) == 0
    }

    /// Check whether the page contains "index" rows.
    #[must_use]
    pub fn is_index_page(&self) -> bool {
        self.0 == 0x64
    }
}

/// An entry in an index page.
#[binrw]
#[derive(PartialEq, Eq, Clone, Copy)]
#[brw(little)]
pub struct IndexEntry(u32);

impl TryFrom<(PageIndex, u8)> for IndexEntry {
    type Error = PdbError;

    fn try_from(value: (PageIndex, u8)) -> Result<Self, Self::Error> {
        let (page_index, index_flags) = value;
        if index_flags & 0b111 != index_flags {
            return Err(PdbError::InvalidIndexFlags(index_flags));
        }
        Ok(Self((page_index.0 << 3) | (index_flags & 0b111) as u32))
    }
}

impl IndexEntry {
    /// Returns bits 31-3 as a `PageIndex` which points to a page containing
    /// data rows, with `page_flags=0x34` and same `page_type` as this page.
    pub fn page_index(&self) -> Result<PageIndex, PdbError> {
        PageIndex::try_from(self.0 >> 3)
    }

    /// Returns the index flags from bits 2-0. Their meaning is currently
    /// unknown.
    #[must_use]
    pub fn index_flags(&self) -> u8 {
        (self.0 & 0b111) as u8
    }

    /// Returns `true` if the entry is an empty slot.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0 == 0x1FFF_FFF8
    }

    /// Creates a new empty `IndexEntry`.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0x1FFF_FFF8)
    }
}

impl fmt::Debug for IndexEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.debug_struct("IndexEntry")
                .field("is_empty", &self.is_empty())
                .finish()
        } else {
            f.debug_struct("IndexEntry")
                .field("is_empty", &self.is_empty())
                .field("page_index", &self.page_index().unwrap())
                .field("index_flags", &self.index_flags())
                .finish()
        }
    }
}

/// The header of the index-containing part of a page.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexPageHeader {
    /// Unknown field, usually `0x1fff` or `0x0001`.
    pub unknown_a: u16,
    /// Unknown field, usually `0x1fff` or `0x0000`.
    pub unknown_b: u16,
    // Magic value `0x03ec`.
    #[brw(magic = 0x03ecu16)]
    /// Offset where the next index entry will be written from the beginning
    /// of the entries array, i.e. if this is 4 it means the next entry should
    /// be written at byte `entries+4*4`. We still do not know why this value
    /// is sometimes different than num_entries.
    pub next_offset: u16,
    /// Redundant page index.
    pub page_index: PageIndex,
    /// Redundant next page index.
    pub next_page: PageIndex,
    // Magic value `0x0000000003ffffff`.
    #[brw(magic = 0x0000_0000_03ff_ffffu64)]
    /// Number of index entries in this page.
    pub num_entries: u16,
    /// Points to the first empty index entry, or `0x1fff` if none.
    ///
    /// In real databases, this has been found to be one of three things:
    /// 1. The same value as `num_entries`.
    /// 2. `0x1fff`. We assume this has the same meaning as **1.**
    /// 3. A number smaller than `num_entries`, indicating the first empty
    /// slot.
    pub first_empty: u16,
}

/// The content of an index page.
#[binread]
#[derive(Debug, PartialEq, Eq, Clone)]
#[br(little)]
pub struct IndexPageContent {
    /// The header of the index page.
    pub header: IndexPageHeader,

    /// The index entries.
    #[br(count = header.num_entries)]
    pub entries: Vec<IndexEntry>,
}

impl BinWrite for IndexPageContent {
    type Args<'a> = (u32,);

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        (page_size,): Self::Args<'_>,
    ) -> BinResult<()> {
        let page_content_start_pos = writer.stream_position()?;

        self.header.write_options(writer, endian, ())?;

        for entry in &self.entries {
            entry.write_options(writer, endian, ())?;
        }

        let after_entries_pos = writer.stream_position()?;
        let written_bytes = after_entries_pos - page_content_start_pos;

        let content_size = page_size - PageHeader::BINARY_SIZE;
        let padding_end_offset = content_size - 20;

        // Fill with empty entries (0x1ffffff8) until the last 20 bytes, which
        // are zeroes. If https://github.com/jam1garner/binrw/issues/205 was ever
        // fixed, this entire BinWrite implementation could possibly be removed.

        if written_bytes < u64::from(padding_end_offset) {
            let empty_entries_to_write = (u64::from(padding_end_offset) - written_bytes) / 4;
            let empty_entry = IndexEntry::empty();
            for _ in 0..empty_entries_to_write {
                empty_entry.write_options(writer, endian, ())?;
            }
        }

        let after_padding_pos = writer.stream_position()?;
        let final_padding_bytes =
            content_size as u64 - (after_padding_pos - page_content_start_pos);

        if final_padding_bytes > 0 {
            let zero_padding = vec![0u8; final_padding_bytes as usize];
            writer.write_all(&zero_padding)?;
        }

        Ok(())
    }
}

/// The content of a page, which can be of different types.
///
/// Does not implement `Eq` due to the `Unknown` variant.
#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[br(little, import { page_size: u32, header: &PageHeader })]
#[bw(little, import { page_size: u32 })]
pub enum PageContent {
    /// The page contains data rows.
    #[br(pre_assert(header.page_flags.page_has_data()))]
    Data(
        #[br(args { page_size, page_header: header })]
        #[bw(args { page_size })]
        DataPageContent,
    ),
    /// The page is an index page.
    #[br(pre_assert(header.page_flags.is_index_page()))]
    Index(#[bw(args(page_size,))] IndexPageContent),
    /// The page is of an unknown or unsupported format.
    Unknown,
}

impl PageContent {
    /// Returns the data content of the page if it is a data page.
    #[must_use]
    pub fn into_data(self) -> Option<DataPageContent> {
        match self {
            PageContent::Data(data) => Some(data),
            _ => None,
        }
    }

    /// Returns the index content of the page if it is an index page.
    #[must_use]
    pub fn into_index(self) -> Option<IndexPageContent> {
        match self {
            PageContent::Index(index) => Some(index),
            _ => None,
        }
    }
}

/// The header of a page.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
#[br(import(db_type: DatabaseType))]
pub struct PageHeader {
    // Magic signature for pages (must be 0).
    #[brw(magic = 0u32)]
    /// Index of the page.
    ///
    /// Should match the index used for lookup and can be used to verify that the correct page was loaded.
    pub page_index: PageIndex,
    /// Type of information that the rows of this page contain.
    ///
    /// Should match the page type of the table that this page belongs to.
    #[br(args(db_type))]
    pub page_type: PageType,
    /// Index of the next page with the same page type.
    ///
    /// If this page is the last one of that type, the page index stored in the field will point
    /// past the end of the file.
    pub next_page: PageIndex,
    /// Unknown field.
    /// Appears to be a number between 1 and ~2500.
    pub unknown1: u32,
    /// Unknown field.
    /// Appears to always be zero.
    pub unknown2: u32,
    /// Packed field containing:
    /// - number of used row offsets in the page (13 bits).
    /// - number of valid rows in the page (11 bits).
    pub packed_row_counts: PackedRowCounts,
    /// Page flags.
    ///
    /// According to [@flesniak](https://github.com/flesniak):
    /// > strange pages: 0x44, 0x64; otherwise seen: 0x24, 0x34
    pub page_flags: PageFlags,
    /// Free space in bytes in the data section of the page (excluding the row offsets in the page footer).
    pub free_size: u16,
    /// Used space in bytes in the data section of the page.
    pub used_size: u16,
}

impl PageHeader {
    /// Size of the page header in bytes.
    pub const BINARY_SIZE: u32 = 0x20;
}

/// A table page.
///
/// Each page consists of a header that contains information about the type, number of rows, etc.,
/// followed by the data section that holds the row data. Each row needs to be located using an
/// offset found in the page footer at the end of the page.
#[binrw]
#[derive(Debug, PartialEq)]
#[brw(little)]
#[br(import(page_size: u32, db_type: DatabaseType))]
#[bw(import(page_size: u32))]
pub struct Page {
    /// The page header.
    #[br(args(db_type))]
    pub header: PageHeader,

    /// The content of the page.
    #[br(args { page_size, header: &header })]
    #[bw(args { page_size })]
    pub content: PageContent,
}

/// The header of the data-containing part of a page.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataPageHeader {
    /// Unknown field.
    /// Often 1 or 0x1fff; also observed: 8, 27, 22, 17, 2.
    ///
    /// According to [@flesniak](https://github.com/flesniak):
    /// > (0->1: 2)
    pub unknown5: u16,
    /// Unknown field related to the number of rows in the table,
    /// but not equal to it.
    pub unknown_not_num_rows_large: u16,
    /// Unknown field (usually zero).
    pub unknown6: u16,
    /// Unknown field (usually zero).
    ///
    /// According to [@flesniak](https://github.com/flesniak):
    /// > always 0, except 1 for history pages, num entries for strange pages?"
    /// @RobinMcCorkell: I don't think this is correct, my DB only has zeros for all pages.
    pub unknown7: u16,
}

impl DataPageHeader {
    /// Size of the page header in bytes.
    pub const BINARY_SIZE: u32 = 0x8;
}

/// The data-containing part of a page.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[br(little, import { page_size: u32, page_header: &PageHeader })]
#[bw(little, import { page_size: u32 })]
pub struct DataPageContent {
    /// The header of the data page.
    pub header: DataPageHeader,

    /// Row groups at the end of the page.
    ///
    /// Seek to the end of the page as we read/write row groups backwards,
    /// but restore the position after to read/write the actual rows.
    #[brw(seek_before(SeekFrom::Current(Self::page_heap_size(page_size) as i64)), restore_position)]
    #[br(args {count: page_header.packed_row_counts.num_row_groups().into()})]
    pub row_groups: Vec<RowGroup>,

    /// Rows belonging to this page by the heap offset at which each is stored.
    ///
    /// The offsets here should match those in `row_groups`.
    #[br(args(page_header.page_type))]
    #[br(parse_with = parse_at_offsets(row_groups.iter().flat_map(RowGroup::present_rows_offsets)))]
    #[bw(write_with = write_at_offsets)]
    #[br(assert(rows.len() == page_header.packed_row_counts.num_rows_valid() as usize, "parsing page {:?}: num_rows_valid {} does not match parsed row count {}", page_header.page_index, page_header.packed_row_counts.num_rows_valid(), rows.len()))]
    pub rows: BTreeMap<u16, Row>,
}

impl DataPageContent {
    fn page_heap_size(page_size: u32) -> u32 {
        page_size - PageHeader::BINARY_SIZE - DataPageHeader::BINARY_SIZE
    }
}

/// A group of row indices, which are built backwards from the end of the page. Holds up to sixteen
/// row offsets, along with a bit mask that indicates whether each row is actually present in the
/// table.
#[binrw]
#[derive(Debug, Clone, Eq)]
pub struct RowGroup {
    /// An offset which points to a row in the table, whose actual presence is controlled by one of the
    /// bits in `row_present_flags`. This instance allows the row itself to be lazily loaded, unless it
    /// is not present, in which case there is no content to be loaded.
    ///
    /// Row groups are read backwards so first seek backwards.
    ///
    /// **Note:** Offsets are filled from the end and may only be partially present, i.e. earlier offsets
    /// may be "uninitialized" and used as part of the page heap instead. We only start writing offsets
    /// from the first present row to avoid clobbering page heap data.
    #[brw(seek_before = SeekFrom::Current(-i64::from(Self::BINARY_SIZE)))]
    #[bw(write_with = Self::row_offsets_writer(*row_presence_flags))]
    pub row_offsets: [u16; Self::MAX_ROW_COUNT],
    /// A bit mask that indicates which rows in this group are actually present.
    pub row_presence_flags: u16,
    /// Unknown field.
    /// Often zero, sometimes a multiple of 2, rarely something else.
    /// When a multiple of 2, the set bit often aligns with the last present row
    /// in the group, so maybe this is a bitset like the flags.
    ///
    /// E.g. for a full Artist rowgroup, this is usually zero.
    /// For the last Artist rowgroup in the page with flags 0x003f, this is often 0x0020.
    pub unknown: u16,

    // Seek to the start of the row group to prepare for reading the next one.
    #[br(temp)]
    #[bw(calc = ())]
    #[brw(seek_before = SeekFrom::Current(-i64::from(Self::BINARY_SIZE)))]
    _dummy: (),
}

impl RowGroup {
    const MAX_ROW_COUNT: usize = 16;
    const BINARY_SIZE: u32 = (Self::MAX_ROW_COUNT as u32) * 2 + 4; // size the serialized structure

    fn present_rows_offsets(&self) -> impl Iterator<Item = u16> + '_ {
        self.row_offsets
            .iter()
            .rev()
            .enumerate()
            .filter_map(move |(i, row_offset)| {
                (self.row_presence_flags & (1 << i) != 0).then_some(*row_offset)
            })
    }

    fn row_offsets_writer<Writer>(
        row_presence_flags: u16,
    ) -> impl FnOnce(&[u16; 16], &mut Writer, Endian, ()) -> BinResult<()>
    where
        Writer: Write + Seek,
    {
        move |row_offsets, writer, endian, ()| {
            let skip = row_presence_flags.leading_zeros() as usize;
            writer.seek(SeekFrom::Current(
                (skip * std::mem::size_of::<u16>()) as i64,
            ))?;
            for offset in row_offsets.iter().skip(skip) {
                offset.write_options(writer, endian, ())?;
            }
            Ok(())
        }
    }
}

impl PartialEq for RowGroup {
    fn eq(&self, other: &Self) -> bool {
        self.unknown == other.unknown
            && self.present_rows_offsets().eq(other.present_rows_offsets())
    }
}

/// Carries additional information about a row (if present, always as the first field of a row)
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct Subtype(pub u16);

impl Subtype {
    /// Returns the offset size (`OffsetSize`) used for this subtype.
    ///
    /// If the 0x04 bit is not set in the subtype, returns `OffsetSize::U8`,
    /// otherwise returns `OffsetSize::U16`.
    #[must_use]
    pub fn get_offset_size(&self) -> OffsetSize {
        if self.0 & 0x04 == 0 {
            OffsetSize::U8
        } else {
            OffsetSize::U16
        }
    }
}

/// Identifies a track.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct TrackId(pub u32);

/// Identifies an artwork item.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct ArtworkId(pub u32);

/// Identifies an album.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct AlbumId(pub u32);

/// Identifies an artist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct ArtistId(pub u32);

/// Identifies a genre.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct GenreId(pub u32);

/// Identifies a key.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct KeyId(pub u32);

/// Identifies a label.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct LabelId(pub u32);

/// Identifies a playlist tree node.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct PlaylistTreeNodeId(pub u32);

/// Identifies a history playlist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct HistoryPlaylistId(pub u32);

#[binrw]
#[brw(little)]
#[brw(import(base: i64, offsets: &OffsetArray<2>, args: ()))]
#[derive(Debug, PartialEq, Clone, Eq)]
/// Represents a trailing name field at the end of a row, used for album and artist names.
pub struct TrailingName {
    #[brw(args(base, args))]
    #[br(parse_with = offsets.read_offset(1))]
    #[bw(write_with = offsets.write_offset(1))]
    /// The name a the end of the row this is used in
    pub name: DeviceSQLString,
}

/// Contains the album name, along with an ID of the corresponding artist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Album {
    /// Unknown field, usually `80 00`.
    subtype: Subtype,
    /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
    /// Appears to always be 0x20 * row index.
    index_shift: u16,
    /// Unknown field.
    unknown2: u32,
    /// ID of the artist row associated with this row.
    artist_id: ArtistId,
    /// ID of this row.
    id: AlbumId,
    /// Unknown field.
    unknown3: u32,
    /// The offsets and its data and the end of this row
    #[brw(args(20, subtype.get_offset_size(), ()))]
    offsets: OffsetArrayContainer<TrailingName, 2>,
}

/// Contains the artist name and ID.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Artist {
    /// Determines if the `name` string is located at the 8-bit offset (0x60) or the 16-bit offset (0x64).
    subtype: Subtype,
    /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
    /// Appears to always be 0x20 * row index.
    index_shift: u16,
    /// ID of this row.
    pub id: ArtistId,
    /// offsets at the row end
    #[brw(args(8, subtype.get_offset_size(), ()))]
    pub offsets: OffsetArrayContainer<TrailingName, 2>,
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
    pub entry_index: u32,
    /// ID of the track played at this position in the playlist.
    pub track_id: TrackId,
    /// ID of the playlist.
    pub playlist_id: PlaylistTreeNodeId,
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

#[binrw]
#[brw(little)]
#[brw(import(base: i64, offsets: &OffsetArray<22>, _args: ()))]
#[derive(Debug, PartialEq, Clone, Eq)]
/// String fields stored via the offset table in Track rows
pub struct TrackStrings {
    /// International Standard Recording Code (ISRC), in mangled format.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(1))]
    #[bw(write_with = offsets.write_offset(1))]
    isrc: DeviceSQLString,
    /// Lyricist of the track.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(2))]
    #[bw(write_with = offsets.write_offset(2))]
    lyricist: DeviceSQLString,
    /// Unknown string field containing a number.
    /// Appears to increment when the track is exported or modified in Rekordbox.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(3))]
    #[bw(write_with = offsets.write_offset(3))]
    unknown_string2: DeviceSQLString,
    /// Unknown string field containing a number.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(4))]
    #[bw(write_with = offsets.write_offset(4))]
    unknown_string3: DeviceSQLString,
    /// Unknown string field.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(5))]
    #[bw(write_with = offsets.write_offset(5))]
    unknown_string4: DeviceSQLString,
    /// Track "message", a field in the Rekordbox UI.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(6))]
    #[bw(write_with = offsets.write_offset(6))]
    message: DeviceSQLString,
    /// "Publish track information" in Rekordbox, value is either "ON" or empty string.
    /// Appears related to the Stagehand product to control DJ equipment remotely.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(7))]
    #[bw(write_with = offsets.write_offset(7))]
    publish_track_information: DeviceSQLString,
    /// Determines if hotcues should be autoloaded. Value is either "ON" or empty string.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(8))]
    #[bw(write_with = offsets.write_offset(8))]
    autoload_hotcues: DeviceSQLString,
    /// Unknown string field (usually empty).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(9))]
    #[bw(write_with = offsets.write_offset(9))]
    unknown_string5: DeviceSQLString,
    /// Unknown string field (usually empty).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(10))]
    #[bw(write_with = offsets.write_offset(10))]
    unknown_string6: DeviceSQLString,
    /// Date when the track was added to the Rekordbox collection (YYYY-MM-DD).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(11))]
    #[bw(write_with = offsets.write_offset(11))]
    date_added: DeviceSQLString,
    /// Date when the track was released (YYYY-MM-DD).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(12))]
    #[bw(write_with = offsets.write_offset(12))]
    release_date: DeviceSQLString,
    /// Name of the remix (if any).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(13))]
    #[bw(write_with = offsets.write_offset(13))]
    mix_name: DeviceSQLString,
    /// Unknown string field (usually empty).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(14))]
    #[bw(write_with = offsets.write_offset(14))]
    unknown_string7: DeviceSQLString,
    /// File path of the track analysis file.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(15))]
    #[bw(write_with = offsets.write_offset(15))]
    analyze_path: DeviceSQLString,
    /// Date when the track analysis was performed (YYYY-MM-DD).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(16))]
    #[bw(write_with = offsets.write_offset(16))]
    analyze_date: DeviceSQLString,
    /// Track comment.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(17))]
    #[bw(write_with = offsets.write_offset(17))]
    comment: DeviceSQLString,
    /// Track title.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(18))]
    #[bw(write_with = offsets.write_offset(18))]
    pub title: DeviceSQLString,
    /// Unknown string field (usually empty).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(19))]
    #[bw(write_with = offsets.write_offset(19))]
    unknown_string8: DeviceSQLString,
    /// Name of the file.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(20))]
    #[bw(write_with = offsets.write_offset(20))]
    filename: DeviceSQLString,
    /// Path of the file.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(21))]
    #[bw(write_with = offsets.write_offset(21))]
    pub file_path: DeviceSQLString,
}

/// Contains the album name, along with an ID of the corresponding artist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Track {
    /// Unknown field, usually `24 00`.
    subtype: Subtype,
    /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
    /// Appears to always be 0x20 * row index.
    index_shift: u16,
    /// Unknown field, called `bitmask` by [@flesniak](https://github.com/flesniak).
    /// Appears to always be 0x000c0700.
    bitmask: u32,
    /// Sample Rate in Hz.
    sample_rate: u32,
    /// Composer of this track as artist row ID (non-zero if set).
    composer_id: ArtistId,
    /// File size in bytes.
    file_size: u32,
    /// Unknown field; observed values are effectively random.
    unknown2: u32,
    /// Unknown field; observed values: 19048, 64128, 31844.
    /// Appears to be the same for all tracks in a given DB.
    unknown3: u16,
    /// Unknown field; observed values: 30967, 1511, 9043.
    /// Appears to be the same for all tracks in a given DB.
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
    pub artist_id: ArtistId,
    /// Row ID of this track (non-zero if set).
    pub id: TrackId,
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
    /// Unknown field, apparently always "0x29".
    unknown5: u16,
    /// Color row ID for this track (non-zero if set).
    color: ColorIndex,
    /// User rating of this track (0 to 5 starts).
    rating: u8,
    /// Format of the file.
    file_type: FileType,
    /// offsets (strings) at row end
    #[brw(args(0x5C, subtype.get_offset_size(), ()))]
    pub offsets: OffsetArrayContainer<TrackStrings, 22>,
}

/// Visibility state for a Menu on the CDJ.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[brw(little)]
pub enum MenuVisibility {
    /// The menu is visible.
    #[brw(magic = 0x00u8)]
    Visible,
    /// The menu is hidden.
    #[brw(magic = 0x01u8)]
    Hidden,
    /// Unknown visibility flag.
    Unknown(u8),
}

/// This table defines the active menus on the CDJ.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Menu {
    /// Determines the Label (e.g. "ARTIST").
    /// Matches IDs in the COLUMN table.
    pub category_id: u16,

    /// Points to the data source, i.e. the list of artists is 0x02.
    pub content_pointer: u16,
    /// Unknown field. Swapping values here appears to have no effect on CDJ-350 behavior.
    ///
    /// Some observed values:
    /// - 0x01: Track
    /// - 0x02: Artist
    /// - 0x03: Album
    /// - 0x05: BPM
    /// - 0x63 (99): Generic List (Playlist, Genre, Key, History)
    pub unknown: u8,

    /// Visibility state of the menu item.
    ///
    /// Experiments confirmed that changing this from `Hidden` to `Visible` makes hidden menus
    /// (like Genre) appear, although some menus do not show in the CDJ-350 even when made
    /// visible here.
    pub visibility: MenuVisibility,

    /// Visual position in the menu list.
    /// 0 is valid and places the item at the very top (if visible).
    pub sort_order: u16,
}

/// A table row contains the actual data.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
#[br(import(page_type: PlainPageType))]
// The large enum size is unfortunate, but since users of this library will probably use iterators
// to consume the results on demand, we can live with this. The alternative of using a `Box` would
// require a heap allocation per row, which is arguably worse. Hence, the warning is disabled for
// this enum.
#[allow(clippy::large_enum_variant)]
pub enum PlainRow {
    /// Contains the album name, along with an ID of the corresponding artist.
    ///
    /// Fresh album rows typically have a bit of padding, presumably to allow
    /// edits on DJ gear.
    #[br(pre_assert(page_type == PlainPageType::Albums))]
    Album(#[bw(pad_after = 6, align_after = 4)] Album),
    /// Contains the artist name and ID.
    ///
    /// Fresh artist rows typically have a bit of padding, presumably to allow
    /// edits on DJ gear.
    #[br(pre_assert(page_type == PlainPageType::Artists))]
    Artist(#[bw(pad_after = 6, align_after = 4)] Artist),
    /// Contains the artwork path and ID.
    #[br(pre_assert(page_type == PlainPageType::Artwork))]
    Artwork(#[bw(pad_after = 0, align_after = 4)] Artwork),
    /// Contains numeric color ID
    #[br(pre_assert(page_type == PlainPageType::Colors))]
    Color(#[bw(pad_after = 0, align_after = 4)] Color),
    /// Represents a musical genre.
    #[br(pre_assert(page_type == PlainPageType::Genres))]
    Genre(#[bw(pad_after = 0, align_after = 4)] Genre),
    /// Represents a history playlist.
    #[br(pre_assert(page_type == PlainPageType::HistoryPlaylists))]
    HistoryPlaylist(#[bw(pad_after = 0, align_after = 4)] HistoryPlaylist),
    /// Represents a history playlist.
    #[br(pre_assert(page_type == PlainPageType::HistoryEntries))]
    HistoryEntry(#[bw(pad_after = 0, align_after = 4)] HistoryEntry),
    /// Represents a musical key.
    #[br(pre_assert(page_type == PlainPageType::Keys))]
    Key(#[bw(pad_after = 0, align_after = 4)] Key),
    /// Represents a record label.
    #[br(pre_assert(page_type == PlainPageType::Labels))]
    Label(#[bw(pad_after = 0, align_after = 4)] Label),
    /// Represents a node in the playlist tree (either a folder or a playlist).
    #[br(pre_assert(page_type == PlainPageType::PlaylistTree))]
    PlaylistTreeNode(#[bw(pad_after = 0, align_after = 4)] PlaylistTreeNode),
    /// Represents a track entry in a playlist.
    #[br(pre_assert(page_type == PlainPageType::PlaylistEntries))]
    PlaylistEntry(#[bw(pad_after = 0, align_after = 4)] PlaylistEntry),
    /// Contains the metadata categories by which Tracks can be browsed by.
    #[br(pre_assert(page_type == PlainPageType::Columns))]
    ColumnEntry(#[bw(pad_after = 0, align_after = 4)] ColumnEntry),
    /// Manages the active menus on the CDJ.
    #[br(pre_assert(page_type == PlainPageType::Menu))]
    Menu(#[bw(pad_after = 0, align_after = 4)] Menu),
    /// Contains a track entry.
    ///
    /// Fresh track rows typically have a bit of padding, presumably to allow
    /// edits on DJ gear.
    #[br(pre_assert(page_type == PlainPageType::Tracks))]
    Track(#[bw(pad_after = 0x30, align_after = 4)] Track),
}

/// A table row contains the actual data.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
#[br(import(page_type: PageType))]
// The large enum size is unfortunate, but since users of this library will probably use iterators
// to consume the results on demand, we can live with this. The alternative of using a `Box` would
// require a heap allocation per row, which is arguably worse. Hence, the warning is disabled for
// this enum.
#[allow(clippy::large_enum_variant)]
pub enum Row {
    // TODO(Swiftb0y: come up with something prettier than the match hell below)
    #[br(pre_assert(matches!(page_type, PageType::Plain(_))))]
    /// A row in a "plain" database (export.pdb), which contains one of the known row types.
    Plain(
        #[br(args(match page_type {
            PageType::Plain(v) => v,
            _ => unreachable!("by above pre_assert")
        }))]
        PlainRow,
    ),
    #[br(pre_assert(matches!(page_type, PageType::Ext(_))))]
    /// A row in an "ext" database (exportExt.pdb), which contains extended track information.
    Ext(
        #[br(args(match page_type {
            PageType::Ext(v) => v,
            _ => unreachable!("by above pre_assert")
        }))]
        ExtRow,
    ),
    /// The row format (and also its size) is unknown, which means it can't be parsed.
    #[br(pre_assert(matches!(page_type, PageType::Plain(PlainPageType::History) | PageType::Unknown(_))))]
    Unknown,
}
