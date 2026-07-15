// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Builds a Rekordbox USB export: the PDB database (`export.pdb`), the `PIONEER/*.DAT` setting
//! files, and the `PIONEER`/`PIONEER/USBANLZ`/`Contents` directory layout.
//!
//! With the `artwork` feature enabled, it also copies each track's [`Track::artwork_path`] into
//! `PIONEER/Artwork/{folder}/`, resizing to the 80×80 thumbnail (`a{id}.jpg`) and the 240×240
//! medium (`a{id}_m.jpg`) that newer Rekordbox versions expect. Without the feature, artwork is a
//! no-op: a raw copy of an unknown format/size can't be guaranteed CDJ-readable, so nothing is
//! written. It never copies audio files into `Contents` or generates `PIONEER/USBANLZ` analysis
//! files (`ANLZ.*`). The API is append-only — existing rows cannot be updated or deleted; re-add
//! via a fresh export instead.

#[cfg(feature = "artwork")]
use crate::device::layout::artwork_folder;
use crate::device::layout::{Layout, DAT_FILES};
use crate::pdb::io::Database;
use crate::pdb::offset_array::OffsetArrayContainer;
use crate::pdb::string::DeviceSQLString;
#[cfg(feature = "artwork")]
use crate::pdb::Artwork;
use crate::pdb::{
    AlbumId, ArtistId, ArtworkId, DatabaseType, GenreId, KeyId, LabelId, PageType, PlainPageType,
    PlainRow, PlaylistEntry, PlaylistTreeNode, PlaylistTreeNodeId, Row, Subtype, Track as PdbTrack,
    TrackId, TrackStrings,
};
use crate::setting::{Setting, SettingType};
use crate::util::{ColorIndex, FileType, MaybeCalculated, Rating};
use crate::{Error, Result};
use binrw::BinWrite;
use fallible_iterator::FallibleIterator;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Cursor;
use std::path::Path;

/// Re-export of [`crate::util::ForeignKeyKind`], for callers reaching it via this module.
pub use crate::util::ForeignKeyKind;

/// A track as a user thinks of it: plain `String` fields, no foreign-key IDs. Input to
/// [`crate::DeviceExportWriter::add_track`], which resolves artists/albums/genres/keys/labels/artwork
/// into deduplicated PDB rows and handles format quirks (the 221-byte minimum row size, centi-BPM
/// tempo). Experts needing fields not exposed here should drive a `pdb::Track` row directly via
/// [`crate::pdb::io::Database`].
///
/// `rating` is a typed [`Rating`] (0–5 stars); the writer encodes it to the raw PDB byte
/// (`0..=5`, confirmed against a real Rekordbox export — not the XML's
/// `0/51/102/153/204/255`).
#[derive(Debug, Clone)]
pub struct Track {
    /// Track title.
    pub title: String,
    /// Performing artist name.
    pub artist: String,
    /// Album name.
    pub album: String,
    /// Genre name.
    pub genre: String,
    /// Musical key name (e.g. "Cmaj", "D\u{266d}min"). Folded to a canonical form for dedup.
    pub key: String,
    /// Record label name.
    pub label: String,
    /// Composer name.
    pub composer: String,
    /// Remixer name.
    pub remixer: String,
    /// Original performer, distinct from `artist` (covers/reworks).
    pub orig_artist: String,
    /// Free-text comment; also the auto-pad target when the row falls under the 221-byte minimum.
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
    /// Host-side path to the artwork image source. Under the `artwork` feature it is decoded,
    /// resized to 80×80 + 240×240, and written into `PIONEER/Artwork/` with an id-derived name;
    /// the PDB `Artwork.path` is derived from the assigned id. Without the feature it is silently
    /// ignored (no row, no file) — set it only when `artwork` is enabled.
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

impl Default for Track {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            genre: String::new(),
            key: String::new(),
            label: String::new(),
            composer: String::new(),
            remixer: String::new(),
            orig_artist: String::new(),
            comment: String::new(),
            isrc: String::new(),
            lyricist: String::new(),
            mix_name: String::new(),
            release_date: String::new(),
            date_added: String::new(),
            file_path: String::new(),
            filename: String::new(),
            artwork_path: String::new(),
            message: String::new(),
            tempo: 0.0,
            bitrate: 0,
            sample_rate: 0,
            sample_depth: 0,
            duration_secs: 0,
            file_size: 0,
            track_number: 0,
            disc_number: 0,
            year: 0,
            play_count: 0,
            rating: Rating::Zero,
            color: ColorIndex::None,
            file_type: FileType::Unknown,
            autoload_hotcues: false,
        }
    }
}

/// The outcome of [`DeviceExportWriter::add_track`]: either a freshly inserted track or an
/// existing one returned because its `file_path` already existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddTrackOutcome {
    /// The track id in the export.
    pub id: TrackId,
    /// `true` if a new row was inserted; `false` if an existing track was returned unchanged.
    pub is_new: bool,
}

/// Builder for a Rekordbox device export: PDB database, setting files, directory structure.
#[derive(Debug)]
pub struct DeviceExportWriter {
    db: Option<Database<fs::File>>,
    #[allow(dead_code)]
    layout: Layout,
    next_track_id: u32,
    next_artist_id: u32,
    next_album_id: u32,
    next_genre_id: u32,
    next_key_id: u32,
    next_label_id: u32,
    #[cfg(feature = "artwork")]
    next_artwork_id: u32,
    next_playlist_node_id: u32,
    track_ids: HashSet<u32>,
    /// `id -> is_folder`. Root (id 0) is implicit — always valid as a parent, always a folder.
    playlist_nodes: HashMap<u32, bool>,
    tracks_by_path: HashMap<String, u32>,
    playlist_entry_counts: HashMap<u32, u32>,
    artists_by_name: HashMap<String, u32>,
    albums_by_artist_and_name: HashMap<(u32, String), u32>,
    genres_by_name: HashMap<String, u32>,
    keys_by_canonical: HashMap<String, u32>,
    labels_by_name: HashMap<String, u32>,
    #[cfg(feature = "artwork")]
    artwork_by_path: HashMap<String, u32>,
}

