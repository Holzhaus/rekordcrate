// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! [`DeviceExport`]: one handle to read a Rekordbox USB export, build one from scratch, or add to
//! an existing one.
//!
//! ```no_run
//! use rekordcrate::{DeviceExport, PlaylistTreeNodeId, Track};
//!
//! let mut export = DeviceExport::create("/mnt/usb")?;
//! let track = export.add_track(&Track {
//!     title: "Song".into(),
//!     artist: "Artist".into(),
//!     file_path: "/Contents/Artist - Song.mp3".into(),
//!     ..Track::default()
//! })?;
//! let playlist = export.create_playlist("Warmup", PlaylistTreeNodeId::root())?;
//! export.add_track_to_playlist(playlist, track.id)?;
//! export.save()?;
//!
//! for (id, track) in export.tracks()? {
//!     println!("{id:?}: {} - {}", track.artist, track.title);
//! }
//! # Ok::<(), rekordcrate::Error>(())
//! ```
//!
//! This module never copies audio files into `Contents` or generates `PIONEER/USBANLZ` analysis
//! files (`ANLZ.*`). Also, the API is append-only, existing rows cannot be updated or deleted.
//! So if you want to make edits, you should either open the `pdb` files with
//! `pdb::io::Database::open` (and thus deal with the format intricacies yourself) or re-add via
//! a fresh export instead.
//!
//! [`Layout`] describes where each of the export's files lives on the device.

#[cfg(feature = "artwork")]
use crate::device::layout::artwork_folder;
use crate::device::layout::{Layout, DAT_FILES};
use crate::device::playlist::{get_playlists, PlaylistNode};
use crate::device::settings::Settings;
use crate::device::track::{Track, ON};
use crate::pdb::ext::{
    ExtPageType, ParentId, TagId, TagOrCategory, TagOrCategoryStrings, TrackTag,
};
use crate::pdb::io::Database;
use crate::pdb::offset_array::OffsetArrayContainer;
use crate::pdb::string::DeviceSQLString;
use crate::pdb::{
    Album, AlbumId, Artist, ArtistId, Artwork, ArtworkId, DatabaseType, Genre, GenreId, Key, KeyId,
    Label, LabelId, PageHeapObject, PageType, PlainPageType, PlaylistEntry, PlaylistTreeNode,
    PlaylistTreeNodeId, RowVariant, Subtype, Track as PdbTrack, TrackId, TrailingName,
};
use crate::setting::{Setting, SettingType};
use crate::util::{ForeignKeyKind, MaybeCalculated, Rating};
use crate::{Error, Result};
use binrw::BinWrite;
use fallible_iterator::FallibleIterator;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::Hash;
use std::io::{Cursor, Write};
use std::num::NonZero;
use std::path::{Path, PathBuf};

/// A PDB database held entirely in memory; [`DeviceExport::save`] writes its image out.
type MemoryDatabase = Database<Cursor<Vec<u8>>>;

/// The outcome of [`DeviceExport::add_track`]: either a freshly inserted track or an
/// existing one returned because its `file_path` already existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddTrackOutcome {
    /// The track id in the export.
    pub id: TrackId,
    /// `true` if a new row was inserted; `false` if an existing track was returned unchanged.
    pub is_new: bool,
}

/// Opaque id of a tag category created by [`DeviceExport::create_tag_category`]. Pass it to
/// [`DeviceExport::add_tags_to_track`] to group leaf tags under the category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagCategoryId(pub u32);

/// A handle to a Rekordbox device export: PDB database, setting files, directory structure.
///
/// [`open`](Self::open) loads an existing export into memory, [`create`](Self::create) starts an
/// empty one. Either way every change stays in memory until [`save`](Self::save); dropping the
/// handle without saving leaves the disk as it was.
#[derive(Debug)]
pub struct DeviceExport {
    layout: Layout,
    db: MemoryDatabase,
    /// `exportExt.pdb`, when the export carries one or a tag call created it.
    ext_db: Option<MemoryDatabase>,
    /// Whether `ext_db` has changes [`save`](Self::save) has not written yet.
    ext_dirty: bool,
    /// Set by [`create`](Self::create): the next [`save`](Self::save) also lays down the default
    /// directory tree and setting files.
    scaffold_pending: bool,
    /// Files other than the databases waiting for [`save`](Self::save) (imported artwork).
    pending_files: Vec<(PathBuf, Vec<u8>)>,
    next_track_id: u32,
    next_artist_id: u32,
    next_album_id: u32,
    next_genre_id: u32,
    next_key_id: u32,
    next_label_id: u32,
    next_artwork_id: u32,
    next_playlist_node_id: u32,
    track_ids: HashSet<u32>,
    tracks_by_path: HashMap<String, u32>,
    /// `id -> is_folder`. Root (id 0) is implicit — always valid as a parent, always a folder.
    playlist_nodes: HashMap<u32, bool>,
    /// `parent id -> next sort_order` among its children (0-based, as on real exports).
    next_sort_order: HashMap<u32, u32>,
    /// `playlist id -> next entry_index` (1-based, as on real exports).
    next_entry_index: HashMap<u32, u32>,
    artists_by_name: HashMap<String, u32>,
    albums_by_artist_and_name: HashMap<(u32, String), u32>,
    genres_by_name: HashMap<String, u32>,
    keys_by_name: HashMap<String, u32>,
    labels_by_name: HashMap<String, u32>,
    /// `Artwork.path -> id`.
    artwork_by_path: HashMap<String, u32>,
    /// Host-side source -> device path, so importing the same image twice reuses one artwork.
    #[cfg(feature = "artwork")]
    artwork_by_source: HashMap<PathBuf, String>,
    tags: TagIndex,
}

/// What the handle remembers about the tag tables of `exportExt.pdb`, so tag calls can append to
/// them without rescanning.
#[derive(Debug, Default)]
struct TagIndex {
    /// Every category and leaf id in use; the two share one id space.
    ids: HashSet<u32>,
    /// Where [`allocate_id`](Self::allocate_id) resumes its search for a free id.
    next_id: u32,
    /// Category ids, for `add_tags_to_track` foreign-key validation.
    categories: HashSet<u32>,
    /// `(category id, label) -> leaf id`.
    leaves: HashMap<(u32, String), u32>,
    /// Next `position` for a category (0-based, as on real exports).
    next_category_position: u32,
    /// `category id -> next leaf position` (dense from 0 within a category).
    next_leaf_position: HashMap<u32, u32>,
    /// Per-row counter driving `index_shift` (see [`tag_row`]).
    next_row_index: u32,
    /// The `(track id, tag id)` junction rows, so a tag is linked to a track only once.
    track_tags: HashSet<(u32, u32)>,
}

impl TagIndex {
    /// Rebuild the index from an existing ext PDB, so later tag calls append to it instead of
    /// colliding with (or duplicating) what is already there.
    fn scan(ext_db: &mut MemoryDatabase) -> Result<Self> {
        let mut index = Self::default();

        ext_db.iter_rows::<TagOrCategory>()?.for_each(|tag| {
            index.ids.insert(tag.id.0);
            index.next_row_index = index
                .next_row_index
                .max(u32::from(tag.index_shift) / 0x20 + 1);
            let next_position = tag.position.saturating_add(1);
            if tag.raw_is_category != 0 {
                index.categories.insert(tag.id.0);
                index.next_category_position = index.next_category_position.max(next_position);
            } else {
                let category = tag.parent_id.0.map_or(0, NonZero::get);
                if let Ok(label) = tag.offsets.inner.name.clone().into_string() {
                    index.leaves.entry((category, label)).or_insert(tag.id.0);
                }
                raise(&mut index.next_leaf_position, category, next_position);
            }
            Ok(())
        })?;

        ext_db.iter_rows::<TrackTag>()?.for_each(|track_tag| {
            index
                .track_tags
                .insert((track_tag.track_id.0, track_tag.tag_id.0));
            Ok(())
        })?;

        Ok(index)
    }

    /// Hand out the lowest free tag id. Ids start at 1 because 0 is the null parent. Rekordbox
    /// gives leaf tags large random ids, so "one past the highest" is not an option here: it
    /// would overflow on real exports.
    fn allocate_id(&mut self) -> u32 {
        let mut id = self.next_id.max(1);
        while self.ids.contains(&id) {
            id = id.wrapping_add(1).max(1);
        }
        self.ids.insert(id);
        self.next_id = id.wrapping_add(1);
        id
    }

    fn allocate_row_index(&mut self) -> u32 {
        let row_index = self.next_row_index;
        self.next_row_index += 1;
        row_index
    }
}

/// Id-derived `Artwork.path`: `/PIONEER/Artwork/{folder}/a{id}.jpg` (thumbnail; `_m` not tracked).
#[cfg(feature = "artwork")]
fn artwork_device_path(id: u32) -> String {
    format!("/PIONEER/Artwork/{}/a{id}.jpg", artwork_folder(id))
}

fn default_setting(setting_type: SettingType) -> Setting {
    match setting_type {
        SettingType::DevSetting => Setting::default_devsetting(),
        SettingType::DJMMySetting => Setting::default_djmmysetting(),
        SettingType::MySetting => Setting::default_mysetting(),
        SettingType::MySetting2 => Setting::default_mysetting2(),
    }
}

impl DeviceExport {
    /// Start a new, empty device export rooted at `device_root`.
    ///
    /// The first [`save`](Self::save) creates the `PIONEER`, `PIONEER/USBANLZ`, and `Contents`
    /// directories, the PDB database, and the default setting files — replacing the database and
    /// setting files of an export already at `device_root`.
    pub fn create(device_root: impl AsRef<Path>) -> Result<Self> {
        let layout = Layout::new(device_root.as_ref().to_path_buf());

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

        let mut db = Database::create(
            Cursor::new(Vec::new()),
            DatabaseType::Plain,
            table_page_types,
        )?;

        crate::pdb::defaults::insert_default_colors(&mut db)?;
        crate::pdb::defaults::insert_default_columns(&mut db)?;
        crate::pdb::defaults::insert_default_menus(&mut db)?;

        let mut export = Self::from_databases(layout, db, None)?;
        export.scaffold_pending = true;
        Ok(export)
    }

