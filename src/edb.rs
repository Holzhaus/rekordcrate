// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Reading the library that Rekordbox 5 keeps in `datafile.edb`.
//!
//! The library holds its rows in the same format as a device export, down to the DeviceSQL
//! strings and the offset tables of a track row, so the row parsers of the [`crate::pdb`] module
//! read them unchanged. What differs is the container: the rows are not held in the pages of a
//! `export.pdb`, but in a flat heap, with each row preceded by a tag naming its type.
//!
//! How that heap is indexed is not worked out yet, so the rows are found by looking for those
//! tags rather than by following an index. That is enough to read a library, with two
//! consequences worth knowing: rows that Rekordbox has deleted but not overwritten are found as
//! well, and a row whose type is not known is not found at all. Neither keeps a library from
//! being read, but it does mean the counts here can exceed what Rekordbox shows.

use crate::anlz::AnalysisPath;
use crate::pdb::{PageType, PlainPageType, PlaylistTreeNode, Row, Track as TrackRow};
use crate::util::{ColorIndex, FileType, RekordcrateResult};
use binrw::BinRead;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Name of the file holding the library.
const LIBRARY_FILE: &str = "datafile.edb";

/// Tag that a track row is preceded by.
const TRACK_TAG: [u8; 2] = [0xc4, 0x02];

/// Tag that a row of the playlist tree is preceded by.
const PLAYLIST_TAG: [u8; 2] = [0xa0, 0x03];

/// Tag that an entry of a playlist is preceded by.
const ENTRY_TAG: [u8; 2] = [0xc0, 0x03];

/// Length of a tag and the index following it.
///
/// Some row types hold the two as their first fields, in which case the row starts at the tag;
/// the others start after it.
const TAG_SIZE: usize = 4;

/// Offset of the string offset table within a track row, and the value it starts with in every
/// track row we have seen, which tells a row apart from a tag appearing by chance.
const TRACK_OFFSETS_AT: [usize; 2] = [0x5c, 0x5e];
const TRACK_FIRST_OFFSET: u16 = 0x0088;

/// Largest row we try to read, which bounds the work done on a tag that is not a row.
const MAX_ROW_SIZE: usize = 8192;

/// Size of an entry of a playlist, which is three 32-bit numbers.
const ENTRY_SIZE: usize = 12;

/// A track of a Rekordbox 5 library.
///
/// The fields a track row holds as ids of other rows, such as its artist, are left out, because
/// the rows they refer to have not been worked out yet.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryTrack {
    /// Id of the track, which playlists refer to it by.
    pub id: u32,
    /// Title of the track.
    pub title: String,
    /// Path of the audio file on the machine the library belongs to.
    pub file_path: String,
    /// Name of the audio file.
    pub filename: String,
    /// Comment shown for the track.
    pub comment: String,
    /// Date the track was added to the library.
    pub date_added: String,
    /// Date the track was analyzed.
    pub analyze_date: String,
    /// Tempo in hundredths of a beat per minute.
    pub tempo: u32,
    /// Playback duration in seconds.
    pub duration: u16,
    /// Bitrate of the audio file in kbit/s.
    pub bitrate: u32,
    /// Sample rate of the audio file in Hz.
    pub sample_rate: u32,
    /// Bits per sample of the audio file.
    pub sample_depth: u16,
    /// Size of the audio file in bytes.
    pub file_size: u32,
    /// Number of the track within its album.
    pub track_number: u32,
    /// Number of the disc the track is on.
    pub disc_number: u16,
    /// Year the track was released.
    pub year: u16,
    /// Rating from zero to five stars.
    pub rating: u8,
    /// Colour the track is labelled with.
    pub color: ColorIndex,
    /// Format of the audio file.
    pub file_type: FileType,
}

impl LibraryTrack {
    /// Tempo in beats per minute.
    #[must_use]
    pub fn tempo_bpm(&self) -> f32 {
        // A library counts the tempo in hundredths of a beat per minute.
        (f64::from(self.tempo) / 100.0) as f32
    }
}

