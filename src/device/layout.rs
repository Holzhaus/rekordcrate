// Copyright (c) 2026 Jan Holthaus <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! On-disk layout of a Rekordbox device export: where `export.pdb`, the `*SETTING.DAT` files,
//! and the `PIONEER`/`USBANLZ`/`Contents` directories live relative to the device root. Shared by
//! [`crate::device::DeviceExportReader`] and [`crate::device::DeviceExportWriter`].

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
/// outside the high-level reader/writer.
#[derive(Debug, Clone, PartialEq)]
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

    /// The `PIONEER/USBANLZ` directory holding per-track analysis files.
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

    /// The `PIONEER/Artwork` directory (under the `artwork` feature).
    #[cfg(feature = "artwork")]
    #[must_use]
    pub fn artwork_dir(&self) -> PathBuf {
        self.pioneer_dir().join("Artwork")
    }

    /// Path to the 80×80 thumbnail `a{id}.jpg` (under the `artwork` feature).
    #[cfg(feature = "artwork")]
    #[must_use]
    pub fn artwork_file(&self, id: u32) -> PathBuf {
        self.artwork_dir()
            .join(artwork_folder(id))
            .join(format!("a{id}.jpg"))
    }

    /// Path to the 240×240 `a{id}_m.jpg` (under the `artwork` feature).
    #[cfg(feature = "artwork")]
    #[must_use]
    pub fn artwork_m_file(&self, id: u32) -> PathBuf {
        self.artwork_dir()
            .join(artwork_folder(id))
            .join(format!("a{id}_m.jpg"))
    }
}

/// Five-digit shard folder name for artwork `id`: `id/20 + 1`, zero-padded (under the `artwork`
/// feature).
#[cfg(feature = "artwork")]
#[must_use]
pub fn artwork_folder(id: u32) -> String {
    format!("{:05}", id / 20 + 1)
}
