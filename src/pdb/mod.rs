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

pub mod offset_array;
pub mod string;

use offset_array::OffsetArrayContainer;

#[cfg(test)]
mod test;

use std::convert::TryInto;

use crate::pdb::offset_array::{OffsetArray, OffsetSize};
use crate::pdb::string::DeviceSQLString;
use crate::util::{ColorIndex, ExplicitPadding};
use binrw::{
    binread, binrw,
    io::{Read, Seek, SeekFrom, Write},
    BinRead, BinResult, BinWrite, Endian,
};

/// Do not read anything, but the return the current stream position of `reader`.
fn current_offset<R: Read + Seek>(reader: &mut R, _: Endian, _: ()) -> BinResult<u64> {
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
        _: Endian,
        args: (&PageIndex, &PageIndex),
    ) -> BinResult<Vec<Page>> {
        let endian = Endian::Little;
        let (first_page, last_page) = args;

        let mut pages = vec![];
        let mut page_index = first_page.clone();
        loop {
            let page_offset = SeekFrom::Start(page_index.offset(self.page_size));
            reader.seek(page_offset).map_err(binrw::Error::Io)?;
            let page = Page::read_options(reader, endian, (self.page_size,))?;
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

#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct PageFlags(u8);

impl PageFlags {
    #[must_use]
    pub fn page_has_data(&self) -> bool {
        (self.0 & 0x40) == 0
    }
}

fn write_page_contents<W: Write + Seek>(
    row_groups: &Vec<RowGroup>,
    writer: &mut W,
    endian: Endian,
    args: (u32,),
) -> BinResult<()> {
    let (page_size,) = args;

    let header_end_pos = writer.stream_position()?;

    let mut relative_row_offset: u64 = 0;

    // Seek to the very end of the page
    writer.seek(SeekFrom::Current((page_size - Page::HEADER_SIZE).into()))?;

    for row_group in row_groups {
        relative_row_offset = row_group.write_options_and_get_row_offset(
            writer,
            endian,
            (header_end_pos, relative_row_offset),
        )?;
    }
    Ok(())
}

/// A table page.
///
/// Each page consists of a header that contains information about the type, number of rows, etc.,
/// followed by the data section that holds the row data. Each row needs to be located using an
/// offset found in the page footer at the end of the page.
#[binrw]
#[derive(Debug, PartialEq)]
#[brw(little, import(page_size: u32))]
pub struct Page {
    /// Stream position at the beginning of the page; used to compute heap base for standalone buffers.
    #[br(temp, parse_with = current_offset)]
    #[bw(ignore)]
    page_start_pos: u64,
    /// Magic signature for pages (must be 0).
    #[br(temp, assert(magic == 0u32))]
    #[bw(calc = 0u32)]
    magic: u32,
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
    #[br(temp, calc = Self::calculate_num_rows(num_rows_small, num_rows_large))]
    #[bw(ignore)]
    num_rows: u16,
    /// Number of rows groups in this page.
    ///
    /// **Note:** This is a virtual field and not actually read from the file.
    #[br(temp, calc = num_rows.div_ceil(RowGroup::MAX_ROW_COUNT as u16))]
    #[bw(ignore)]
    num_row_groups: u16,
    /// The offset at which the row data for this page are located.
    ///
    /// **Note:** This is a virtual field and not actually read from the file.
    #[br(temp, calc = page_start_pos + u64::from(Self::HEADER_SIZE))]
    #[bw(ignore)]
    page_heap_offset: u64,
    /// Row groups belonging to this page.
    #[br(seek_before(SeekFrom::Current(Self::heap_padding_size(page_size, num_row_groups).into())))]
    #[br(args {count: num_row_groups.into(), inner: (page_type, page_heap_offset)})]
    #[br(map(|mut vec: Vec<RowGroup>| {vec.reverse(); vec}))]
    #[br(if(num_row_groups > 0 && page_flags.page_has_data()))]
    #[bw(write_with = write_page_contents, args(page_size))]
    pub row_groups: Vec<RowGroup>,
}

impl Page {
    /// Size of the page header in bytes.
    pub const HEADER_SIZE: u32 = 0x28;

    /// Calculate the size of the empty space between the header and the footer.
    fn heap_padding_size(page_size: u32, num_row_groups: u16) -> u32 {
        // Size of all row offsets
        let row_groups_footer_size = u32::from(num_row_groups) * RowGroup::BINARY_SIZE;
        page_size - Self::HEADER_SIZE - row_groups_footer_size
    }

    #[must_use]
    /// Returns `true` if the page actually contains row data.
    pub fn has_data(&self) -> bool {
        self.page_flags.page_has_data()
    }

    #[must_use]
    /// Number of rows on this page.
    ///
    /// Note that this number includes rows that have been flagged as missing by the row group.
    pub fn num_rows(&self) -> u16 {
        Self::calculate_num_rows(self.num_rows_small, self.num_rows_large)
    }

    fn calculate_num_rows(num_rows_small: u8, num_rows_large: u16) -> u16 {
        if num_rows_large > num_rows_small.into() && num_rows_large != 0x1fff {
            num_rows_large
        } else {
            num_rows_small.into()
        }
    }
}

/// A group of row indices, which are built backwards from the end of the page. Holds up to sixteen
/// row offsets, along with a bit mask that indicates whether each row is actually present in the
/// table.
#[binread]
#[br(import(page_type: PageType, page_heap_position: u64))]
#[derive(Debug, Clone)]
pub struct RowGroup {
    /// An offset which points to a row in the table, whose actual presence is controlled by one of the
    /// bits in `row_present_flags`. This instance allows the row itself to be lazily loaded, unless it
    /// is not present, in which case there is no content to be loaded.
    // rustc doesn't seem to recognize that this is used below, ignore for now
    #[allow(dead_code)]
    row_offsets: [u16; Self::MAX_ROW_COUNT],
    row_presence_flags: u16,
    /// Unknown field, probably padding.
    ///
    /// Apparently this is not always zero, so it might also be something different.
    unknown: u16,
    // build rows from offsets collected above
    #[br(seek_before=SeekFrom::Start(page_heap_position))]
    #[br(args(page_type))]
    #[br(parse_with = binrw::file_ptr::parse_from_iter(Self::present_rows_offsets(&row_offsets, row_presence_flags)))]
    #[br(restore_position)] // ensure the parser points to just after this instance, this is important
    /// Access rows in this RowGroup
    rows: Vec<Row>,
}

impl RowGroup {
    const MAX_ROW_COUNT: usize = 16;
    const BINARY_SIZE: u32 = (Self::MAX_ROW_COUNT as u32) * 2 + 4; // size the serialized structure

    /// Get all rows present in this rowgroup
    #[must_use]
    pub fn present_rows(&self) -> &[Row] {
        &self.rows
    }
    // TODO(Swiftb0y): Add a new error category for user APIs and add the correct
    // error herer
    #[allow(clippy::result_unit_err)]
    /// Add a row to this rowgroup
    pub fn add_row(&mut self, row: Row) -> Result<(), ()> {
        if self.rows.len() >= Self::MAX_ROW_COUNT {
            return Err(());
        }
        self.row_presence_flags |= 1 << self.rows.len() as u16;
        self.rows.push(row);
        Ok(())
    }

    fn present_rows_offsets(
        row_offsets: &[u16; Self::MAX_ROW_COUNT],
        row_presence_flags: u16,
    ) -> impl Iterator<Item = u16> + '_ {
        row_offsets
            .iter()
            .rev()
            .enumerate()
            .filter_map(move |(i, row_offset)| {
                (row_presence_flags & (1 << i) != 0).then_some(*row_offset)
            })
    }
}

impl PartialEq for RowGroup {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows
    }
}

