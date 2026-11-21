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

use crate::pdb::{
    Album, Artist, Artwork, Genre, Label, PageType, PlainPageType, PlaylistTreeNode, Row,
    Track as TrackRow,
};
use crate::util::{ColorIndex, FileType, RekordcrateResult};
use binrw::BinRead;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Name of the file holding the library.
const LIBRARY_FILE: &str = "datafile.edb";

/// Name of the file holding the profile of the DJ, which an export carries a copy of.
const PROFILE_FILE: &str = "djprofile.nxs";

/// Name of the file holding the tags that tracks are labelled with.
const EXT_DATA_FILE: &str = "ExtData.edb";

/// Tag of the rows holding a tag or a category of tags.
const TAG_TAG: [u8; 2] = [0x60, 0x05];

/// Tag of the rows saying that a track carries a tag.
const TRACK_TAG_TAG: [u8; 2] = [0x80, 0x05];

/// Value marking a tag row as holding a category rather than a tag.
const TAG_IS_CATEGORY: u32 = 0x0100_0000;

/// Tag that a track row is preceded by.
const TRACK_TAG: [u8; 2] = [0xc4, 0x02];

/// Tag that a row of the playlist tree is preceded by.
const PLAYLIST_TAG: [u8; 2] = [0xa0, 0x03];

/// Tag that an entry of a playlist is preceded by.
const ENTRY_TAG: [u8; 2] = [0xc0, 0x03];

/// Tags of the rows holding the names that a track refers to by id.
const ARTIST_TAG: [u8; 2] = [0x00, 0x03];
const ALBUM_TAG: [u8; 2] = [0x20, 0x03];
const GENRE_TAG: [u8; 2] = [0xe0, 0x02];
const LABEL_TAG: [u8; 2] = [0x40, 0x03];

/// Tag of the rows naming the image files that hold the artwork of a track.
const ARTWORK_TAG: [u8; 2] = [0x20, 0x04];

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
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LibraryTrack {
    /// Id of the track, which playlists refer to it by.
    pub id: u32,
    /// Title of the track.
    pub title: String,
    /// Name of the artist, or empty if the track names none.
    pub artist: String,
    /// Name of the album, or empty if the track names none.
    pub album: String,
    /// Name of the artist that the album names, which is not always the artist of the track.
    pub album_artist: String,
    /// Name of the genre, or empty if the track names none.
    pub genre: String,
    /// Name of the record label, or empty if the track names none.
    pub label: String,
    /// Path of the image file holding the artwork of the track on the machine the library
    /// belongs to, or empty if the track names none.
    pub artwork_path: String,
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
    /// Path of the analysis file of the track on the machine the library belongs to.
    pub analyze_path: String,
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
    /// Number of times the track was played.
    pub play_count: u16,
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

/// A tag that tracks can be labelled with, or a category holding tags.
///
/// Rekordbox keeps these apart from the playlist tree, in a file of their own, though a library
/// may well also hold playlists named after the tags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryTag {
    /// Id of the tag, which the tracks carrying it refer to it by.
    pub id: u32,
    /// Id of the category holding the tag, or zero for a category.
    pub parent_id: u32,
    /// Name of the tag.
    pub name: String,
    /// Place of the tag among the tags of its category, counted from zero.
    pub position: u32,
    /// Whether the row holds a category rather than a tag.
    pub is_category: bool,
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
    tags: Vec<LibraryTag>,
    /// The tags of a track, by the id of the track.
    track_tags: BTreeMap<u32, Vec<u32>>,
}

impl Library {
    /// Reads the library in the given directory, which is the one holding `datafile.edb`.
    pub fn open(root: impl Into<PathBuf>) -> RekordcrateResult<Self> {
        let root = root.into();
        let data = std::fs::read(root.join(LIBRARY_FILE))?;

        let names = read_names(&data);
        let tracks = read_tracks(&data, &names);
        let playlists = read_playlists(&data);
        let entries = read_entries(
            &data,
            &tracks.keys().copied().collect(),
            &playlists.iter().map(|node| node.id).collect(),
        );

        // The tags live in a file of their own, which a library need not have.
        let (tags, track_tags) = match std::fs::read(root.join(EXT_DATA_FILE)) {
            Ok(data) => (
                read_tags(&data),
                read_track_tags(&data, &tracks.keys().copied().collect()),
            ),
            Err(_) => (Vec::new(), BTreeMap::new()),
        };

        Ok(Self {
            root,
            tracks,
            playlists,
            entries,
            tags,
            track_tags,
        })
    }

    /// The directory the library was read from.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The tags that tracks can be labelled with, and the categories holding them.
    #[must_use]
    pub fn tags(&self) -> &[LibraryTag] {
        &self.tags
    }

