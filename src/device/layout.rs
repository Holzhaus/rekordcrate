// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! On-disk layout of a Rekordbox device export: where `export.pdb`, `exportExt.pdb`, the
//! `SETTING.DAT` files and the `PIONEER`/`USBANLZ`/`Contents` directories live relative
//! to the device root. Used by [`crate::device::DeviceExport`].

use crate::setting::SettingType;
use std::path::{Path, PathBuf};

/// The `*SETTING.DAT` files in a device export, in the order Rekordbox writes them.
pub const DAT_FILES: &[(&str, SettingType)] = &[
    ("DEVSETTING.DAT", SettingType::DevSetting),
    ("DJMMYSETTING.DAT", SettingType::DJMMySetting),
    ("MYSETTING.DAT", SettingType::MySetting),
    ("MYSETTING2.DAT", SettingType::MySetting2),
];

/// On-disk layout of a device export rooted at `root`. Derives all paths from it on demand.
///
/// Exposed so expert callers can locate [`Self::export_pdb`] / [`Self::export_ext_pdb`] and the
/// surrounding `PIONEER` / `Contents` directories directly, for manual inspection or modification
/// outside the high-level [`crate::device::DeviceExport`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    root: PathBuf,
}

impl Layout {
    /// Wrap a device-export root directory.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// The device root directory this layout was built from.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The `PIONEER` directory.
    #[must_use]
    pub fn pioneer_dir(&self) -> PathBuf {
        self.root.join("PIONEER")
    }

    /// The `PIONEER/rekordbox` directory holding the PDB files.
    #[must_use]
    pub fn rekordbox_dir(&self) -> PathBuf {
        self.pioneer_dir().join("rekordbox")
    }

    /// Path to `export.pdb`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rekordcrate::device::layout::Layout;
    /// let layout = Layout::new("/srv/export".into());
    /// assert!(layout.export_pdb().ends_with("export.pdb"));
    /// ```
    #[must_use]
    pub fn export_pdb(&self) -> PathBuf {
        self.rekordbox_dir().join("export.pdb")
    }

    /// Path to `exportExt.pdb`.
    #[must_use]
    pub fn export_ext_pdb(&self) -> PathBuf {
        self.rekordbox_dir().join("exportExt.pdb")
    }

    /// The `PIONEER/USBANLZ` directory holding per-track analysis files, each track's in the
    /// subfolder [`anlz_folder`] names.
    #[must_use]
    pub fn usbanlz_dir(&self) -> PathBuf {
        self.pioneer_dir().join("USBANLZ")
    }

    /// The `Contents` directory holding audio files.
    #[must_use]
    pub fn contents_dir(&self) -> PathBuf {
        self.root.join("Contents")
    }

    /// Path to a `*SETTING.DAT` file by name, under `PIONEER`.
    #[must_use]
    pub fn dat_path(&self, filename: &str) -> PathBuf {
        self.pioneer_dir().join(filename)
    }

    /// The `PIONEER/Artwork` directory holding track artwork.
    #[must_use]
    pub fn artwork_dir(&self) -> PathBuf {
        self.pioneer_dir().join("Artwork")
    }

    /// Path to the 80×80 thumbnail `a{id}.jpg` of artwork `id`, the file an `Artwork` row names.
    #[must_use]
    pub fn artwork_file(&self, id: u32) -> PathBuf {
        self.artwork_dir()
            .join(artwork_folder(id))
            .join(format!("a{id}.jpg"))
    }

    /// Path to the 240×240 `a{id}_m.jpg` of artwork `id`, which players look for next to the
    /// thumbnail.
    #[must_use]
    pub fn artwork_m_file(&self, id: u32) -> PathBuf {
        self.artwork_dir()
            .join(artwork_folder(id))
            .join(format!("a{id}_m.jpg"))
    }
}

/// Five-digit folder name Rekordbox files artwork `id` under: `id / 20 + 1`, zero-padded.
#[must_use]
pub fn artwork_folder(id: u32) -> String {
    format!("{:05}", id / 20 + 1)
}

const ANLZ_HASH_FIRST_MULTIPLIER: u32 = 0x5BC9;
const ANLZ_HASH_SECOND_MULTIPLIER: u32 = 0x93B5;
const ANLZ_HASH_MODULUS: u32 = 200_003;