/// A node of the playlist tree of a Rekordbox 5 library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryPlaylist {
    /// Id of the node, which its children and entries refer to it by.
    pub id: u32,
    /// Id of the node holding this one, or zero for a node at the root of the tree.
    pub parent_id: u32,
    /// Name of the node.
    pub name: String,
    /// Position of the node among the nodes beside it.
    pub sort_order: u32,
    /// What kind of node this is, which Rekordbox calls the attribute of a playlist: zero for a
    /// playlist, one for a folder, and four for a playlist whose content follows a condition.
    pub attribute: u32,
}

impl LibraryPlaylist {
    /// Whether the node holds other nodes rather than tracks.
    #[must_use]
    pub fn is_folder(&self) -> bool {
        self.attribute == 1
    }
}

/// The library that Rekordbox 5 keeps in a directory.
#[derive(Debug)]
pub struct Library {
    root: PathBuf,
    tracks: BTreeMap<u32, LibraryTrack>,
    playlists: Vec<LibraryPlaylist>,
    entries: BTreeMap<u32, Vec<u32>>,
}

impl Library {
    /// Reads the library in the given directory, which is the one holding `datafile.edb`.
    pub fn open(root: impl Into<PathBuf>) -> RekordcrateResult<Self> {
        let root = root.into();
        let data = std::fs::read(root.join(LIBRARY_FILE))?;

        let tracks = read_tracks(&data);
        let playlists = read_playlists(&data);
        let entries = read_entries(
            &data,
            &tracks.keys().copied().collect(),
            &playlists.iter().map(|node| node.id).collect(),
        );

        Ok(Self {
            root,
            tracks,
            playlists,
            entries,
        })
    }

    /// The directory the library was read from.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The nodes of the playlist tree.
    #[must_use]
    pub fn playlists(&self) -> &[LibraryPlaylist] {
        &self.playlists
    }

    /// The tracks of the library, by id.
    #[must_use]
    pub fn tracks(&self) -> &BTreeMap<u32, LibraryTrack> {
        &self.tracks
    }

    /// The node of the playlist tree with the given name, if there is exactly one.
    #[must_use]
    pub fn find_playlist(&self, name: &str) -> Option<&LibraryPlaylist> {
        let mut found = self.playlists.iter().filter(|node| node.name == name);
        let first = found.next()?;
        found.next().is_none().then_some(first)
    }

    /// The tracks of a playlist, in the order they are held in.
    #[must_use]
    pub fn playlist_tracks(&self, playlist_id: u32) -> Vec<&LibraryTrack> {
        self.entries
            .get(&playlist_id)
            .map(|ids| ids.iter().filter_map(|id| self.tracks.get(id)).collect())
            .unwrap_or_default()
    }

    /// The analysis files that the library holds for a track, if they are there.
    ///
    /// The library keeps them under its own `PIONEER/USBANLZ` directory, in the place that
    /// follows from the path of the audio file.
    #[must_use]
    pub fn analysis_files(&self, track: &LibraryTrack) -> Vec<PathBuf> {
        let dir = self
            .root
            .join("PIONEER")
            .join("USBANLZ")
            .join(AnalysisPath::for_audio_path(&track.file_path).relative_dir());
        ["DAT", "EXT", "2EX"]
            .into_iter()
            .map(|extension| dir.join(format!("{}.{extension}", AnalysisPath::FILE_STEM)))
            .filter(|path| path.is_file())
            .collect()
    }
}

/// Reads a row of the given type at an offset, or returns `None` if there is no row there.
///
/// A tag can appear by chance in the middle of something else, in which case the bytes that
/// follow it are not a row at all and reading them fails.
fn read_row_at(data: &[u8], offset: usize, page_type: PageType) -> Option<Row> {
    let end = (offset + MAX_ROW_SIZE).min(data.len());
    let slice = data.get(offset..end)?;
    let mut cursor = Cursor::new(slice);
    Row::read_args(&mut cursor, (page_type,)).ok()
}

/// Finds the offsets of the tags naming rows of the given type.
fn tagged_rows(data: &[u8], tag: [u8; 2]) -> impl Iterator<Item = usize> + '_ {
    (0..data.len().saturating_sub(TAG_SIZE)).filter_map(move |offset| {
        (data[offset..offset + 2] == tag).then_some(())?;
        // The tag is followed by the index of the row within its table, which counts in steps.
        let index = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        index.is_multiple_of(0x20).then_some(offset)
    })
}

