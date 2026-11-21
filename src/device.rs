// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! High-level API for working with Rekordbox device exports.

use crate::{
    anlz::AnalysisPath,
    pdb::defaults,
    pdb::io::Database,
    pdb::offset_array::OffsetArrayContainer,
    pdb::string::DeviceSQLString,
    pdb::{
        Album, AlbumId, Artist, ArtistId, Artwork, ArtworkId, DatabaseType, Genre, GenreId, Key,
        KeyId, Label, LabelId, PlaylistEntry, PlaylistTreeNode, PlaylistTreeNodeId, TrackId,
        TrackStrings, TrailingName,
    },
    setting,
    setting::{Setting, SettingType},
    util::{ColorIndex, FileType, MaybeCalculated},
};
use binrw::{BinRead, BinWrite};
use fallible_iterator::FallibleIterator;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

/// The paths that make up a Rekordbox device export.
///
/// The export is just a directory tree, so this knows nothing about the device the tree is on and
/// works the same for a mounted USB drive and a directory to collect an export in before copying
/// it to one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportLayout(PathBuf);

impl ExportLayout {
    /// Lay out an export in the given directory, which is the root of the device.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self(root.into())
    }

    /// The root of the device, which holds the `PIONEER` directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.0
    }

    /// The directory holding the databases, the settings and the analysis files.
    #[must_use]
    pub fn pioneer_dir(&self) -> PathBuf {
        self.0.join("PIONEER")
    }

    /// The directory holding the databases.
    #[must_use]
    pub fn rekordbox_dir(&self) -> PathBuf {
        self.pioneer_dir().join("rekordbox")
    }

    /// The database holding the tracks, playlists and the metadata they are browsed by.
    #[must_use]
    pub fn export_pdb(&self) -> PathBuf {
        self.rekordbox_dir().join("export.pdb")
    }

    /// The database holding the extended track information, which only newer players read.
    #[must_use]
    pub fn export_ext_pdb(&self) -> PathBuf {
        self.rekordbox_dir().join("exportExt.pdb")
    }

    /// The directory holding the audio files of the tracks.
    #[must_use]
    pub fn contents_dir(&self) -> PathBuf {
        self.0.join("Contents")
    }

    /// Where the audio file of a track belongs on the device.
    ///
    /// Rekordbox holds the audio of a track under the artist and the album it names, cutting
    /// each name to what a path allows, and a track naming no album is held under a name
    /// standing for that.
    #[must_use]
    pub fn contents_file(&self, artist: &str, album: &str, filename: &str) -> PathBuf {
        let directory = |name: &str, unknown: &str| {
            let name = name.trim();
            let name = if name.is_empty() { unknown } else { name };
            cut_to_length(&replace_path_characters(name), MAX_PATH_PART)
        };
        self.contents_dir()
            .join(directory(artist, UNKNOWN_ARTIST))
            .join(directory(album, UNKNOWN_ALBUM))
            .join(cut_file_name(&replace_path_characters(filename)))
    }

    /// The directory holding the images that hold the artwork of the tracks.
    #[must_use]
    pub fn artwork_dir(&self) -> PathBuf {
        self.pioneer_dir().join("Artwork")
    }

    /// The directory holding the analysis files of the tracks.
    #[must_use]
    pub fn usbanlz_dir(&self) -> PathBuf {
        self.pioneer_dir().join("USBANLZ")
    }

    /// The directory holding the analysis files of the track with the given audio file.
    ///
    /// The path is the path of the audio file on the device, as the track row holds it, e.g.
    /// `/Contents/Artist/Album/Track.mp3`.
    #[must_use]
    pub fn anlz_dir_for_audio(&self, device_audio_path: &str) -> PathBuf {
        self.usbanlz_dir()
            .join(AnalysisPath::for_audio_path(device_audio_path).relative_dir())
    }

    /// The analysis file with the given extension of the track with the given audio file.
    ///
    /// The extension is one of `DAT`, `EXT` or `2EX`.
    #[must_use]
    pub fn anlz_file_for_audio(&self, device_audio_path: &str, extension: &str) -> PathBuf {
        self.anlz_dir_for_audio(device_audio_path)
            .join(format!("{}.{extension}", AnalysisPath::FILE_STEM))
    }

    /// The file holding settings of the given type.
    #[must_use]
    pub fn setting_file(&self, setting_type: SettingType) -> PathBuf {
        self.pioneer_dir().join(setting_type.filename())
    }
}

/// Represents a Rekordbox device export.
#[derive(Debug, PartialEq)]
pub struct DeviceExportLoader(PathBuf);

impl DeviceExportLoader {
    /// Load device export from the given path.
    ///
    /// The path should contain a `PIONEER` directory.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Get the device path.
    #[must_use]
    pub fn get_path(&self) -> &Path {
        &self.0
    }

    fn read_setting_file(path: &PathBuf, setting_type: SettingType) -> crate::Result<Setting> {
        let mut reader = std::fs::File::open(path)?;
        let setting = Setting::read_args(&mut reader, (setting_type,))?;
        Ok(setting)
    }

    /// The paths of the export.
    #[must_use]
    pub fn layout(&self) -> ExportLayout {
        ExportLayout::new(self.0.clone())
    }

    /// Load setting files. If a file is missing or cannot be read,
    /// a warning will be printed.
    #[must_use]
    pub fn load_settings(&self) -> Settings {
        let layout = self.layout();

        let load_setting = |setting_type: SettingType| -> Option<Setting> {
            let file_path = layout.setting_file(setting_type);
            match Self::read_setting_file(&file_path, setting_type) {
                Ok(setting) => Some(setting),
                Err(e) => {
                    eprintln!("Warning: Could not load {}: {}", file_path.display(), e);
                    None
                }
            }
        };

        let mut settings = Settings::default();
        if let Some(devsetting) = load_setting(SettingType::DevSetting) {
            settings.set_devsetting(devsetting.data.as_dev_setting().unwrap());
        }
        if let Some(djmmysetting) = load_setting(SettingType::DJMMySetting) {
            settings.set_djmmysetting(djmmysetting.data.as_djm_my_setting().unwrap());
        }
        if let Some(mysetting) = load_setting(SettingType::MySetting) {
            settings.set_mysetting(mysetting.data.as_my_setting().unwrap());
        }
        if let Some(mysetting2) = load_setting(SettingType::MySetting2) {
            settings.set_mysetting2(mysetting2.data.as_my_setting2().unwrap());
        }

        settings
    }