impl RowGroup {
    // This helper function now lives in the main impl block for RowGroup
    // Assumes we point just past the rowgroup we're trying to write.
    fn write_options_and_get_row_offset<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: (u64, u64),
    ) -> binrw::BinResult<u64> {
        let (heap_start, relative_row_offset) = args;

        let rows_to_write_count = self.present_rows().len();

        // The number of flags set should match the number of present rows.
        if rows_to_write_count != self.row_presence_flags.count_ones() as usize {
            return Err(binrw::Error::AssertFail {
                pos: heap_start,
                message: "Mismatch between present row count and row_presence_flags".to_string(),
            });
        }

        let rowgroup_start = writer.stream_position()? - u64::from(Self::BINARY_SIZE);

        let free_space_start = heap_start + relative_row_offset;
        const INVALID_ROW_OFFSET: u16 = u16::MAX;
        let mut row_offsets = [INVALID_ROW_OFFSET; Self::MAX_ROW_COUNT];

        // Write rows
        writer.seek(SeekFrom::Start(free_space_start))?;
        for (i, row) in self.present_rows().iter().enumerate() {
            let row_position = writer.stream_position()?;
            let aligned_position = row.align_by(row_position);
            writer.seek(SeekFrom::Start(aligned_position))?;
            row.write_options(writer, endian, ())?;

            let large_offset = aligned_position.checked_sub(heap_start).ok_or_else(|| {
                binrw::Error::AssertFail {
                    pos: aligned_position,
                    message: "Wraparound while calculating row offset".to_string(),
                }
            })?;
            row_offsets[i] = large_offset
                .try_into()
                .map_err(|error| binrw::Error::AssertFail {
                    pos: aligned_position,
                    message: format!("Error converting offset: {:?}", error),
                })?;
        }
        let written_space_end = writer.stream_position()?;
        writer.seek(SeekFrom::Start(rowgroup_start))?;

        // Write the offsets in reverse order, which matches the file format.
        for offset in row_offsets.into_iter().rev() {
            if offset == INVALID_ROW_OFFSET {
                // Just skip the row, don't write zeros to it as
                // there may be valid content there
                writer.seek_relative(2)?;
            } else {
                offset.write_options(writer, endian, ())?;
            }
        }
        self.row_presence_flags.write_options(writer, endian, ())?;
        self.unknown.write_options(writer, endian, ())?;
        // Seek back to the beginning of this rowgroup (which is the end of the next rowgroup)
        writer.seek(SeekFrom::Start(rowgroup_start))?;

        Ok(written_space_end - heap_start)
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
    name: DeviceSQLString,
}

