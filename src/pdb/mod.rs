// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
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
pub mod io;
pub mod offset_array;
pub mod string;

use bitfields::{PackedRowCounts, PageFlags};
use offset_array::{OffsetArrayContainer, OffsetArrayItems};

#[cfg(test)]
mod test_roundtrip;

#[cfg(test)]
mod test_modification;

use std::collections::BTreeMap;
use std::fmt;

use crate::pdb::ext::{ExtPageType, ExtRow};
use crate::pdb::offset_array::OffsetSize;
use crate::pdb::string::DeviceSQLString;
use crate::util::{parse_at_offsets, write_at_offsets, ColorIndex, FileType, TableIndex};
use binrw::{binrw, BinRead, BinResult, BinWrite, Endian};
use std::io::{Read, Seek, SeekFrom, Write};
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
    /// Holds information about synchronization of the USB with Rekordbox or a device.
    #[brw(magic = 19u32)]
    History,
}

/// A row variant that can be extracted from a generic `Row`.
pub trait RowVariant {
    /// The page type that contains rows of this variant.
    const PAGE_TYPE: PageType;

    /// Extracts a reference to this row variant from a generic `Row`.
    fn from_row(row: &Row) -> Option<&Self>;
    /// Extracts a mutable reference to this row variant from a generic `Row`.
    fn from_row_mut(row: &mut Row) -> Option<&mut Self>;
}

/// Points to a table page and can be used to calculate the page's file offset by multiplying it
/// with the page size (found in the file header).
#[binrw]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash)]
#[brw(little)]
pub struct PageIndex(pub(crate) u32);

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
    /// Finds the table for a given page type.
    #[must_use]
    pub fn find_table(&self, page_type: PageType) -> Option<(TableIndex, &Table)> {
        self.tables
            .iter()
            .enumerate()
            .find(|(_, table)| table.page_type == page_type)
            .map(|(i, table)| (TableIndex::from(i), table))
    }

    /// Finds the table for a given page type.
    #[must_use]
    pub fn find_table_mut(&mut self, page_type: PageType) -> Option<(TableIndex, &mut Table)> {
        self.tables
            .iter_mut()
            .enumerate()
            .find(|(_, table)| table.page_type == page_type)
            .map(|(i, table)| (TableIndex::from(i), table))
    }
}

/// An entry in an index page.
#[binrw]
#[derive(PartialEq, Eq, Clone, Copy)]
#[brw(little)]
pub struct IndexEntry(u32);

impl IndexEntry {
    /// Size of the index entry in bytes.
    pub const BINARY_SIZE: u32 = 4;
}

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

impl IndexPageHeader {
    /// Size of the index page header in bytes.
    pub const BINARY_SIZE: u32 = 28;
}

/// The content of an index page.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[br(little)]
#[bw(little, import { page_size: u32 })]
pub struct IndexPageContent {
    /// The header of the index page.
    pub header: IndexPageHeader,

    /// The index entries.
    #[br(count = header.num_entries)]
    pub entries: Vec<IndexEntry>,

    // Write empty entries to pad out the rest of the page, except the last
    // 20 bytes which are zeros instead.
    #[br(temp)]
    #[bw(calc = EmptyIndexEntries(
        Self::total_entries(page_size) - usize::from(header.num_entries)
    ))]
    #[bw(pad_after = 20)]
    _empty_entries: EmptyIndexEntries,
}

impl IndexPageContent {
    fn total_entries(page_size: u32) -> usize {
        // The last 20 bytes in an index page are zeros.
        let entries_space = page_size - PageHeader::BINARY_SIZE - IndexPageHeader::BINARY_SIZE - 20;
        (entries_space / IndexEntry::BINARY_SIZE)
            .try_into()
            .unwrap()
    }
}

/// Helper struct to write empty index entries while reading nothing.
struct EmptyIndexEntries(usize);

impl BinRead for EmptyIndexEntries {
    type Args<'a> = ();

    fn read_options<Reader>(_: &mut Reader, _: Endian, (): Self::Args<'_>) -> BinResult<Self>
    where
        Reader: Read + Seek,
    {
        Ok(Self(0))
    }
}

impl BinWrite for EmptyIndexEntries {
    type Args<'a> = ();

