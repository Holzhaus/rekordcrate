// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Tests for writing a device export.
//!
//! The export that Rekordbox writes for a device without any tracks on it is the reference for
//! what a written export should look like, so these tests compare against the whole of it.

use fallible_iterator::FallibleIterator;
use rekordcrate::device::{get_playlists, PlaylistNode, Track};
use rekordcrate::pdb::{Album, Artist, Genre, Key, Label, PlaylistEntry, RowVariant};
use rekordcrate::pdb::{Track as TrackRow, TrackId};
use rekordcrate::setting::{Setting, SettingType};
use rekordcrate::{DeviceExportLoader, DeviceExportWriter};
use std::path::{Path, PathBuf};

/// The export that Rekordbox writes for a device without any tracks on it.
const REFERENCE_EXPORT: &str = "data/complete_export/empty";

/// The date of the export that the reference was taken from.
const REFERENCE_DATE: &str = "2022-02-02";

/// Offset of the sequence number in a database header, which counts the edits made to the
/// database and so cannot match an export written by Rekordbox.
const SEQUENCE_OFFSET: usize = 0x14;

/// A directory that is removed again when it goes out of scope.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("rekordcrate-test-{}-{name}", std::process::id()));
        // Leftovers from a run that was interrupted would make the test fail for the wrong
        // reason, because an export is never written over.
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("failed to create temporary directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Returns the paths of all files in a directory, relative to it, in a stable order.
fn files_in(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(current) = dirs.pop() {
        for entry in std::fs::read_dir(&current).expect("failed to read directory") {
            let path = entry.expect("failed to read directory entry").path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                files.push(
                    path.strip_prefix(dir)
                        .expect("path is below the directory")
                        .to_path_buf(),
                );
            }
        }
    }
    files.sort();
    files
}

/// A written export must hold the same files as one written by Rekordbox.
#[test]
fn test_create_writes_the_files_of_an_export() {
    let dir = TempDir::new("files");
    DeviceExportWriter::create(dir.path(), REFERENCE_DATE)
        .expect("failed to create export")
        .close()
        .expect("failed to close export");

    assert_eq!(
        files_in(dir.path()),
        files_in(Path::new(REFERENCE_EXPORT)),
        "written export does not hold the files of an export"
    );
}

/// The databases of a written export must be the ones Rekordbox writes for an export without
/// tracks, apart from the sequence number that counts edits.
#[test]
fn test_create_writes_the_databases_of_an_export() {
    let dir = TempDir::new("databases");
    DeviceExportWriter::create(dir.path(), REFERENCE_DATE)
        .expect("failed to create export")
        .close()
        .expect("failed to close export");

    let layout = DeviceExportLoader::new(dir.path().to_path_buf()).layout();
    let reference = DeviceExportLoader::new(PathBuf::from(REFERENCE_EXPORT)).layout();

    for (written, expected) in [
        (layout.export_pdb(), reference.export_pdb()),
        (layout.export_ext_pdb(), reference.export_ext_pdb()),
    ] {
        let mut written_bytes = std::fs::read(&written).expect("failed to read written database");
        let expected_bytes = std::fs::read(&expected).expect("failed to read reference database");
        assert_eq!(
            written_bytes.len(),
            expected_bytes.len(),
            "{} has the wrong size",
            written.display()
        );

        // Take the sequence number out of the comparison rather than skipping the bytes, so that
        // a difference anywhere else is still reported.
        written_bytes[SEQUENCE_OFFSET..SEQUENCE_OFFSET + 4]
            .copy_from_slice(&expected_bytes[SEQUENCE_OFFSET..SEQUENCE_OFFSET + 4]);
        assert!(
            written_bytes == expected_bytes,
            "{} differs from the database Rekordbox writes, at page {:?}",
            written.display(),
            written_bytes
                .chunks(4096)
                .zip(expected_bytes.chunks(4096))
                .position(|(ours, theirs)| ours != theirs)
        );
    }
}

