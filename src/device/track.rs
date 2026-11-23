// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! The [`Track`] type: a track as a user thinks of it, used both to add tracks to a
//! [`DeviceExport`](crate::DeviceExport) and to read them back.

use crate::device::layout::anlz_folder;
use crate::pdb::offset_array::OffsetArrayContainer;
use crate::pdb::string::DeviceSQLString;
use crate::pdb::{
    AlbumId, ArtistId, ArtworkId, GenreId, KeyId, LabelId, PageHeapObject, Subtype,
    Track as PdbTrack, TrackId, TrackStrings,
};
use crate::util::{ColorIndex, FileType, MaybeCalculated, Rating};
use crate::Result;

/// Minimum allocated (4-byte-aligned) size of a Track row, in bytes. A CDJ-350 crashes entering
/// its "TRACK" menu when any track row is shorter than this; 221 was the smallest value found to
/// avoid the crash (by trial and error). [`DeviceExport::add_track`](crate::DeviceExport::add_track)
/// pads `comment` to satisfy it; the low-level [`pdb`](crate::pdb) layer does not enforce it.
pub(crate) const MIN_TRACK_ROW_SIZE: u16 = 221;

/// The string value of the PDB's boolean-like string fields when they are on.
pub(crate) const ON: &str = "ON";

/// A track as a user thinks of it: plain `String` fields, no foreign-key IDs. An empty string
/// means "none", as in the PDB itself.
///
/// [`DeviceExport::add_track`](crate::DeviceExport::add_track) takes one and
/// [`DeviceExport::tracks`](crate::DeviceExport::tracks) hands them back. Fields not exposed
/// here need a `pdb::Track` row driven directly via [`crate::pdb::io::Database`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Track {
    /// Track title.
    pub title: String,
    /// Performing artist name.
    pub artist: String,
    /// Album name.
    pub album: String,
    /// Genre name.
    pub genre: String,
    /// Musical key name (e.g. "Am", "F#m", "8A"), stored as given.
    pub key: String,
    /// Record label name.
    pub label: String,
    /// Composer name.
    pub composer: String,
    /// Remixer name.
    pub remixer: String,
    /// Original performer, distinct from `artist` (covers/reworks).
    pub orig_artist: String,
    /// Free-text comment. Padded with trailing spaces when the row would fall under the minimum
    /// size some players need; trailing spaces are trimmed when reading back.
    pub comment: String,
    /// ISRC, in rekordbox's mangled format.
    pub isrc: String,
    /// Lyricist name.
    pub lyricist: String,
    /// Remix/mix name.
    pub mix_name: String,
    /// Free text. Rekordbox itself writes strict `YYYY-MM-DD` or empty here, but this field is
    /// passed through verbatim so callers can probe hardware behavior with arbitrary strings.
    pub release_date: String,
    /// Free text, commonly `YYYY-MM-DD` (see [`Self::release_date`]).
    pub date_added: String,
    /// Device-relative file path (e.g. `/Contents/Artist - Title.mp3`). Dedup key when non-empty.
    pub file_path: String,
    /// File name without path.
    pub filename: String,
    /// Device path of the artwork thumbnail (e.g. `/PIONEER/Artwork/00001/a1.jpg`), stored
    /// verbatim in the `Artwork` row. Empty = none. The caller owns placing the image files it
    /// names — or, with the `artwork` feature, gets a path whose files the export writes itself
    /// from `DeviceExport::import_artwork`.
    pub artwork_path: String,
    /// Track "message" field shown in Rekordbox.
    pub message: String,
    /// Tempo in BPM; encoded to centi-BPM (× 100) in the PDB.
    pub tempo: f32,
    /// Bitrate in kbps.
    pub bitrate: u32,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Bits per sample of the audio file.
    pub sample_depth: u16,
    /// Playback duration in seconds. PDB field is `u16` (ceiling ~18.2 h).
    pub duration_secs: u16,
    /// File size in bytes.
    pub file_size: u32,
    /// Track number within the album.
    pub track_number: u32,
    /// Disc number.
    pub disc_number: u16,
    /// Release year.
    pub year: u16,
    /// Number of times the track was played.
    pub play_count: u16,
    /// Star rating.
    pub rating: Rating,
    /// Color label.
    pub color: ColorIndex,
    /// Audio file format.
    pub file_type: FileType,
    /// Whether stored hotcues auto-load on a CDJ. Maps to the PDB string `"ON"` / empty.
    pub autoload_hotcues: bool,
}