    /// Open an existing device export at the given path, loading its databases into memory.
    ///
    /// Scans the tables for their highest ids, so rows added later get ids that do not conflict,
    /// and for the names already present, so they are reused rather than duplicated. Tags are
    /// recovered from `exportExt.pdb` the same way; if that file is absent it is created on the
    /// first tag call, as in [`create`](Self::create).
    pub fn open(device_root: impl AsRef<Path>) -> Result<Self> {
        let layout = Layout::new(device_root.as_ref().to_path_buf());
        let db = read_database(&layout.export_pdb(), DatabaseType::Plain)?;
        let ext_path = layout.export_ext_pdb();
        let ext_db = if ext_path.exists() {
            Some(read_database(&ext_path, DatabaseType::Ext)?)
        } else {
            None
        };
        Self::from_databases(layout, db, ext_db)
    }

    /// Build the handle around loaded (or freshly created) databases, scanning them for the
    /// next free ids and the dedup indices that keep later lookups O(1).
    fn from_databases(
        layout: Layout,
        mut db: MemoryDatabase,
        mut ext_db: Option<MemoryDatabase>,
    ) -> Result<Self> {
        // A name that cannot be decoded is left out of its index: the row keeps its id, it just
        // cannot be found (and so reused) by name.
        let text = |string: &DeviceSQLString| string.clone().into_string().ok();

        let mut track_ids = HashSet::new();
        let mut tracks_by_path = HashMap::new();
        let mut next_track_id = 1;
        db.iter_rows::<PdbTrack>()?.for_each(|track| {
            track_ids.insert(track.id.0);
            next_track_id = next_track_id.max(track.id.0.saturating_add(1));
            if let Some(path) = text(&track.offsets.file_path).filter(|path| !path.is_empty()) {
                tracks_by_path.entry(path).or_insert(track.id.0);
            }
            Ok(())
        })?;

        let (next_artist_id, artists_by_name) =
            scan_table::<Artist, _>(&mut db, |row| (row.id.0, text(&row.offsets.name)))?;
        let (next_album_id, albums_by_artist_and_name) = scan_table::<Album, _>(&mut db, |row| {
            let key = text(&row.offsets.name).map(|name| (row.artist_id.0, name));
            (row.id.0, key)
        })?;
        let (next_genre_id, genres_by_name) =
            scan_table::<Genre, _>(&mut db, |row| (row.id.0, text(&row.name)))?;
        let (next_key_id, keys_by_name) =
            scan_table::<Key, _>(&mut db, |row| (row.id.0, text(&row.name)))?;
        let (next_label_id, labels_by_name) =
            scan_table::<Label, _>(&mut db, |row| (row.id.0, text(&row.name)))?;
        let (next_artwork_id, artwork_by_path) =
            scan_table::<Artwork, _>(&mut db, |row| (row.id.0, text(&row.path)))?;

        let mut playlist_nodes = HashMap::new();
        let mut next_sort_order = HashMap::new();
        let mut next_playlist_node_id = 1;
        db.iter_rows::<PlaylistTreeNode>()?.for_each(|node| {
            playlist_nodes.insert(node.id.0, node.is_folder());
            next_playlist_node_id = next_playlist_node_id.max(node.id.0.saturating_add(1));
            raise(
                &mut next_sort_order,
                node.parent_id.0,
                node.sort_order.saturating_add(1),
            );
            Ok(())
        })?;

        // One past the highest index rather than the row count, so sparse indices in an existing
        // export are never reused.
        let mut next_entry_index = HashMap::new();
        db.iter_rows::<PlaylistEntry>()?.for_each(|entry| {
            raise(
                &mut next_entry_index,
                entry.playlist_id.0,
                entry.entry_index.saturating_add(1),
            );
            Ok(())
        })?;

        let tags = match ext_db.as_mut() {
            Some(ext_db) => TagIndex::scan(ext_db)?,
            None => TagIndex::default(),
        };

        Ok(Self {
            layout,
            db,
            ext_db,
            ext_dirty: false,
            scaffold_pending: false,
            pending_files: Vec::new(),
            next_track_id,
            next_artist_id,
            next_album_id,
            next_genre_id,
            next_key_id,
            next_label_id,
            next_artwork_id,
            next_playlist_node_id,
            track_ids,
            tracks_by_path,
            playlist_nodes,
            next_sort_order,
            next_entry_index,
            artists_by_name,
            albums_by_artist_and_name,
            genres_by_name,
            keys_by_name,
            labels_by_name,
            artwork_by_path,
            #[cfg(feature = "artwork")]
            artwork_by_source: HashMap::new(),
            tags,
        })
    }

    /// Write the export to disk.
    ///
    /// Every file is written to a temporary sibling and renamed into place, and `export.pdb`
    /// goes last, so an interrupted save leaves the previous `export.pdb` in place rather than a
    /// half-written one. The handle stays usable: keep adding and save again, or retry a failed
    /// save.
    pub fn save(&mut self) -> Result<()> {
        fs::create_dir_all(self.layout.rekordbox_dir())?;

        if self.scaffold_pending {
            fs::create_dir_all(self.layout.usbanlz_dir())?;
            fs::create_dir_all(self.layout.contents_dir())?;
            for &(filename, setting_type) in DAT_FILES {
                write_atomic(
                    &self.layout.dat_path(filename),
                    &setting_bytes(&default_setting(setting_type))?,
                )?;
            }
        }

        for (path, bytes) in &self.pending_files {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            write_atomic(path, bytes)?;
        }

        if self.ext_dirty {
            if let Some(ext_db) = self.ext_db.as_mut() {
                write_database(ext_db, &self.layout.export_ext_pdb())?;
            }
        }

        write_database(&mut self.db, &self.layout.export_pdb())?;

        // Cleared only once everything landed, so a failed save can be retried as a whole.
        self.scaffold_pending = false;
        self.pending_files.clear();
        self.ext_dirty = false;
        Ok(())
    }