    /// Open a PDB database without persistence back to disk.
    /// Still allows modifying data in memory.
    pub fn open_pdb_non_persistent(&self) -> crate::Result<Database<File>> {
        let path = self.layout().export_pdb();
        Database::open_non_persistent(File::open(path)?, DatabaseType::Plain)
    }
}

/// Longest a part of a path below `Contents` is allowed to be.
///
/// Rekordbox cuts the name of an artist, an album or a file to this, which the exports it writes
/// bear out: nothing in them is longer, and the names it had to cut end exactly here.
const MAX_PATH_PART: usize = 48;

/// Name standing in for the album of a track that names none, as Rekordbox writes it.
const UNKNOWN_ALBUM: &str = "UnknownAlbum";

/// Name standing in for the artist of a track that names none.
///
/// No export we have holds a track without an artist, so this follows the name used for an
/// album.
const UNKNOWN_ARTIST: &str = "UnknownArtist";
/// Unused page heap space left after a track row.
///
/// Rekordbox leaves this much space after the track rows of the exports we have fixtures for,
/// where a row needing about 290 bytes was given 336. Players have been reported to crash on
/// track rows that were given too little space, so this is not only a matter of writing the same
/// bytes as Rekordbox.
const TRACK_SLACK_BYTES: u16 = 48;

/// A track to add to a device export.
///
/// Only the fields that are known are set; everything else a track row holds is left at what
/// Rekordbox writes for a track it has nothing to put there. The paths are the paths on the
/// device, not on the machine writing the export, and the files they name are not touched: a
/// track is added to the database of an export, and placing its audio alongside is left to the
/// caller.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Track {
    /// Path of the audio file on the device, e.g. `/Contents/Artist/Album/Track.mp3`.
    pub file_path: String,
    /// Title of the track.
    pub title: String,
    /// Name of the artist, which is added to the artists of the export if it is new.
    pub artist: String,
    /// Name of the album, which is added to the albums of the export if it is new.
    pub album: String,
    /// Name of the artist of the album, which an album names rather than naming the artist of a
    /// track on it. Falls back to the artist of the track when empty.
    pub album_artist: String,
    /// Name of the genre, which is added to the genres of the export if it is new.
    pub genre: String,
    /// Name of the record label, which is added to the labels of the export if it is new.
    pub label: String,
    /// Musical key, which is added to the keys of the export if it is new.
    pub key: String,
    /// Comment shown for the track.
    pub comment: String,
    /// Date the track was added to the collection, in `YYYY-MM-DD` form.
    pub date_added: String,
    /// Date the track was released, in `YYYY-MM-DD` form.
    pub release_date: String,
    /// Name of the remix, if any.
    pub mix_name: String,
    /// Path of the analysis file of the track on the device, below `/PIONEER/USBANLZ`.
    pub analyze_path: String,
    /// Path of the image file holding the artwork of the track on the device, below
    /// `/PIONEER/Artwork`. The image itself is not touched, as with the audio file.
    pub artwork_path: String,
    /// Date the track was analyzed, in `YYYY-MM-DD` form.
    pub analyze_date: String,
    /// Tempo in beats per minute.
    pub tempo: f32,
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
    /// Color the track is labelled with.
    pub color: ColorIndex,
    /// Format of the audio file.
    pub file_type: FileType,
}

/// The setting types that an export holds a file for.
const SETTING_TYPES: [SettingType; 4] = [
    SettingType::DevSetting,
    SettingType::DJMMySetting,
    SettingType::MySetting,
    SettingType::MySetting2,
];

/// Writes a device export.
///
/// Creating one lays out the files that every export has: the two databases, holding the content
/// that Rekordbox puts in every export, and the four settings files, holding the default
/// settings. The databases stay open so that content can be added to them, and are written back
/// to the device by [`DeviceExportWriter::close`].
#[derive(Debug)]
pub struct DeviceExportWriter {
    layout: ExportLayout,
    plain: Database<File>,
    ext: Database<File>,
    /// The id a track was given, by the path of its audio file, so that a track that is added
    /// twice does not end up in the export twice.
    track_ids: HashMap<String, TrackId>,
    /// The rows that tracks refer to by name, so that they are only added once each.
    artist_ids: HashMap<String, ArtistId>,
    album_ids: HashMap<String, AlbumId>,
    artwork_ids: HashMap<String, ArtworkId>,
    genre_ids: HashMap<String, GenreId>,
    label_ids: HashMap<String, LabelId>,
    key_ids: HashMap<String, KeyId>,
    /// The nodes of the playlist tree by the folder holding them and their name, so that a
    /// playlist that is already in the export is added to rather than added again. Two playlists
    /// in different folders may have the same name, which is why the folder is part of the key.
    playlist_ids: HashMap<(PlaylistTreeNodeId, String), PlaylistTreeNodeId>,
    /// The position of the last entry of a playlist, by its id.
    playlist_lengths: HashMap<PlaylistTreeNodeId, u32>,
    /// The tracks a playlist already holds, so that one is not put in it twice.
    playlist_members: HashMap<PlaylistTreeNodeId, HashSet<TrackId>>,
    /// The id to give the next row of each kind, which continues where an export that was opened
    /// left off.
    next_ids: NextIds,
}

/// The id to give the next row of each kind.
#[derive(Debug, Default)]
struct NextIds {
    track: u32,
    artist: u32,
    album: u32,
    artwork: u32,
    genre: u32,
    label: u32,
    key: u32,
    playlist: u32,
}

impl NextIds {
    /// The ids of an export that holds nothing, which are counted from one.
    fn initial() -> Self {
        Self {
            track: 1,
            artist: 1,
            album: 1,
            artwork: 1,
            genre: 1,
            label: 1,
            key: 1,
            playlist: 1,
        }
    }

    /// Takes the next id of a kind, and moves on to the one after it.
    fn take(counter: &mut u32) -> u32 {
        let id = *counter;
        *counter += 1;
        id
    }
}

