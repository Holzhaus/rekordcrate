// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Tests for working out where the analysis files of a track belong.

use fallible_iterator::FallibleIterator;
use rekordcrate::anlz::AnalysisPath;
use rekordcrate::pdb::io::Database;
use rekordcrate::pdb::{DatabaseType, Track};
use std::path::{Path, PathBuf};

/// An export written by Rekordbox, whose tracks hold the path of their analysis files.
const REFERENCE_EXPORT: &str = "data/complete_export/demo_tracks";

/// The directory worked out for a track must be the one Rekordbox put its analysis files in.
///
/// Every track of the export is checked against the path that its own row holds, so this needs
/// nothing to be stated here that the fixture does not already say.
#[test]
fn test_analysis_path_matches_the_paths_rekordbox_wrote() {
    let path = Path::new(REFERENCE_EXPORT)
        .join("PIONEER")
        .join("rekordbox")
        .join("export.pdb");
    let file = std::fs::File::open(path).expect("failed to open database");
    let mut db =
        Database::open_non_persistent(file, DatabaseType::Plain).expect("failed to open database");

    let tracks: Vec<(String, String)> = db
        .iter_rows::<Track>()
        .expect("failed to load tracks")
        .map(|track| {
            Ok((
                track.offsets.file_path.clone().into_string()?,
                track.offsets.analyze_path.clone().into_string()?,
            ))
        })
        .collect()
        .expect("failed to read tracks");
    assert!(!tracks.is_empty(), "the fixture should hold tracks");

    for (file_path, analyze_path) in tracks {
        let analysis = AnalysisPath::for_audio_path(&file_path);
        let expected = Path::new(&analyze_path)
            .parent()
            .expect("the analysis path should name a file in a directory")
            .strip_prefix("/PIONEER/USBANLZ")
            .expect("the analysis path should be below the USBANLZ directory");

        assert_eq!(
            analysis.relative_dir(),
            expected,
            "wrong analysis directory for {file_path}"
        );
        // The file itself is the one the track row names, and it is really there.
        assert_eq!(
            PathBuf::from("/PIONEER/USBANLZ")
                .join(analysis.relative_dir())
                .join(format!("{}.DAT", AnalysisPath::FILE_STEM)),
            Path::new(&analyze_path)
        );
        assert!(
            Path::new(REFERENCE_EXPORT)
                .join(analyze_path.trim_start_matches('/'))
                .is_file(),
            "the export should hold the analysis file of {file_path}"
        );
    }
}

/// The directory must match the paths that were taken off hardware by the research this follows.
///
/// These two are from the notes linked in the documentation of `AnalysisPath`, and were checked
/// against an XDJ-XZ rather than against an export, so they cover the claim that players work
/// the directory out for themselves.
#[test]
fn test_analysis_path_matches_the_published_paths() {
    for (audio_path, group, hash) in [
        (
            "/Contents/ARTISTTEST1/ALBUMTEST1/TITLETEST1.mp3",
            0x051,
            0x0001_D603,
        ),
        (
            "/Contents/ARTISTTEST2/ALBUMTEST2/TITLETEST2.mp3",
            0x03C,
            0x0000_A6CA,
        ),
    ] {
        let analysis = AnalysisPath::for_audio_path(audio_path);
        assert_eq!(analysis.group(), group, "wrong group for {audio_path}");
        assert_eq!(analysis.hash_value(), hash, "wrong hash for {audio_path}");
        assert_eq!(
            analysis.relative_dir(),
            PathBuf::from(format!("P{group:03X}")).join(format!("{hash:08X}")),
            "wrong directory for {audio_path}"
        );
    }
}

/// The path is worked out from the whole path of the audio file, so tracks that differ anywhere
/// in it end up apart.
#[test]
fn test_analysis_path_follows_the_whole_audio_path() {
    let paths = [
        "/Contents/Artist/Album/Track.mp3",
        "/Contents/Artist/Album/track.mp3",
        "/Contents/Artist/Other/Track.mp3",
        "/Contents/Artist/Album/Track.flac",
    ];
    let dirs: Vec<PathBuf> = paths
        .iter()
        .map(|path| AnalysisPath::for_audio_path(path).relative_dir())
        .collect();

    for (index, dir) in dirs.iter().enumerate() {
        assert!(
            !dirs[index + 1..].contains(dir),
            "{} shares a directory with another path: {}",
            paths[index],
            dir.display()
        );
    }
    // The same path always gives the same answer.
    assert_eq!(
        AnalysisPath::for_audio_path(paths[0]),
        AnalysisPath::for_audio_path(paths[0])
    );
}

/// A path with characters outside ASCII must be hashed as the UTF-16 the players read.
#[test]
fn test_analysis_path_handles_characters_outside_ascii() {
    // Nothing is known to compare these against, so this only holds them to being worked out at
    // all and kept apart, which hashing the bytes of the path would not do.
    let first = AnalysisPath::for_audio_path("/Contents/Björk/Homogenic/Jóga.mp3");
    let second = AnalysisPath::for_audio_path("/Contents/Bjork/Homogenic/Joga.mp3");
    assert_ne!(first, second);
    assert!(first.group() <= 0x7F, "the group should fit in seven bits");
}