/// The settings of a written export must be the settings Rekordbox writes.
///
/// The files cannot be compared byte for byte, because they also record the version of the
/// software that wrote them.
#[test]
fn test_create_writes_the_settings_of_an_export() {
    let dir = TempDir::new("settings");
    DeviceExportWriter::create(dir.path(), REFERENCE_DATE)
        .expect("failed to create export")
        .close()
        .expect("failed to close export");

    let written = DeviceExportLoader::new(dir.path().to_path_buf()).load_settings();
    let expected = DeviceExportLoader::new(PathBuf::from(REFERENCE_EXPORT)).load_settings();
    assert_eq!(written, expected);

    // The brand is the one the player expects to find, so it is part of the file rather than of
    // the settings it holds.
    let layout = DeviceExportLoader::new(dir.path().to_path_buf()).layout();
    for setting_type in [
        SettingType::DevSetting,
        SettingType::DJMMySetting,
        SettingType::MySetting,
        SettingType::MySetting2,
    ] {
        let read = |path: PathBuf| {
            let mut file = std::fs::File::open(&path).expect("failed to open settings file");
            <Setting as binrw::BinRead>::read_args(&mut file, (setting_type,))
                .expect("failed to parse settings file")
        };
        let written = read(layout.setting_file(setting_type));
        let expected = read(
            Path::new(REFERENCE_EXPORT)
                .join("PIONEER")
                .join(setting_type.filename()),
        );
        assert_eq!(
            written.brand,
            expected.brand,
            "wrong brand in {}",
            setting_type.filename()
        );
        assert_eq!(
            written.data,
            expected.data,
            "wrong settings in {}",
            setting_type.filename()
        );
    }
}

/// An export must never be written over, so that one that took a long time to collect cannot be
/// destroyed by accident.
#[test]
fn test_create_does_not_overwrite_an_export() {
    let dir = TempDir::new("overwrite");
    DeviceExportWriter::create(dir.path(), REFERENCE_DATE)
        .expect("failed to create export")
        .close()
        .expect("failed to close export");
    let before = std::fs::read(
        DeviceExportLoader::new(dir.path().to_path_buf())
            .layout()
            .export_pdb(),
    )
    .expect("failed to read written database");

    let result = DeviceExportWriter::create(dir.path(), REFERENCE_DATE);

    assert!(result.is_err(), "an existing export was written over");
    let after = std::fs::read(
        DeviceExportLoader::new(dir.path().to_path_buf())
            .layout()
            .export_pdb(),
    )
    .expect("failed to read written database");
    assert_eq!(before, after, "the existing database was modified");
}

/// Counts the rows of a table in the export written to `dir`.
fn row_count<RowT: RowVariant>(dir: &Path) -> usize {
    let mut db = DeviceExportLoader::new(dir.to_path_buf())
        .open_pdb_non_persistent()
        .expect("failed to open database");
    db.iter_rows::<RowT>()
        .expect("failed to load rows")
        .count()
        .expect("failed to count rows")
}

/// A track with the fields that are worth checking set to something recognizable.
fn demo_track(path: &str, title: &str) -> Track {
    Track {
        file_path: path.to_owned(),
        title: title.to_owned(),
        artist: "Loopmasters".to_owned(),
        album: "Unknown Album".to_owned(),
        genre: "House".to_owned(),
        label: "Loopmasters".to_owned(),
        key: "Dm".to_owned(),
        comment: "Tracks by www.loopmasters.com".to_owned(),
        date_added: "2022-02-02".to_owned(),
        analyze_path: "/PIONEER/USBANLZ/P016/0000875E/ANLZ0000.DAT".to_owned(),
        analyze_date: "2022-02-02".to_owned(),
        tempo: 128.0,
        duration: 172,
        bitrate: 320,
        sample_rate: 44100,
        sample_depth: 16,
        file_size: 6_899_624,
        rating: 4,
        file_type: rekordcrate::util::FileType::Mp3,
        ..Default::default()
    }
}

/// An export takes more tracks than a `u16` could number in steps of 0x20, which is the form the
/// `index_shift` field of a row takes. The field counts the place a row has in its own page, so
/// the size of a table is not bound by it.
#[test]
fn test_add_track_past_a_table_wide_index_shift() {
    let dir = TempDir::new("many-tracks");
    let mut writer =
        DeviceExportWriter::create(dir.path(), REFERENCE_DATE).expect("failed to create export");

    // One more than the number of rows that 0x20 times a table-wide row index could number.
    let tracks = usize::from(u16::MAX / 0x20) + 2;
    for index in 0..tracks {
        let path = format!("/Contents/Demo Track {index}.mp3");
        let title = format!("Demo Track {index}");
        // Each track names its own artist and album, so those tables grow past the limit too.
        let mut track = demo_track(&path, &title);
        track.artist = format!("Artist {index}");
        track.album = format!("Album {index}");
        writer
            .add_track(&track)
            .unwrap_or_else(|error| panic!("track {index} of {tracks} was refused: {error}"));
    }
    writer.close().expect("failed to close export");

    assert_eq!(tracks, row_count::<TrackRow>(dir.path()));
    assert_eq!(tracks, row_count::<Artist>(dir.path()));
    assert_eq!(tracks, row_count::<Album>(dir.path()));
}