impl DeviceExportWriter {
    /// Creates the files of an export in the given directory.
    ///
    /// `date` is the date the export is written, in the `YYYY-MM-DD` form that Rekordbox records
    /// it in. Fails without writing anything if the directory already holds an export, so that an
    /// export that took a long time to collect cannot be destroyed by accident.
    pub fn create(root: impl Into<PathBuf>, date: &str) -> crate::Result<Self> {
        let layout = ExportLayout::new(root);
        std::fs::create_dir_all(layout.rekordbox_dir())?;

        for setting_type in SETTING_TYPES {
            let setting = match setting_type {
                SettingType::DevSetting => Setting::default_devsetting(),
                SettingType::DJMMySetting => Setting::default_djmmysetting(),
                SettingType::MySetting => Setting::default_mysetting(),
                SettingType::MySetting2 => Setting::default_mysetting2(),
            };
            write_setting_file(&layout.setting_file(setting_type), &setting)?;
        }

        let mut plain = create_database(&layout.export_pdb(), DatabaseType::Plain)?;
        defaults::insert_defaults(&mut plain, date, 0)?;
        let mut ext = create_database(&layout.export_ext_pdb(), DatabaseType::Ext)?;
        defaults::insert_ext_defaults(&mut ext)?;

        Ok(Self {
            layout,
            plain,
            ext,
            track_ids: HashMap::new(),
            artist_ids: HashMap::new(),
            album_ids: HashMap::new(),
            artwork_ids: HashMap::new(),
            genre_ids: HashMap::new(),
            label_ids: HashMap::new(),
            key_ids: HashMap::new(),
            playlist_ids: HashMap::new(),
            playlist_lengths: HashMap::new(),
            playlist_members: HashMap::new(),
            next_ids: NextIds::initial(),
        })
    }

    /// Opens an export that is already on a device, to add to it.
    ///
    /// What the export already holds is read back, so that a track whose audio file is already
    /// there keeps the id it has, the rows that tracks share are reused, and a playlist that is
    /// already there is added to rather than added again. That makes it possible to put a
    /// playlist that has grown on a device without copying what is on it already.
    pub fn open(root: impl Into<PathBuf>) -> crate::Result<Self> {
        let layout = ExportLayout::new(root);
        let plain = open_database(&layout.export_pdb(), DatabaseType::Plain)?;
        let ext = open_database(&layout.export_ext_pdb(), DatabaseType::Ext)?;

        let mut writer = Self {
            layout,
            plain,
            ext,
            track_ids: HashMap::new(),
            artist_ids: HashMap::new(),
            album_ids: HashMap::new(),
            artwork_ids: HashMap::new(),
            genre_ids: HashMap::new(),
            label_ids: HashMap::new(),
            key_ids: HashMap::new(),
            playlist_ids: HashMap::new(),
            playlist_lengths: HashMap::new(),
            playlist_members: HashMap::new(),
            next_ids: NextIds::initial(),
        };
        writer.read_existing()?;
        Ok(writer)
    }

    /// Reads back what an export already holds.
    fn read_existing(&mut self) -> crate::Result<()> {
        // The rows are collected first, as reading them borrows the database.
        let tracks: Vec<(String, TrackId)> = self
            .plain
            .iter_rows::<crate::pdb::Track>()?
            .map(|row| Ok((row.offsets.file_path.clone().into_string()?, row.id)))
            .collect()?;
        let artists: Vec<(String, ArtistId)> = self
            .plain
            .iter_rows::<Artist>()?
            .map(|row| Ok((row.offsets.name.clone().into_string()?, row.id)))
            .collect()?;
        let albums: Vec<(String, AlbumId)> = self
            .plain
            .iter_rows::<Album>()?
            .map(|row| Ok((row.offsets.name.clone().into_string()?, row.id)))
            .collect()?;
        let genres: Vec<(String, GenreId)> = self
            .plain
            .iter_rows::<Genre>()?
            .map(|row| Ok((row.name.clone().into_string()?, row.id)))
            .collect()?;
        let labels: Vec<(String, LabelId)> = self
            .plain
            .iter_rows::<Label>()?
            .map(|row| Ok((row.name.clone().into_string()?, row.id)))
            .collect()?;
        let keys: Vec<(String, KeyId)> = self
            .plain
            .iter_rows::<Key>()?
            .map(|row| Ok((row.name.clone().into_string()?, row.id)))
            .collect()?;
        let artwork: Vec<(String, ArtworkId)> = self
            .plain
            .iter_rows::<Artwork>()?
            .map(|row| Ok((row.path.clone().into_string()?, row.id)))
            .collect()?;
        let playlists: Vec<(PlaylistTreeNodeId, String, PlaylistTreeNodeId)> = self
            .plain
            .iter_rows::<PlaylistTreeNode>()?
            .map(|row| Ok((row.parent_id, row.name.clone().into_string()?, row.id)))
            .collect()?;
        let entries: Vec<(PlaylistTreeNodeId, u32, TrackId)> = self
            .plain
            .iter_rows::<PlaylistEntry>()?
            .map(|row| Ok((row.playlist_id, row.entry_index, row.track_id)))
            .collect()?;

        // An id that is already taken must not be given out again.
        self.next_ids.track = next_after(tracks.iter().map(|(_, id)| id.0));
        self.next_ids.artist = next_after(artists.iter().map(|(_, id)| id.0));
        self.next_ids.album = next_after(albums.iter().map(|(_, id)| id.0));
        self.next_ids.genre = next_after(genres.iter().map(|(_, id)| id.0));
        self.next_ids.label = next_after(labels.iter().map(|(_, id)| id.0));
        self.next_ids.key = next_after(keys.iter().map(|(_, id)| id.0));
        self.next_ids.artwork = next_after(artwork.iter().map(|(_, id)| id.0));
        self.next_ids.playlist = next_after(playlists.iter().map(|(_, _, id)| id.0));

        self.track_ids = tracks.into_iter().collect();
        self.artist_ids = artists.into_iter().collect();
        self.album_ids = albums.into_iter().collect();
        self.genre_ids = genres.into_iter().collect();
        self.label_ids = labels.into_iter().collect();
        self.key_ids = keys.into_iter().collect();
        self.artwork_ids = artwork.into_iter().collect();
        self.playlist_ids = playlists
            .iter()
            .map(|(parent, name, id)| ((*parent, name.clone()), *id))
            .collect();
        for (_, _, id) in playlists {
            self.playlist_lengths.entry(id).or_insert(0);
            self.playlist_members.entry(id).or_default();
        }
        for (playlist_id, position, track_id) in entries {
            let length = self.playlist_lengths.entry(playlist_id).or_insert(0);
            *length = (*length).max(position);
            self.playlist_members
                .entry(playlist_id)
                .or_default()
                .insert(track_id);
        }
        Ok(())
    }

