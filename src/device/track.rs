// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! The [`Track`] type: a track as a user thinks of it, used both to add tracks to a
//! [`DeviceExport`](crate::DeviceExport) and to read them back.

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
/// The same type goes both ways: [`DeviceExport::add_track`](crate::DeviceExport::add_track)
/// takes one and resolves artists/albums/genres/keys/labels/artwork into deduplicated PDB rows,
/// handling the format quirks (the 221-byte minimum row size, centi-BPM tempo), and
/// [`DeviceExport::tracks`](crate::DeviceExport::tracks) hands them back with the foreign keys
/// resolved to names. Experts needing fields not exposed here should drive a `pdb::Track` row
/// directly via [`crate::pdb::io::Database`].
///
/// `rating` is a typed [`Rating`] (0–5 stars); the writer encodes it to the raw PDB byte
/// (`0..=5`, confirmed against a real Rekordbox export — not the XML's
/// `0/51/102/153/204/255`).
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
    /// Musical key name (e.g. "Cmaj", "D\u{266d}min"). Folded to a canonical form for dedup, so
    /// it reads back in that form.
    pub key: String,
    /// Record label name.
    pub label: String,
    /// Composer name.
    pub composer: String,
    /// Remixer name.
    pub remixer: String,
    /// Original performer, distinct from `artist` (covers/reworks).
    pub orig_artist: String,
    /// Free-text comment. Also the auto-pad target when the row falls under the 221-byte
    /// minimum: the writer appends spaces, and trailing spaces are trimmed when reading back.
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
    /// Star rating (0–5); see the struct docs.
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
    /// Build the PDB row: encode all strings, set scalar fields, grow `comment` past the 221-byte
    /// minimum. The foreign-key ids (and the track's own id) are left at 0 for the caller to fill.
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
            // PDB tempo is centi-BPM (BPM × 100).
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
                    unknown_string2: DeviceSQLString::new("1")?,
                    unknown_string3: DeviceSQLString::new("1")?,
                    unknown_string4: DeviceSQLString::empty(),
                    message,
                    publish_track_information: DeviceSQLString::new(ON)?,
                    autoload_hotcues,
                    unknown_string5: DeviceSQLString::empty(),
                    unknown_string6: DeviceSQLString::empty(),
                    date_added,
                    release_date,
                    mix_name,
                    unknown_string7: DeviceSQLString::empty(),
                    analyze_path: DeviceSQLString::empty(),
                    analyze_date: DeviceSQLString::empty(),
                    comment: DeviceSQLString::empty(),
                    title,
                    unknown_string8: DeviceSQLString::empty(),
                    filename,
                    file_path,
                },
            },
        };

        // Grow `comment` with trailing spaces (semantically harmless free text) until the row
        // meets the 221-byte minimum. Only `comment` is re-encoded each pass.
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

/// Fold a key name to a canonical form for dedup (`C Major`/`Cmaj`/`C MAJOR`/`Cmajor` → `Cmaj`),
/// so they share one PDB Key row. The note letter keeps its case.
///
/// String-equality only: enharmonic equivalents (`B♭m` ≠ `A#m`), Camelot, and Open Key notation
/// are not resolved — different spellings of the same pitch still create distinct rows.
pub(crate) fn canonical_key_name(name: &str) -> String {
    let trimmed = name.trim();
    let lower = trimmed.to_lowercase();
    let mut out = String::with_capacity(trimmed.len());
    let mut i = 0;
    while i < lower.len() {
        let rest = &lower[i..];
        // Order matters: longest tokens first. Each folds a mode/accidental word to its short form.
        let folded = [
            ("major", "maj"),
            ("minor", "min"),
            ("flat", "b"),
            ("sharp", "#"),
            ("maj", "maj"),
            ("min", "min"),
        ]
        .into_iter()
        .find_map(|(tok, emit)| rest.starts_with(tok).then_some((emit, tok.len())));
        if let Some((emit, skip)) = folded {
            out.push_str(emit);
            i += skip;
        } else {
            // Not a recognized token: copy one original char through (preserves note case + any
            // unicode like ♭/♯ fixed in the second pass below).
            let ch = trimmed[i..].chars().next().expect("aligned with lower");
            out.push(ch);
            i += ch.len_utf8();
        }
    }

    // Bare trailing 'm' is minor (e.g. "Cm" → "Cmin"). Only at end-of-string, and only if we
    // didn't already end in maj/min.
    if out.ends_with('m') && !out.ends_with("min") && !out.ends_with("maj") {
        out.push_str("in");
    }

    // Fold unicode accidentals.
    out.replace('\u{266d}', "b")
        .replace('\u{266f}', "#")
        .replace(' ', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_key_name_major_forms() {
        assert_eq!(canonical_key_name("C Major"), "Cmaj");
        assert_eq!(canonical_key_name("Cmaj"), "Cmaj");
        assert_eq!(canonical_key_name("C MAJOR"), "Cmaj");
        assert_eq!(canonical_key_name("Cmajor"), "Cmaj");
        // The note letter's case is preserved as given.
        assert_eq!(canonical_key_name("c MAJOR"), "cmaj");
    }

    #[test]
    fn test_canonical_key_name_minor_forms() {
        assert_eq!(canonical_key_name("A Minor"), "Amin");
        assert_eq!(canonical_key_name("Amin"), "Amin");
        assert_eq!(canonical_key_name("A MINOR"), "Amin");
        // Bare 'm' suffix is minor.
        assert_eq!(canonical_key_name("Am"), "Amin");
    }

    #[test]
    fn test_canonical_key_name_accidentals() {
        // Unicode ♭/♯ and the words flat/sharp all collapse to ascii.
        assert_eq!(canonical_key_name("B\u{266d}m"), "Bbmin");
        assert_eq!(canonical_key_name("B flat minor"), "Bbmin");
        assert_eq!(canonical_key_name("F\u{266f}m"), "F#min");
        assert_eq!(canonical_key_name("F sharp Minor"), "F#min");
    }

    #[test]
    fn test_canonical_key_name_dedup_equivalence() {
        // The whole point: these must all collide so they share one PDB Key row.
        let a = canonical_key_name("D Major");
        let b = canonical_key_name("Dmaj");
        assert_eq!(a, b);
    }
}