/// A tempo that cannot be counted in hundredths of a beat per minute is refused rather than
/// written as some other number.
#[test]
fn test_add_track_refuses_a_tempo_that_is_not_a_number() {
    let dir = TempDir::new("bad-tempo");
    let mut writer =
        DeviceExportWriter::create(dir.path(), REFERENCE_DATE).expect("failed to create export");
    for tempo in [f32::NAN, f32::INFINITY, -1.0, f32::MAX] {
        let mut track = demo_track("/Contents/Demo Track 1.mp3", "Demo Track 1");
        track.tempo = tempo;
        assert!(
            writer.add_track(&track).is_err(),
            "a tempo of {tempo} was accepted"
        );
    }
}

/// A track that was added must be readable from the export with the fields it was given.
#[test]
fn test_add_track_writes_the_track() {
    let dir = TempDir::new("add-track");
    let mut writer =
        DeviceExportWriter::create(dir.path(), REFERENCE_DATE).expect("failed to create export");
    let id = writer
        .add_track(&demo_track("/Contents/Demo Track 1.mp3", "Demo Track 1"))
        .expect("failed to add track");
    writer.close().expect("failed to close export");

    assert_eq!(id, TrackId(1));
    let mut db = DeviceExportLoader::new(dir.path().to_path_buf())
        .open_pdb_non_persistent()
        .expect("failed to open database");
    let tracks: Vec<TrackRow> = db
        .iter_rows::<TrackRow>()
        .expect("failed to load tracks")
        .map(|track| Ok(track.clone()))
        .collect()
        .expect("failed to read tracks");

    assert_eq!(tracks.len(), 1);
    let track = &tracks[0];
    assert_eq!(track.id, TrackId(1));
    assert_eq!(
        track.offsets.title.clone().into_string().unwrap(),
        "Demo Track 1"
    );
    assert_eq!(
        track.offsets.file_path.clone().into_string().unwrap(),
        "/Contents/Demo Track 1.mp3"
    );
    // The name of the file is held separately from the path it is at.
    assert_eq!(
        track.offsets.filename.clone().into_string().unwrap(),
        "Demo Track 1.mp3"
    );
    // The tempo is stored in hundredths of a beat per minute.
    assert_eq!(track.tempo, 12800);
    assert_eq!(track.duration, 172);
    assert_eq!(track.bitrate, 320);
    assert_eq!(track.sample_rate, 44100);
    assert_eq!(track.rating, 4);
    assert_eq!(track.file_size, 6_899_624);
}

/// The rows that tracks refer to by name must be shared between the tracks that refer to them.
#[test]
fn test_add_track_shares_the_rows_tracks_refer_to() {
    let dir = TempDir::new("share-rows");
    let mut writer =
        DeviceExportWriter::create(dir.path(), REFERENCE_DATE).expect("failed to create export");
    let first = writer
        .add_track(&demo_track("/Contents/One.mp3", "One"))
        .expect("failed to add track");
    let second = writer
        .add_track(&demo_track("/Contents/Two.mp3", "Two"))
        .expect("failed to add track");
    // A track with nothing but a path must not add empty rows for the metadata it lacks.
    writer
        .add_track(&Track {
            file_path: "/Contents/Three.mp3".to_owned(),
            ..Default::default()
        })
        .expect("failed to add track");
    writer.close().expect("failed to close export");

    assert_ne!(first, second);
    assert_eq!(row_count::<TrackRow>(dir.path()), 3);
    assert_eq!(row_count::<Artist>(dir.path()), 1);
    assert_eq!(row_count::<Album>(dir.path()), 1);
    assert_eq!(row_count::<Genre>(dir.path()), 1);
    assert_eq!(row_count::<Label>(dir.path()), 1);
    assert_eq!(row_count::<Key>(dir.path()), 1);
}

