// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
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
pub(crate) const DAT_FILES: &[(&str, SettingType)] = &[
    ("DEVSETTING.DAT", SettingType::DevSetting),
    ("DJMMYSETTING.DAT", SettingType::DJMMySetting),
    ("MYSETTING.DAT", SettingType::MySetting),
    ("MYSETTING2.DAT", SettingType::MySetting2),
];

/// On-disk layout of a device export rooted at `root`. Derives all paths from it on demand.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Layout {
    root: PathBuf,
}

impl Layout {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn pioneer_dir(&self) -> PathBuf {
        self.root.join("PIONEER")
    }

    pub(crate) fn rekordbox_dir(&self) -> PathBuf {
        self.pioneer_dir().join("rekordbox")
    }

    pub(crate) fn export_pdb(&self) -> PathBuf {
        self.rekordbox_dir().join("export.pdb")
    }

    pub(crate) fn dat_path(&self, filename: &str) -> PathBuf {
        self.pioneer_dir().join(filename)
    }
}