/// Fold a key name to a canonical form for dedup (`C Major`/`Cmaj`/`C MAJOR`/`Cmajor` → `Cmaj`),
/// so they share one PDB Key row. The note letter keeps its case.
///
/// String-equality only: enharmonic equivalents (`B♭m` ≠ `A#m`), Camelot, and Open Key notation
/// are not resolved — different spellings of the same pitch still create distinct rows.
fn canonical_key_name(name: &str) -> String {
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

/// PDB `Artwork.path` for `id`: `/PIONEER/Artwork/{folder}/a{id}.jpg`. Always references the
/// thumbnail; the medium-resolution `_m` file is not tracked in the PDB.
#[cfg(feature = "artwork")]
fn artwork_device_path(id: u32) -> String {
    format!("/PIONEER/Artwork/{}/a{id}.jpg", artwork_folder(id))
}

impl Drop for DeviceExportWriter {
    /// Best-effort flush if `close` wasn't called. Cannot surface errors — call
    /// [`DeviceExportWriter::close`] for a clean shutdown.
    fn drop(&mut self) {
        if let Some(db) = self.db.as_mut() {
            let _ = db.flush();
        }
    }
}

/// Default `Setting` for a type. The `SettingType`→`default_*` mapping lives here (not in
/// `layout`) because `Setting::default_*` is named per variant, not keyed by `SettingType`.
fn default_setting(setting_type: SettingType) -> Setting {
    match setting_type {
        SettingType::DevSetting => Setting::default_devsetting(),
        SettingType::DJMMySetting => Setting::default_djmmysetting(),
        SettingType::MySetting => Setting::default_mysetting(),
        SettingType::MySetting2 => Setting::default_mysetting2(),
    }
}

impl DeviceExportWriter {
    /// Borrows the live `Database`. `db` is `Some` until `close()` (which consumes `self`), so
    /// no method observes a `None`.
    fn db(&mut self) -> &mut Database<fs::File> {
        self.db
            .as_mut()
            .expect("DeviceExportWriter.db always Some until close")
    }
    /// Create a new device export at `device_root`.
    ///
    /// Creates the `PIONEER`, `PIONEER/USBANLZ`, and `Contents` directories, an empty PDB
    /// database, and the default setting files. Audio files and `ANLZ.*` analysis files are never
    /// produced; artwork is copied only under the `artwork` feature (see the module docs).
    ///
    /// # Errors
    ///
    /// If directory creation, PDB creation, or setting file writing fails.
    pub fn create(device_root: impl AsRef<Path>) -> Result<Self> {
        let layout = Layout::new(device_root.as_ref().to_path_buf());

        fs::create_dir_all(layout.rekordbox_dir())?;
        fs::create_dir_all(layout.usbanlz_dir())?;
        fs::create_dir_all(layout.contents_dir())?;

        // PDB table layout, in the fixed index order Rekordbox expects on a device export. Each
        // entry occupies one table index (0=Tracks, 1=Genres, …). The `Unknown(N)` entries are
        // currently unknown tables, but they must stay in place so CDJ players don't crash.
        let table_page_types: &[PageType] = &[
            PageType::Plain(PlainPageType::Tracks),
            PageType::Plain(PlainPageType::Genres),
            PageType::Plain(PlainPageType::Artists),
            PageType::Plain(PlainPageType::Albums),
            PageType::Plain(PlainPageType::Labels),
            PageType::Plain(PlainPageType::Keys),
            PageType::Plain(PlainPageType::Colors),
            PageType::Plain(PlainPageType::PlaylistTree),
            PageType::Plain(PlainPageType::PlaylistEntries),
            PageType::Unknown(9),
            PageType::Unknown(10),
            PageType::Plain(PlainPageType::HistoryPlaylists),
            PageType::Plain(PlainPageType::HistoryEntries),
            PageType::Plain(PlainPageType::Artwork),
            PageType::Unknown(14),
            PageType::Unknown(15),
            PageType::Plain(PlainPageType::Columns),
            PageType::Plain(PlainPageType::Menu),
            PageType::Unknown(18),
            PageType::Plain(PlainPageType::History),
        ];

        let file = fs::File::create(layout.export_pdb())?;
        let mut db = Database::create(file, DatabaseType::Plain, table_page_types)?;

        crate::pdb::defaults::insert_default_colors(&mut db)?;
        crate::pdb::defaults::insert_default_columns(&mut db)?;
        crate::pdb::defaults::insert_default_menus(&mut db)?;

        for &(filename, setting_type) in DAT_FILES {
            write_setting_file(&layout.dat_path(filename), &default_setting(setting_type))?;
        }

        Ok(Self {
            db: Some(db),
            layout,
            // Counters start at 1: id 0 is the null/empty foreign key (returned by the empty-name
            // branches of get_or_create_* and used for unset composer/orig-artist/remixer ids), so
            // the first real row must be 1. Do not change to 0.
            next_track_id: 1,
            next_artist_id: 1,
            next_album_id: 1,
            next_genre_id: 1,
            next_key_id: 1,
            next_label_id: 1,
            #[cfg(feature = "artwork")]
            next_artwork_id: 1,
            next_playlist_node_id: 1,
            track_ids: HashSet::new(),
            playlist_nodes: HashMap::new(),
            tracks_by_path: HashMap::new(),
            playlist_entry_counts: HashMap::new(),
            artists_by_name: HashMap::new(),
            albums_by_artist_and_name: HashMap::new(),
            genres_by_name: HashMap::new(),
            keys_by_canonical: HashMap::new(),
            labels_by_name: HashMap::new(),
            #[cfg(feature = "artwork")]
            artwork_by_path: HashMap::new(),
        })
    }

    /// Open an existing device export at the given path.
    ///
    /// Opens the existing PDB database and scans for the highest IDs in each
    /// table category to continue adding rows with non-conflicting IDs.
    ///
    /// # Errors
    ///
    /// If the PDB cannot be opened or the table scanning fails.
    pub fn open(device_root: impl AsRef<Path>) -> Result<Self> {
        let layout = Layout::new(device_root.as_ref().to_path_buf());
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(layout.export_pdb())?;
        let mut db = Database::open(file, DatabaseType::Plain)?;

        // Each table's next free ID is one past the highest existing ID (0 when empty). We also
        // rebuild the in-memory dedup/index maps so add_track stays O(1) per lookup.
        let mut track_ids = HashSet::new();
        let mut tracks_by_path = HashMap::new();
        let next_track_id = db.iter_rows::<crate::pdb::Track>()?.fold(0u32, |max, t| {
            track_ids.insert(t.id.0);
            if let Ok(path) = t.offsets.inner.file_path.clone().into_string() {
                if !path.is_empty() {
                    tracks_by_path.entry(path).or_insert(t.id.0);
                }
            }
            Ok(max.max(t.id.0))
        })? + 1;

        let mut artists_by_name = HashMap::new();
        let next_artist_id = db.iter_rows::<crate::pdb::Artist>()?.fold(0u32, |max, a| {
            if let Ok(name) = a.offsets.name.clone().into_string() {
                artists_by_name.entry(name).or_insert(a.id.0);
            }
            Ok(max.max(a.id.0))
        })? + 1;

        let mut albums_by_artist_and_name = HashMap::new();
        let next_album_id = db.iter_rows::<crate::pdb::Album>()?.fold(0u32, |max, a| {
            if let Ok(name) = a.offsets.name.clone().into_string() {
                albums_by_artist_and_name
                    .entry((a.artist_id.0, name))
                    .or_insert(a.id.0);
            }
            Ok(max.max(a.id.0))
        })? + 1;

        let mut genres_by_name = HashMap::new();
        let next_genre_id = db.iter_rows::<crate::pdb::Genre>()?.fold(0u32, |max, g| {
            if let Ok(name) = g.name.clone().into_string() {
                genres_by_name.entry(name).or_insert(g.id.0);
            }
            Ok(max.max(g.id.0))
        })? + 1;

        let mut keys_by_canonical = HashMap::new();
        let next_key_id = db.iter_rows::<crate::pdb::Key>()?.fold(0u32, |max, k| {
            if let Ok(name) = k.name.clone().into_string() {
                // Index under canonical form so later lookups collide.
                keys_by_canonical
                    .entry(canonical_key_name(&name))
                    .or_insert(k.id.0);
            }
            Ok(max.max(k.id.0))
        })? + 1;

        let mut labels_by_name = HashMap::new();
        let next_label_id = db.iter_rows::<crate::pdb::Label>()?.fold(0u32, |max, l| {
            if let Ok(name) = l.name.clone().into_string() {
                labels_by_name.entry(name).or_insert(l.id.0);
            }
            Ok(max.max(l.id.0))
        })? + 1;

        #[cfg(feature = "artwork")]
        let (next_artwork_id, artwork_by_path) = {
            let mut artwork_by_path = HashMap::new();
            let next_artwork_id = db
                .iter_rows::<crate::pdb::Artwork>()?
                .fold(0u32, |max, a| {
                    if let Ok(path) = a.path.clone().into_string() {
                        artwork_by_path.entry(path).or_insert(a.id.0);
                    }
                    Ok(max.max(a.id.0))
                })?
                + 1;
            (next_artwork_id, artwork_by_path)
        };

        let mut playlist_nodes = HashMap::new();
        let next_playlist_node_id =
            db.iter_rows::<crate::pdb::PlaylistTreeNode>()?
                .fold(0u32, |max, p| {
                    playlist_nodes.insert(p.id.0, p.is_folder());
                    Ok(max.max(p.id.0))
                })?
                + 1;

        // `max(entry_index) + 1` (not row count) so a reopened export with sparse indices doesn't
        // collide with an existing index.
        let mut playlist_entry_counts: HashMap<u32, u32> = HashMap::new();
        db.iter_rows::<crate::pdb::PlaylistEntry>()?.for_each(|e| {
            let entry = playlist_entry_counts.entry(e.playlist_id.0).or_insert(0);
            *entry = (*entry).max(e.entry_index + 1);
            Ok(())
        })?;

        Ok(Self {
            db: Some(db),
            layout,
            next_track_id,
            next_artist_id,
            next_album_id,
            next_genre_id,
            next_key_id,
            next_label_id,
            #[cfg(feature = "artwork")]
            next_artwork_id,
            next_playlist_node_id,
            track_ids,
            playlist_nodes,
            tracks_by_path,
            playlist_entry_counts,
            artists_by_name,
            albums_by_artist_and_name,
            genres_by_name,
            keys_by_canonical,
            labels_by_name,
            #[cfg(feature = "artwork")]
            artwork_by_path,
        })
    }

    /// Add a track to the export from a domain [`Track`].
    ///
    /// Resolves (creating where needed) the Artist, Album, Genre, Key, Label, and Artwork rows,
    /// then inserts a Track row pointing at them by ID.
    ///
    /// Idempotent on non-empty `file_path`: if a track with that path was already added (this
    /// session or read back by [`open`](Self::open)), the existing [`TrackId`] is returned and no
    /// new row is inserted. Tracks with an empty `file_path` are always inserted. Use
    /// [`AddTrackOutcome::is_new`] to tell the cases apart.
    ///
    /// Pads `comment` with spaces if the row would fall under Rekordbox's 221-byte minimum.
    ///
    /// # Errors
    ///
    /// If any of the track's strings cannot be encoded.
    ///
    /// String encoding and row building happen before any IDs are allocated or rows inserted, so
    /// a bad string leaves the export untouched. A mid-write IO failure can leave orphaned
    /// dimension rows on an `open()`-ed device; this is not recovered automatically.
    pub fn add_track(&mut self, track: &Track) -> Result<AddTrackOutcome> {
        if !track.file_path.is_empty() {
            if let Some(&id) = self.tracks_by_path.get(&track.file_path) {
                return Ok(AddTrackOutcome {
                    id: TrackId(id),
                    is_new: false,
                });
            }
        }

        let (mut pdb_track, comment) = self.build_pdb_track(track)?;

        let artist_id = self.get_or_create_artist(&track.artist)?;
        let album_id = self.get_or_create_album(&track.album, artist_id)?;
        let genre_id = self.get_or_create_genre(&track.genre)?;
        let key_id = self.get_or_create_key(&track.key)?;
        let label_id = self.get_or_create_label(&track.label)?;
        let artwork_id = self.get_or_create_artwork(&track.artwork_path)?;

        let composer_id = if track.composer.is_empty() {
            0
        } else {
            self.get_or_create_artist(&track.composer)?
        };
        let orig_artist_id = if track.orig_artist.is_empty() {
            0
        } else {
            self.get_or_create_artist(&track.orig_artist)?
        };
        let remixer_id = if track.remixer.is_empty() {
            0
        } else {
            self.get_or_create_artist(&track.remixer)?
        };

        let track_id = self.next_track_id;
        self.next_track_id += 1;

        pdb_track.id = TrackId(track_id);
        pdb_track.artist_id = ArtistId(artist_id);
        pdb_track.album_id = AlbumId(album_id);
        pdb_track.genre_id = GenreId(genre_id);
        pdb_track.key_id = KeyId(key_id);
        pdb_track.label_id = LabelId(label_id);
        pdb_track.artwork_id = ArtworkId(artwork_id);
        pdb_track.composer_id = ArtistId(composer_id);
        pdb_track.orig_artist_id = ArtistId(orig_artist_id);
        pdb_track.remixer_id = ArtistId(remixer_id);

        self.db().add_row(Row::Plain(PlainRow::Track(pdb_track)))?;

        self.track_ids.insert(track_id);
        if !track.file_path.is_empty() {
            self.tracks_by_path
                .insert(track.file_path.clone(), track_id);
        }

        // `comment` isn't observable after insertion; kept for the dedicated auto-pad test.
        let _ = comment;

        Ok(AddTrackOutcome {
            id: TrackId(track_id),
            is_new: true,
        })
    }

    /// Encode all strings, set scalar fields, grow `comment` past the 221-byte minimum. Allocates
    /// no IDs, inserts nothing.
    fn build_pdb_track(&self, track: &Track) -> Result<(crate::pdb::Track, String)> {
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
            DeviceSQLString::new("ON")?
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
                    publish_track_information: DeviceSQLString::new("ON")?,
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
            if Database::<fs::File>::validate_track_row_size(&pdb_track).is_ok() {
                break;
            }
            comment.push(' ');
        }

        Ok((pdb_track, comment))
    }

    /// Create a playlist folder and return its node ID.
    ///
    /// `parent_id` is [`PlaylistTreeNodeId::root()`] or an existing folder created by
    /// [`create_playlist_folder`](Self::create_playlist_folder).
    ///
    /// # Errors
    ///
    /// [`Error::UnknownForeignKey`] if `parent_id` isn't the root or an existing folder (a playlist
    /// id is rejected — only folders hold children).
    pub fn create_playlist_folder(
        &mut self,
        name: &str,
        parent_id: PlaylistTreeNodeId,
    ) -> Result<PlaylistTreeNodeId> {
        self.create_playlist_node(name, parent_id, true)
    }

    /// Create a playlist and return its node ID.
    ///
    /// `parent_id` is [`PlaylistTreeNodeId::root()`] or an existing folder created by
    /// [`create_playlist_folder`](Self::create_playlist_folder).
    ///
    /// # Errors
    ///
    /// [`Error::UnknownForeignKey`] if `parent_id` isn't the root or an existing folder.
    pub fn create_playlist(
        &mut self,
        name: &str,
        parent_id: PlaylistTreeNodeId,
    ) -> Result<PlaylistTreeNodeId> {
        self.create_playlist_node(name, parent_id, false)
    }

    fn create_playlist_node(
        &mut self,
        name: &str,
        parent_id: PlaylistTreeNodeId,
        is_folder: bool,
    ) -> Result<PlaylistTreeNodeId> {
        if parent_id.0 != 0
            && !self
                .playlist_nodes
                .get(&parent_id.0)
                .copied()
                .unwrap_or(false)
        {
            return Err(Error::UnknownForeignKey {
                kind: ForeignKeyKind::PlaylistNode,
                id: parent_id.0,
            });
        }

        let id = self.next_playlist_node_id;
        self.next_playlist_node_id += 1;

        let node = PlaylistTreeNode::new(
            PlaylistTreeNodeId(id),
            parent_id,
            DeviceSQLString::new(name)?,
            is_folder,
            0,
        );

        self.db()
            .add_row(Row::Plain(PlainRow::PlaylistTreeNode(node)))?;

        self.playlist_nodes.insert(id, is_folder);

        Ok(PlaylistTreeNodeId(id))
    }

    /// Append a track to the end of a playlist. The entry position is assigned automatically.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownForeignKey`] if `playlist_id` isn't an existing *playlist* (a folder id is
    /// rejected — tracks go into playlists only), or `track_id` isn't an existing track.
    pub fn add_track_to_playlist(
        &mut self,
        playlist_id: PlaylistTreeNodeId,
        track_id: TrackId,
    ) -> Result<()> {
        if !self
            .playlist_nodes
            .get(&playlist_id.0)
            .copied()
            .map(|is_folder| !is_folder)
            .unwrap_or(false)
        {
            return Err(Error::UnknownForeignKey {
                kind: ForeignKeyKind::PlaylistNode,
                id: playlist_id.0,
            });
        }
        if !self.track_ids.contains(&track_id.0) {
            return Err(Error::UnknownForeignKey {
                kind: ForeignKeyKind::Track,
                id: track_id.0,
            });
        }

        let entry_index = self
            .playlist_entry_counts
            .get(&playlist_id.0)
            .copied()
            .unwrap_or(0);

        let entry = PlaylistEntry {
            entry_index,
            track_id,
            playlist_id,
        };

        self.db()
            .add_row(Row::Plain(PlainRow::PlaylistEntry(entry)))?;

        self.playlist_entry_counts
            .insert(playlist_id.0, entry_index + 1);

        Ok(())
    }

    /// Flush pending writes and close the PDB. Prefer this over relying on `Drop`, which can't
    /// surface errors.
    ///
    /// # Errors
    ///
    /// If the flush or close fails.
    pub fn close(mut self) -> Result<()> {
        self.db
            .take()
            .expect("DeviceExportWriter.db always Some until close")
            .close()
    }

    fn get_or_create<K, RowFn>(
        db: &mut Database<fs::File>,
        map: &mut HashMap<K, u32>,
        next_id: &mut u32,
        key: K,
        key_empty: bool,
        build: RowFn,
    ) -> Result<u32>
    where
        K: Eq + std::hash::Hash + Clone,
        RowFn: FnOnce(u32, &K) -> Result<Row>,
    {
        if key_empty {
            return Ok(0);
        }
        if let Some(&id) = map.get(&key) {
            return Ok(id);
        }

        let id = *next_id;
        *next_id += 1;
        let row = build(id, &key)?;
        db.add_row(row)?;
        map.insert(key.clone(), id);
        Ok(id)
    }

    fn get_or_create_artist(&mut self, name: &str) -> Result<u32> {
        Self::get_or_create(
            self.db
                .as_mut()
                .expect("DeviceExportWriter.db always Some until close"),
            &mut self.artists_by_name,
            &mut self.next_artist_id,
            name.to_string(),
            name.is_empty(),
            |id, name| {
                Ok(Row::Plain(PlainRow::Artist(crate::pdb::Artist {
                    subtype: crate::pdb::Subtype(0x60),
                    index_shift: 0,
                    id: crate::pdb::ArtistId(id),
                    offsets: OffsetArrayContainer {
                        offsets: MaybeCalculated::Calculated,
                        inner: crate::pdb::TrailingName {
                            name: DeviceSQLString::new(name)?,
                        },
                    },
                })))
            },
        )
    }

    fn get_or_create_album(&mut self, name: &str, artist_id: u32) -> Result<u32> {
        Self::get_or_create(
            self.db
                .as_mut()
                .expect("DeviceExportWriter.db always Some until close"),
            &mut self.albums_by_artist_and_name,
            &mut self.next_album_id,
            (artist_id, name.to_string()),
            name.is_empty(),
            |id, (artist, name): &(u32, String)| {
                Ok(Row::Plain(PlainRow::Album(crate::pdb::Album {
                    subtype: crate::pdb::Subtype(0x80),
                    index_shift: 0,
                    unknown2: 0,
                    artist_id: crate::pdb::ArtistId(*artist),
                    id: crate::pdb::AlbumId(id),
                    unknown3: 0,
                    offsets: OffsetArrayContainer {
                        offsets: MaybeCalculated::Calculated,
                        inner: crate::pdb::TrailingName {
                            name: DeviceSQLString::new(name)?,
                        },
                    },
                })))
            },
        )
    }

    fn get_or_create_genre(&mut self, name: &str) -> Result<u32> {
        Self::get_or_create(
            self.db
                .as_mut()
                .expect("DeviceExportWriter.db always Some until close"),
            &mut self.genres_by_name,
            &mut self.next_genre_id,
            name.to_string(),
            name.is_empty(),
            |id, name| {
                Ok(Row::Plain(PlainRow::Genre(crate::pdb::Genre {
                    id: crate::pdb::GenreId(id),
                    name: DeviceSQLString::new(name)?,
                })))
            },
        )
    }

    fn get_or_create_key(&mut self, name: &str) -> Result<u32> {
        let canonical = canonical_key_name(name);
        Self::get_or_create(
            self.db
                .as_mut()
                .expect("DeviceExportWriter.db always Some until close"),
            &mut self.keys_by_canonical,
            &mut self.next_key_id,
            canonical.clone(),
            name.is_empty(),
            |id, _canonical| {
                // Store the canonical form so future lookups collide.
                let key_name = if canonical.is_empty() {
                    name
                } else {
                    &canonical
                };
                Ok(Row::Plain(PlainRow::Key(crate::pdb::Key {
                    id: crate::pdb::KeyId(id),
                    id2: id,
                    name: DeviceSQLString::new(key_name)?,
                })))
            },
        )
    }

    fn get_or_create_label(&mut self, name: &str) -> Result<u32> {
        Self::get_or_create(
            self.db
                .as_mut()
                .expect("DeviceExportWriter.db always Some until close"),
            &mut self.labels_by_name,
            &mut self.next_label_id,
            name.to_string(),
            name.is_empty(),
            |id, name| {
                Ok(Row::Plain(PlainRow::Label(crate::pdb::Label {
                    id: crate::pdb::LabelId(id),
                    name: DeviceSQLString::new(name)?,
                })))
            },
        )
    }

    /// Without the `artwork` feature, producing no artwork at all: a raw copy of an unknown
    /// format/size cannot be guaranteed readable by a CDJ, so we write nothing rather than risk a
    /// broken image. Returns the null id `0`; no PDB row is created.
    #[cfg(not(feature = "artwork"))]
    fn get_or_create_artwork(&mut self, _path: &str) -> Result<u32> {
        Ok(0)
    }

    /// With the `artwork` feature, resolve `path` (a host source file) to an artwork id: reuse an
    /// existing one for the same source, or copy the image into the export (resizing to 80×80
    /// thumbnail and 240×240 medium) and insert a new PDB `Artwork` row whose path is derived from
    /// the assigned id. An empty `path` yields the null id `0` (no row, no copy).
    #[cfg(feature = "artwork")]
    fn get_or_create_artwork(&mut self, path: &str) -> Result<u32> {
        if path.is_empty() {
            return Ok(0);
        }
        if let Some(&id) = self.artwork_by_path.get(path) {
            return Ok(id);
        }

        let id = self.next_artwork_id;
        self.copy_artwork_file(path, id)?;
        // Bump only after the copy succeeds, so a failure leaves no id gap and no orphan row.
        self.next_artwork_id += 1;

        let row = Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(id),
            path: DeviceSQLString::new(&artwork_device_path(id))?,
        }));
        self.db().add_row(row)?;

        self.artwork_by_path.insert(path.to_string(), id);
        Ok(id)
    }

    /// Decode `source`, resize to the 80×80 thumbnail and 240×240 medium, and write both as JPEG
    /// under their id-derived shard paths. Creates the shard directory. Decode/encode/IO errors
    /// propagate to the caller.
    #[cfg(feature = "artwork")]
    fn copy_artwork_file(&self, source: &str, id: u32) -> Result<()> {
        use image::imageops::FilterType;
        use image::ImageReader;
        use std::path::PathBuf;

        let source_path = PathBuf::from(source);
        let img = ImageReader::open(&source_path)
            .map_err(|e| Error::ArtworkError {
                path: source_path.clone(),
                message: e.to_string(),
            })?
            .decode()
            .map_err(|e| Error::ArtworkError {
                path: source_path.clone(),
                message: e.to_string(),
            })?;

        let dest = self.layout.artwork_file(id);
        fs::create_dir_all(dest.parent().expect("artwork path always has a folder"))?;

        let thumb = img.resize_to_fill(80, 80, FilterType::Lanczos3);
        thumb
            .to_rgb8()
            .save(&dest)
            .map_err(|e| Error::ArtworkError {
                path: dest.clone(),
                message: e.to_string(),
            })?;

        let medium_dest = self.layout.artwork_m_file(id);
        let medium = img.resize_to_fill(240, 240, FilterType::Lanczos3);
        medium
            .to_rgb8()
            .save(&medium_dest)
            .map_err(|e| Error::ArtworkError {
                path: medium_dest.clone(),
                message: e.to_string(),
            })?;

        Ok(())
    }
}