    /// The device root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        self.layout.root()
    }

    /// Where the export's files live on disk, for callers that want to reach them directly.
    #[must_use]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Load the setting files from disk; see [`Settings::load`]. A [`create`](Self::create)d
    /// export has none until its first [`save`](Self::save).
    #[must_use]
    pub fn load_settings(&self) -> Settings {
        Settings::load(&self.layout)
    }

    /// All tracks of the export, with their foreign keys resolved to names — the same [`Track`]
    /// shape [`add_track`](Self::add_track) takes. A foreign key naming no row reads as empty.
    pub fn tracks(&mut self) -> Result<Vec<(TrackId, Track)>> {
        let db = &mut self.db;
        let artists = names_by_id::<Artist>(db, |r| (r.id.0, &r.offsets.name))?;
        let albums = names_by_id::<Album>(db, |r| (r.id.0, &r.offsets.name))?;
        let genres = names_by_id::<Genre>(db, |r| (r.id.0, &r.name))?;
        let keys = names_by_id::<Key>(db, |r| (r.id.0, &r.name))?;
        let labels = names_by_id::<Label>(db, |r| (r.id.0, &r.name))?;
        let artwork = names_by_id::<Artwork>(db, |r| (r.id.0, &r.path))?;

        let name = |names: &HashMap<u32, String>, id: u32| -> String {
            names.get(&id).cloned().unwrap_or_default()
        };

        db.iter_rows::<PdbTrack>()?
            .map(|row| {
                let text = |string: &DeviceSQLString| string.clone().into_string();
                let strings = &row.offsets.inner;
                let track = Track {
                    title: text(&strings.title)?,
                    artist: name(&artists, row.artist_id.0),
                    album: name(&albums, row.album_id.0),
                    genre: name(&genres, row.genre_id.0),
                    key: name(&keys, row.key_id.0),
                    label: name(&labels, row.label_id.0),
                    composer: name(&artists, row.composer_id.0),
                    remixer: name(&artists, row.remixer_id.0),
                    orig_artist: name(&artists, row.orig_artist_id.0),
                    // Undo the minimum-row-size padding `add_track` may have appended.
                    comment: text(&strings.comment)?.trim_end_matches(' ').to_string(),
                    isrc: text(&strings.isrc)?,
                    lyricist: text(&strings.lyricist)?,
                    mix_name: text(&strings.mix_name)?,
                    release_date: text(&strings.release_date)?,
                    date_added: text(&strings.date_added)?,
                    file_path: text(&strings.file_path)?,
                    filename: text(&strings.filename)?,
                    artwork_path: name(&artwork, row.artwork_id.0),
                    message: text(&strings.message)?,
                    tempo: row.tempo as f32 / 100.0,
                    bitrate: row.bitrate,
                    sample_rate: row.sample_rate,
                    sample_depth: row.sample_depth,
                    duration_secs: row.duration,
                    file_size: row.file_size,
                    track_number: row.track_number,
                    disc_number: row.disc_number,
                    year: row.year,
                    play_count: row.play_count,
                    rating: Rating::from(row.rating),
                    color: row.color.clone(),
                    file_type: row.file_type.clone(),
                    autoload_hotcues: text(&strings.autoload_hotcues)? == ON,
                };
                Ok((row.id, track))
            })
            .collect()
    }

    /// The playlist tree: the root's folders and playlists, folders carrying their children, each
    /// level in display order.
    pub fn playlists(&mut self) -> Result<Vec<PlaylistNode>> {
        get_playlists(&mut self.db)
    }

    /// The tracks of a playlist, in playlist order.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownForeignKey`] if `playlist_id` isn't an existing *playlist* (folders hold
    /// no tracks).
    pub fn playlist_tracks(&mut self, playlist_id: PlaylistTreeNodeId) -> Result<Vec<TrackId>> {
        self.require_playlist(playlist_id)?;
        let mut entries: Vec<(u32, TrackId)> = self
            .db
            .iter_rows::<PlaylistEntry>()?
            .filter(|entry| Ok(entry.playlist_id == playlist_id))
            .map(|entry| Ok((entry.entry_index, entry.track_id)))
            .collect()?;
        entries.sort_by_key(|&(index, _)| index);
        Ok(entries.into_iter().map(|(_, track_id)| track_id).collect())
    }

    /// Add a track to the export.
    ///
    /// Resolves (creating where needed) the Artist, Album, Genre, Key, Label, and Artwork rows,
    /// then inserts a Track row pointing at them by ID.
    ///
    /// Idempotent on non-empty `file_path`: if a track with that path was already added (this
    /// session or read back by [`open`](Self::open)), the existing [`TrackId`] is returned and no
    /// new row is inserted. Tracks with an empty `file_path` are always inserted. Use
    /// [`AddTrackOutcome::is_new`] to tell the cases apart.
    ///
    /// # Errors
    ///
    /// If any of the track's strings cannot be encoded, or a row is too large for a page. The
    /// track row itself is encoded and measured before any id is allocated or row inserted, so
    /// a bad one leaves the export untouched.
    pub fn add_track(&mut self, track: &Track) -> Result<AddTrackOutcome> {
        if !track.file_path.is_empty() {
            if let Some(&id) = self.tracks_by_path.get(&track.file_path) {
                return Ok(AddTrackOutcome {
                    id: TrackId(id),
                    is_new: false,
                });
            }
        }

        let mut pdb_track = track.to_row()?;
        // The foreign keys filled in below are fixed-width, so the size is already final.
        self.db.check_row_size(pdb_track.heap_bytes_required(()))?;

        let artist_id = self.get_or_create_artist(&track.artist)?;
        pdb_track.artist_id = ArtistId(artist_id);
        pdb_track.album_id = AlbumId(self.get_or_create_album(&track.album, artist_id)?);
        pdb_track.genre_id = GenreId(self.get_or_create_genre(&track.genre)?);
        pdb_track.key_id = KeyId(self.get_or_create_key(&track.key)?);
        pdb_track.label_id = LabelId(self.get_or_create_label(&track.label)?);
        pdb_track.artwork_id = ArtworkId(self.get_or_create_artwork(&track.artwork_path)?);
        pdb_track.composer_id = ArtistId(self.get_or_create_artist(&track.composer)?);
        pdb_track.orig_artist_id = ArtistId(self.get_or_create_artist(&track.orig_artist)?);
        pdb_track.remixer_id = ArtistId(self.get_or_create_artist(&track.remixer)?);

        let track_id = self.next_track_id;
        pdb_track.id = TrackId(track_id);
        self.db.add_row(pdb_track)?;

        self.next_track_id += 1;
        self.track_ids.insert(track_id);
        if !track.file_path.is_empty() {
            self.tracks_by_path
                .insert(track.file_path.clone(), track_id);
        }

        Ok(AddTrackOutcome {
            id: TrackId(track_id),
            is_new: true,
        })
    }

    /// Create a playlist folder at the end of `parent_id`'s children and return its node ID.
    ///
    /// `parent_id` is [`PlaylistTreeNodeId::root()`] or an existing folder.
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

    /// Create a playlist at the end of `parent_id`'s children and return its node ID.
    ///
    /// `parent_id` is [`PlaylistTreeNodeId::root()`] or an existing folder.
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
        if parent_id != PlaylistTreeNodeId::root()
            && self.playlist_nodes.get(&parent_id.0) != Some(&true)
        {
            return Err(unknown_foreign_key(
                ForeignKeyKind::PlaylistNode,
                parent_id.0,
            ));
        }

        let id = self.next_playlist_node_id;
        let sort_order = self.next_sort_order.get(&parent_id.0).copied().unwrap_or(0);
        let node = PlaylistTreeNode::new(
            PlaylistTreeNodeId(id),
            parent_id,
            DeviceSQLString::new(name)?,
            is_folder,
            sort_order,
        );
        self.db.add_row(node)?;

        self.next_playlist_node_id += 1;
        self.next_sort_order.insert(parent_id.0, sort_order + 1);
        self.playlist_nodes.insert(id, is_folder);

        Ok(PlaylistTreeNodeId(id))
    }

    /// Append a track to the end of a playlist. The entry position is assigned automatically.
    /// A track may appear in a playlist more than once.
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
        self.require_playlist(playlist_id)?;
        self.require_track(track_id)?;

        let entry_index = self
            .next_entry_index
            .get(&playlist_id.0)
            .copied()
            .unwrap_or(1);
        let entry = PlaylistEntry {
            entry_index,
            track_id,
            playlist_id,
        };
        self.db.add_row(entry)?;

        self.next_entry_index.insert(playlist_id.0, entry_index + 1);

        Ok(())
    }

    /// Create a top-level tag category (e.g. "Genre", "My Tags") in `exportExt.pdb` and return its
    /// id. Leaf tags are grouped under a category via [`add_tags_to_track`](Self::add_tags_to_track).
    /// The ext PDB is created lazily on first use, and written by [`save`](Self::save).
    ///
    /// # Errors
    ///
    /// [`Error::TagNameTooLong`] if `name` does not fit in a tag row.
    pub fn create_tag_category(&mut self, name: &str) -> Result<TagCategoryId> {
        check_tag_name(name)?;
        let id = self.tags.allocate_id();
        let row = tag_row(
            ParentId(None),
            self.tags.next_category_position,
            TagId(id),
            true,
            self.tags.allocate_row_index(),
            name,
        )?;
        self.ext_db_mut()?.add_row(row)?;

        self.tags.next_category_position += 1;
        self.tags.categories.insert(id);
        Ok(TagCategoryId(id))
    }

    /// Tag `track_id` with `tags` under `category` in `exportExt.pdb`.
    ///
    /// Empty labels are skipped. A label that already exists under the category reuses its tag
    /// instead of creating another, and a tag the track already carries is not linked again, so
    /// repeating a label (in this call, an earlier one, or before [`open`](Self::open)) changes
    /// nothing.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownForeignKey`] if `track_id` isn't an existing track or `category` isn't an
    /// existing category.
    /// [`Error::TagNameTooLong`] if a label does not fit in a tag row; the labels before it stay
    /// added.
    pub fn add_tags_to_track(
        &mut self,
        track_id: TrackId,
        category: TagCategoryId,
        tags: &[impl AsRef<str>],
    ) -> Result<()> {
        self.require_track(track_id)?;
        if !self.tags.categories.contains(&category.0) {
            return Err(unknown_foreign_key(ForeignKeyKind::TagCategory, category.0));
        }

        for label in tags.iter().map(AsRef::as_ref) {
            if label.is_empty() {
                continue;
            }
            let tag_id = self.get_or_create_tag(category, label)?;
            if self.tags.track_tags.contains(&(track_id.0, tag_id)) {
                continue;
            }
            self.ext_db_mut()?.add_row(TrackTag {
                track_id,
                tag_id: TagId(tag_id),
                // TODO(acrilique): we should add an exportExt.pdb fixture
                // with TrackTag rows generated by Rekordbox to data/ and
                // test this against it.
                unknown_const: 3,
            })?;
            self.tags.track_tags.insert((track_id.0, tag_id));
        }
        Ok(())
    }

    /// Reuse the leaf tag for `(category, label)` if it exists, else insert a new leaf row under
    /// `category` and remember it.
    fn get_or_create_tag(&mut self, category: TagCategoryId, label: &str) -> Result<u32> {
        let key = (category.0, label.to_string());
        if let Some(&id) = self.tags.leaves.get(&key) {
            return Ok(id);
        }
        check_tag_name(label)?;

        let id = self.tags.allocate_id();
        let position = self
            .tags
            .next_leaf_position
            .get(&category.0)
            .copied()
            .unwrap_or(0);
        let row = tag_row(
            ParentId(NonZero::new(category.0)),
            position,
            TagId(id),
            false,
            self.tags.allocate_row_index(),
            label,
        )?;
        self.ext_db_mut()?.add_row(row)?;

        self.tags
            .next_leaf_position
            .insert(category.0, position + 1);
        self.tags.leaves.insert(key, id);
        Ok(id)
    }

    /// Creates the ext PDB if the export has none, and marks it for the next [`save`](Self::save).
    fn ext_db_mut(&mut self) -> Result<&mut MemoryDatabase> {
        let ext_db = match self.ext_db.take() {
            Some(ext_db) => ext_db,
            // TODO(acrilique): the `exportExt.pdb` of both exports in `data/complete_export`
            // holds nine tables (types 0 to 8), not just these two.
            None => Database::create(
                Cursor::new(Vec::new()),
                DatabaseType::Ext,
                &[
                    PageType::Ext(ExtPageType::Tag),
                    PageType::Ext(ExtPageType::TrackTag),
                ],
            )?,
        };
        self.ext_dirty = true;
        Ok(self.ext_db.insert(ext_db))
    }

    fn require_track(&self, id: TrackId) -> Result<()> {
        if self.track_ids.contains(&id.0) {
            Ok(())
        } else {
            Err(unknown_foreign_key(ForeignKeyKind::Track, id.0))
        }
    }

    fn require_playlist(&self, id: PlaylistTreeNodeId) -> Result<()> {
        if self.playlist_nodes.get(&id.0) == Some(&false) {
            Ok(())
        } else {
            Err(unknown_foreign_key(ForeignKeyKind::PlaylistNode, id.0))
        }
    }

    /// Resolve `key` to the id of its row in a dimension table, inserting the row `build` makes
    /// on first use. An empty key resolves to id 0, the null foreign key, without a row.
    fn get_or_create<K: Eq + Hash, RowT: RowVariant>(
        db: &mut MemoryDatabase,
        ids_by_key: &mut HashMap<K, u32>,
        next_id: &mut u32,
        key: K,
        key_empty: bool,
        build: impl FnOnce(u32, &K) -> Result<RowT>,
    ) -> Result<u32> {
        if key_empty {
            return Ok(0);
        }
        if let Some(&id) = ids_by_key.get(&key) {
            return Ok(id);
        }

        let id = *next_id;
        db.add_row(build(id, &key)?)?;
        *next_id += 1;
        ids_by_key.insert(key, id);
        Ok(id)
    }

    fn get_or_create_artist(&mut self, name: &str) -> Result<u32> {
        Self::get_or_create(
            &mut self.db,
            &mut self.artists_by_name,
            &mut self.next_artist_id,
            name.to_string(),
            name.is_empty(),
            |id, name| {
                Ok(Artist {
                    subtype: Subtype(0x60),
                    index_shift: 0,
                    id: ArtistId(id),
                    offsets: OffsetArrayContainer {
                        offsets: MaybeCalculated::Calculated,
                        inner: TrailingName {
                            name: DeviceSQLString::new(name)?,
                        },
                    },
                })
            },
        )
    }

    fn get_or_create_album(&mut self, name: &str, artist_id: u32) -> Result<u32> {
        Self::get_or_create(
            &mut self.db,
            &mut self.albums_by_artist_and_name,
            &mut self.next_album_id,
            (artist_id, name.to_string()),
            name.is_empty(),
            |id, (artist_id, name)| {
                Ok(Album {
                    subtype: Subtype(0x80),
                    index_shift: 0,
                    unknown2: 0,
                    artist_id: ArtistId(*artist_id),
                    id: AlbumId(id),
                    unknown3: 0,
                    offsets: OffsetArrayContainer {
                        offsets: MaybeCalculated::Calculated,
                        inner: TrailingName {
                            name: DeviceSQLString::new(name)?,
                        },
                    },
                })
            },
        )
    }

    fn get_or_create_genre(&mut self, name: &str) -> Result<u32> {
        Self::get_or_create(
            &mut self.db,
            &mut self.genres_by_name,
            &mut self.next_genre_id,
            name.to_string(),
            name.is_empty(),
            |id, name| {
                Ok(Genre {
                    id: GenreId(id),
                    name: DeviceSQLString::new(name)?,
                })
            },
        )
    }

    fn get_or_create_key(&mut self, name: &str) -> Result<u32> {
        Self::get_or_create(
            &mut self.db,
            &mut self.keys_by_name,
            &mut self.next_key_id,
            name.to_string(),
            name.is_empty(),
            |id, name| {
                Ok(Key {
                    id: KeyId(id),
                    id2: id,
                    name: DeviceSQLString::new(name)?,
                })
            },
        )
    }

    fn get_or_create_label(&mut self, name: &str) -> Result<u32> {
        Self::get_or_create(
            &mut self.db,
            &mut self.labels_by_name,
            &mut self.next_label_id,
            name.to_string(),
            name.is_empty(),
            |id, name| {
                Ok(Label {
                    id: LabelId(id),
                    name: DeviceSQLString::new(name)?,
                })
            },
        )
    }

    fn get_or_create_artwork(&mut self, path: &str) -> Result<u32> {
        Self::get_or_create(
            &mut self.db,
            &mut self.artwork_by_path,
            &mut self.next_artwork_id,
            path.to_string(),
            path.is_empty(),
            |id, path| {
                Ok(Artwork {
                    id: ArtworkId(id),
                    path: DeviceSQLString::new(path)?,
                })
            },
        )
    }

    /// Import a host-side image as track artwork and return the device path to put in
    /// [`Track::artwork_path`].
    ///
    /// The image is decoded and resized right away into the 80×80 thumbnail and the 240×240
    /// medium Rekordbox uses; both are written as JPEG under `PIONEER/Artwork/` by the next
    /// [`save`](Self::save). Importing the same `source` again returns the same path, so tracks
    /// sharing a cover share one artwork.
    ///
    /// # Errors
    ///
    /// [`Error::ArtworkError`] if the image cannot be read, decoded, or encoded.
    #[cfg(feature = "artwork")]
    pub fn import_artwork(&mut self, source: impl AsRef<Path>) -> Result<String> {
        use image::imageops::FilterType;
        use image::{DynamicImage, ImageFormat, ImageReader};

        let source = source.as_ref();
        if let Some(device_path) = self.artwork_by_source.get(source) {
            return Ok(device_path.clone());
        }

        let artwork_error = |e: image::ImageError| Error::ArtworkError {
            path: source.to_path_buf(),
            message: e.to_string(),
        };
        let img = ImageReader::open(source)
            .map_err(|e| artwork_error(e.into()))?
            .decode()
            .map_err(artwork_error)?;
        let encode = |img: &DynamicImage, size: u32| -> Result<Vec<u8>> {
            let mut bytes = Vec::new();
            img.resize_to_fill(size, size, FilterType::Lanczos3)
                .to_rgb8()
                .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Jpeg)
                .map_err(artwork_error)?;
            Ok(bytes)
        };
        let thumbnail = encode(&img, 80)?;
        let medium = encode(&img, 240)?;

        // The file names derive from the artwork id. Skip any id whose path a caller-supplied
        // `Track::artwork_path` already claimed, so the import never shares (or overwrites) it.
        while self
            .artwork_by_path
            .contains_key(&artwork_device_path(self.next_artwork_id))
        {
            self.next_artwork_id += 1;
        }
        let id = self.next_artwork_id;
        let device_path = artwork_device_path(id);
        let row_id = self.get_or_create_artwork(&device_path)?;
        debug_assert_eq!(row_id, id, "a fresh artwork path allocates the next id");

        self.pending_files
            .push((self.layout.artwork_file(id), thumbnail));
        self.pending_files
            .push((self.layout.artwork_m_file(id), medium));
        self.artwork_by_source
            .insert(source.to_path_buf(), device_path.clone());
        Ok(device_path)
    }
}

