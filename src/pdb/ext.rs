// Copyright (c) 2025 Nikolaus Einhauser <nikolaus.einhauser@web.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Parser for the extended Pioneer DeviceSQL database exports (exportExt.pdb).
//!
//! The Rekordbox DJ software uses writes PDB files to `/PIONEER/rekordbox/exportExt.pdb`.
//!
//! Most of the base file format has been reverse-engineered by Henry Betts, Fabian Lesniak and James
//! Elliott.
//! The exportExt specifics have been reversed engineered by Dominik Stolz (@voidc).
//!
//! - <https://github.com/Deep-Symmetry/crate-digger/blob/master/doc/Analysis.pdf>
//! - <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html>
//! - <https://github.com/henrybetts/Rekordbox-Decoding>
//! - <https://github.com/flesniak/python-prodj-link/tree/master/prodj/pdblib>

use crate::pdb::{DeviceSQLString, OffsetArray, OffsetArrayContainer, Subtype, TrackId};
use crate::util::ExplicitPadding;
use binrw::binrw;
use std::num::NonZero;

/// A unique identifier for a tag or category.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
pub struct TagId(pub u32);

#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[brw(little)]
/// A possibly absent parent ID. If the ID is zero (None), then there is no parent.
pub struct ParentId(
    #[br(try)] // failing to parse is fine, since then its just non-zero
    #[bw(map = |&x| x.map_or(0, |v| v.get()))]
    pub Option<NonZero<u32>>,
);

#[binrw]
#[brw(little)]
#[brw(import(base: i64, offsets: &OffsetArray<3>, args: ()))]
#[derive(Debug, PartialEq, Clone, Eq)]
/// The strings associated with a tag or category.
pub struct TagOrCategoryStrings {
    #[brw(args(base, args))]
    #[br(parse_with = offsets.read_offset(1))]
    #[bw(write_with = offsets.write_offset(1))]
    /// The name of the tag or category.
    pub name: DeviceSQLString,
    #[brw(args(base, args))]
    #[br(parse_with = offsets.read_offset(2))]
    #[bw(write_with = offsets.write_offset(2))]
    /// String with unknown purpose, often empty.
    pub unknown: DeviceSQLString,
}

/// A tag or category that can be assigned to tracks for the purpose of categorization.
// https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html#tag-rows
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct TagOrCategory {
    /// Determines if an 8-bit offset (0x0680) or a 16-bit offset (0x0684) is used for the strings.
    pub subtype: Subtype,
    // also called tag_index. Seems to increment by 0x20 every row.
    /// Incrementing index (0x20 for each row).
    pub index_shift: u16,
    // no idea what these two do, but they aren't always zero
    // as described on https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html#tag-rows
    /// Unknown purpose.
    pub unknown1: u32,
    /// Unknown purpose.
    pub unknown2: u32,
    /// The ID of the parent category, if any.
    pub parent_id: ParentId,
    /// Zero-based position at which this tag should be displayed within its category.
    /// If the row represents a category rather than a tag, then this is the zero-based
    /// position of the category itself within the category list.
    pub position: u32,
    /// Numeric ID of the tag or category.
    pub id: TagId,
    /// Non-zero if this row represents a category rather than a tag.
    pub raw_is_category: u32,
    #[brw(args(0x1C, subtype.get_offset_size(), ()))]
    /// The strings associated with this tag or category.
    pub offsets: OffsetArrayContainer<TagOrCategoryStrings, 3>,
    #[br(args(0x20))]
    /// Padding at the end of the struct (observed 11 bytes for this rows)
    pub padding: ExplicitPadding,
}

// https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html#tag-track-rows
/// M*N junction table between tags and tracks.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct TrackTag {
    #[brw(magic(0u32))]
    /// The ID of the track.
    pub track_id: TrackId,
    /// The ID of the tag.
    pub tag_id: TagId,
    /// Unknown purpose, seems to be always 3.
    pub unknown_const: u32, // always 3?
}

/// The type of ext pages found inside a `Table`.
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExtPageType {
    /// can be assigned to tracks for the purpose of categorization.
    #[brw(magic = 3u32)]
    Tag,
    /// holds the associations between tag ids and track ids.
    #[brw(magic = 4u32)]
    TrackTag,
}

/// A table row contains the actual data.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
// #[br(import(page_type: PageType))]
// The large enum size is unfortunate, but since users of this library will probably use iterators
// to consume the results on demand, we can live with this. The alternative of using a `Box` would
// require a heap allocation per row, which is arguably worse. Hence, the warning is disabled for
// this enum.
#[br(import(page_type: ExtPageType))]
pub enum ExtRow {
    /// Contains the album name, along with an ID of the corresponding artist.
    #[br(pre_assert(page_type == ExtPageType::Tag))]
    Tag(TagOrCategory),
    /// Contains the artist name and ID.
    #[br(pre_assert(page_type == ExtPageType::TrackTag))]
    TrackTag(TrackTag),
}

impl ExtRow {
    #[must_use]
    pub(crate) const fn align_by(&self, offset: u64) -> u64 {
        use crate::util::align_by;
        use std::mem::align_of_val;
        match self {
            ExtRow::Tag(_) => align_by(4, offset),
            ExtRow::TrackTag(r) => align_by(align_of_val(r) as u64, offset),
        }
    }
}