    /// The paths of the export being written.
    #[must_use]
    pub fn layout(&self) -> &ExportLayout {
        &self.layout
    }

    /// The database holding the tracks, playlists and the metadata they are browsed by.
    pub fn plain_database(&mut self) -> &mut Database<File> {
        &mut self.plain
    }

    /// The database holding the extended track information.
    pub fn ext_database(&mut self) -> &mut Database<File> {
        &mut self.ext
    }

    /// Adds a track to the export.
    ///
    /// The artist, album, genre, label and key of the track are added to the export as well if
    /// they are not in it yet, and shared with the tracks added before. A track whose audio file
    /// was added before is not added again, and keeps the id it was given then.
    ///
    /// Returns the id the track has in the export, which playlists refer to it by.
    pub fn add_track(&mut self, track: &Track) -> crate::Result<TrackId> {
        if let Some(&id) = self.track_ids.get(&track.file_path) {
            return Ok(id);
        }

        let artist_id = self.add_artist(&track.artist)?;
        // An album names its own artist, which is the artist of the track where the track names
        // none of its own.
        let album_artist_id = if track.album_artist.is_empty() {
            artist_id
        } else {
            self.add_artist(&track.album_artist)?
        };
        let album_id = self.add_album(&track.album, album_artist_id)?;
        let genre_id = self.add_genre(&track.genre)?;
        let label_id = self.add_label(&track.label)?;
        let key_id = self.add_key(&track.key)?;
        let artwork_id = self.add_artwork(&track.artwork_path)?;

        let id = TrackId(NextIds::take(&mut self.next_ids.track));
        let row = crate::pdb::Track {
            id,
            artist_id,
            album_id,
            genre_id,
            label_id,
            key_id,
            artwork_id,
            sample_rate: track.sample_rate,
            sample_depth: track.sample_depth,
            file_size: track.file_size,
            bitrate: track.bitrate,
            track_number: track.track_number,
            disc_number: track.disc_number,
            year: track.year,
            duration: track.duration,
            rating: track.rating,
            play_count: track.play_count,
            color: track.color.clone(),
            file_type: track.file_type.clone(),
            // The tempo is stored in hundredths of a beat per minute.
            tempo: tempo_hundredths(track.tempo).ok_or(crate::Error::IntegrityError(
                "the tempo of a track is not a number of beats per minute that can be recorded",
            ))?,
            offsets: OffsetArrayContainer {
                offsets: MaybeCalculated::Calculated,
                inner: TrackStrings {
                    // Rekordbox keeps a counter beside the play count, which holds the count
                    // plus one in the exports we have to go by.
                    unknown_string2: DeviceSQLString::new(
                        &track.play_count.saturating_add(1).to_string(),
                    )?,
                    title: DeviceSQLString::new(&track.title)?,
                    file_path: DeviceSQLString::new(&track.file_path)?,
                    filename: DeviceSQLString::new(file_name_of(&track.file_path))?,
                    comment: DeviceSQLString::new(&track.comment)?,
                    date_added: DeviceSQLString::new(&track.date_added)?,
                    release_date: DeviceSQLString::new(&track.release_date)?,
                    mix_name: DeviceSQLString::new(&track.mix_name)?,
                    analyze_path: DeviceSQLString::new(&track.analyze_path)?,
                    analyze_date: DeviceSQLString::new(&track.analyze_date)?,
                    ..Default::default()
                },
            },
            ..Default::default()
        };
        self.plain.add_row_with_slack(row, TRACK_SLACK_BYTES)?;

        self.track_ids.insert(track.file_path.clone(), id);
        Ok(id)
    }

    /// Adds a folder to the playlist tree, which playlists and further folders can be put in.
    ///
    /// Returns the id of the folder, which is what `parent` takes to put something in it.
    pub fn create_playlist_folder(
        &mut self,
        parent: Option<PlaylistTreeNodeId>,
        name: &str,
    ) -> crate::Result<PlaylistTreeNodeId> {
        self.create_playlist_node(parent, name, true)
    }

    /// Adds a playlist to the playlist tree, which tracks can be added to.
    ///
    /// Returns the id of the playlist, which is what [`DeviceExportWriter::add_track_to_playlist`]
    /// takes.
    pub fn create_playlist(
        &mut self,
        parent: Option<PlaylistTreeNodeId>,
        name: &str,
    ) -> crate::Result<PlaylistTreeNodeId> {
        self.create_playlist_node(parent, name, false)
    }

    /// Adds a track to the end of a playlist.
    pub fn add_track_to_playlist(
        &mut self,
        playlist_id: PlaylistTreeNodeId,
        track_id: TrackId,
    ) -> crate::Result<bool> {
        let length = self.playlist_lengths.get(&playlist_id).copied().ok_or(
            crate::Error::IntegrityError("no playlist of the export has the given id"),
        )?;
        if self
            .playlist_members
            .get(&playlist_id)
            .is_some_and(|members| members.contains(&track_id))
        {
            return Ok(false);
        }

        self.plain.add_row(PlaylistEntry {
            // Positions in a playlist are counted from one.
            entry_index: length + 1,
            track_id,
            playlist_id,
        })?;
        self.playlist_lengths.insert(playlist_id, length + 1);
        self.playlist_members
            .entry(playlist_id)
            .or_default()
            .insert(track_id);
        Ok(true)
    }

    /// The node of the playlist tree with the given name, in the given folder or at the root of
    /// the tree, if the export holds one.
    #[must_use]
    pub fn playlist_by_name(
        &self,
        parent: Option<PlaylistTreeNodeId>,
        name: &str,
    ) -> Option<PlaylistTreeNodeId> {
        self.playlist_ids
            .get(&(parent.unwrap_or(PlaylistTreeNodeId(0)), name.to_owned()))
            .copied()
    }