/// Adding the same audio file twice must not put it in the export twice.
#[test]
fn test_add_track_ignores_a_track_that_was_added_before() {
    let dir = TempDir::new("add-twice");
    let mut writer =
        DeviceExportWriter::create(dir.path(), REFERENCE_DATE).expect("failed to create export");
    let track = demo_track("/Contents/Demo Track 1.mp3", "Demo Track 1");
    let first = writer.add_track(&track).expect("failed to add track");
    let second = writer.add_track(&track).expect("failed to add track");
    writer.close().expect("failed to close export");

    assert_eq!(first, second);
    assert_eq!(row_count::<TrackRow>(dir.path()), 1);
}

/// Playlists and the folders holding them must be readable as the tree they were built as.
#[test]
fn test_playlists_are_written_as_a_tree() {
    let dir = TempDir::new("playlists");
    let mut writer =
        DeviceExportWriter::create(dir.path(), REFERENCE_DATE).expect("failed to create export");
    let track = writer
        .add_track(&demo_track("/Contents/Demo Track 1.mp3", "Demo Track 1"))
        .expect("failed to add track");
    let other = writer
        .add_track(&demo_track("/Contents/Demo Track 2.mp3", "Demo Track 2"))
        .expect("failed to add track");

    let folder = writer
        .create_playlist_folder(None, "Sets")
        .expect("failed to create folder");
    let playlist = writer
        .create_playlist(Some(folder), "Opening")
        .expect("failed to create playlist");
    let root_playlist = writer
        .create_playlist(None, "Everything")
        .expect("failed to create playlist");
    writer
        .add_track_to_playlist(playlist, track)
        .expect("failed to add track to playlist");
    writer
        .add_track_to_playlist(playlist, other)
        .expect("failed to add track to playlist");
    writer
        .add_track_to_playlist(root_playlist, track)
        .expect("failed to add track to playlist");
    writer.close().expect("failed to close export");

    let mut db = DeviceExportLoader::new(dir.path().to_path_buf())
        .open_pdb_non_persistent()
        .expect("failed to open database");
    let tree = get_playlists(&mut db).expect("failed to read playlists");

    let names: Vec<String> = tree
        .iter()
        .map(|node| match node {
            PlaylistNode::Folder(folder) => format!("folder {}", folder.name),
            PlaylistNode::Playlist(playlist) => format!("playlist {}", playlist.name),
        })
        .collect();
    assert_eq!(
        names,
        vec!["folder Sets".to_string(), "playlist Everything".to_string()]
    );
    let PlaylistNode::Folder(sets) = &tree[0] else {
        panic!("the first node should be a folder: {:?}", tree[0]);
    };
    assert_eq!(sets.children.len(), 1);
    let PlaylistNode::Playlist(opening) = &sets.children[0] else {
        panic!("the folder should hold a playlist: {:?}", sets.children[0]);
    };
    assert_eq!(opening.name, "Opening");

    // The tracks of a playlist are held in the order they were added, counted from one.
    let mut entries: Vec<(u32, u32, u32)> = db
        .iter_rows::<PlaylistEntry>()
        .expect("failed to load playlist entries")
        .map(|entry| Ok((entry.playlist_id.0, entry.entry_index, entry.track_id.0)))
        .collect()
        .expect("failed to read playlist entries");
    entries.sort();
    assert_eq!(
        entries,
        vec![
            (opening.id.0, 1, track.0),
            (opening.id.0, 2, other.0),
            (root_playlist.0, 1, track.0),
        ]
    );
}

/// A playlist that is not in the export cannot be added to.
#[test]
fn test_playlists_reject_an_unknown_id() {
    let dir = TempDir::new("playlist-id");
    let mut writer =
        DeviceExportWriter::create(dir.path(), REFERENCE_DATE).expect("failed to create export");
    let track = writer
        .add_track(&demo_track("/Contents/Demo Track 1.mp3", "Demo Track 1"))
        .expect("failed to add track");
    let unknown = rekordcrate::pdb::PlaylistTreeNodeId(42);

    assert!(writer.add_track_to_playlist(unknown, track).is_err());
    assert!(writer.create_playlist(Some(unknown), "Nested").is_err());
}