/// Contains the album name, along with an ID of the corresponding artist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Album {
    /// Unknown field, usually `80 00`.
    subtype: Subtype,
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
    /// The offsets and its data and the end of this row
    #[brw(args(20, subtype.get_offset_size(), ()))]
    offsets: OffsetArrayContainer<TrailingName, 2>,
    /// Explicit padding, used to align rows in a page (manually)
    padding: ExplicitPadding,
}

/// Contains the artist name and ID.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Artist {
    /// Determines if the `name` string is located at the 8-bit offset (0x60) or the 16-bit offset (0x64).
    subtype: Subtype,
    /// Unknown field, called `index_shift` by [@flesniak](https://github.com/flesniak).
    index_shift: u16,
    /// ID of this row.
    id: ArtistId,
    /// offsets at the row end
    #[brw(args(8, subtype.get_offset_size(), ()))]
    offsets: OffsetArrayContainer<TrailingName, 2>,
    /// Explicit padding, used to align rows in a page (manually)
    #[br(args(0x30))]
    padding: ExplicitPadding,
}

// impl Artist {
//
// This is the auto alignment code for artist rows specifically, it has been retired temporarily
// by manual padding settings. Its stays here commented so it can be reused later.
//
// #[writer(writer: writer, endian: endian)]
// fn write_string_with_padding(str: &DeviceSQLString, subtype: u16) -> BinResult<()> {
//     str.write_options(writer, endian, ())?;
//     let string_end = writer.stream_position()?;

//     let aligned_down = string_end & !0b11;
//     let next_position = match (subtype, string_end % 4) {
//         (0x64, _) => align_by(4, string_end) + 4,
//         (_, 3) => aligned_down + 12,
//         (_, _) => aligned_down + 8,
//     };
//     let total_pad = next_position - string_end;
//     // TODO(Swiftb0y): https://github.com/jam1garner/binrw/discussions/344
//     writer.write_all(&vec![0u8; total_pad as usize])?;
//     Ok(())
// }
// }

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