    /// Adds a node to the playlist tree, which is either a folder or a playlist.
    fn create_playlist_node(
        &mut self,
        parent: Option<PlaylistTreeNodeId>,
        name: &str,
        is_folder: bool,
    ) -> crate::Result<PlaylistTreeNodeId> {
        if let Some(parent) = parent {
            if !self.playlist_lengths.contains_key(&parent) {
                return Err(crate::Error::IntegrityError(
                    "no node of the playlist tree has the given id",
                ));
            }
        }

        let index = self.next_ids.playlist - 1;
        let id = PlaylistTreeNodeId(NextIds::take(&mut self.next_ids.playlist));
        self.plain.add_row(PlaylistTreeNode {
            // A node with no parent sits at the root of the tree, which is what a parent of zero
            // means.
            parent_id: parent.unwrap_or(PlaylistTreeNodeId(0)),
            sort_order: index,
            id,
            node_is_folder: u32::from(is_folder),
            name: DeviceSQLString::new(name)?,
            ..Default::default()
        })?;

        self.playlist_ids.insert(
            (parent.unwrap_or(PlaylistTreeNodeId(0)), name.to_owned()),
            id,
        );
        self.playlist_lengths.insert(id, 0);
        self.playlist_members.entry(id).or_default();
        Ok(id)
    }

    /// Adds the image holding the artwork of a track unless it is already in the export, and
    /// returns its id.
    fn add_artwork(&mut self, device_path: &str) -> crate::Result<ArtworkId> {
        if device_path.is_empty() {
            return Ok(ArtworkId(0));
        }
        if let Some(&id) = self.artwork_ids.get(device_path) {
            return Ok(id);
        }

        let id = ArtworkId(
            u32::try_from(self.artwork_ids.len())
                .map_err(|_| crate::Error::IntegrityError("too many artwork images"))?
                + 1,
        );
        self.plain.add_row(Artwork {
            id,
            path: DeviceSQLString::new(device_path)?,
        })?;
        self.artwork_ids.insert(device_path.to_owned(), id);
        Ok(id)
    }

    /// Adds an artist to the export unless it is already in it, and returns its id.
    fn add_artist(&mut self, name: &str) -> crate::Result<ArtistId> {
        if name.is_empty() {
            return Ok(ArtistId(0));
        }
        if let Some(&id) = self.artist_ids.get(name) {
            return Ok(id);
        }

        let id = ArtistId(NextIds::take(&mut self.next_ids.artist));
        self.plain.add_row(Artist {
            id,
            offsets: OffsetArrayContainer {
                offsets: MaybeCalculated::Calculated,
                inner: TrailingName {
                    name: DeviceSQLString::new(name)?,
                },
            },
            ..Default::default()
        })?;
        self.artist_ids.insert(name.to_owned(), id);
        Ok(id)
    }

    /// Adds an album to the export unless it is already in it, and returns its id.
    fn add_album(&mut self, name: &str, artist_id: ArtistId) -> crate::Result<AlbumId> {
        if name.is_empty() {
            return Ok(AlbumId(0));
        }
        if let Some(&id) = self.album_ids.get(name) {
            return Ok(id);
        }

        let id = AlbumId(NextIds::take(&mut self.next_ids.album));
        self.plain.add_row(Album {
            id,
            artist_id,
            offsets: OffsetArrayContainer {
                offsets: MaybeCalculated::Calculated,
                inner: TrailingName {
                    name: DeviceSQLString::new(name)?,
                },
            },
            ..Default::default()
        })?;
        self.album_ids.insert(name.to_owned(), id);
        Ok(id)
    }

    /// Adds a genre to the export unless it is already in it, and returns its id.
    fn add_genre(&mut self, name: &str) -> crate::Result<GenreId> {
        if name.is_empty() {
            return Ok(GenreId(0));
        }
        if let Some(&id) = self.genre_ids.get(name) {
            return Ok(id);
        }

        let id = GenreId(NextIds::take(&mut self.next_ids.genre));
        self.plain.add_row(Genre {
            id,
            name: DeviceSQLString::new(name)?,
        })?;
        self.genre_ids.insert(name.to_owned(), id);
        Ok(id)
    }

    /// Adds a record label to the export unless it is already in it, and returns its id.
    fn add_label(&mut self, name: &str) -> crate::Result<LabelId> {
        if name.is_empty() {
            return Ok(LabelId(0));
        }
        if let Some(&id) = self.label_ids.get(name) {
            return Ok(id);
        }

        let id = LabelId(NextIds::take(&mut self.next_ids.label));
        self.plain.add_row(Label {
            id,
            name: DeviceSQLString::new(name)?,
        })?;
        self.label_ids.insert(name.to_owned(), id);
        Ok(id)
    }

    /// Adds a musical key to the export unless it is already in it, and returns its id.
    fn add_key(&mut self, name: &str) -> crate::Result<KeyId> {
        if name.is_empty() {
            return Ok(KeyId(0));
        }
        if let Some(&id) = self.key_ids.get(name) {
            return Ok(id);
        }

        let index = self.next_ids.key - 1;
        let id = KeyId(NextIds::take(&mut self.next_ids.key));
        self.plain.add_row(Key {
            id,
            // The second copy of the id that a key row holds.
            id2: index + 1,
            name: DeviceSQLString::new(name)?,
        })?;
        self.key_ids.insert(name.to_owned(), id);
        Ok(id)
    }

    /// Writes the databases to the device and makes sure everything written has reached it.
    ///
    /// Call this before the device is unmounted, as nothing guarantees that the export has been
    /// written otherwise.
    pub fn close(mut self) -> crate::Result<()> {
        self.plain.flush()?;
        self.ext.flush()?;
        // The databases point at the analysis and audio files, so they are the last thing that
        // should reach the device.
        self.ext.into_inner().sync_all()?;
        self.plain.into_inner().sync_all()?;
        Ok(())
    }
}

/// A tempo in the hundredths of a beat per minute that a track row counts it in, or `None` for
/// one that cannot be counted that way, such as a tempo that is not a number at all.
fn tempo_hundredths(tempo: f32) -> Option<u32> {
    let hundredths = (f64::from(tempo) * 100.0).round();
    if !hundredths.is_finite() || !(0.0..=f64::from(u32::MAX)).contains(&hundredths) {
        return None;
    }
    // Whole, finite and within range, as checked just above.
    Some(hundredths as u32)
}

