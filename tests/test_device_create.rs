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