/// Folder where Rekordbox puts the analysis files of a track (`ANLZ0000.DAT`, `.EXT` and `.2EX`),
/// relative to `PIONEER/USBANLZ`.
///
/// `audio_path` must be the absolute device path stored for the track, including its leading slash,
/// for example `/Contents/Artist/Album/Track.flac`. Case, separators, and Unicode are significant;
/// this function deliberately performs no path normalization. The returned path always has the
/// form `Pxxx/yyyyyyyy`, using uppercase, zero-padded hexadecimal components.
///
/// The path is processed as UTF-16 code units by a custom rolling hash, reduced modulo 200003,
/// which is the second component. Selected bits of that hash form the first.
///
/// Players have been seen to recompute this folder from the audio path instead of relying on the
/// `analyze_path` stored in `export.pdb` (confirmed on the XDJ-XZ), so analysis files have to be
/// placed here to be found.
///
/// The algorithm was contributed by AnnoyingTechnology from [interoperability research][research]
/// published in December 2025.
///
/// [research]: https://github.com/AnnoyingTechnology/rhythmbox-to-pioneer-xdj-exporter/blob/master/posterity/WAVEFORMS.md
#[must_use]
pub fn anlz_folder(audio_path: &str) -> String {
    let mut hash = 0u32;
    for code_unit in audio_path.encode_utf16().map(u32::from) {
        let intermediate = hash
            .wrapping_mul(ANLZ_HASH_FIRST_MULTIPLIER)
            .wrapping_add(code_unit);
        hash = intermediate
            .wrapping_mul(ANLZ_HASH_SECOND_MULTIPLIER)
            .wrapping_add(code_unit);
    }

    let hash = hash % ANLZ_HASH_MODULUS;
    let p_value = (hash & 0x01)
        | ((hash >> 1) & 0x02)
        | ((hash >> 4) & 0x04)
        | ((hash >> 4) & 0x08)
        | ((hash >> 5) & 0x10)
        | ((hash >> 8) & 0x20)
        | ((hash >> 10) & 0x40);

    format!("P{p_value:03X}/{hash:08X}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdb::io::Database;
    use crate::pdb::{DatabaseType, Track};
    use fallible_iterator::FallibleIterator;
    use std::fs::File;

    /// Every track of an export Rekordbox wrote names the analysis file that follows from its
    /// audio path, and the export holds the two analysis folders it comes with.
    #[test]
    fn anlz_paths_match_a_rekordbox_export() {
        let layout = Layout::new("data/complete_export/demo_tracks".into());
        let mut db = Database::open_non_persistent(
            File::open(layout.export_pdb()).unwrap(),
            DatabaseType::Plain,
        )
        .unwrap();
        let paths: Vec<(String, String)> = db
            .iter_rows::<Track>()
            .unwrap()
            .map(|track| {
                let strings = &track.offsets.inner;
                Ok((
                    strings.file_path.clone().into_string()?,
                    strings.analyze_path.clone().into_string()?,
                ))
            })
            .collect()
            .unwrap();
        assert!(!paths.is_empty());

        for (audio_path, analyze_path) in paths {
            assert_eq!(
                format!("/PIONEER/USBANLZ/{}/ANLZ0000.DAT", anlz_folder(&audio_path)),
                analyze_path,
                "unexpected analysis path for {audio_path}"
            );
            for extension in ["DAT", "EXT", "2EX"] {
                assert!(
                    layout
                        .usbanlz_dir()
                        .join(anlz_folder(&audio_path))
                        .join(format!("ANLZ0000.{extension}"))
                        .is_file(),
                    "no {extension} analysis file for {audio_path}"
                );
            }
        }
    }

    /// Paths and folders taken from Rekordbox exports by AnnoyingTechnology.
    #[test]
    fn anlz_folder_matches_rekordbox_exports() {
        let cases = [
            (
                "/Contents/ARTISTTEST1/ALBUMTEST1/TITLETEST1.mp3",
                "P051/0001D603",
            ),
            (
                "/Contents/ARTISTTEST2/ALBUMTEST2/TITLETEST2.mp3",
                "P03C/0000A6CA",
            ),
            (
                "/Contents/ARTISTTEST3/ALBUMTEST3/TITLETEST3.mp3",
                "P045/0001096B",
            ),
        ];

        for (audio_path, expected) in cases {
            assert_eq!(
                anlz_folder(audio_path),
                expected,
                "unexpected folder for {audio_path}"
            );
        }
    }

    #[test]
    fn anlz_folder_takes_surrogate_pairs_as_two_code_units() {
        assert_eq!(anlz_folder("/Contents/Artist/🚀.flac"), "P074/00012258");
    }
}