/// Replaces the characters that a part of a path cannot hold.
fn replace_path_characters(name: &str) -> String {
    name.replace(['/', ':'], "_")
}

/// Cuts a name to a number of characters, counting characters rather than bytes so that a name
/// holding more than ASCII is not cut through the middle of one.
fn cut_to_length(name: &str, length: usize) -> String {
    name.chars().take(length).collect()
}

/// Cuts the name of a file, keeping the extension so that the file is still what it was.
fn cut_file_name(filename: &str) -> String {
    if filename.chars().count() <= MAX_PATH_PART {
        return filename.to_owned();
    }
    match filename.rsplit_once('.') {
        Some((stem, extension)) => {
            let room = MAX_PATH_PART.saturating_sub(extension.chars().count() + 1);
            format!("{}.{extension}", cut_to_length(stem, room))
        }
        None => cut_to_length(filename, MAX_PATH_PART),
    }
}
/// The name of the file a path names, which a track row holds alongside the path itself.
fn file_name_of(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// The id to give out next, after the ids that are already taken.
fn next_after(ids: impl Iterator<Item = u32>) -> u32 {
    ids.max().unwrap_or(0) + 1
}

/// Opens a database that a device already holds, for reading and writing.
fn open_database(path: &Path, db_type: DatabaseType) -> crate::Result<Database<File>> {
    let file = File::options().read(true).write(true).open(path)?;
    Database::open(file, db_type)
}

/// Creates a database file and an empty database in it.
fn create_database(path: &Path, db_type: DatabaseType) -> crate::Result<Database<File>> {
    let file = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)?;
    Database::create(file, db_type)
}

/// Writes a settings file and makes sure it has reached the device.
fn write_setting_file(path: &Path, setting: &Setting) -> crate::Result<()> {
    let mut file = File::options().write(true).create_new(true).open(path)?;
    setting.write(&mut file)?;
    file.sync_all()?;
    Ok(())
}

/// Settings object containing for all device settings.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Settings {
    // MYSETTING.DAT
    /// "ON AIR DISPLAY" setting.
    pub on_air_display: Option<setting::OnAirDisplay>,
    /// "LCD BRIGHTNESS" setting.
    pub lcd_brightness: Option<setting::LCDBrightness>,
    /// "QUANTIZE" setting.
    pub quantize: Option<setting::Quantize>,
    /// "AUTO CUE LEVEL" setting.
    pub auto_cue_level: Option<setting::AutoCueLevel>,
    /// "LANGUAGE" setting.
    pub language: Option<setting::Language>,
    /// "JOG RING BRIGHTNESS" setting.
    pub jog_ring_brightness: Option<setting::JogRingBrightness>,
    /// "JOG RING INDICATOR" setting.
    pub jog_ring_indicator: Option<setting::JogRingIndicator>,
    /// "SLIP FLASHING" setting.
    pub slip_flashing: Option<setting::SlipFlashing>,
    /// "DISC SLOT ILLUMINATION" setting.
    pub disc_slot_illumination: Option<setting::DiscSlotIllumination>,
    /// "EJECT/LOAD LOCK" setting.
    pub eject_lock: Option<setting::EjectLock>,
    /// "SYNC" setting.
    pub sync: Option<setting::Sync>,
    /// "PLAY MODE / AUTO PLAY MODE" setting.
    pub play_mode: Option<setting::PlayMode>,
    /// Quantize Beat Value setting.
    pub quantize_beat_value: Option<setting::QuantizeBeatValue>,
    /// "HOT CUE AUTO LOAD" setting.
    pub hotcue_autoload: Option<setting::HotCueAutoLoad>,
    /// "HOT CUE COLOR" setting.
    pub hotcue_color: Option<setting::HotCueColor>,
    /// "NEEDLE LOCK" setting.
    pub needle_lock: Option<setting::NeedleLock>,
    /// "TIME MODE" setting.
    pub time_mode: Option<setting::TimeMode>,
    /// "TIME MODE" setting.
    pub jog_mode: Option<setting::JogMode>,
    /// "AUTO CUE" setting.
    pub auto_cue: Option<setting::AutoCue>,
    /// "MASTER TEMPO" setting.
    pub master_tempo: Option<setting::MasterTempo>,
    /// "TEMPO RANGE" setting.
    pub tempo_range: Option<setting::TempoRange>,
    /// "PHASE METER" setting.
    pub phase_meter: Option<setting::PhaseMeter>,

    // MYSETTING2.DAT
    /// "VINYL SPEED ADJUST" setting.
    pub vinyl_speed_adjust: Option<setting::VinylSpeedAdjust>,
    /// "JOG DISPLAY MODE" setting.
    pub jog_display_mode: Option<setting::JogDisplayMode>,
    /// "PAD/BUTTON BRIGHTNESS" setting.
    pub pad_button_brightness: Option<setting::PadButtonBrightness>,
    /// "JOG LCD BRIGHTNESS" setting.
    pub jog_lcd_brightness: Option<setting::JogLCDBrightness>,
    /// "WAVEFORM DIVISIONS" setting.
    pub waveform_divisions: Option<setting::WaveformDivisions>,
    /// "WAVEFORM / PHASE METER" setting.
    pub waveform: Option<setting::Waveform>,
    /// "BEAT JUMP BEAT VALUE" setting.
    pub beat_jump_beat_value: Option<setting::BeatJumpBeatValue>,

    // DJMSETTING.DAT
    /// "CH FADER CURVE" setting.
    pub channel_fader_curve: Option<setting::ChannelFaderCurve>,
    /// "CROSSFADER CURVE" setting.
    pub crossfader_curve: Option<setting::CrossfaderCurve>,
    /// "HEADPHONES PRE EQ" setting.
    pub headphones_pre_eq: Option<setting::HeadphonesPreEQ>,
    /// "HEADPHONES MONO SPLIT" setting.
    pub headphones_mono_split: Option<setting::HeadphonesMonoSplit>,
    /// "BEAT FX QUANTIZE" setting.
    pub beat_fx_quantize: Option<setting::BeatFXQuantize>,
    /// "MIC LOW CUT" setting.
    pub mic_low_cut: Option<setting::MicLowCut>,
    /// "TALK OVER MODE" setting.
    pub talk_over_mode: Option<setting::TalkOverMode>,
    /// "TALK OVER LEVEL" setting.
    pub talk_over_level: Option<setting::TalkOverLevel>,
    /// "MIDI CH" setting.
    pub midi_channel: Option<setting::MidiChannel>,
    /// "MIDI BUTTON TYPE" setting.
    pub midi_button_type: Option<setting::MidiButtonType>,
    /// "BRIGHTNESS > DISPLAY" setting.
    pub display_brightness: Option<setting::MixerDisplayBrightness>,
    /// "BRIGHTNESS > INDICATOR" setting.
    pub indicator_brightness: Option<setting::MixerIndicatorBrightness>,
    /// "CH FADER CURVE (LONG FADER)" setting.
    pub channel_fader_curve_long_fader: Option<setting::ChannelFaderCurveLongFader>,

    // DEVSETTING.DAT
    /// "Type of the overview Waveform" setting.
    pub overview_waveform_type: Option<setting::OverviewWaveformType>,
    /// "Waveform color" setting.
    pub waveform_color: Option<setting::WaveformColor>,
    /// "Key display format" setting.
    pub key_display_format: Option<setting::KeyDisplayFormat>,
    /// "Waveform Current Position" setting.
    pub waveform_current_position: Option<setting::WaveformCurrentPosition>,
}