    fn write_options<Writer>(
        &self,
        writer: &mut Writer,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<()>
    where
        Writer: Write + Seek,
    {
        const EMPTY: IndexEntry = IndexEntry::empty();
        for _ in 0..self.0 {
            EMPTY.write_options(writer, endian, ())?;
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
    #[br(pre_assert(!header.page_flags.is_index_page()))]
    Data(
        #[br(args { page_size, page_header: header })]
        #[bw(args { page_size })]
        DataPageContent,
    ),
    /// The page is an index page.
    #[br(pre_assert(header.page_flags.is_index_page()))]
    Index(#[bw(args { page_size })] IndexPageContent),
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

    /// Returns a reference to the data content of the page if it is a data page.
    #[must_use]
    pub fn as_data(&self) -> Option<&DataPageContent> {
        match self {
            PageContent::Data(data) => Some(data),
            _ => None,
        }
    }

    /// Returns a mutable reference to the data content of the page if it is a data page.
    #[must_use]
    pub fn as_data_mut(&mut self) -> Option<&mut DataPageContent> {
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

    /// Returns a reference to the index content of the page if it is an index page.
    #[must_use]
    pub fn as_index(&self) -> Option<&IndexPageContent> {
        match self {
            PageContent::Index(index) => Some(index),
            _ => None,
        }
    }

    /// Returns a mutable reference to the index content of the page if it is an index page.
    #[must_use]
    pub fn as_index_mut(&mut self) -> Option<&mut IndexPageContent> {
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
#[derive(Debug, PartialEq, Clone)]
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

impl Page {
    /// Allocate space for a new row in the page heap and return a function to
    /// insert the row at the allocated offset. Returns `None` if there is
    /// insufficient free space in the page.
    ///
    /// We do this allocate-then-insert dance so that we only take ownership of
    /// the Row once we know we can insert it, avoiding unnecessary copies.
    pub fn allocate_row<'a>(&'a mut self, bytes: u16) -> Option<impl FnOnce(Row) + 'a> {
        match self.content {
            PageContent::Index(_) => None,
            PageContent::Data(ref mut dpc) => {
                // Always align rows to 4 bytes.
                let bytes = bytes.next_multiple_of(4);

                // Assume the upper bound of required space.
                let required_bytes = bytes + RowGroup::HEADER_SIZE + RowGroup::OFFSET_SIZE;
                if self.header.free_size < required_bytes {
                    return None;
                }

                let offset = self.header.used_size;
                self.header.used_size += bytes;
                self.header.free_size -= bytes;
                let row_counts = &mut self.header.packed_row_counts;
                row_counts.increment_num_rows();
                let (row_group_index, row_subindex) = row_counts.last_row_index().unwrap();

                if dpc.row_groups.get(row_group_index as usize).is_none() {
                    dpc.row_groups.push(RowGroup::empty());
                    self.header.free_size -= RowGroup::HEADER_SIZE;
                }

                let row_group = dpc.row_groups.get_mut(row_group_index as usize).unwrap();
                row_group.insert_offset(row_subindex, offset);
                self.header.free_size -= RowGroup::OFFSET_SIZE;

                let rows = &mut dpc.rows;

                Some(move |row: Row| {
                    let prev_entry = rows.insert(offset, row);
                    if let Some(prev_entry) = prev_entry {
                        panic!(
                            "Offset {} was already occupied by row {:?}",
                            offset, prev_entry
                        );
                    }
                    row_group.mark_offset_present(row_subindex);
                    row_counts.increment_num_rows_valid();
                })
            }
        }
    }
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
    // Seek to the end of the page as we read/write row groups backwards,
    // but restore the position after to read/write the actual rows.
    #[brw(seek_before(SeekFrom::Current(Self::page_heap_size(page_size) as i64)), restore_position)]
    #[br(args {count: page_header.packed_row_counts.num_row_groups().into()})]
    pub row_groups: Vec<RowGroup>,

    /// Rows belonging to this page by the heap offset at which each is stored.
    ///
    /// The offsets here should match those in `row_groups`.
    #[br(args(page_header.page_type))]
    #[br(parse_with = parse_at_offsets(row_groups.iter().flat_map(RowGroup::present_rows_offsets)))]
    // `write_at_offsets` restores the writer position after writing.
    #[bw(write_with = write_at_offsets)]
    #[br(assert(rows.len() == page_header.packed_row_counts.num_rows_valid().into(), "parsing page {:?}: num_rows_valid {} does not match parsed row count {}", page_header.page_index, page_header.packed_row_counts.num_rows_valid(), rows.len()))]
    pub rows: BTreeMap<u16, Row>,

    // Seek to the end of the data content area.
    //
    // It's tempting to use `bw(align_after = ...)` or `bw(pad_size_to = ...)`
    // instead of this manual seek, but those cause binrw to write zeros over the top
    // of the row groups and rows above!
    #[br(temp)]
    #[bw(calc = ())]
    #[brw(seek_before = SeekFrom::Current(Self::page_heap_size(page_size) as i64))]
    _dummy: (),
}

impl DataPageContent {
    fn page_heap_size(page_size: u32) -> u32 {
        page_size - PageHeader::BINARY_SIZE - DataPageHeader::BINARY_SIZE
    }
}

// Usage of PageHeapObject is coming in a future PR.
#[allow(dead_code)]
trait PageHeapObject {
    type Args<'a>;

    /// Required page heap space in bytes to store the object.
    fn heap_bytes_required(&self, args: Self::Args<'_>) -> u16;
}

impl PageHeapObject for u8 {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        std::mem::size_of::<u8>() as u16
    }
}

impl PageHeapObject for u16 {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        std::mem::size_of::<u16>() as u16
    }
}

impl PageHeapObject for u32 {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        std::mem::size_of::<u32>() as u16
    }
}

impl PageHeapObject for ColorIndex {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        (0u8).heap_bytes_required(())
    }
}

impl PageHeapObject for FileType {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        (0u16).heap_bytes_required(())
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
    #[bw(write_with = Self::write_row_offsets, args(*row_presence_flags))]
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
    /// Maximum number of rows in a row group.
    pub const MAX_ROW_COUNT: usize = 16;
    const HEADER_SIZE: u16 = 4; // row_presence_flags and unknown fields.
    const OFFSET_SIZE: u16 = 2;
    const BINARY_SIZE: u16 = (Self::MAX_ROW_COUNT as u16) * Self::OFFSET_SIZE + Self::HEADER_SIZE;

    fn empty() -> Self {
        Self {
            row_offsets: [0; Self::MAX_ROW_COUNT],
            row_presence_flags: 0,
            unknown: 0,
        }
    }

    fn present_rows_offsets(&self) -> impl Iterator<Item = u16> + '_ {
        self.row_offsets
            .iter()
            .rev()
            .enumerate()
            .filter_map(move |(i, row_offset)| {
                (self.row_presence_flags & (1 << i) != 0).then_some(*row_offset)
            })
    }

    fn write_row_offsets<Writer>(
        row_offsets: &[u16; 16],
        writer: &mut Writer,
        endian: Endian,
        (row_presence_flags,): (u16,),
    ) -> BinResult<()>
    where
        Writer: Write + Seek,
    {
        const U16_SIZE: u32 = std::mem::size_of::<u16>() as u32;
        let skip = row_presence_flags.leading_zeros();
        writer.seek(SeekFrom::Current((skip * U16_SIZE).into()))?;
        for offset in row_offsets.iter().skip(skip.try_into().unwrap()) {
            offset.write_options(writer, endian, ())?;
        }
        Ok(())
    }

    /// Insert a row offset into the group at the given subindex
    /// but do not mark it present yet (see `mark_offset_present`).
    fn insert_offset(&mut self, subindex: u16, offset: u16) {
        self.row_offsets[(Self::MAX_ROW_COUNT as u16 - 1 - subindex) as usize] = offset;
    }

    /// Mark an inserted row offset as present.
    fn mark_offset_present(&mut self, subindex: u16) {
        self.row_presence_flags |= 1 << subindex;
    }
}

impl Default for RowGroup {
    fn default() -> Self {
        Self::empty()
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

impl PageHeapObject for Subtype {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

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

impl PageHeapObject for TrackId {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

/// Identifies an artwork item.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct ArtworkId(pub u32);

impl PageHeapObject for ArtworkId {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

/// Identifies an album.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct AlbumId(pub u32);

impl PageHeapObject for AlbumId {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

/// Identifies an artist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct ArtistId(pub u32);

impl PageHeapObject for ArtistId {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

/// Identifies a genre.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct GenreId(pub u32);

impl PageHeapObject for GenreId {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

/// Identifies a key.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct KeyId(pub u32);

impl PageHeapObject for KeyId {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

/// Identifies a label.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct LabelId(pub u32);

impl PageHeapObject for LabelId {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

/// Identifies a playlist tree node.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct PlaylistTreeNodeId(pub u32);

impl PageHeapObject for PlaylistTreeNodeId {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

/// Identifies a history playlist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct HistoryPlaylistId(pub u32);

impl PageHeapObject for HistoryPlaylistId {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        self.0.heap_bytes_required(())
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
/// Represents a trailing name field at the end of a row, used for album and artist names.
pub struct TrailingName {
    /// The name a the end of the row this is used in
    pub name: DeviceSQLString,
}

impl OffsetArrayItems<1> for TrailingName {
    type Item = DeviceSQLString;

    fn as_items(&self) -> [&Self::Item; 1] {
        [&self.name]
    }

    fn from_items(items: [Self::Item; 1]) -> Self {
        let [name] = items;
        Self { name }
    }
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
    offsets: OffsetArrayContainer<TrailingName, 1>,
}

impl RowVariant for Album {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Albums);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::Album(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::Album(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for Album {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.subtype.heap_bytes_required(()),
            self.index_shift.heap_bytes_required(()),
            self.unknown2.heap_bytes_required(()),
            self.artist_id.heap_bytes_required(()),
            self.id.heap_bytes_required(()),
            self.unknown3.heap_bytes_required(()),
            self.offsets
                .heap_bytes_required(self.subtype.get_offset_size()),
        ]
        .iter()
        .sum()
    }
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
    pub offsets: OffsetArrayContainer<TrailingName, 1>,
}

impl RowVariant for Artist {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Artists);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::Artist(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::Artist(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for Artist {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.subtype.heap_bytes_required(()),
            self.index_shift.heap_bytes_required(()),
            self.id.heap_bytes_required(()),
            self.offsets
                .heap_bytes_required(self.subtype.get_offset_size()),
        ]
        .iter()
        .sum()
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

impl RowVariant for Artwork {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Artwork);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::Artwork(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::Artwork(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for Artwork {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.id.heap_bytes_required(()),
            self.path.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
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

impl RowVariant for Color {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Colors);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::Color(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::Color(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for Color {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.unknown1.heap_bytes_required(()),
            self.unknown2.heap_bytes_required(()),
            self.color.heap_bytes_required(()),
            self.unknown3.heap_bytes_required(()),
            self.name.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
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

impl RowVariant for Genre {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Genres);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::Genre(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::Genre(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for Genre {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.id.heap_bytes_required(()),
            self.name.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
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

impl RowVariant for HistoryPlaylist {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::HistoryPlaylists);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::HistoryPlaylist(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::HistoryPlaylist(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for HistoryPlaylist {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.id.heap_bytes_required(()),
            self.name.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
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

impl RowVariant for HistoryEntry {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::HistoryEntries);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::HistoryEntry(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::HistoryEntry(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for HistoryEntry {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.track_id.heap_bytes_required(()),
            self.playlist_id.heap_bytes_required(()),
            self.entry_index.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
}

/// Represents a sync log row, used to track synchronization events.
///
/// At least one entry is written every time a synchronization event (e.g. a track is added
/// or removed) occurs. Previous rows are marked as deleted, but rekordbox doesn't write over
/// them, so they can be used to track the history of changes to the database.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct History {
    /// Subtype field, in this case usually `80 02` (hex) or `640` (decimal).
    subtype: Subtype,
    /// Unknown field. I'm assuming this is the `index_shift` found in other row types.
    index_shift: u16,
    /// Tracks present in the database after this sync event.
    num_tracks: u32,
    // Magic value, always zero.
    #[brw(magic = 0u32)]
    /// Sync date, e.g. "2022-02-02".
    date: DeviceSQLString,
    // Magic value, always `7705` -> `0x1E19`.
    #[brw(magic = 0x1E19u16)]
    /// Format/protocol version string. In all known exports this is the string "1000".
    // We could make this magic, but for now this seems fine.
    version: DeviceSQLString,
    /// Device or backup label. Can be empty.
    label: DeviceSQLString,
}

impl PageHeapObject for History {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.subtype.heap_bytes_required(()),
            self.index_shift.heap_bytes_required(()),
            self.num_tracks.heap_bytes_required(()),
            (0u32).heap_bytes_required(()),
            self.date.heap_bytes_required(()),
            (0u16).heap_bytes_required(()),
            self.version.heap_bytes_required(()),
            self.label.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
}

impl RowVariant for History {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::History);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::History(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::History(row)) => Some(row),
            _ => None,
        }
    }
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

impl RowVariant for Key {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Keys);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::Key(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::Key(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for Key {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.id.heap_bytes_required(()),
            self.id2.heap_bytes_required(()),
            self.name.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
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

impl RowVariant for Label {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Labels);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::Label(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::Label(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for Label {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.id.heap_bytes_required(()),
            self.name.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
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

impl RowVariant for PlaylistTreeNode {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::PlaylistTree);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::PlaylistTreeNode(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::PlaylistTreeNode(row)) => Some(row),
            _ => None,
        }
    }
}

impl PlaylistTreeNode {
    /// Indicates whether the node is a folder or a playlist.
    #[must_use]
    pub fn is_folder(&self) -> bool {
        self.node_is_folder > 0
    }
}

impl PageHeapObject for PlaylistTreeNode {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.parent_id.heap_bytes_required(()),
            self.unknown.heap_bytes_required(()),
            self.sort_order.heap_bytes_required(()),
            self.id.heap_bytes_required(()),
            self.node_is_folder.heap_bytes_required(()),
            self.name.heap_bytes_required(()),
        ]
        .iter()
        .sum()
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

impl RowVariant for PlaylistEntry {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::PlaylistEntries);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::PlaylistEntry(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::PlaylistEntry(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for PlaylistEntry {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.entry_index.heap_bytes_required(()),
            self.track_id.heap_bytes_required(()),
            self.playlist_id.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
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

impl RowVariant for ColumnEntry {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Columns);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::ColumnEntry(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::ColumnEntry(row)) => Some(row),
            _ => None,
        }
    }
}

impl PageHeapObject for ColumnEntry {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.id.heap_bytes_required(()),
            self.unknown0.heap_bytes_required(()),
            self.column_name.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
/// String fields stored via the offset table in Track rows
pub struct TrackStrings {
    /// International Standard Recording Code (ISRC), in mangled format.
    isrc: DeviceSQLString,
    /// Lyricist of the track.
    lyricist: DeviceSQLString,
    /// Unknown string field containing a number.
    /// Appears to increment when the track is exported or modified in Rekordbox.
    unknown_string2: DeviceSQLString,
    /// Unknown string field containing a number.
    unknown_string3: DeviceSQLString,
    /// Unknown string field.
    unknown_string4: DeviceSQLString,
    /// Track "message", a field in the Rekordbox UI.
    message: DeviceSQLString,
    /// "Publish track information" in Rekordbox, value is either "ON" or empty string.
    /// Appears related to the Stagehand product to control DJ equipment remotely.
    publish_track_information: DeviceSQLString,
    /// Determines if hotcues should be autoloaded. Value is either "ON" or empty string.
    autoload_hotcues: DeviceSQLString,
    /// Unknown string field (usually empty).
    unknown_string5: DeviceSQLString,
    /// Unknown string field (usually empty).
    unknown_string6: DeviceSQLString,
    /// Date when the track was added to the Rekordbox collection (YYYY-MM-DD).
    date_added: DeviceSQLString,
    /// Date when the track was released (YYYY-MM-DD).
    release_date: DeviceSQLString,
    /// Name of the remix (if any).
    mix_name: DeviceSQLString,
    /// Unknown string field (usually empty).
    unknown_string7: DeviceSQLString,
    /// File path of the track analysis file.
    analyze_path: DeviceSQLString,
    /// Date when the track analysis was performed (YYYY-MM-DD).
    analyze_date: DeviceSQLString,
    /// Track comment.
    comment: DeviceSQLString,
    /// Track title.
    pub title: DeviceSQLString,
    /// Unknown string field (usually empty).
    unknown_string8: DeviceSQLString,
    /// Name of the file.
    filename: DeviceSQLString,
    /// Path of the file.
    pub file_path: DeviceSQLString,
}

impl OffsetArrayItems<21> for TrackStrings {
    type Item = DeviceSQLString;

    fn as_items(&self) -> [&Self::Item; 21] {
        [
            &self.isrc,
            &self.lyricist,
            &self.unknown_string2,
            &self.unknown_string3,
            &self.unknown_string4,
            &self.message,
            &self.publish_track_information,
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
    }

    fn from_items(items: [Self::Item; 21]) -> Self {
        let [isrc, lyricist, unknown_string2, unknown_string3, unknown_string4, message, publish_track_information, autoload_hotcues, unknown_string5, unknown_string6, date_added, release_date, mix_name, unknown_string7, analyze_path, analyze_date, comment, title, unknown_string8, filename, file_path] =
            items;
        Self {
            isrc,
            lyricist,
            unknown_string2,
            unknown_string3,
            unknown_string4,
            message,
            publish_track_information,
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
        }
    }
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
    pub rating: u8,
    /// Format of the file.
    file_type: FileType,
    /// offsets (strings) at row end
    #[brw(args(0x5C, subtype.get_offset_size(), ()))]
    pub offsets: OffsetArrayContainer<TrackStrings, 21>,
}

impl RowVariant for Track {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Tracks);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::Track(track)) => Some(track),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::Track(track)) => Some(track),
            _ => None,
        }
    }
}

impl PageHeapObject for Track {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.subtype.heap_bytes_required(()),
            self.index_shift.heap_bytes_required(()),
            self.bitmask.heap_bytes_required(()),
            self.sample_rate.heap_bytes_required(()),
            self.composer_id.heap_bytes_required(()),
            self.file_size.heap_bytes_required(()),
            self.unknown2.heap_bytes_required(()),
            self.unknown3.heap_bytes_required(()),
            self.unknown4.heap_bytes_required(()),
            self.artwork_id.heap_bytes_required(()),
            self.key_id.heap_bytes_required(()),
            self.orig_artist_id.heap_bytes_required(()),
            self.label_id.heap_bytes_required(()),
            self.remixer_id.heap_bytes_required(()),
            self.bitrate.heap_bytes_required(()),
            self.track_number.heap_bytes_required(()),
            self.tempo.heap_bytes_required(()),
            self.genre_id.heap_bytes_required(()),
            self.album_id.heap_bytes_required(()),
            self.artist_id.heap_bytes_required(()),
            self.id.heap_bytes_required(()),
            self.disc_number.heap_bytes_required(()),
            self.play_count.heap_bytes_required(()),
            self.year.heap_bytes_required(()),
            self.sample_depth.heap_bytes_required(()),
            self.duration.heap_bytes_required(()),
            self.unknown5.heap_bytes_required(()),
            self.color.heap_bytes_required(()),
            self.rating.heap_bytes_required(()),
            self.file_type.heap_bytes_required(()),
            self.offsets
                .heap_bytes_required(self.subtype.get_offset_size()),
        ]
        .iter()
        .sum()
    }
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

impl PageHeapObject for MenuVisibility {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        (0u8).heap_bytes_required(())
    }
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

impl PageHeapObject for Menu {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        [
            self.category_id.heap_bytes_required(()),
            self.content_pointer.heap_bytes_required(()),
            self.unknown.heap_bytes_required(()),
            self.visibility.heap_bytes_required(()),
            self.sort_order.heap_bytes_required(()),
        ]
        .iter()
        .sum()
    }
}

impl RowVariant for Menu {
    const PAGE_TYPE: PageType = PageType::Plain(PlainPageType::Menu);

    fn from_row(row: &Row) -> Option<&Self> {
        match row {
            Row::Plain(PlainRow::Menu(row)) => Some(row),
            _ => None,
        }
    }
    fn from_row_mut(row: &mut Row) -> Option<&mut Self> {
        match row {
            Row::Plain(PlainRow::Menu(row)) => Some(row),
            _ => None,
        }
    }
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
    // FIXME: Fresh album rows typically have about 6 bytes of padding,
    // presumably to allow edits on DJ gear.
    #[br(pre_assert(page_type == PlainPageType::Albums))]
    Album(Album),
    /// Contains the artist name and ID.
    // FIXME: Fresh artist rows typically have about 6 bytes of padding,
    // presumably to allow edits on DJ gear.
    #[br(pre_assert(page_type == PlainPageType::Artists))]
    Artist(Artist),
    /// Contains the artwork path and ID.
    #[br(pre_assert(page_type == PlainPageType::Artwork))]
    Artwork(Artwork),
    /// Contains numeric color ID
    #[br(pre_assert(page_type == PlainPageType::Colors))]
    Color(Color),
    /// Represents a musical genre.
    #[br(pre_assert(page_type == PlainPageType::Genres))]
    Genre(Genre),
    /// Represents a history playlist.
    #[br(pre_assert(page_type == PlainPageType::HistoryPlaylists))]
    HistoryPlaylist(HistoryPlaylist),
    /// Represents a history playlist.
    #[br(pre_assert(page_type == PlainPageType::HistoryEntries))]
    HistoryEntry(HistoryEntry),
    /// Represents a history sync row.
    #[br(pre_assert(page_type == PlainPageType::History))]
    History(History),
    /// Represents a musical key.
    #[br(pre_assert(page_type == PlainPageType::Keys))]
    Key(Key),
    /// Represents a record label.
    #[br(pre_assert(page_type == PlainPageType::Labels))]
    Label(Label),
    /// Represents a node in the playlist tree (either a folder or a playlist).
    #[br(pre_assert(page_type == PlainPageType::PlaylistTree))]
    PlaylistTreeNode(PlaylistTreeNode),
    /// Represents a track entry in a playlist.
    #[br(pre_assert(page_type == PlainPageType::PlaylistEntries))]
    PlaylistEntry(PlaylistEntry),
    /// Contains the metadata categories by which Tracks can be browsed by.
    #[br(pre_assert(page_type == PlainPageType::Columns))]
    ColumnEntry(ColumnEntry),
    /// Manages the active menus on the CDJ.
    #[br(pre_assert(page_type == PlainPageType::Menu))]
    Menu(Menu),
    /// Contains a track entry.
    // FIXME: Fresh track rows typically have about 48 bytes of padding,
    // presumably to allow edits on DJ gear.
    #[br(pre_assert(page_type == PlainPageType::Tracks))]
    Track(Track),
}

impl PageHeapObject for PlainRow {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        match self {
            PlainRow::Album(album) => album.heap_bytes_required(()),
            PlainRow::Artist(artist) => artist.heap_bytes_required(()),
            PlainRow::Artwork(artwork) => artwork.heap_bytes_required(()),
            PlainRow::Color(color) => color.heap_bytes_required(()),
            PlainRow::Genre(genre) => genre.heap_bytes_required(()),
            PlainRow::HistoryPlaylist(history_playlist) => history_playlist.heap_bytes_required(()),
            PlainRow::HistoryEntry(history_entry) => history_entry.heap_bytes_required(()),
            PlainRow::History(history) => history.heap_bytes_required(()),
            PlainRow::Key(key) => key.heap_bytes_required(()),
            PlainRow::Label(label) => label.heap_bytes_required(()),
            PlainRow::PlaylistTreeNode(playlist_tree_node) => {
                playlist_tree_node.heap_bytes_required(())
            }
            PlainRow::PlaylistEntry(playlist_entry) => playlist_entry.heap_bytes_required(()),
            PlainRow::ColumnEntry(column_entry) => column_entry.heap_bytes_required(()),
            PlainRow::Menu(menu) => menu.heap_bytes_required(()),
            PlainRow::Track(track) => track.heap_bytes_required(()),
        }
    }
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
//
// FIXME: Rows must always be aligned to 4 bytes, and certain row types typically have padding
// after each row too (see PlainRow). This is irrelevant while we write rows to precise offsets
// but needs to be considered when we generate row offsets from scratch.
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
    #[br(pre_assert(matches!(page_type, PageType::Unknown(_))))]
    Unknown,
}

impl Row {
    /// Attempt to convert this row into a reference to the given variant type.
    #[must_use]
    pub fn as_variant<T: RowVariant>(&self) -> Option<&T> {
        T::from_row(self)
    }
    /// Attempt to convert this row into a mutable reference to the given variant type.
    #[must_use]
    pub fn as_variant_mut<T: RowVariant>(&mut self) -> Option<&mut T> {
        T::from_row_mut(self)
    }
}

impl PageHeapObject for Row {
    type Args<'a> = ();
    fn heap_bytes_required(&self, _: ()) -> u16 {
        match self {
            Row::Plain(plain_row) => plain_row.heap_bytes_required(()),
            Row::Ext(ext_row) => ext_row.heap_bytes_required(()),
            Row::Unknown => panic!("Unable to determine required bytes for unknown row type"),
        }
    }
}
