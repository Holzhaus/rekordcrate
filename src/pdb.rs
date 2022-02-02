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
pub struct PageIndex(u32);

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
}