fn unknown_foreign_key(kind: ForeignKeyKind, id: u32) -> Error {
    Error::UnknownForeignKey { kind, id }
}

fn raise(map: &mut HashMap<u32, u32>, key: u32, value: u32) {
    let current = map.entry(key).or_insert(0);
    *current = (*current).max(value);
}

/// Scan one dimension table for its next free id (one past the highest, and at least 1 because 0
/// is the null foreign key) and a `key -> id` index of its rows. `entry` returns a row's id and
/// its key, or no key to leave the row out of the index; the first row wins a duplicated key.
fn scan_table<RowT: RowVariant, K: Eq + Hash>(
    db: &mut MemoryDatabase,
    entry: impl Fn(&RowT) -> (u32, Option<K>),
) -> Result<(u32, HashMap<K, u32>)> {
    let mut next_id = 1;
    let mut ids_by_key = HashMap::new();
    db.iter_rows::<RowT>()?.for_each(|row| {
        let (id, key) = entry(row);
        next_id = next_id.max(id.saturating_add(1));
        if let Some(key) = key {
            ids_by_key.entry(key).or_insert(id);
        }
        Ok(())
    })?;
    Ok((next_id, ids_by_key))
}

fn setting_bytes(setting: &Setting) -> Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    setting
        .write_options(&mut buf, binrw::Endian::Little, (false,))
        .map_err(Error::BinrwError)?;
    Ok(buf.into_inner())
}

/// Write `bytes` to `path` without ever exposing a half-written file under that name: the bytes
/// go to a sibling temporary file, which is synced and then renamed over `path`.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut tmp_name = path
        .file_name()
        .expect("export files always have a file name")
        .to_os_string();
    tmp_name.push(".tmp");
    let tmp_path = path.with_file_name(tmp_name);

    let write = || -> std::io::Result<()> {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&tmp_path, path)
    };
    write().inspect_err(|_| {
        let _ = fs::remove_file(&tmp_path);
    })?;
    Ok(())
}

fn read_database(path: &Path, db_type: DatabaseType) -> Result<MemoryDatabase> {
    Database::open(Cursor::new(fs::read(path)?), db_type)
}

fn write_database(db: &mut MemoryDatabase, path: &Path) -> Result<()> {
    db.flush()?;
    write_atomic(path, db.get_ref().get_ref())
}

fn names_by_id<RowT: RowVariant>(
    db: &mut MemoryDatabase,
    entry: impl Fn(&RowT) -> (u32, &DeviceSQLString),
) -> Result<HashMap<u32, String>> {
    db.iter_rows::<RowT>()?
        .map(|row| {
            let (id, name) = entry(row);
            Ok((id, name.clone().into_string()?))
        })
        .collect()
}

/// Build a category or leaf tag row. `is_category` selects the `raw_is_category` flag
/// (`0x01000000` for categories, `0` for leaves — confirmed against a real Rekordbox export).
/// `row_index` is the per-row monotonic counter that drives `index_shift` (`0x20` per row, as
/// observed on real exports; the field is 16 bits wide, so it wraps after 2048 rows).
///
/// Leaf tag ids are small sequential numbers here, not the large random 32-bit values Rekordbox
/// writes. Whether any player cares is unknown.
fn tag_row(
    parent_id: ParentId,
    position: u32,
    id: TagId,
    is_category: bool,
    row_index: u32,
    name: &str,
) -> Result<TagOrCategory> {
    Ok(TagOrCategory {
        subtype: Subtype(0x0680),
        index_shift: (row_index.wrapping_mul(0x20)) as u16,
        unknown1: 0,
        unknown2: 0,
        parent_id,
        position,
        id,
        raw_is_category: u32::from(is_category) << 24,
        offsets: OffsetArrayContainer {
            offsets: MaybeCalculated::Calculated,
            inner: TagOrCategoryStrings {
                name: name.parse()?,
                unknown: DeviceSQLString::empty(),
            },
        },
    })
}