impl Settings {
    fn set_mysetting(&mut self, data: &setting::MySetting) {
        self.on_air_display = data.on_air_display.into();
        self.lcd_brightness = data.lcd_brightness.into();
        self.quantize = data.quantize.into();
        self.auto_cue_level = data.auto_cue_level.into();
        self.language = data.language.into();
        self.jog_ring_brightness = data.jog_ring_brightness.into();
        self.jog_ring_indicator = data.jog_ring_indicator.into();
        self.slip_flashing = data.slip_flashing.into();
        self.disc_slot_illumination = data.disc_slot_illumination.into();
        self.eject_lock = data.eject_lock.into();
        self.sync = data.sync.into();
        self.play_mode = data.play_mode.into();
        self.quantize_beat_value = data.quantize_beat_value.into();
        self.hotcue_autoload = data.hotcue_autoload.into();
        self.hotcue_color = data.hotcue_color.into();
        self.needle_lock = data.needle_lock.into();
        self.time_mode = data.time_mode.into();
        self.jog_mode = data.jog_mode.into();
        self.auto_cue = data.auto_cue.into();
        self.master_tempo = data.master_tempo.into();
        self.tempo_range = data.tempo_range.into();
        self.phase_meter = data.phase_meter.into();
    }

    fn set_mysetting2(&mut self, data: &setting::MySetting2) {
        self.vinyl_speed_adjust = data.vinyl_speed_adjust.into();
        self.jog_display_mode = data.jog_display_mode.into();
        self.pad_button_brightness = data.pad_button_brightness.into();
        self.jog_lcd_brightness = data.jog_lcd_brightness.into();
        self.waveform_divisions = data.waveform_divisions.into();
        self.waveform = data.waveform.into();
        self.beat_jump_beat_value = data.beat_jump_beat_value.into();
    }

    fn set_djmmysetting(&mut self, data: &setting::DJMMySetting) {
        self.channel_fader_curve = data.channel_fader_curve.into();
        self.crossfader_curve = data.crossfader_curve.into();
        self.headphones_pre_eq = data.headphones_pre_eq.into();
        self.headphones_mono_split = data.headphones_mono_split.into();
        self.beat_fx_quantize = data.beat_fx_quantize.into();
        self.mic_low_cut = data.mic_low_cut.into();
        self.talk_over_mode = data.talk_over_mode.into();
        self.talk_over_level = data.talk_over_level.into();
        self.midi_channel = data.midi_channel.into();
        self.midi_button_type = data.midi_button_type.into();
        self.display_brightness = data.display_brightness.into();
        self.indicator_brightness = data.indicator_brightness.into();
        self.channel_fader_curve_long_fader = data.channel_fader_curve_long_fader.into();
    }

