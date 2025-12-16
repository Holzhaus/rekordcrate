// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
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