/// Reject a name whose row could not be written, before anything is allocated for it: a row
/// only fails on [`save`](DeviceExport::save), and one that does can never be removed again.
fn check_tag_name(name: &str) -> Result<()> {
    tag_row(ParentId(None), 0, TagId(0), false, 0, name)?
        .write_le(&mut Cursor::new(Vec::new()))
        .map_err(|_| Error::TagNameTooLong(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::playlist::{Playlist, PlaylistFolder};
    use crate::util::{ColorIndex, FileType};

    /// A directory for a test's export: unique per test and per run, absent until the export is
    /// saved, and removed again when the test ends (pass or fail).
    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "rekordcrate-export-{name}-test-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&dir);
            Self(dir)
        }
    }

    impl std::ops::Deref for TestDir {
        type Target = Path;

        fn deref(&self) -> &Path {
            &self.0
        }
    }

    impl AsRef<Path> for TestDir {
        fn as_ref(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// A minimal track: a title, and the file name and path derived from it.
    fn track(name: &str) -> Track {
        Track {
            title: name.into(),
            filename: format!("{name}.mp3"),
            file_path: format!("/Contents/{name}.mp3"),
            ..Default::default()
        }
    }

    fn rows<RowT: RowVariant + Clone>(db: &mut MemoryDatabase) -> Vec<RowT> {
        db.iter_rows::<RowT>()
            .unwrap()
            .map(|row| Ok(row.clone()))
            .collect()
            .unwrap()
    }

    /// The tag and track-tag rows of the `exportExt.pdb` saved under `dir`.
    fn saved_ext_rows(dir: &Path) -> (Vec<TagOrCategory>, Vec<TrackTag>) {
        let mut ext_db =
            read_database(&Layout::new(dir.into()).export_ext_pdb(), DatabaseType::Ext).unwrap();
        (rows(&mut ext_db), rows(&mut ext_db))
    }

    fn tag_name(tag: &TagOrCategory) -> String {
        tag.offsets.inner.name.clone().into_string().unwrap()
    }

    fn assert_unknown_foreign_key<T: std::fmt::Debug>(
        result: Result<T>,
        expected_kind: ForeignKeyKind,
        expected_id: u32,
    ) {
        match result {
            Err(Error::UnknownForeignKey { kind, id })
                if kind == expected_kind && id == expected_id => {}
            other => panic!(
                "expected an unknown {expected_kind:?} foreign key {expected_id}, got {other:?}"
            ),
        }
    }

    #[test]
    fn nothing_is_written_before_save() {
        let dir = TestDir::new("unsaved");

        let mut dev = DeviceExport::create(&dir).unwrap();
        dev.add_track(&track("song")).unwrap();
        let cat = dev.create_tag_category("Mood").unwrap();
        assert_eq!(cat, TagCategoryId(1));
        drop(dev);

        assert!(!dir.exists(), "an unsaved export must not touch the disk");
    }

    #[test]
    fn save_writes_the_export_and_can_repeat() {
        let dir = TestDir::new("save");

        let mut dev = DeviceExport::create(&dir).unwrap();
        dev.add_track(&track("song0")).unwrap();
        dev.save().unwrap();

        for path in [
            "PIONEER/rekordbox/export.pdb",
            "PIONEER/USBANLZ",
            "Contents",
            "PIONEER/DEVSETTING.DAT",
            "PIONEER/DJMMYSETTING.DAT",
            "PIONEER/MYSETTING.DAT",
            "PIONEER/MYSETTING2.DAT",
        ] {
            assert!(dir.join(path).exists(), "{path} must exist after save()");
        }
        assert_eq!(
            dev.load_settings(),
            Settings::load(&Layout::new(dir.to_path_buf()))
        );
        assert!(dev.load_settings().quantize.is_some());

        dev.add_track(&track("song1")).unwrap();
        dev.save().unwrap();

        let leftovers: Vec<_> = ["PIONEER", "PIONEER/rekordbox"]
            .iter()
            .flat_map(|sub| fs::read_dir(dir.join(sub)).unwrap())
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .filter(|name| name.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temporary files left: {leftovers:?}");

        let mut dev = DeviceExport::open(&dir).unwrap();
        assert_eq!(dev.tracks().unwrap().len(), 2);
    }

    // A real Rekordbox export opens, reads back, and can be added to without clashing with what
    // it already holds. Nothing is saved, so the fixture stays untouched.
    #[test]
    fn open_reads_a_rekordbox_export() {
        let mut dev = DeviceExport::open("data/complete_export/demo_tracks").unwrap();

        let tracks = dev.tracks().unwrap();
        let titles: Vec<&str> = tracks.iter().map(|(_, t)| t.title.as_str()).collect();
        assert_eq!(titles, ["Demo Track 1", "Demo Track 2"]);
        assert!(tracks.iter().all(|(_, t)| t.artist == "Loopmasters"));

        // An existing path is recognised, a new one gets an id past the existing ones.
        let existing = dev.add_track(&tracks[0].1).unwrap();
        assert_eq!((existing.id, existing.is_new), (tracks[0].0, false));
        let added = dev.add_track(&track("new")).unwrap();
        assert!(added.is_new);
        assert!(tracks.iter().all(|(id, _)| *id != added.id));

        // Rekordbox gives leaf tags random 32-bit ids; a new category must still get a free one.
        let existing_tags: Vec<TagOrCategory> = rows(dev.ext_db.as_mut().unwrap());
        assert!(!existing_tags.is_empty());
        let cat = dev.create_tag_category("Mine").unwrap();
        assert!(cat.0 != 0 && existing_tags.iter().all(|tag| tag.id.0 != cat.0));
        dev.add_tags_to_track(added.id, cat, &["Tag"]).unwrap();
    }

    #[test]
    fn tracks_read_back_what_was_added() {
        let dir = TestDir::new("readback");

        let full = Track {
            title: "Title".into(),
            artist: "Artist".into(),
            album: "Album".into(),
            genre: "Genre".into(),
            key: "Amin".into(),
            label: "Label".into(),
            composer: "Composer".into(),
            remixer: "Remixer".into(),
            orig_artist: "Original".into(),
            comment: "A comment".into(),
            isrc: "USRC17607839".into(),
            lyricist: "Lyricist".into(),
            mix_name: "Extended Mix".into(),
            release_date: "2024-01-02".into(),
            date_added: "2024-03-04".into(),
            file_path: "/Contents/Artist/Title.mp3".into(),
            filename: "Title.mp3".into(),
            artwork_path: "/PIONEER/Artwork/00001/a1.jpg".into(),
            message: "Message".into(),
            tempo: 126.5,
            bitrate: 320,
            sample_rate: 44_100,
            sample_depth: 16,
            duration_secs: 301,
            file_size: 12_345_678,
            track_number: 3,
            disc_number: 1,
            year: 2024,
            play_count: 7,
            rating: Rating::Four,
            color: ColorIndex::Green,
            file_type: FileType::Mp3,
            autoload_hotcues: true,
        };
        // Short enough to need the minimum-row-size padding, which must not leak into `comment`.
        let tiny = Track {
            title: "tiny".into(),
            ..Default::default()
        };

        let mut dev = DeviceExport::create(&dir).unwrap();
        let full_id = dev.add_track(&full).unwrap().id;
        let tiny_id = dev.add_track(&tiny).unwrap().id;
        let expected = vec![(full_id, full), (tiny_id, tiny)];
        assert_eq!(dev.tracks().unwrap(), expected);

        dev.save().unwrap();
        let mut dev = DeviceExport::open(&dir).unwrap();
        assert_eq!(dev.tracks().unwrap(), expected);
    }

    #[test]
    fn playlists_read_back() {
        let dir = TestDir::new("playlist-readback");

        let mut dev = DeviceExport::create(&dir).unwrap();
        let ids: Vec<TrackId> = (0..3)
            .map(|n| dev.add_track(&track(&format!("song{n}"))).unwrap().id)
            .collect();
        let folder = dev
            .create_playlist_folder("Folder", PlaylistTreeNodeId::root())
            .unwrap();
        let playlist = dev.create_playlist("Playlist", folder).unwrap();
        for &id in &[ids[2], ids[0], ids[1]] {
            dev.add_track_to_playlist(playlist, id).unwrap();
        }
        dev.save().unwrap();

        let mut dev = DeviceExport::open(&dir).unwrap();
        assert_eq!(
            dev.playlists().unwrap(),
            vec![PlaylistNode::Folder(PlaylistFolder {
                id: folder,
                name: "Folder".into(),
                children: vec![PlaylistNode::Playlist(Playlist {
                    id: playlist,
                    name: "Playlist".into(),
                })],
            })]
        );
        assert_eq!(
            dev.playlist_tracks(playlist).unwrap(),
            vec![ids[2], ids[0], ids[1]]
        );
        assert_unknown_foreign_key(
            dev.playlist_tracks(folder),
            ForeignKeyKind::PlaylistNode,
            folder.0,
        );
    }

    // Siblings are numbered in creation order, per parent, and the numbering carries on after
    // open(); playlists() returns each level in that order.
    #[test]
    fn playlist_nodes_keep_creation_order() {
        let dir = TestDir::new("playlist-order");
        let root = PlaylistTreeNodeId::root();

        let mut dev = DeviceExport::create(&dir).unwrap();
        let folder = dev.create_playlist_folder("Folder", root).unwrap();
        let a = dev.create_playlist("A", root).unwrap();
        let inner = dev.create_playlist("Inner", folder).unwrap();
        dev.save().unwrap();

        let mut dev = DeviceExport::open(&dir).unwrap();
        let b = dev.create_playlist("B", root).unwrap();
        dev.save().unwrap();

        let mut dev = DeviceExport::open(&dir).unwrap();
        let mut sort_orders: Vec<(PlaylistTreeNodeId, u32)> = rows::<PlaylistTreeNode>(&mut dev.db)
            .iter()
            .map(|node| (node.id, node.sort_order))
            .collect();
        sort_orders.sort_by_key(|&(id, _)| id.0);
        assert_eq!(sort_orders, [(folder, 0), (a, 1), (inner, 0), (b, 2)]);

        let top_level: Vec<PlaylistTreeNodeId> = dev
            .playlists()
            .unwrap()
            .iter()
            .map(|node| match node {
                PlaylistNode::Folder(folder) => folder.id,
                PlaylistNode::Playlist(playlist) => playlist.id,
            })
            .collect();
        assert_eq!(top_level, [folder, a, b]);
    }

    #[test]
    fn open_then_add_does_not_duplicate_named_row() {
        let dir = TestDir::new("named-row-dedup");

        let t = Track {
            artist: "Dup Artist".into(),
            album: "Dup Album".into(),
            genre: "Dup Genre".into(),
            ..track("song")
        };

        let mut dev = DeviceExport::create(&dir).unwrap();
        dev.add_track(&t).unwrap();
        dev.save().unwrap();

        // A different track by the same artist, so add_track gets as far as resolving names.
        let mut dev = DeviceExport::open(&dir).unwrap();
        dev.add_track(&Track {
            file_path: "/Contents/other.mp3".into(),
            ..t.clone()
        })
        .unwrap();
        dev.save().unwrap();

        // Dedup must survive to disk: exactly one row of each.
        let mut dev = DeviceExport::open(&dir).unwrap();
        assert_eq!(rows::<PdbTrack>(&mut dev.db).len(), 2);
        assert_eq!(rows::<Artist>(&mut dev.db).len(), 1);
        assert_eq!(rows::<Album>(&mut dev.db).len(), 1);
        assert_eq!(rows::<Genre>(&mut dev.db).len(), 1);
    }

    #[test]
    fn add_track_dedups_on_file_path() {
        let dir = TestDir::new("track-dedup");
        let t = track("song");

        let mut dev = DeviceExport::create(&dir).unwrap();
        let first = dev.add_track(&t).unwrap();
        let second = dev.add_track(&t).unwrap();
        assert_eq!(
            first.id, second.id,
            "re-adding the same file_path must return the existing id"
        );
        assert!(first.is_new, "the first add must be flagged new");
        assert!(!second.is_new, "the duplicate add must be flagged not new");
        dev.save().unwrap();

        let mut dev = DeviceExport::open(&dir).unwrap();
        let third = dev.add_track(&t).unwrap();
        assert_eq!(
            first.id, third.id,
            "re-adding after open() must return the existing id"
        );
        assert!(!third.is_new, "the post-reopen add must be flagged not new");
        dev.save().unwrap();

        let mut dev = DeviceExport::open(&dir).unwrap();
        assert_eq!(
            rows::<PdbTrack>(&mut dev.db).len(),
            1,
            "dedup must keep exactly one Track row on disk"
        );
    }

    #[test]
    fn add_track_without_file_path_always_inserts() {
        let mut dev = DeviceExport::create(TestDir::new("no-path")).unwrap();
        let t = Track {
            title: "untitled".into(),
            ..Default::default()
        };

        let first = dev.add_track(&t).unwrap();
        let second = dev.add_track(&t).unwrap();
        assert!(first.is_new && second.is_new);
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn playlist_methods_reject_unknown_foreign_keys() {
        let mut dev = DeviceExport::create(TestDir::new("fk")).unwrap();

        // parent_id root() is the tree root and always valid.
        let folder = dev
            .create_playlist_folder("Folder", PlaylistTreeNodeId::root())
            .unwrap();
        let playlist = dev.create_playlist("Playlist", folder).unwrap();
        let track = dev.add_track(&track("song")).unwrap().id;

        assert_unknown_foreign_key(
            dev.create_playlist("Orphan", PlaylistTreeNodeId(999)),
            ForeignKeyKind::PlaylistNode,
            999,
        );
        assert_unknown_foreign_key(
            dev.add_track_to_playlist(PlaylistTreeNodeId(999), track),
            ForeignKeyKind::PlaylistNode,
            999,
        );
        assert_unknown_foreign_key(
            dev.add_track_to_playlist(playlist, TrackId(999)),
            ForeignKeyKind::Track,
            999,
        );

        dev.add_track_to_playlist(playlist, track).unwrap();
    }

    // A playlist id is rejected as a parent (only folders hold children), and a folder id is
    // rejected as an add_track_to_playlist target (only playlists hold tracks).
    #[test]
    fn playlist_tree_enforces_folder_vs_playlist_roles() {
        let mut dev = DeviceExport::create(TestDir::new("tree-shape")).unwrap();
        let folder = dev
            .create_playlist_folder("Folder", PlaylistTreeNodeId::root())
            .unwrap();
        let playlist = dev.create_playlist("Playlist", folder).unwrap();
        let track = dev.add_track(&track("song")).unwrap().id;

        assert_unknown_foreign_key(
            dev.create_playlist("Child", playlist),
            ForeignKeyKind::PlaylistNode,
            playlist.0,
        );
        assert_unknown_foreign_key(
            dev.add_track_to_playlist(folder, track),
            ForeignKeyKind::PlaylistNode,
            folder.0,
        );
    }

    #[test]
    fn add_track_to_playlist_auto_indexes() {
        let dir = TestDir::new("autoindex");

        let mut dev = DeviceExport::create(&dir).unwrap();
        let playlist = dev
            .create_playlist("P", PlaylistTreeNodeId::root())
            .unwrap();
        for n in 0..3 {
            let id = dev.add_track(&track(&format!("song{n}"))).unwrap().id;
            dev.add_track_to_playlist(playlist, id).unwrap();
        }
        dev.save().unwrap();

        let mut dev = DeviceExport::open(&dir).unwrap();
        let id = dev.add_track(&track("song3")).unwrap().id;
        dev.add_track_to_playlist(playlist, id).unwrap();
        dev.save().unwrap();

        let mut dev = DeviceExport::open(&dir).unwrap();
        let mut indices: Vec<u32> = rows::<PlaylistEntry>(&mut dev.db)
            .iter()
            .map(|entry| entry.entry_index)
            .collect();
        indices.sort_unstable();
        assert_eq!(indices, [1, 2, 3, 4], "entry indices must be dense from 1");
    }

    #[test]
    fn add_track_auto_pads_under_minimum_row_size() {
        let dir = TestDir::new("autopad");

        let mut dev = DeviceExport::create(&dir).unwrap();
        dev.add_track(&Track {
            title: "tiny".into(),
            ..Default::default()
        })
        .unwrap();
        dev.save().unwrap();

        let mut dev = DeviceExport::open(&dir).unwrap();
        let row = rows::<PdbTrack>(&mut dev.db).remove(0);
        let comment = row.offsets.inner.comment.into_string().unwrap();
        assert!(
            !comment.is_empty() && comment.chars().all(|c| c == ' '),
            "auto-pad must grow the comment with spaces only, got {comment:?}"
        );
    }

    // The folder rule id/20+1, zero-padded to 5 digits, derived from the captured fixtures.
    #[test]
    fn artwork_paths_follow_the_folder_rule() {
        use crate::device::layout::artwork_folder;

        assert_eq!(artwork_folder(1), "00001");
        assert_eq!(artwork_folder(19), "00001");
        assert_eq!(artwork_folder(20), "00002");
        assert_eq!(artwork_folder(39), "00002");
        assert_eq!(artwork_folder(40), "00003");

        let layout = Layout::new("/usb".into());
        assert_eq!(
            layout.artwork_file(20),
            Path::new("/usb/PIONEER/Artwork/00002/a20.jpg")
        );
        assert_eq!(
            layout.artwork_m_file(20),
            Path::new("/usb/PIONEER/Artwork/00002/a20_m.jpg")
        );
        // The PDB path always references the thumbnail.
        #[cfg(feature = "artwork")]
        assert_eq!(artwork_device_path(20), "/PIONEER/Artwork/00002/a20.jpg");
    }

    /// Write a `w`×`h` test image to `path`, in the format its extension names.
    #[cfg(feature = "artwork")]
    fn write_test_image(path: &Path, w: u32, h: u32) {
        use image::{ImageBuffer, Rgb, RgbImage};
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let img: RgbImage = ImageBuffer::from_fn(w, h, |x, y| {
            Rgb([(x * 10 % 255) as u8, (y * 10 % 255) as u8, 128])
        });
        img.save(path).unwrap();
    }

    // import_artwork returns the thumbnail path the PDB stores, and save() writes the source
    // under it as an 80×80 thumbnail plus a 240×240 medium, whatever the source's size.
    #[cfg(feature = "artwork")]
    #[test]
    fn import_artwork_writes_thumbnail_and_medium() {
        use image::ImageReader;

        let dir = TestDir::new("artwork-import");
        let src = dir.join("src.png");
        write_test_image(&src, 200, 120);

        let mut dev = DeviceExport::create(&dir).unwrap();
        let artwork_path = dev.import_artwork(&src).unwrap();
        assert_eq!(artwork_path, "/PIONEER/Artwork/00001/a1.jpg");
        dev.add_track(&Track {
            artwork_path,
            ..track("song")
        })
        .unwrap();

        let thumb = dir.join("PIONEER/Artwork/00001/a1.jpg");
        let medium = dir.join("PIONEER/Artwork/00001/a1_m.jpg");
        assert!(!thumb.exists(), "nothing is written before save()");
        dev.save().unwrap();

        let dimensions = |path: &Path| ImageReader::open(path).unwrap().into_dimensions().unwrap();
        assert_eq!(dimensions(&thumb), (80, 80));
        assert_eq!(dimensions(&medium), (240, 240));

        let mut dev = DeviceExport::open(&dir).unwrap();
        let artwork = rows::<Artwork>(&mut dev.db);
        assert_eq!(artwork.len(), 1);
        assert_eq!(
            artwork[0].path.clone().into_string().unwrap(),
            "/PIONEER/Artwork/00001/a1.jpg",
            "PDB Artwork.path must reference the thumbnail"
        );
    }

    // Two tracks sharing one source dedup to a single artwork: one thumbnail, one medium on disk,
    // one PDB row.
    #[cfg(feature = "artwork")]
    #[test]
    fn import_artwork_dedups_by_source() {
        let dir = TestDir::new("artwork-dedup");
        let src = dir.join("cover.png");
        write_test_image(&src, 100, 100);

        let mut dev = DeviceExport::create(&dir).unwrap();
        let artwork_path = dev.import_artwork(&src).unwrap();
        for name in ["song0", "song1"] {
            dev.add_track(&Track {
                artwork_path: artwork_path.clone(),
                ..track(name)
            })
            .unwrap();
        }
        assert_eq!(dev.import_artwork(&src).unwrap(), artwork_path);
        dev.save().unwrap();

        let folder = dir.join("PIONEER/Artwork/00001");
        assert_eq!(
            fs::read_dir(&folder).unwrap().count(),
            2,
            "expected exactly thumbnail + medium in {folder:?}"
        );

        let mut dev = DeviceExport::open(&dir).unwrap();
        assert_eq!(
            rows::<Artwork>(&mut dev.db).len(),
            1,
            "dedup must keep exactly one Artwork row on disk"
        );
    }

    // An import never lands on a path a caller already put in `Track::artwork_path`.
    #[cfg(feature = "artwork")]
    #[test]
    fn import_artwork_skips_paths_claimed_by_callers() {
        let dir = TestDir::new("artwork-claimed");
        let src = dir.join("cover.png");
        write_test_image(&src, 100, 100);

        let mut dev = DeviceExport::create(&dir).unwrap();
        // Takes artwork id 1, but names the file the next import would otherwise derive (id 2).
        let claimed = "/PIONEER/Artwork/00001/a2.jpg";
        dev.add_track(&Track {
            artwork_path: claimed.into(),
            ..track("song")
        })
        .unwrap();

        assert_eq!(
            dev.import_artwork(&src).unwrap(),
            "/PIONEER/Artwork/00001/a3.jpg"
        );
        assert_eq!(rows::<Artwork>(&mut dev.db).len(), 2);
    }

    #[test]
    fn artwork_path_is_stored_verbatim_without_copy() {
        let dir = TestDir::new("artwork-verbatim");
        let caller_path = "/PIONEER/Artwork/00007/a137.jpg";

        let mut dev = DeviceExport::create(&dir).unwrap();
        dev.add_track(&Track {
            artwork_path: caller_path.into(),
            ..track("song")
        })
        .unwrap();
        dev.save().unwrap();

        // No image files written — they are the caller's responsibility.
        assert!(
            !dir.join("PIONEER/Artwork").exists(),
            "no artwork dir/files must be created for a caller-supplied path"
        );

        let mut dev = DeviceExport::open(&dir).unwrap();
        let artwork = rows::<Artwork>(&mut dev.db);
        assert_eq!(artwork.len(), 1, "one Artwork row must be written");
        assert_eq!(
            artwork[0].path.clone().into_string().unwrap(),
            caller_path,
            "Artwork.path must be the caller-supplied string, stored verbatim"
        );
        assert_eq!(
            rows::<PdbTrack>(&mut dev.db)[0].artwork_id,
            artwork[0].id,
            "the track must reference the artwork row"
        );
    }

    #[test]
    fn tags_not_written_when_unused() {
        let dir = TestDir::new("tags-none");

        let mut dev = DeviceExport::create(&dir).unwrap();
        dev.add_track(&track("song")).unwrap();
        dev.save().unwrap();

        assert!(
            !dir.join("PIONEER/rekordbox/exportExt.pdb").exists(),
            "no exportExt.pdb must be written when no track has tags"
        );
    }

    #[test]
    fn add_tags_creates_category_leaves_and_junctions() {
        let dir = TestDir::new("tags-basic");

        let mut dev = DeviceExport::create(&dir).unwrap();
        let id1 = dev.add_track(&track("a")).unwrap().id;
        let id2 = dev.add_track(&track("b")).unwrap().id;

        let cat = dev.create_tag_category("My Tags").unwrap();
        dev.add_tags_to_track(id1, cat, &["Techno", "Dub", "Techno"])
            .unwrap();
        dev.add_tags_to_track(id2, cat, &["Dub", "House"]).unwrap();
        dev.save().unwrap();

        let (tags, track_tags) = saved_ext_rows(&dir);

        let categories: Vec<_> = tags.iter().filter(|t| t.raw_is_category != 0).collect();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].raw_is_category, 0x01000000);
        assert_eq!(categories[0].index_shift, 0x0000);
        assert_eq!(tag_name(categories[0]), "My Tags");
        assert_eq!(categories[0].id.0, cat.0);
        assert_eq!(categories[0].parent_id.0, None);

        // The category took row index 0, so the leaves' `index_shift` starts at 0x20.
        let mut leaves: Vec<_> = tags.iter().filter(|t| t.raw_is_category == 0).collect();
        leaves.sort_unstable_by_key(|t| t.index_shift);
        let leaf_fields: Vec<_> = leaves
            .iter()
            .map(|t| (tag_name(t), t.position, t.index_shift))
            .collect();
        assert_eq!(
            leaf_fields,
            [
                ("Techno".to_string(), 0, 0x20),
                ("Dub".to_string(), 1, 0x40),
                ("House".to_string(), 2, 0x60),
            ]
        );
        for leaf in &leaves {
            assert_eq!(leaf.parent_id.0.map(NonZero::get), Some(cat.0));
        }

        // t1: 2 tags, t2: 2 tags; the duplicate "Techno" within t1 must not add a junction, and the
        // shared "Dub" must reuse one leaf row.
        assert_eq!(track_tags.len(), 4);
        for tt in &track_tags {
            assert_eq!(tt.unknown_const, 3);
        }
    }

    // Tagging a track with a tag it already carries adds no second junction row, whether the
    // first one came from the same call, an earlier call, or an export opened from disk.
    #[test]
    fn add_tags_links_a_tag_to_a_track_once() {
        let dir = TestDir::new("tags-once");

        let mut dev = DeviceExport::create(&dir).unwrap();
        let id = dev.add_track(&track("a")).unwrap().id;
        let cat = dev.create_tag_category("My Tags").unwrap();
        dev.add_tags_to_track(id, cat, &["Dub", "Dub"]).unwrap();
        dev.add_tags_to_track(id, cat, &["Dub"]).unwrap();
        dev.save().unwrap();

        let mut dev = DeviceExport::open(&dir).unwrap();
        dev.add_tags_to_track(id, cat, &["Dub"]).unwrap();
        assert!(
            !dev.ext_dirty,
            "a no-op tag call must not dirty the ext PDB"
        );
        dev.save().unwrap();

        let (tags, track_tags) = saved_ext_rows(&dir);
        assert_eq!(tags.len(), 2, "one category and one leaf");
        assert_eq!(track_tags.len(), 1, "one junction row");
    }

    #[test]
    fn add_tags_rejects_unknown_foreign_keys() {
        let mut dev = DeviceExport::create(TestDir::new("tags-fk")).unwrap();
        let id = dev.add_track(&track("a")).unwrap().id;
        let cat = dev.create_tag_category("My Tags").unwrap();

        assert_unknown_foreign_key(
            dev.add_tags_to_track(TrackId(999), cat, &["x"]),
            ForeignKeyKind::Track,
            999,
        );
        assert_unknown_foreign_key(
            dev.add_tags_to_track(id, TagCategoryId(999), &["x"]),
            ForeignKeyKind::TagCategory,
            999,
        );
        // The track is checked before the category, so a bad track id is reported first.
        assert_unknown_foreign_key(
            dev.add_tags_to_track(TrackId(888), TagCategoryId(999), &["x"]),
            ForeignKeyKind::Track,
            888,
        );
    }

    #[test]
    fn overlong_tag_names_are_rejected_and_leave_the_export_saveable() {
        let dir = TestDir::new("tags-long");
        let long = "x".repeat(300);

        let mut dev = DeviceExport::create(&dir).unwrap();
        let id = dev.add_track(&track("song")).unwrap().id;
        let cat = dev.create_tag_category("Mood").unwrap();
        assert!(matches!(
            dev.add_tags_to_track(id, cat, &[&long]),
            Err(Error::TagNameTooLong(_))
        ));
        assert!(matches!(
            dev.create_tag_category(&long),
            Err(Error::TagNameTooLong(_))
        ));
        dev.add_tags_to_track(id, cat, &["Happy"]).unwrap();
        dev.save().unwrap();

        let (tags, _) = saved_ext_rows(&dir);
        let ids: Vec<u32> = tags.iter().map(|tag| tag.id.0).collect();
        assert_eq!(ids, [1, 2]);
    }

    #[test]
    fn add_tags_ignores_empty_labels() {
        let dir = TestDir::new("tags-empty");

        let mut dev = DeviceExport::create(&dir).unwrap();
        let id = dev.add_track(&track("a")).unwrap().id;
        let cat = dev.create_tag_category("My Tags").unwrap();
        dev.add_tags_to_track(id, cat, &["", ""]).unwrap();
        dev.save().unwrap();

        let (tags, track_tags) = saved_ext_rows(&dir);
        assert_eq!(tags.len(), 1);
        assert_eq!(track_tags.len(), 0);
    }

    // open() must not truncate exportExt.pdb: prior categories/leaves/junctions survive, and later
    // tag calls append with fresh ids and row indices (no collision).
    #[test]
    fn open_preserves_existing_tags() {
        let dir = TestDir::new("tags-open-preserve");

        let mut dev = DeviceExport::create(&dir).unwrap();
        let id = dev.add_track(&track("a")).unwrap().id;
        let cat = dev.create_tag_category("My Tags").unwrap();
        dev.add_tags_to_track(id, cat, &["Techno", "Dub"]).unwrap();
        dev.save().unwrap();

        // Reopen and append a new leaf under the existing category plus a brand-new category.
        let mut dev = DeviceExport::open(&dir).unwrap();
        dev.add_tags_to_track(id, cat, &["House", "Dub"]).unwrap();
        let cat2 = dev.create_tag_category("Mood").unwrap();
        dev.add_tags_to_track(id, cat2, &["Dark"]).unwrap();
        dev.save().unwrap();

        let (tags, track_tags) = saved_ext_rows(&dir);

        let mut categories: Vec<_> = tags
            .iter()
            .filter(|t| t.raw_is_category != 0)
            .map(|t| (t.position, tag_name(t)))
            .collect();
        categories.sort();
        assert_eq!(
            categories,
            [(0, "My Tags".to_string()), (1, "Mood".to_string())]
        );

        // The two original leaves plus the two new ones (House, Dark); the pre-existing "Dub"
        // must not have been duplicated.
        let mut leaves: Vec<String> = tags
            .iter()
            .filter(|t| t.raw_is_category == 0)
            .map(tag_name)
            .collect();
        leaves.sort();
        assert_eq!(leaves, ["Dark", "Dub", "House", "Techno"]);

        assert_eq!(track_tags.len(), 4);

        // No two Tag rows share an id or an index_shift (would mean a counter wasn't recovered).
        let ids: HashSet<u32> = tags.iter().map(|t| t.id.0).collect();
        assert_eq!(ids.len(), tags.len(), "tag ids collided after open()");
        let shifts: HashSet<u16> = tags.iter().map(|t| t.index_shift).collect();
        assert_eq!(shifts.len(), tags.len(), "index_shift values collided");
    }

    #[test]
    fn tag_row_encodings() {
        let cat = tag_row(ParentId(None), 0, TagId(1), true, 0, "C").unwrap();
        assert_eq!(cat.raw_is_category, 0x01000000);
        assert_eq!(cat.index_shift, 0x0000);

        let leaf = tag_row(ParentId(NonZero::new(1)), 0, TagId(2), false, 3, "L").unwrap();
        assert_eq!(leaf.raw_is_category, 0);
        assert_eq!(leaf.index_shift, 0x0060);
    }

    const REKORDBOX_PDBS: [&str; 3] = [
        "data/pdb/num_rows/export.pdb",
        "data/complete_export/demo_tracks/PIONEER/rekordbox/export.pdb",
        "data/complete_export/empty/PIONEER/rekordbox/export.pdb",
    ];

    const REKORDBOX_EXT_PDBS: [&str; 2] = [
        "data/complete_export/demo_tracks/PIONEER/rekordbox/exportExt.pdb",
        "data/complete_export/empty/PIONEER/rekordbox/exportExt.pdb",
    ];

    fn rekordbox_rows<RowT: RowVariant + Clone>(path: &str, db_type: DatabaseType) -> Vec<RowT> {
        rows(&mut read_database(Path::new(path), db_type).unwrap())
    }

    // utility to test several reverse-engineering claims against our Rekordbox
    // generated fixtures
    fn all_rekordbox_rows<RowT: RowVariant + Clone>() -> Vec<RowT> {
        let all: Vec<RowT> = REKORDBOX_PDBS
            .iter()
            .flat_map(|path| rekordbox_rows(path, DatabaseType::Plain))
            .collect();
        assert!(
            !all.is_empty(),
            "the fixtures hold no row to compare against"
        );
        all
    }

    /// An unsaved export holding one track that names every row a track can share, in a playlist
    /// inside a folder, labelled with two tags.
    fn written_export() -> DeviceExport {
        let mut dev = DeviceExport::create("unsaved-export").unwrap();
        let added = dev
            .add_track(&Track {
                artist: "Artist".into(),
                album: "Album".into(),
                key: "Amin".into(),
                ..track("song")
            })
            .unwrap();
        let other = dev.add_track(&track("other")).unwrap();

        let folder = dev
            .create_playlist_folder("Folder", PlaylistTreeNodeId::root())
            .unwrap();
        dev.create_playlist("Root", PlaylistTreeNodeId::root())
            .unwrap();
        let playlist = dev.create_playlist("Nested", folder).unwrap();
        dev.add_track_to_playlist(playlist, added.id).unwrap();
        dev.add_track_to_playlist(playlist, other.id).unwrap();

        let category = dev.create_tag_category("Mood").unwrap();
        dev.add_tags_to_track(added.id, category, &["Dark", "Deep"])
            .unwrap();
        dev
    }

    /// Asserts that each group of values, once sorted, counts up from `first` without a gap.
    fn assert_counts_up_from(first: u32, groups: HashMap<u32, Vec<u32>>, what: &str) {
        assert!(!groups.is_empty(), "no {what} to check");
        for (group, mut values) in groups {
            values.sort_unstable();
            let expected: Vec<u32> = (first..).take(values.len()).collect();
            assert_eq!(values, expected, "{what} of {group}");
        }
    }

    fn sort_orders_by_parent(nodes: &[PlaylistTreeNode]) -> HashMap<u32, Vec<u32>> {
        let mut groups: HashMap<u32, Vec<u32>> = HashMap::new();
        for node in nodes {
            groups
                .entry(node.parent_id.0)
                .or_default()
                .push(node.sort_order);
        }
        groups
    }

    fn entry_indices_by_playlist(entries: &[PlaylistEntry]) -> HashMap<u32, Vec<u32>> {
        let mut groups: HashMap<u32, Vec<u32>> = HashMap::new();
        for entry in entries {
            groups
                .entry(entry.playlist_id.0)
                .or_default()
                .push(entry.entry_index);
        }
        groups
    }

    #[test]
    fn table_order_matches_rekordbox() {
        let page_types = |db: &MemoryDatabase| -> Vec<PageType> {
            db.get_header()
                .tables
                .iter()
                .map(|table| table.page_type)
                .collect()
        };

        let ours = page_types(&written_export().db);
        for path in REKORDBOX_PDBS {
            let theirs = read_database(Path::new(path), DatabaseType::Plain).unwrap();
            assert_eq!(page_types(&theirs), ours, "{path}");
        }
    }

    #[test]
    fn track_row_constants_match_rekordbox() {
        let ours = rows::<PdbTrack>(&mut written_export().db).remove(0);

        for theirs in all_rekordbox_rows::<PdbTrack>() {
            assert_eq!(theirs.subtype.0, ours.subtype.0, "{:?}", theirs.id);
            assert_eq!(theirs.bitmask, ours.bitmask, "{:?}", theirs.id);
            assert_eq!(theirs.unknown5, ours.unknown5, "{:?}", theirs.id);
        }
    }

    #[test]
    fn artist_and_album_row_constants_match_rekordbox() {
        let mut dev = written_export();
        let our_artist = rows::<Artist>(&mut dev.db).remove(0);
        let our_album = rows::<Album>(&mut dev.db).remove(0);

        for theirs in all_rekordbox_rows::<Artist>() {
            assert_eq!(theirs.subtype.0, our_artist.subtype.0, "{:?}", theirs.id);
        }
        for theirs in all_rekordbox_rows::<Album>() {
            assert_eq!(theirs.subtype.0, our_album.subtype.0, "{:?}", theirs.id);
            assert_eq!(theirs.unknown2, our_album.unknown2, "{:?}", theirs.id);
            assert_eq!(theirs.unknown3, our_album.unknown3, "{:?}", theirs.id);
        }
    }

    #[test]
    fn key_rows_repeat_their_id_as_in_rekordbox() {
        let ours = rows::<Key>(&mut written_export().db);
        assert!(!ours.is_empty());

        for key in all_rekordbox_rows::<Key>().iter().chain(&ours) {
            assert_eq!(key.id2, key.id.0, "{:?}", key.id);
        }
    }

    #[test]
    fn ratings_are_star_counts_in_rekordbox() {
        let mut ratings: Vec<u8> = all_rekordbox_rows::<PdbTrack>()
            .iter()
            .map(|track| track.rating)
            .collect();
        ratings.sort_unstable();
        ratings.dedup();

        // Every star count occurs, and nothing else does.
        assert_eq!(ratings, [0, 1, 2, 3, 4, 5]);
        for rating in ratings {
            assert_eq!(Rating::from(rating).to_byte(), rating);
        }
    }

    #[test]
    fn playlist_nodes_are_ordered_as_in_rekordbox() {
        let theirs = all_rekordbox_rows::<PlaylistTreeNode>();
        assert_counts_up_from(0, sort_orders_by_parent(&theirs), "sort orders");

        let ours = rows::<PlaylistTreeNode>(&mut written_export().db);
        assert_counts_up_from(0, sort_orders_by_parent(&ours), "sort orders");
    }

    #[test]
    fn playlist_entries_are_indexed_as_in_rekordbox() {
        let theirs = all_rekordbox_rows::<PlaylistEntry>();
        assert_counts_up_from(1, entry_indices_by_playlist(&theirs), "entry indices");

        let ours = rows::<PlaylistEntry>(&mut written_export().db);
        assert_counts_up_from(1, entry_indices_by_playlist(&ours), "entry indices");
    }

    #[test]
    fn artwork_paths_follow_the_folder_rule_in_rekordbox() {
        use crate::device::layout::artwork_folder;

        for artwork in all_rekordbox_rows::<Artwork>() {
            let id = artwork.id.0;
            assert_eq!(
                artwork.path.clone().into_string().unwrap(),
                format!("/PIONEER/Artwork/{}/a{id}.jpg", artwork_folder(id))
            );
        }
    }

    #[test]
    fn default_rows_match_rekordbox() {
        use crate::pdb::{Color, ColumnEntry, Menu};

        let mut dev = DeviceExport::create("unsaved-export").unwrap();
        let colors = rows::<Color>(&mut dev.db);
        let columns = rows::<ColumnEntry>(&mut dev.db);
        let menu = rows::<Menu>(&mut dev.db);
        assert!(!colors.is_empty() && !columns.is_empty() && !menu.is_empty());

        for path in REKORDBOX_PDBS {
            assert_eq!(
                rekordbox_rows::<Color>(path, DatabaseType::Plain),
                colors,
                "{path}"
            );
            assert_eq!(
                rekordbox_rows::<ColumnEntry>(path, DatabaseType::Plain),
                columns,
                "{path}"
            );
        }
        // `data/pdb/num_rows` holds another menu than the two complete exports, so the menu
        // is only held against those.
        for path in &REKORDBOX_PDBS[1..] {
            assert_eq!(
                rekordbox_rows::<Menu>(path, DatabaseType::Plain),
                menu,
                "{path}"
            );
        }
    }

    #[test]
    fn tag_rows_match_rekordbox() {
        let our_tags = rows::<TagOrCategory>(written_export().ext_db.as_mut().unwrap());
        let our_category = our_tags.iter().find(|t| t.parent_id.0.is_none()).unwrap();
        let our_leaf = our_tags.iter().find(|t| t.parent_id.0.is_some()).unwrap();

        // What tells a category from a tag, and where each sits among its siblings.
        let check_structure = |tags: &[TagOrCategory], origin: &str| {
            let mut positions: HashMap<u32, Vec<u32>> = HashMap::new();
            for tag in tags {
                let ours = match tag.parent_id.0 {
                    None => our_category,
                    Some(_) => our_leaf,
                };
                assert_eq!(tag.subtype.0, ours.subtype.0, "{origin}: {:?}", tag.id);
                assert_eq!(
                    tag.raw_is_category, ours.raw_is_category,
                    "{origin}: {:?}",
                    tag.id
                );
                assert_eq!(tag.unknown1, ours.unknown1, "{origin}: {:?}", tag.id);
                assert_eq!(tag.unknown2, ours.unknown2, "{origin}: {:?}", tag.id);
                positions
                    .entry(tag.parent_id.0.map_or(0, NonZero::get))
                    .or_default()
                    .push(tag.position);
            }
            assert_counts_up_from(0, positions, "tag positions");

            // Each row is one step of the index shift after the row before it.
            for pair in tags.windows(2) {
                assert_eq!(
                    pair[1].index_shift,
                    pair[0].index_shift + 0x20,
                    "{origin}: {:?}",
                    pair[1].id
                );
            }
        };

        for path in REKORDBOX_EXT_PDBS {
            let theirs = rekordbox_rows::<TagOrCategory>(path, DatabaseType::Ext);
            assert!(!theirs.is_empty(), "{path} holds no tags");
            check_structure(&theirs, path);
        }
        check_structure(&our_tags, "written export");
    }
}