    fn set_devsetting(&mut self, data: &setting::DevSetting) {
        self.overview_waveform_type = data.overview_waveform_type.into();
        self.waveform_color = data.waveform_color.into();
        self.key_display_format = data.key_display_format.into();
        self.waveform_current_position = data.waveform_current_position.into();
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_option<T: fmt::Display>(opt: Option<T>) -> String {
            opt.map(|v| v.to_string())
                .unwrap_or_else(|| "<missing>".to_string())
        }

        writeln!(
            f,
            "On Air Display:                 {}",
            format_option(self.on_air_display)
        )?;
        writeln!(
            f,
            "LCD Brightness:                 {}",
            format_option(self.lcd_brightness)
        )?;
        writeln!(
            f,
            "Quantize:                       {}",
            format_option(self.quantize)
        )?;
        writeln!(
            f,
            "Auto Cue Level:                 {}",
            format_option(self.auto_cue_level)
        )?;
        writeln!(
            f,
            "Language:                       {}",
            format_option(self.language)
        )?;
        writeln!(
            f,
            "Jog Ring Brightness:            {}",
            format_option(self.jog_ring_brightness)
        )?;
        writeln!(
            f,
            "Jog Ring Indicator:             {}",
            format_option(self.jog_ring_indicator)
        )?;
        writeln!(
            f,
            "Slip Flashing:                  {}",
            format_option(self.slip_flashing)
        )?;
        writeln!(
            f,
            "Disc Slot Illumination:         {}",
            format_option(self.disc_slot_illumination)
        )?;
        writeln!(
            f,
            "Eject Lock:                     {}",
            format_option(self.eject_lock)
        )?;
        writeln!(
            f,
            "Sync:                           {}",
            format_option(self.sync)
        )?;
        writeln!(
            f,
            "Play Mode:                      {}",
            format_option(self.play_mode)
        )?;
        writeln!(
            f,
            "Quantize Beat Value:            {}",
            format_option(self.quantize_beat_value)
        )?;
        writeln!(
            f,
            "Hotcue Autoload:                {}",
            format_option(self.hotcue_autoload)
        )?;
        writeln!(
            f,
            "Hotcue Color:                   {}",
            format_option(self.hotcue_color)
        )?;
        writeln!(
            f,
            "Needle Lock:                    {}",
            format_option(self.needle_lock)
        )?;
        writeln!(
            f,
            "Time Mode:                      {}",
            format_option(self.time_mode)
        )?;
        writeln!(
            f,
            "Jog Mode:                       {}",
            format_option(self.jog_mode)
        )?;
        writeln!(
            f,
            "Auto Cue:                       {}",
            format_option(self.auto_cue)
        )?;
        writeln!(
            f,
            "Master Tempo:                   {}",
            format_option(self.master_tempo)
        )?;
        writeln!(
            f,
            "Tempo Range:                    {}",
            format_option(self.tempo_range)
        )?;
        writeln!(
            f,
            "Phase Meter:                    {}",
            format_option(self.phase_meter)
        )?;
        writeln!(
            f,
            "Vinyl Speed Adjust:             {}",
            format_option(self.vinyl_speed_adjust)
        )?;
        writeln!(
            f,
            "Jog Display Mode:               {}",
            format_option(self.jog_display_mode)
        )?;
        writeln!(
            f,
            "Pad Button Brightness:          {}",
            format_option(self.pad_button_brightness)
        )?;
        writeln!(
            f,
            "Jog LCD Brightness:             {}",
            format_option(self.jog_lcd_brightness)
        )?;
        writeln!(
            f,
            "Waveform Divisions:             {}",
            format_option(self.waveform_divisions)
        )?;
        writeln!(
            f,
            "Waveform:                       {}",
            format_option(self.waveform)
        )?;
        writeln!(
            f,
            "Beat Jump Beat Value:           {}",
            format_option(self.beat_jump_beat_value)
        )?;
        writeln!(
            f,
            "Channel Fader Curve:            {}",
            format_option(self.channel_fader_curve)
        )?;
        writeln!(
            f,
            "Crossfader Curve:               {}",
            format_option(self.crossfader_curve)
        )?;
        writeln!(
            f,
            "Headphones Pre Eq:              {}",
            format_option(self.headphones_pre_eq)
        )?;
        writeln!(
            f,
            "Headphones Mono Split:          {}",
            format_option(self.headphones_mono_split)
        )?;
        writeln!(
            f,
            "Beat FX Quantize:               {}",
            format_option(self.beat_fx_quantize)
        )?;
        writeln!(
            f,
            "Mic Low Cut:                    {}",
            format_option(self.mic_low_cut)
        )?;
        writeln!(
            f,
            "Talk Over Mode:                 {}",
            format_option(self.talk_over_mode)
        )?;
        writeln!(
            f,
            "Talk Over Level:                {}",
            format_option(self.talk_over_level)
        )?;
        writeln!(
            f,
            "MIDI Channel:                   {}",
            format_option(self.midi_channel)
        )?;
        writeln!(
            f,
            "MIDI Button Type:               {}",
            format_option(self.midi_button_type)
        )?;
        writeln!(
            f,
            "Display Brightness:             {}",
            format_option(self.display_brightness)
        )?;
        writeln!(
            f,
            "Indicator Brightness:           {}",
            format_option(self.indicator_brightness)
        )?;
        writeln!(
            f,
            "Channel Fader Curve Long Fader: {}",
            format_option(self.channel_fader_curve_long_fader)
        )?;
        writeln!(
            f,
            "Overview Waveform Type:         {}",
            format_option(self.overview_waveform_type)
        )?;
        writeln!(
            f,
            "Waveform Color:                 {}",
            format_option(self.waveform_color)
        )?;
        writeln!(
            f,
            "Key Display Format:             {}",
            format_option(self.key_display_format)
        )?;
        write!(
            f,
            "Waveform Current Position:      {}",
            format_option(self.waveform_current_position)
        )
    }
}

/// Represents either a playlist folder or a playlist.
#[derive(Debug, PartialEq)]
pub enum PlaylistNode {
    /// Represents a playlist folder that contains `PlaylistNode`s.
    Folder(PlaylistFolder),
    /// Represents a playlist.
    Playlist(Playlist),
}

/// Represents a playlist folder that contains `PlaylistNode`s.
#[derive(Debug, PartialEq)]
pub struct PlaylistFolder {
    /// ID of this node in the playlist tree.
    pub id: PlaylistTreeNodeId,
    /// Name of the playlist folder.
    pub name: String,
    /// Child nodes of the playlist folder.
    pub children: Vec<PlaylistNode>,
}

/// Represents a playlist.
#[derive(Debug, PartialEq, Eq)]
pub struct Playlist {
    /// ID of this node in the playlist tree.
    pub id: PlaylistTreeNodeId,
    /// Name of the playlist.
    pub name: String,
}

/// Get playlist tree.
pub fn get_playlists<R: Read + Seek>(db: &mut Database<R>) -> crate::Result<Vec<PlaylistNode>> {
    let mut playlists: HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>> = HashMap::new();

    db.iter_rows::<PlaylistTreeNode>()?.for_each(|node| {
        playlists
            .entry(node.parent_id)
            .or_default()
            .push(node.clone());
        Ok(())
    })?;

    fn get_child_nodes(
        playlists: &HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>>,
        id: PlaylistTreeNodeId,
    ) -> impl Iterator<Item = crate::Result<PlaylistNode>> + '_ {
        playlists
            .get(&id)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
            .map(|node| -> crate::Result<PlaylistNode> {
                let child_node = if node.is_folder() {
                    let folder = PlaylistFolder {
                        id: node.id,
                        name: node.name.clone().into_string()?,
                        children: get_child_nodes(playlists, node.id)
                            .collect::<crate::Result<Vec<PlaylistNode>>>()?,
                    };
                    PlaylistNode::Folder(folder)
                } else {
                    let playlist = Playlist {
                        id: node.id,
                        name: node.name.clone().into_string()?,
                    };
                    PlaylistNode::Playlist(playlist)
                };
                Ok(child_node)
            })
    }

    get_child_nodes(&playlists, PlaylistTreeNodeId(0)).collect::<crate::Result<Vec<PlaylistNode>>>()
}