/// Reads the tracks of the library.
fn read_tracks(data: &[u8]) -> BTreeMap<u32, LibraryTrack> {
    let mut tracks = BTreeMap::new();
    for offset in tagged_rows(data, TRACK_TAG) {
        // A track row holds a table of the offsets of its strings, which starts with the same
        // value in every library we have seen. Checking it keeps the rest of the work for rows.
        let starts_with_offsets = TRACK_OFFSETS_AT.into_iter().any(|at| {
            data.get(offset + at..offset + at + 2)
                .and_then(|bytes| bytes.try_into().ok())
                .map(|bytes| u16::from_le_bytes(bytes) == TRACK_FIRST_OFFSET)
                .unwrap_or(false)
        });
        if !starts_with_offsets {
            continue;
        }

        let Some(row) = read_row_at(data, offset, PageType::Plain(PlainPageType::Tracks)) else {
            continue;
        };
        let Some(row) = row.as_variant::<TrackRow>() else {
            continue;
        };
        let string = |value: &crate::pdb::string::DeviceSQLString| {
            value.clone().into_string().unwrap_or_default()
        };
        let file_path = string(&row.offsets.file_path);
        if row.id.0 == 0 || !file_path.starts_with('/') {
            continue;
        }

        tracks.insert(
            row.id.0,
            LibraryTrack {
                id: row.id.0,
                title: string(&row.offsets.title),
                file_path,
                filename: string(&row.offsets.filename),
                comment: string(&row.offsets.comment),
                date_added: string(&row.offsets.date_added),
                analyze_date: string(&row.offsets.analyze_date),
                tempo: row.tempo,
                duration: row.duration,
                bitrate: row.bitrate,
                sample_rate: row.sample_rate,
                sample_depth: row.sample_depth,
                file_size: row.file_size,
                track_number: row.track_number,
                disc_number: row.disc_number,
                year: row.year,
                rating: row.rating,
                color: row.color.clone(),
                file_type: row.file_type.clone(),
            },
        );
    }
    tracks
}

/// Reads the playlist tree of the library.
fn read_playlists(data: &[u8]) -> Vec<LibraryPlaylist> {
    let mut playlists: BTreeMap<u32, LibraryPlaylist> = BTreeMap::new();
    for tag in tagged_rows(data, PLAYLIST_TAG) {
        let offset = tag + TAG_SIZE;
        let Some(row) = read_row_at(data, offset, PageType::Plain(PlainPageType::PlaylistTree))
        else {
            continue;
        };
        let Some(row) = row.as_variant::<PlaylistTreeNode>() else {
            continue;
        };
        let Ok(name) = row.name.clone().into_string() else {
            continue;
        };
        if row.id.0 == 0 || name.is_empty() {
            continue;
        }
        playlists.insert(
            row.id.0,
            LibraryPlaylist {
                id: row.id.0,
                parent_id: row.parent_id.0,
                name,
                sort_order: row.sort_order,
                attribute: row.node_is_folder,
            },
        );
    }
    playlists.into_values().collect()
}

/// Reads which tracks the playlists hold.
///
/// An entry holds the position of a track in a playlist, the track, and the playlist. Both of the
/// latter are checked against the rows that were read, which is what tells an entry apart from a
/// tag that appears by chance.
fn read_entries(
    data: &[u8],
    track_ids: &BTreeSet<u32>,
    playlist_ids: &BTreeSet<u32>,
) -> BTreeMap<u32, Vec<u32>> {
    let number = |offset: usize| {
        data.get(offset..offset + 4)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u32::from_le_bytes)
    };

    // Ordered by the position the entry holds, which is what the playlist is played in.
    let mut entries: BTreeMap<u32, BTreeMap<u32, u32>> = BTreeMap::new();
    for tag in tagged_rows(data, ENTRY_TAG) {
        let offset = tag + TAG_SIZE;
        if offset + ENTRY_SIZE > data.len() {
            break;
        }
        let (Some(position), Some(track_id), Some(playlist_id)) =
            (number(offset), number(offset + 4), number(offset + 8))
        else {
            continue;
        };
        if !playlist_ids.contains(&playlist_id) || !track_ids.contains(&track_id) {
            continue;
        }
        entries
            .entry(playlist_id)
            .or_default()
            .insert(position, track_id);
    }

    entries
        .into_iter()
        .map(|(playlist_id, tracks)| (playlist_id, tracks.into_values().collect()))
        .collect()
}
