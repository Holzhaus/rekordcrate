// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! High-level API for Rekordbox device exports.
//!
//! A device export is a directory tree holding `PIONEER/` (the `*SETTING.DAT` files and
//! `rekordbox/export.pdb`), `PIONEER/USBANLZ/` (per-track analysis files), and `Contents/` (audio
//! files). The shared on-disk layout lives in [`layout`].
//!
//! [`DeviceExportReader`] inspects an existing export.

pub mod layout;
pub mod reader;

pub use crate::device::reader::DeviceExportReader;
pub use crate::device::reader::{get_playlists, Playlist, PlaylistFolder, PlaylistNode};