/// Allocated size of a Track row on the page heap: its serialized size, 4-byte aligned.
fn allocated_row_size(row: &PdbTrack) -> u16 {
    row.heap_bytes_required(()).next_multiple_of(4)
}

impl Track {
    /// The foreign-key ids (and the track's own id) are left at 0 for the caller to fill.
    pub(crate) fn to_row(&self) -> Result<PdbTrack> {
        let track = self;
        // Only `comment` grows below; hoist the rest so they're encoded once.
        let isrc = DeviceSQLString::new(&track.isrc)?;
        let lyricist = DeviceSQLString::new(&track.lyricist)?;
        let message = DeviceSQLString::new(&track.message)?;
        let mix_name = DeviceSQLString::new(&track.mix_name)?;
        let release_date = DeviceSQLString::new(&track.release_date)?;
        let date_added = DeviceSQLString::new(&track.date_added)?;
        let title = DeviceSQLString::new(&track.title)?;
        let filename = DeviceSQLString::new(&track.filename)?;
        let file_path = DeviceSQLString::new(&track.file_path)?;

        let analyze_path = if track.file_path.is_empty() {
            DeviceSQLString::empty()
        } else {
            DeviceSQLString::new(&format!(
                "/PIONEER/USBANLZ/{}/ANLZ0000.DAT",
                anlz_folder(&track.file_path)
            ))?
        };
        let autoload_hotcues = if track.autoload_hotcues {
            DeviceSQLString::new(ON)?
        } else {
            DeviceSQLString::empty()
        };

        let mut pdb_track = PdbTrack {
            // Reverse-engineered constants observed on fresh Rekordbox track rows.
            subtype: Subtype(0x24),
            index_shift: 0,
            bitmask: 788_224,
            unknown5: 41,
            sample_rate: track.sample_rate,
            sample_depth: track.sample_depth,
            bitrate: track.bitrate,
            duration: track.duration_secs,
            file_size: track.file_size,
            tempo: (track.tempo * 100.0).round() as u32,
            file_type: track.file_type.clone(),
            track_number: track.track_number,
            disc_number: track.disc_number,
            year: track.year,
            play_count: track.play_count,
            rating: track.rating.to_byte(),
            color: track.color.clone(),
            composer_id: ArtistId(0),
            artwork_id: ArtworkId(0),
            key_id: KeyId(0),
            orig_artist_id: ArtistId(0),
            label_id: LabelId(0),
            remixer_id: ArtistId(0),
            genre_id: GenreId(0),
            album_id: AlbumId(0),
            artist_id: ArtistId(0),
            id: TrackId(0),
            unknown2: 0,
            unknown3: 0,
            unknown4: 0,
            offsets: OffsetArrayContainer {
                offsets: MaybeCalculated::Calculated,
                inner: TrackStrings {
                    isrc,
                    lyricist,
                    // TODO(acrilique): "1" is only the most common value on
                    // current fixtures. The real meaning behind these fields
                    // is still TBD.
                    unknown_string2: DeviceSQLString::new("1")?,
                    unknown_string3: DeviceSQLString::new("1")?,
                    unknown_string4: DeviceSQLString::empty(),
                    message,
                    // TODO(acrilique): this is not constant on real exports.
                    // Maybe some users of the library will want to be able to
                    // choose the value for it...
                    publish_track_information: DeviceSQLString::new(ON)?,
                    autoload_hotcues,
                    unknown_string5: DeviceSQLString::empty(),
                    unknown_string6: DeviceSQLString::empty(),
                    date_added,
                    release_date,
                    mix_name,
                    unknown_string7: DeviceSQLString::empty(),
                    analyze_path,
                    analyze_date: DeviceSQLString::empty(),
                    comment: DeviceSQLString::empty(),
                    title,
                    unknown_string8: DeviceSQLString::empty(),
                    filename,
                    file_path,
                },
            },
        };

        // `comment` absorbs the padding up to `MIN_TRACK_ROW_SIZE`: trailing spaces are harmless
        // in free text.
        let mut comment = track.comment.clone();
        loop {
            pdb_track.offsets.inner.comment = DeviceSQLString::new(&comment)?;
            if allocated_row_size(&pdb_track) >= MIN_TRACK_ROW_SIZE {
                break;
            }
            comment.push(' ');
        }

        Ok(pdb_track)
    }
}