#[binrw]
#[brw(little)]
#[brw(import(base: i64, offsets: &OffsetArray<23>, _args: ()))]
#[derive(Debug, PartialEq, Clone, Eq)]
/// String fields stored via the offset table in Track rows
pub struct TrackStrings {
    /// International Standard Recording Code (ISRC), in mangled format.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(2))]
    #[bw(write_with = offsets.write_offset(2))]
    isrc: DeviceSQLString,
    /// Unknown string field.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(3))]
    #[bw(write_with = offsets.write_offset(3))]
    unknown_string1: DeviceSQLString,
    /// Unknown string field.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(4))]
    #[bw(write_with = offsets.write_offset(4))]
    unknown_string2: DeviceSQLString,
    /// Unknown string field.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(5))]
    #[bw(write_with = offsets.write_offset(5))]
    unknown_string3: DeviceSQLString,
    /// Unknown string field.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(6))]
    #[bw(write_with = offsets.write_offset(6))]
    unknown_string4: DeviceSQLString,
    /// Unknown string field (named by [@flesniak](https://github.com/flesniak)).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(7))]
    #[bw(write_with = offsets.write_offset(7))]
    message: DeviceSQLString,
    /// Probably describes whether the track is public on kuvo.com (?). Value is either "ON" or empty string.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(8))]
    #[bw(write_with = offsets.write_offset(8))]
    kuvo_public: DeviceSQLString,
    /// Determines if hotcues should be autoloaded. Value is either "ON" or empty string.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(9))]
    #[bw(write_with = offsets.write_offset(9))]
    autoload_hotcues: DeviceSQLString,
    /// Unknown string field.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(10))]
    #[bw(write_with = offsets.write_offset(10))]
    unknown_string5: DeviceSQLString,
    /// Unknown string field (usually empty).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(11))]
    #[bw(write_with = offsets.write_offset(11))]
    unknown_string6: DeviceSQLString,
    /// Date when the track was added to the Rekordbox collection.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(12))]
    #[bw(write_with = offsets.write_offset(12))]
    date_added: DeviceSQLString,
    /// Date when the track was released.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(13))]
    #[bw(write_with = offsets.write_offset(13))]
    release_date: DeviceSQLString,
    /// Name of the remix (if any).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(14))]
    #[bw(write_with = offsets.write_offset(14))]
    mix_name: DeviceSQLString,
    /// Unknown string field (usually empty).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(15))]
    #[bw(write_with = offsets.write_offset(15))]
    unknown_string7: DeviceSQLString,
    /// File path of the track analysis file.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(16))]
    #[bw(write_with = offsets.write_offset(16))]
    analyze_path: DeviceSQLString,
    /// Date when the track analysis was performed.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(17))]
    #[bw(write_with = offsets.write_offset(17))]
    analyze_date: DeviceSQLString,
    /// Track comment.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(18))]
    #[bw(write_with = offsets.write_offset(18))]
    comment: DeviceSQLString,
    /// Track title.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(19))]
    #[bw(write_with = offsets.write_offset(19))]
    title: DeviceSQLString,
    /// Unknown string field (usually empty).
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(20))]
    #[bw(write_with = offsets.write_offset(20))]
    unknown_string8: DeviceSQLString,
    /// Name of the file.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(21))]
    #[bw(write_with = offsets.write_offset(21))]
    filename: DeviceSQLString,
    /// Path of the file.
    #[brw(args(base, ()))]
    #[br(parse_with = offsets.read_offset(22))]
    #[bw(write_with = offsets.write_offset(22))]
    file_path: DeviceSQLString,
}

/// Contains the album name, along with an ID of the corresponding artist.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct Track {
    /// Unknown field, usually `24 00`.
    subtype: Subtype,
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
    #[brw(args(0x5A, subtype.get_offset_size(), ()))]
    offsets: OffsetArrayContainer<TrackStrings, 23>,
    // Track paddings in general seem to follow this odd formula.
    // A similar oddity is the case with other rows employing an OffsetArray
    // (though with different padding_base)
    // let mut padding_base = 0x34;
    // // This is a heuristic that seems to match the padding behavior of the
    // // original file for the `track_page` test case. The actual logic
    // // is unknown.
    // // We're assigning a different padding base for even and odd tracks
    // if self.id.0 % 2 == 0 {
    //     padding_base += 4;
    // }
    // padding_base = ((end_of_row + padding_base) & !0b11) - end_of_row;
    // writer.seek(SeekFrom::Current(padding_base as i64))?;
    #[br(args(0x40))]
    padding: ExplicitPadding,
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

impl Row {
    #[must_use]
    const fn align_by(&self, offset: u64) -> u64 {
        use crate::pdb::Row::*;
        use crate::util::align_by;
        use std::mem::align_of_val;
        // unfortunately I couldn't find any less copy-pastey way of doing this
        // without unnecessarily complex macros.
        match &self {
            Album(_) => offset,
            Artist(_) => offset,
            Artwork(_) => align_by(4, offset),
            Color(_) => align_by(4, offset),
            ColumnEntry(r) => align_by(align_of_val(r) as u64, offset),
            Genre(_) => align_by(4, offset), // fixed alignment to 4 bytes
            HistoryPlaylist(r) => align_by(align_of_val(r) as u64, offset),
            HistoryEntry(r) => align_by(align_of_val(r) as u64, offset),
            Key(_) => align_by(4, offset),
            Label(_) => align_by(4, offset),
            PlaylistTreeNode(_) => align_by(4, offset),
            PlaylistEntry(r) => align_by(align_of_val(r) as u64, offset),
            Track(_) => offset, // already handled by track serialization
            Unknown => offset,
        }
    }
}