fn write_setting_file(path: &Path, setting: &Setting) -> Result<()> {
    let mut buf = Cursor::new(Vec::new());
    setting
        .write_options(&mut buf, binrw::Endian::Little, (false,))
        .map_err(Error::BinrwError)?;
    fs::write(path, buf.into_inner())?;
    Ok(())
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

    // open() must reuse an existing named row instead of duplicating it.
    #[test]
    fn open_then_add_does_not_duplicate_named_row() {
        let dir =
            std::env::temp_dir().join(format!("rekordcrate-export-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let t = Track {
            title: "song".into(),
            artist: "Dup Artist".into(),
            album: "Dup Album".into(),
            genre: "Dup Genre".into(),
            filename: "song.mp3".into(),
            file_path: "/Contents/song.mp3".into(),
            ..Default::default()
        };

        let mut dev = DeviceExportWriter::create(&dir).unwrap();
        dev.add_track(&t).unwrap();
        dev.close().unwrap();

        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        dev.add_track(&t).unwrap();
        dev.close().unwrap();

        // Dedup must survive to disk: exactly one "Dup Artist".
        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        let count = dev
            .db()
            .iter_rows::<crate::pdb::Artist>()
            .unwrap()
            .filter(|row| Ok(row.offsets.name.clone().into_string().unwrap() == "Dup Artist"))
            .count()
            .unwrap();
        assert_eq!(count, 1, "open() must not duplicate an existing artist");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // add_track is idempotent on a non-empty file_path, in-session and across open()/close().
    #[test]
    fn add_track_dedups_on_file_path() {
        let dir = std::env::temp_dir().join(format!(
            "rekordcrate-export-track-dedup-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);

        let t = Track {
            title: "song".into(),
            filename: "song.mp3".into(),
            file_path: "/Contents/song.mp3".into(),
            ..Default::default()
        };

        let mut dev = DeviceExportWriter::create(&dir).unwrap();
        let first = dev.add_track(&t).unwrap();
        let second = dev.add_track(&t).unwrap();
        assert_eq!(
            first.id, second.id,
            "re-adding the same file_path must return the existing id"
        );
        assert!(first.is_new, "the first add must be flagged new");
        assert!(!second.is_new, "the duplicate add must be flagged not new");
        dev.close().unwrap();

        // Across open(): the path is read back, so re-adding still dedups.
        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        let third = dev.add_track(&t).unwrap();
        assert_eq!(
            first.id, third.id,
            "re-adding after open() must return the existing id"
        );
        assert!(!third.is_new, "the post-reopen add must be flagged not new");
        dev.close().unwrap();

        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        let count = dev
            .db()
            .iter_rows::<crate::pdb::Track>()
            .unwrap()
            .count()
            .unwrap();
        assert_eq!(count, 1, "dedup must keep exactly one Track row on disk");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // The playlist methods reject ids that don't reference an existing row.
    #[test]
    fn playlist_methods_reject_unknown_foreign_keys() {
        let dir =
            std::env::temp_dir().join(format!("rekordcrate-export-fk-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let mut dev = DeviceExportWriter::create(&dir).unwrap();

        // parent_id root() is the tree root and always valid.
        let folder = dev
            .create_playlist_folder("Folder", PlaylistTreeNodeId::root())
            .unwrap();
        let playlist = dev.create_playlist("Playlist", folder).unwrap();

        // Unknown parent: 999 was never created.
        let err = dev
            .create_playlist("Orphan", PlaylistTreeNodeId(999))
            .unwrap_err();
        assert!(
            matches!(err, Error::UnknownForeignKey { kind, id } if kind == ForeignKeyKind::PlaylistNode && id == 999),
            "unknown parent_id must be rejected, got {err:?}"
        );

        // A valid track so add_track_to_playlist's track check has something to find.
        let t = Track {
            title: "song".into(),
            filename: "song.mp3".into(),
            file_path: "/Contents/song.mp3".into(),
            ..Default::default()
        };
        let track = dev.add_track(&t).unwrap().id;

        // Unknown playlist id.
        let err = dev
            .add_track_to_playlist(PlaylistTreeNodeId(999), track)
            .unwrap_err();
        assert!(
            matches!(err, Error::UnknownForeignKey { kind, id } if kind == ForeignKeyKind::PlaylistNode && id == 999),
            "unknown playlist_id must be rejected, got {err:?}"
        );

        // Unknown track id.
        let err = dev
            .add_track_to_playlist(playlist, TrackId(999))
            .unwrap_err();
        assert!(
            matches!(err, Error::UnknownForeignKey { kind, id } if kind == ForeignKeyKind::Track && id == 999),
            "unknown track_id must be rejected, got {err:?}"
        );

        // The happy path still works.
        dev.add_track_to_playlist(playlist, track).unwrap();

        let _ = std::fs::remove_dir_all(&dir);
    }

    // A playlist id is rejected as a parent (only folders hold children), and a folder id is
    // rejected as an add_track_to_playlist target (only playlists hold tracks).
    #[test]
    fn playlist_tree_enforces_folder_vs_playlist_roles() {
        let dir = std::env::temp_dir().join(format!(
            "rekordcrate-export-tree-shape-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);

        let mut dev = DeviceExportWriter::create(&dir).unwrap();
        let folder = dev
            .create_playlist_folder("Folder", PlaylistTreeNodeId::root())
            .unwrap();
        let playlist = dev.create_playlist("Playlist", folder).unwrap();

        // The playlist exists, but it's a leaf, not a folder.
        let err = dev.create_playlist("Child", playlist).unwrap_err();
        assert!(
            matches!(err, Error::UnknownForeignKey { kind, id } if kind == ForeignKeyKind::PlaylistNode && id == playlist.0),
            "a playlist id must be rejected as a parent, got {err:?}"
        );

        // The folder exists, but tracks go into playlists only.
        let t = Track {
            title: "song".into(),
            filename: "song.mp3".into(),
            file_path: "/Contents/song.mp3".into(),
            ..Default::default()
        };
        let track = dev.add_track(&t).unwrap().id;
        let err = dev.add_track_to_playlist(folder, track).unwrap_err();
        assert!(
            matches!(err, Error::UnknownForeignKey { kind, id } if kind == ForeignKeyKind::PlaylistNode && id == folder.0),
            "a folder id must be rejected as a track container, got {err:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // add_track_to_playlist assigns dense entry indices 0,1,2,…, in-session and across
    // open()/close() (where the counter is rebuilt from existing rows).
    #[test]
    fn add_track_to_playlist_auto_indexes() {
        let dir = std::env::temp_dir().join(format!(
            "rekordcrate-export-autoindex-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);

        let mk = |i: u32| Track {
            title: format!("song{i}"),
            filename: format!("song{i}.mp3"),
            file_path: format!("/Contents/song{i}.mp3"),
            ..Default::default()
        };

        let mut dev = DeviceExportWriter::create(&dir).unwrap();
        let playlist = dev
            .create_playlist("P", PlaylistTreeNodeId::root())
            .unwrap();
        let t0 = dev.add_track(&mk(0)).unwrap().id;
        let t1 = dev.add_track(&mk(1)).unwrap().id;
        let t2 = dev.add_track(&mk(2)).unwrap().id;
        dev.add_track_to_playlist(playlist, t0).unwrap();
        dev.add_track_to_playlist(playlist, t1).unwrap();
        dev.add_track_to_playlist(playlist, t2).unwrap();
        dev.close().unwrap();

        // After open() the counter is rebuilt, so one more append lands at index 3.
        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        let t3 = dev.add_track(&mk(3)).unwrap().id;
        dev.add_track_to_playlist(playlist, t3).unwrap();
        dev.close().unwrap();

        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        let mut indices: Vec<u32> = dev
            .db()
            .iter_rows::<crate::pdb::PlaylistEntry>()
            .unwrap()
            .map(|e| Ok(e.entry_index))
            .collect()
            .unwrap();
        indices.sort_unstable();
        assert_eq!(
            indices,
            vec![0, 1, 2, 3],
            "entry indices must be dense from 0"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // A bare title + file_path (well under 221 bytes) must succeed, with the writer growing
    // `comment` with spaces internally.
    #[test]
    fn add_track_auto_pads_under_minimum_row_size() {
        let dir = std::env::temp_dir().join(format!(
            "rekordcrate-export-autopad-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);

        let t = Track {
            title: "tiny".into(),
            file_path: "/Contents/tiny.mp3".into(),
            ..Default::default()
        };

        let mut dev = DeviceExportWriter::create(&dir).unwrap();
        let outcome = dev.add_track(&t).unwrap();
        assert!(outcome.is_new, "a fresh track must be flagged new");

        dev.close().unwrap();
        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        let row = dev
            .db()
            .iter_rows::<crate::pdb::Track>()
            .unwrap()
            .find(|row| Ok(row.id == outcome.id))
            .unwrap()
            .unwrap();
        let comment = row.offsets.inner.comment.clone().into_string().unwrap();
        assert!(
            comment.chars().all(|c| c == ' '),
            "auto-pad must grow the comment with spaces only, got {comment:?}"
        );
        assert!(
            !comment.is_empty(),
            "auto-pad must have grown the comment past empty"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // The sharding rule id/20+1, zero-padded to 5 digits, derived from the captured fixtures.
    #[cfg(feature = "artwork")]
    #[test]
    fn artwork_device_path_sharding() {
        assert_eq!(artwork_folder(1), "00001");
        assert_eq!(artwork_folder(19), "00001");
        assert_eq!(artwork_folder(20), "00002");
        assert_eq!(artwork_folder(39), "00002");
        assert_eq!(artwork_folder(40), "00003");
        // The PDB path always references the thumbnail under its shard folder.
        assert_eq!(artwork_device_path(1), "/PIONEER/Artwork/00001/a1.jpg");
        assert_eq!(artwork_device_path(20), "/PIONEER/Artwork/00002/a20.jpg");
    }

    #[cfg(feature = "artwork")]
    fn write_test_jpeg(path: &std::path::Path, w: u32, h: u32) {
        use image::{ImageBuffer, Rgb, RgbImage};
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let img: RgbImage = ImageBuffer::from_fn(w, h, |x, y| {
            Rgb([(x * 10 % 255) as u8, (y * 10 % 255) as u8, 128])
        });
        img.save(path).unwrap();
    }

    // Under the artwork feature, add_track copies the source into the sharded dir under both the
    // 80×80 thumbnail name and the 240×240 medium name, and stores the thumbnail path in the PDB.
    #[cfg(feature = "artwork")]
    #[test]
    fn add_track_copies_artwork() {
        let dir = std::env::temp_dir().join(format!(
            "rekordcrate-export-artwork-copy-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);

        let src = dir.join("src.png");
        write_test_jpeg(&src, 200, 200);

        let t = Track {
            title: "song".into(),
            filename: "song.mp3".into(),
            file_path: "/Contents/song.mp3".into(),
            artwork_path: src.to_string_lossy().into_owned(),
            ..Default::default()
        };

        let mut dev = DeviceExportWriter::create(&dir).unwrap();
        dev.add_track(&t).unwrap();
        dev.close().unwrap();

        let thumb = dir.join("PIONEER/Artwork/00001/a1.jpg");
        let medium = dir.join("PIONEER/Artwork/00001/a1_m.jpg");
        assert!(thumb.exists(), "thumbnail must be written at {thumb:?}");
        assert!(medium.exists(), "medium must be written at {medium:?}");

        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        let row = dev
            .db()
            .iter_rows::<crate::pdb::Artwork>()
            .unwrap()
            .find(|_| Ok(true))
            .unwrap()
            .unwrap();
        let path = row.path.clone().into_string().unwrap();
        assert_eq!(
            path, "/PIONEER/Artwork/00001/a1.jpg",
            "PDB Artwork.path must reference the thumbnail"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // Two tracks sharing one source dedup to a single artwork: one thumbnail, one medium on disk,
    // one PDB row, and both tracks reference the same artwork id.
    #[cfg(feature = "artwork")]
    #[test]
    fn add_track_dedups_artwork_by_source() {
        let dir = std::env::temp_dir().join(format!(
            "rekordcrate-export-artwork-dedup-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);

        let src = dir.join("cover.png");
        write_test_jpeg(&src, 100, 100);

        let mk = |n: usize| Track {
            title: format!("song{n}"),
            filename: format!("song{n}.mp3"),
            file_path: format!("/Contents/song{n}.mp3"),
            artwork_path: src.to_string_lossy().into_owned(),
            ..Default::default()
        };

        let mut dev = DeviceExportWriter::create(&dir).unwrap();
        dev.add_track(&mk(0)).unwrap();
        dev.add_track(&mk(1)).unwrap();
        dev.close().unwrap();

        // One shard folder, one thumbnail, one medium.
        let shard = dir.join("PIONEER/Artwork/00001");
        let entries: Vec<_> = std::fs::read_dir(&shard)
            .unwrap()
            .collect::<std::io::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(
            entries.len(),
            2,
            "expected exactly thumbnail + medium in {shard:?}"
        );

        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        let count = dev
            .db()
            .iter_rows::<crate::pdb::Artwork>()
            .unwrap()
            .count()
            .unwrap();
        assert_eq!(count, 1, "dedup must keep exactly one Artwork row on disk");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // The written files are resized to the Rekordbox dimensions regardless of source size.
    #[cfg(feature = "artwork")]
    #[test]
    fn add_track_artwork_dimensions() {
        use image::ImageReader;

        let dir = std::env::temp_dir().join(format!(
            "rekordcrate-export-artwork-dims-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);

        let src = dir.join("cover.png");
        write_test_jpeg(&src, 200, 200);

        let t = Track {
            title: "song".into(),
            filename: "song.mp3".into(),
            file_path: "/Contents/song.mp3".into(),
            artwork_path: src.to_string_lossy().into_owned(),
            ..Default::default()
        };

        let mut dev = DeviceExportWriter::create(&dir).unwrap();
        dev.add_track(&t).unwrap();
        dev.close().unwrap();

        let thumb = ImageReader::open(dir.join("PIONEER/Artwork/00001/a1.jpg"))
            .unwrap()
            .into_dimensions()
            .unwrap();
        assert_eq!(thumb, (80, 80), "thumbnail must be 80×80");

        let medium = ImageReader::open(dir.join("PIONEER/Artwork/00001/a1_m.jpg"))
            .unwrap()
            .into_dimensions()
            .unwrap();
        assert_eq!(medium, (240, 240), "medium must be 240×240");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // Without the artwork feature, setting artwork_path is a silent no-op: no Artwork dir, no row,
    // and the track's artwork_id is the null id 0.
    #[cfg(not(feature = "artwork"))]
    #[test]
    fn artwork_noop_without_feature() {
        let dir = std::env::temp_dir().join(format!(
            "rekordcrate-export-artwork-noop-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);

        let t = Track {
            title: "song".into(),
            filename: "song.mp3".into(),
            file_path: "/Contents/song.mp3".into(),
            artwork_path: "/nonexistent/cover.jpg".into(),
            ..Default::default()
        };

        let mut dev = DeviceExportWriter::create(&dir).unwrap();
        let outcome = dev.add_track(&t).unwrap();
        dev.close().unwrap();

        assert!(
            !dir.join("PIONEER/Artwork").exists(),
            "no artwork dir must be created without the feature"
        );

        let mut dev = DeviceExportWriter::open(&dir).unwrap();
        let row = dev
            .db()
            .iter_rows::<crate::pdb::Track>()
            .unwrap()
            .find(|row| Ok(row.id == outcome.id))
            .unwrap()
            .unwrap();
        assert_eq!(
            row.artwork_id.0, 0,
            "without the feature the track must reference the null artwork id"
        );

        let artwork_count = dev
            .db()
            .iter_rows::<crate::pdb::Artwork>()
            .unwrap()
            .count()
            .unwrap();
        assert_eq!(artwork_count, 0, "no Artwork row must be written");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
