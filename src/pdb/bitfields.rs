// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Packed fields/bitfields used in PDB files.

// This is necessary since `bitfield` generates methods that otherwise violate this lint.
#![allow(clippy::must_use_candidate)]

use binrw::{BinRead, BinWrite};
use modular_bitfield::prelude::*;

use crate::pdb::RowGroup;

/// Packed field found in the page header containing:
/// - number of used row offsets in the page (13 bits).
/// - number of valid rows in the page (11 bits).
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(little, map = Self::from_bytes)]
#[bw(little, map = |x: &PackedRowCounts| x.into_bytes())]
pub struct PackedRowCounts {
    pub num_rows: B13,
    pub num_rows_valid: B11,
}

impl Default for PackedRowCounts {
    fn default() -> Self {
        Self::new()
    }
}

impl PackedRowCounts {
    /// Create a `PackedRowCounts` assuming all rows in the page are valid,
    /// e.g. when we are serializing a page without any deleted rows.
    pub fn from_all_valid(num_rows: usize) -> Self {
        Self::new()
            .with_num_rows(num_rows as u16)
            .with_num_rows_valid(num_rows as u16)
    }

    /// Get the number of row groups in the page.
    pub(crate) fn num_row_groups(&self) -> u16 {
        self.num_rows().div_ceil(RowGroup::MAX_ROW_COUNT as u16)
    }
}

/// Page flags stored in the page header.
///
/// Note that `modular-bitfield` stores the bits in LSB-first order so the
/// bitfield definition is reversed compared to typical notation.
#[bitfield(bits = 8)]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(little, map = Self::from_bytes)]
#[bw(little, map = |x: &Self| x.into_bytes())]
pub struct PageFlags {
    /// Unknown flag that appears to never be set.
    pub unknown0: bool,
    /// Unknown flag that appears to never be set.
    pub unknown1: bool,
    /// Unknown flag that appears to always be set.
    pub unknown2: bool,
    /// Unknown flag that appears to never be set.
    pub unknown3: bool,
    /// Flag set when the page contains a deleted row.
    pub contains_deleted: bool,
    /// Unknown flag that appears to always be set.
    pub unknown5: bool,
    /// Determines if the page is an index page.
    pub is_index_page: bool,
    /// Unknown flag that appears to never be set.
    pub unknown7: bool,
}

impl Default for PageFlags {
    fn default() -> Self {
        Self::new()
            .with_unknown0(false)
            .with_unknown1(false)
            .with_unknown2(true)
            .with_unknown3(false)
            .with_contains_deleted(false)
            .with_unknown5(true)
            .with_is_index_page(false)
            .with_unknown7(false)
    }
}

impl PageFlags {
    /// Create a `PageFlags` for a typical data page.
    pub fn new_data_page() -> Self {
        Self::default().with_is_index_page(false)
    }

    /// Create a `PageFlags` for a typical index page.
    pub fn new_index_page() -> Self {
        Self::default().with_is_index_page(true)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_page_flags_index() {
        let flags = PageFlags::new_index_page();
        assert_eq!(flags.into_bytes(), [0x64]);
    }

    #[test]
    fn test_page_flags_data() {
        let mut flags = PageFlags::new_data_page();
        assert_eq!(flags.into_bytes(), [0x24]);

        flags.set_contains_deleted(true);
        assert_eq!(flags.into_bytes(), [0x34]);
    }
}