    /// The tags a track is labelled with.
    #[must_use]
    pub fn tags_of_track(&self, track_id: u32) -> &[u32] {
        self.track_tags
            .get(&track_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
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

    /// The node of the playlist tree with the given id.
    #[must_use]
    pub fn playlist_by_id(&self, id: u32) -> Option<&LibraryPlaylist> {
        self.playlists.iter().find(|node| node.id == id)
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

    /// The profile of the DJ, if the library holds one.
    ///
    /// Rekordbox puts a copy of this file in an export. What it holds beyond the name of the DJ
    /// is not worked out, and it names ids of the installation it came from, so it is carried
    /// over as it is rather than written.
    #[must_use]
    pub fn profile_file(&self) -> Option<PathBuf> {
        let path = self.root.join(PROFILE_FILE);
        path.is_file().then_some(path)
    }

    /// The file that the library holds at a path it recorded, if it is there.
    ///
    /// The paths in a library are absolute paths on the machine it belongs to, which a copy of
    /// the library is not at, so the part below `PIONEER` is taken and looked for in the library
    /// itself.
    #[must_use]
    pub fn file_below_pioneer(&self, recorded_path: &str) -> Option<PathBuf> {
        let below = recorded_path.split_once("PIONEER/").map(|(_, rest)| rest)?;
        let path = self.root.join("PIONEER").join(below);
        path.is_file().then_some(path)
    }

    /// The analysis files that the library holds for a track, if they are there.
    ///
    /// The track names the one holding the beat grid and the cues, and the others sit beside it
    /// under the same name. The name is not always the first one: where the paths of two tracks
    /// lead to the same directory, which happens, Rekordbox counts the name up, so the name has
    /// to be taken from the track rather than assumed.
    ///
    /// The path the track names is a path on the machine the library belongs to, so the file is
    /// looked for in the library itself.
    #[must_use]
    pub fn analysis_files(&self, track: &LibraryTrack) -> Vec<PathBuf> {
        let Some(named) = self.file_below_pioneer(&track.analyze_path) else {
            return Vec::new();
        };
        ["DAT", "EXT", "2EX"]
            .into_iter()
            .map(|extension| named.with_extension(extension))
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

/// The names that tracks refer to by id, by the kind of row holding them.
#[derive(Debug, Default)]
struct Names {
    artists: BTreeMap<u32, String>,
    albums: BTreeMap<u32, String>,
    /// The artist that an album names, by the id of the album.
    album_artists: BTreeMap<u32, u32>,
    genres: BTreeMap<u32, String>,
    labels: BTreeMap<u32, String>,
    artwork: BTreeMap<u32, String>,
}

/// Reads the tags of a library, along with the categories holding them.
///
/// A tag row holds its fields in the order an export does, bar one: an export holds a second
/// field of unknown purpose which a library does not, so the rows are read here rather than by
/// the parser of the export.
fn read_tags(data: &[u8]) -> Vec<LibraryTag> {
    let number = |offset: usize| {
        data.get(offset..offset + 4)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u32::from_le_bytes)
    };

    let mut tags: BTreeMap<u32, LibraryTag> = BTreeMap::new();
    for tag in tagged_rows(data, TAG_TAG) {
        let row = tag + TAG_SIZE;
        let (Some(parent_id), Some(position), Some(id), Some(is_category)) = (
            number(row + 4),
            number(row + 8),
            number(row + 12),
            number(row + 16),
        ) else {
            continue;
        };
        if id == 0 {
            continue;
        }
        // The fields are followed by the offsets of the strings and then the name itself.
        let Some(name) = read_name(data, row + 20 + 2) else {
            continue;
        };
        tags.insert(
            id,
            LibraryTag {
                id,
                parent_id,
                name,
                position,
                is_category: is_category == TAG_IS_CATEGORY,
            },
        );
    }
    tags.into_values().collect()
}

/// Reads which tags the tracks of a library are labelled with.
fn read_track_tags(data: &[u8], track_ids: &BTreeSet<u32>) -> BTreeMap<u32, Vec<u32>> {
    let number = |offset: usize| {
        data.get(offset..offset + 4)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u32::from_le_bytes)
    };

    let mut found: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for tag in tagged_rows(data, TRACK_TAG_TAG) {
        let row = tag + TAG_SIZE;
        let (Some(track_id), Some(tag_id)) = (number(row + 4), number(row + 8)) else {
            continue;
        };
        // A row of another table carrying the same tag is told apart by naming a track that the
        // library holds.
        if tag_id == 0 || !track_ids.contains(&track_id) {
            continue;
        }
        found.entry(track_id).or_default().insert(tag_id);
    }
    found
        .into_iter()
        .map(|(track, tags)| (track, tags.into_iter().collect()))
        .collect()
}

/// Reads a DeviceSQL string of plain characters at an offset, if there is one.
fn read_name(data: &[u8], offset: usize) -> Option<String> {
    let header = *data.get(offset)?;
    // A string of plain characters holds its length in the upper bits, and the lowest bit set.
    if header & 1 == 0 || header < 3 {
        return None;
    }
    let length = usize::from(header >> 1).checked_sub(1)?;
    let name = data.get(offset + 1..offset + 1 + length)?;
    let name = std::str::from_utf8(name).ok()?;
    (!name.is_empty() && !name.chars().any(char::is_control)).then(|| name.to_owned())
}

/// Reads the rows holding the names that a track refers to by id.
fn read_names(data: &[u8]) -> Names {
    let mut names = Names::default();
    for (tag, page_type) in [
        (ARTIST_TAG, PlainPageType::Artists),
        (ALBUM_TAG, PlainPageType::Albums),
        (GENRE_TAG, PlainPageType::Genres),
        (LABEL_TAG, PlainPageType::Labels),
        (ARTWORK_TAG, PlainPageType::Artwork),
    ] {
        // An artist or album row holds the tag as its first two fields, while the others start
        // after it.
        let skip_tag = matches!(
            page_type,
            PlainPageType::Genres | PlainPageType::Labels | PlainPageType::Artwork
        );
        for tagged in tagged_rows(data, tag) {
            let offset = if skip_tag { tagged + TAG_SIZE } else { tagged };
            let Some(row) = read_row_at(data, offset, PageType::Plain(page_type)) else {
                continue;
            };
            let named = match page_type {
                PlainPageType::Artists => row
                    .as_variant::<Artist>()
                    .map(|row| (row.id.0, row.offsets.name.clone())),
                PlainPageType::Albums => row.as_variant::<Album>().map(|row| {
                    // An album names an artist of its own, which is kept to be resolved once
                    // every artist has been read.
                    names.album_artists.insert(row.id.0, row.artist_id.0);
                    (row.id.0, row.offsets.name.clone())
                }),
                PlainPageType::Genres => row
                    .as_variant::<Genre>()
                    .map(|row| (row.id.0, row.name.clone())),
                PlainPageType::Labels => row
                    .as_variant::<Label>()
                    .map(|row| (row.id.0, row.name.clone())),
                PlainPageType::Artwork => row
                    .as_variant::<Artwork>()
                    .map(|row| (row.id.0, row.path.clone())),
                _ => None,
            };
            let Some((id, name)) = named else { continue };
            let Ok(name) = name.into_string() else {
                continue;
            };
            // A tag can appear by chance, so a row is only kept if it names something.
            if id == 0 || name.is_empty() || name.chars().any(char::is_control) {
                continue;
            }
            let table = match page_type {
                PlainPageType::Artists => &mut names.artists,
                PlainPageType::Albums => &mut names.albums,
                PlainPageType::Genres => &mut names.genres,
                PlainPageType::Artwork => &mut names.artwork,
                _ => &mut names.labels,
            };
            table.insert(id, name);
        }
    }
    names
}

/// Reads the tracks of the library.
fn read_tracks(data: &[u8], names: &Names) -> BTreeMap<u32, LibraryTrack> {
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
                artist: names
                    .artists
                    .get(&row.artist_id.0)
                    .cloned()
                    .unwrap_or_default(),
                album: names
                    .albums
                    .get(&row.album_id.0)
                    .cloned()
                    .unwrap_or_default(),
                album_artist: names
                    .album_artists
                    .get(&row.album_id.0)
                    .and_then(|artist_id| names.artists.get(artist_id))
                    .cloned()
                    .unwrap_or_default(),
                genre: names
                    .genres
                    .get(&row.genre_id.0)
                    .cloned()
                    .unwrap_or_default(),
                label: names
                    .labels
                    .get(&row.label_id.0)
                    .cloned()
                    .unwrap_or_default(),
                artwork_path: names
                    .artwork
                    .get(&row.artwork_id.0)
                    .cloned()
                    .unwrap_or_default(),
                file_path,
                filename: string(&row.offsets.filename),
                comment: string(&row.offsets.comment),
                date_added: string(&row.offsets.date_added),
                analyze_date: string(&row.offsets.analyze_date),
                analyze_path: string(&row.offsets.analyze_path),
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
                play_count: row.play_count,
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

#[cfg(test)]
mod test {
    use super::*;

    /// A directory holding nothing but the file that makes it a library.
    fn empty_library(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("rekordcrate-library-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join(LIBRARY_FILE), []).unwrap();
        path
    }

    /// The profile of the DJ is found in the library, and is only reported when it is there.
    #[test]
    fn test_profile_file() {
        let root = empty_library("profile");
        let library = Library::open(&root).unwrap();
        assert_eq!(library.profile_file(), None);

        std::fs::write(root.join(PROFILE_FILE), b"a profile").unwrap();
        let library = Library::open(&root).unwrap();
        assert_eq!(library.profile_file(), Some(root.join(PROFILE_FILE)));

        let _ = std::fs::remove_dir_all(&root);
    }
}
