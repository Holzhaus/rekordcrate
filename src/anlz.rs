// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Parser for Rekordbox analysis files, that can be found inside nested subdirectories of the
//! `PIONEER/USBANLZ` directory and can have the extensions `.DAT`, `.EXT` or `.2EX`. Note that
//! these files are not only used for device exports, but also for local rekordbox databases. In
//! that case, the directory can be found at `%APPDATA%\Pioneer\rekordbox\share\PIONEER\USBANLZ`.
//!
//! These files contain additional data (such as beatgrids, hotcues, waveforms and song structure
//! information) that is not part of the PDB file.
//!
//! The file is divided in section, where each section consists of a tag, header, and content.
//!
//! With the evolution of the Pioneer hardware line, new section types were added (e.g.
//! for high-resolution colored waveforms). To avoid issues with older hardware that cannot handle
//! the additional data due to their memory limitations, the new sections were only added to a copy
//! of the original file (`.DAT`) and saved with another extension (`.EXT`).
//!
//! - <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html>
//! - <https://reverseengineering.stackexchange.com/questions/4311/help-reversing-a-edb-database-file-for-pioneers-rekordbox-software>

#![allow(clippy::must_use_candidate)]

use crate::util::ColorIndex;
use binrw::{
    binrw,
    io::{Read, Seek},
    BinRead, BinResult, BinWrite, NullWideString, ReadOptions,
};
use modular_bitfield::prelude::*;

/// The kind of section.
#[derive(Debug, PartialEq, Clone)]
#[binrw]
#[brw(big)]
pub enum ContentKind {
    /// File section that contains all other sections.
    #[brw(magic = b"PMAI")]
    File,
    /// All beats found in the track.
    #[brw(magic = b"PQTZ")]
    BeatGrid,
    /// Either memory points and loops or hotcues and hot loops of the track.
    ///
    /// *Note:* Since the release of the Nexus 2 series, there also exists the `ExtendedCueList`
    /// section which can carry additional information.
    #[brw(magic = b"PCOB")]
    CueList,
    /// Extended version of the `CueList` section (since Nexus 2 series).
    #[brw(magic = b"PCO2")]
    ExtendedCueList,
    /// Single cue entry inside a `ExtendedCueList` section.
    #[brw(magic = b"PCP2")]
    ExtendedCue,
    /// Single cue entry inside a `CueList` section.
    #[brw(magic = b"PCPT")]
    Cue,
    /// File path of the audio file.
    #[brw(magic = b"PPTH")]
    Path,
    /// Seek information for variable bitrate files.
    #[brw(magic = b"PVBR")]
    VBR,
    /// Fixed-width monochrome preview of the track waveform.
    #[brw(magic = b"PWAV")]
    WaveformPreview,
    /// Smaller version of the fixed-width monochrome preview of the track waveform (for the
    /// CDJ-900).
    #[brw(magic = b"PWV2")]
    TinyWaveformPreview,
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PWV3")]
    WaveformDetail,
    /// Fixed-width colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PWV4")]
    WaveformColorPreview,
    /// Variable-width large colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PWV5")]
    WaveformColorDetail,
    /// Describes the structure of a sond (Intro, Chrous, Verse, etc.).
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PSSI")]
    SongStructure,
    /// Unknown Kind.
    ///
    /// This allows handling files that contain unknown section types and allows to access later
    /// sections in the file that have a known type instead of failing to parse the whole file.
    Unknown([u8; 4]),
}

/// Header of a section that contains type and size information.
#[derive(Debug, PartialEq, Clone)]
#[binrw]
#[brw(big)]
pub struct Header {
    /// Kind of content in this item.
    pub kind: ContentKind,
    /// Length of the header data (including `kind`, `size` and `total_size`).
    pub size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
}

impl Header {
    fn remaining_size(&self) -> u32 {
        self.size - 12
    }

    fn content_size(&self) -> u32 {
        self.total_size - self.size
    }
}

/// A single beat inside the beat grid.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(big)]
pub struct Beat {
    /// Beat number inside the bar (1-4).
    pub beat_number: u16,
    /// Current tempo in centi-BPM (= 1/100 BPM).
    pub tempo: u16,
    /// Time in milliseconds after which this beat would occur (at normal playback speed).
    pub time: u32,
}

/// Describes the types of entries found in a Cue List section.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(big, repr = u32)]
pub enum CueListType {
    /// Memory cues or loops.
    MemoryCues = 0,
    /// Hot cues or loops.
    HotCues = 1,
}

/// Indicates if the cue is point or a loop.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum CueType {
    /// Cue is a single point.
    Point = 0,
    /// Cue is a loop.
    Loop = 2,
}

/// A memory or hot cue (or loop).
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(big)]
pub struct Cue {
    /// Cue entry header.
    pub header: Header,
    /// Hot cue number.
    /// | Value | Hot cue        |
    /// | ----- | -------------- |
    /// |     0 | Not a hot cue. |
    /// |     1 | A              |
    /// |     2 | B              |
    /// | ...   | ...            |
    pub hot_cue: u32,
    /// Loop status. `4` if this cue is an active loop, `0` otherwise.
    pub status: u32,
    /// Unknown field. Seems to always have the value `0x00100000`.
    pub unknown1: u32,
    /// Somehow used for sorting cues.
    ///
    /// | Value    | Cue    |
    /// | -------- | ------ |
    /// | `0xFFFF` | 1      |
    /// | `0x0000` | 2      |
    /// | `0x0002` | 3      |
    /// | `0x0003` | 4      |
    /// | ...      | ...    |
    ///
    /// It is unknown why both `order_first` and `order_last` exist, when on of those values should
    /// suffice.
    pub order_first: u16,
    /// Somehow used for sorting cues.
    ///
    /// | Value    | Cue    |
    /// | -------- | ------ |
    /// | `0x0001` | 1      |
    /// | `0x0002` | 2      |
    /// | `0x0003` | 3      |
    /// | `0x0004` | 4      |
    /// | ...      | ...    |
    /// | `0xFFFF` | *last* |
    ///
    /// It is unknown why both `order_first` and `order_last` exist, when on of those values should
    /// suffice.
    pub order_last: u16,
    /// Type of this cue (`2` if this cue is a loop).
    pub cue_type: CueType,
    /// Unknown field. Seems always have the value `0`.
    pub unknown2: u8,
    /// Unknown field. Seems always have the value `0x03E8` (= decimal 1000).
    pub unknown3: u16,
    /// Time in milliseconds after which this cue would occur (at normal playback speed).
    pub time: u32,
    /// Time in milliseconds after which this the loop would jump back to `time` (at normal playback speed).
    pub loop_time: u32,
    /// Unknown field.
    pub unknown4: u32,
    /// Unknown field.
    pub unknown5: u32,
    /// Unknown field.
    pub unknown6: u32,
    /// Unknown field.
    pub unknown7: u32,
}

/// A memory or hot cue (or loop).
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(big)]
pub struct ExtendedCue {
    /// Cue entry header.
    pub header: Header,
    /// Hot cue number.
    ///
    /// | Value | Hot cue        |
    /// | ----- | -------------- |
    /// |     0 | Not a hot cue. |
    /// |     1 | A              |
    /// |     2 | B              |
    /// | ...   | ...            |
    pub hot_cue: u32,
    /// Type of this cue (`2` if this cue is a loop).
    pub cue_type: CueType,
    /// Unknown field. Seems always have the value `0`.
    pub unknown1: u8,
    /// Unknown field. Seems always have the value `0x03E8` (= decimal 1000).
    pub unknown2: u16,
    /// Time in milliseconds after which this cue would occur (at normal playback speed).
    pub time: u32,
    /// Time in milliseconds after which this the loop would jump back to `time` (at normal playback speed).
    pub loop_time: u32,
    /// Color assigned to this cue.
    ///
    /// Only used by memory cues, hot cues use a different value (see below).
    pub color: ColorIndex,
    /// Unknown field.
    pub unknown3: u8,
    /// Unknown field.
    pub unknown4: u16,
    /// Unknown field.
    pub unknown5: u32,
    /// Represents the loop size numerator (if this is a quantized loop).
    pub loop_numerator: u16,
    /// Represents the loop size denominator (if this is a quantized loop).
    pub loop_denominator: u16,
    /// Length of the comment string in bytes.
    pub len_comment: u32,
    /// An UTF-16BE encoded string, followed by a trailing  `0x0000`.
    #[br(assert((comment.len() as u32 + 1) * 2 == len_comment))]
    pub comment: NullWideString,
    /// Rekordbox hotcue color index.
    ///
    /// | Value  | Color                       |
    /// | ------ | --------------------------- |
    /// | `0x00` | None (Green on older CDJs). |
    /// | `0x01` | `#305aff`                   |
    /// | `0x02` | `#5073ff`                   |
    /// | `0x03` | `#508cff`                   |
    /// | `0x04` | `#50a0ff`                   |
    /// | `0x05` | `#50b4ff`                   |
    /// | `0x06` | `#50b0f2`                   |
    /// | `0x07` | `#50aee8`                   |
    /// | `0x08` | `#45acdb`                   |
    /// | `0x09` | `#00e0ff`                   |
    /// | `0x0a` | `#19daf0`                   |
    /// | `0x0b` | `#32d2e6`                   |
    /// | `0x0c` | `#21b4b9`                   |
    /// | `0x0d` | `#20aaa0`                   |
    /// | `0x0e` | `#1fa392`                   |
    /// | `0x0f` | `#19a08c`                   |
    /// | `0x10` | `#14a584`                   |
    /// | `0x11` | `#14aa7d`                   |
    /// | `0x12` | `#10b176`                   |
    /// | `0x13` | `#30d26e`                   |
    /// | `0x14` | `#37de5a`                   |
    /// | `0x15` | `#3ceb50`                   |
    /// | `0x16` | `#28e214`                   |
    /// | `0x17` | `#7dc13d`                   |
    /// | `0x18` | `#8cc832`                   |
    /// | `0x19` | `#9bd723`                   |
    /// | `0x1a` | `#a5e116`                   |
    /// | `0x1b` | `#a5dc0a`                   |
    /// | `0x1c` | `#aad208`                   |
    /// | `0x1d` | `#b4c805`                   |
    /// | `0x1e` | `#b4be04`                   |
    /// | `0x1f` | `#bab404`                   |
    /// | `0x20` | `#c3af04`                   |
    /// | `0x21` | `#e1aa00`                   |
    /// | `0x22` | `#ffa000`                   |
    /// | `0x23` | `#ff9600`                   |
    /// | `0x24` | `#ff8c00`                   |
    /// | `0x25` | `#ff7500`                   |
    /// | `0x26` | `#e0641b`                   |
    /// | `0x27` | `#e0461e`                   |
    /// | `0x28` | `#e0301e`                   |
    /// | `0x29` | `#e02823`                   |
    /// | `0x2a` | `#e62828`                   |
    /// | `0x2b` | `#ff376f`                   |
    /// | `0x2c` | `#ff2d6f`                   |
    /// | `0x2d` | `#ff127b`                   |
    /// | `0x2e` | `#f51e8c`                   |
    /// | `0x2f` | `#eb2da0`                   |
    /// | `0x30` | `#e637b4`                   |
    /// | `0x31` | `#de44cf`                   |
    /// | `0x32` | `#de448d`                   |
    /// | `0x33` | `#e630b4`                   |
    /// | `0x34` | `#e619dc`                   |
    /// | `0x35` | `#e600ff`                   |
    /// | `0x36` | `#dc00ff`                   |
    /// | `0x37` | `#cc00ff`                   |
    /// | `0x38` | `#b432ff`                   |
    /// | `0x39` | `#b93cff`                   |
    /// | `0x3a` | `#c542ff`                   |
    /// | `0x3b` | `#aa5aff`                   |
    /// | `0x3c` | `#aa72ff`                   |
    /// | `0x3d` | `#8272ff`                   |
    /// | `0x3e` | `#6473ff`                   |
    pub hot_cue_color_index: u8,
    /// Rekordbot hot cue color RGB value.
    ///
    /// This color is similar but not identical to the color that Rekordbox displays, and possibly
    /// used to illuminate the RGB LEDs in a player that has loaded the cue. If not color is
    /// associated with this hot cue, the value is `(0, 0, 0)`.
    pub hot_cue_color_rgb: (u8, u8, u8),
    /// Unknown field.
    pub unknown6: u32,
    /// Unknown field.
    pub unknown7: u32,
    /// Unknown field.
    pub unknown8: u32,
    /// Unknown field.
    pub unknown9: u32,
    /// Unknown field.
    pub unknown10: u32,
}

/// Single Column value in a Waveform Preview.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Clone, Copy)]
#[br(big, map = Self::from_bytes)]
#[bw(big, map = |x: &WaveformPreviewColumn| x.into_bytes())]
pub struct WaveformPreviewColumn {
    /// Height of the Column in pixels.
    pub height: B5,
    /// Shade of white.
    pub whiteness: B3,
}

/// Single Column value in a Tiny Waveform Preview.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Clone, Copy)]
#[br(big, map = Self::from_bytes)]
#[bw(big, map = |x: &TinyWaveformPreviewColumn| x.into_bytes())]
pub struct TinyWaveformPreviewColumn {
    /// Height of the Column in pixels.
    pub unused: B4,
    pub beight: B4,
}

/// Single Column value in a Waveform Color Preview.
///
/// See these the documentation for details:
/// <https://djl-analysis.deepsymmetry.org/djl-analysis/track_metadata.html#color-preview-analysis>
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(big)]
pub struct WaveformColorPreviewColumn {
    /// Unknown field (somehow encodes the "whiteness").
    pub unknown1: u8,
    /// Unknown field (somehow encodes the "whiteness").
    pub unknown2: u8,
    /// Sound energy in the bottom half of the frequency range (<10 KHz).
    pub energy_bottom_half_freq: u8,
    /// Sound energy in the bottom third of the frequency range.
    pub energy_bottom_third_freq: u8,
    /// Sound energy in the mid of the frequency range.
    pub energy_mid_third_freq: u8,
    /// Sound energy in the top of the frequency range.
    pub energy_top_third_freq: u8,
}

/// Single Column value in a Waveform Color Detail section.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Clone, Copy)]
#[br(map = Self::from_bytes)]
#[bw(big, map = |x: &WaveformColorDetailColumn| x.into_bytes())]
pub struct WaveformColorDetailColumn {
    /// Red color component.
    pub red: B3,
    /// Green color component.
    pub green: B3,
    /// Blue color component.
    pub blue: B3,
    /// Height of the column.
    pub height: B5,
    /// Unknown field
    pub unknown: B2,
}

/// Music classification that is used for Lightnight mode and based on rhythm, tempo kick drum and
/// sound density.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(big)]
pub enum Mood {
    /// Phrase types consist of "Intro", "Up", "Down", "Chorus", and "Outro". Other values in each
    /// phrase entry cause the intro, chorus, and outro phrases to have their labels subdivided
    /// into styes "1" or "2" (for example, "Intro 1"), and "up" is subdivided into style "Up 1",
    /// "Up 2", or "Up 3".
    #[brw(magic = 1u16)]
    High,
    /// Phrase types are labeled "Intro", "Verse 1" through "Verse 6", "Chorus", "Bridge", and
    /// "Outro".
    #[brw(magic = 2u16)]
    Mid,
    /// Phrase types are labeled "Intro", "Verse 1", "Verse 2", "Chorus", "Bridge", and "Outro".
    /// There are three different phrase type values for each of "Verse 1" and "Verse 2", but
    /// rekordbox makes no distinction between them.
    #[brw(magic = 3u16)]
    Low,
    /// Unknown value.
    ///
    /// TODO: Remove this once https://github.com/Holzhaus/rekordcrate/issues/13 has been fixed.
    Unknown(u16),
}

/// Stylistic track bank for Lightning mode.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum Bank {
    /// Default bank variant, treated as `Cool`.
    #[brw(magic = 0u8)]
    Default,
    /// "Cool" bank variant.
    #[brw(magic = 1u8)]
    Cool,
    /// "Natural" bank variant.
    #[brw(magic = 2u8)]
    Natural,
    /// "Hot" bank variant.
    #[brw(magic = 3u8)]
    Hot,
    /// "Subtle" bank variant.
    #[brw(magic = 4u8)]
    Subtle,
    /// "Warm" bank variant.
    #[brw(magic = 5u8)]
    Warm,
    /// "Vivid" bank variant.
    #[brw(magic = 6u8)]
    Vivid,
    /// "Club 1" bank variant.
    #[brw(magic = 7u8)]
    Club1,
    /// "Club 2" bank variant.
    #[brw(magic = 8u8)]
    Club2,
    /// Unknown value.
    ///
    /// TODO: Remove this once https://github.com/Holzhaus/rekordcrate/issues/13 has been fixed.
    Unknown(u8),
}

/// A song structure entry that represents a phrase in the track.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(big)]
pub struct Phrase {
    /// Phrase number (starting at 1).
    pub index: u16,
    /// Beat number where this phrase begins.
    pub beat: u16,
    /// Kind of phrase that rekordbox has identified (?).
    pub kind: u16,
    /// Unknown field.
    #[allow(dead_code)]
    unknown1: u8,
    /// Flag byte used for numbered variations (in case of the `High` mood).
    ///
    /// See the documentation for details:
    /// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#high-phrase-variants>
    pub k1: u8,
    /// Unknown field.
    #[allow(dead_code)]
    unknown2: u8,
    /// Flag byte used for numbered variations (in case of the `High` mood).
    ///
    /// See the documentation for details:
    /// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#high-phrase-variants>
    pub k2: u8,
    /// Unknown field.
    #[allow(dead_code)]
    unknown3: u8,
    /// Flag that determined if only `beat2` is used (0), or if `beat2`, `beat3` and `beat4` are
    /// used (1).
    pub b: u8,
    /// Beat number.
    pub beat2: u16,
    /// Beat number.
    pub beat3: u16,
    /// Beat number.
    pub beat4: u16,
    /// Unknown field.
    #[allow(dead_code)]
    unknown4: u8,
    /// Flag byte used for numbered variations (in case of the `High` mood).
    ///
    /// See the documentation for details:
    /// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#high-phrase-variants>
    pub k3: u8,
    /// Unknown field.
    #[allow(dead_code)]
    unknown5: u8,
    /// Indicates if there are fill (non-phrase) beats at the end of the phrase.
    pub fill: u8,
    /// Beat number where the fill begins (if `fill` is non-zero).
    pub beat_fill: u16,
}

/// Section content which differs depending on the section type.
#[derive(Debug, PartialEq)]
#[binrw]
#[br(import(header: Header))]
pub enum Content {
    /// All beats in the track.
    #[brw(pre_assert(header.kind == ContentKind::BeatGrid))]
    BeatGrid(BeatGrid),
    /// List of cue points or loops (either hot cues or memory cues).
    #[brw(pre_assert(header.kind == ContentKind::CueList))]
    CueList(CueList),
    /// List of cue points or loops (either hot cues or memory cues, extended version).
    ///
    /// Variation of the original `CueList` that also adds support for more metadata such as
    /// comments and colors. Introduces with the Nexus 2 series players.
    #[brw(pre_assert(header.kind == ContentKind::ExtendedCueList))]
    ExtendedCueList(ExtendedCueList),
    /// Path of the audio file that this analysis belongs to.
    #[brw(pre_assert(header.kind == ContentKind::Path))]
    Path(#[br(args(header.clone()))] Path),
    /// Seek information for variable bitrate files (probably).
    #[brw(pre_assert(header.kind == ContentKind::VBR))]
    VBR(#[br(args(header.clone()))] VBR),
    /// Fixed-width monochrome preview of the track waveform.
    #[brw(pre_assert(header.kind == ContentKind::WaveformPreview))]
    WaveformPreview(#[br(args(header.clone()))] WaveformPreview),
    /// Smaller version of the fixed-width monochrome preview of the track waveform.
    #[brw(pre_assert(header.kind == ContentKind::TinyWaveformPreview))]
    TinyWaveformPreview(#[br(args(header.clone()))] TinyWaveformPreview),
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(pre_assert(header.kind == ContentKind::WaveformDetail))]
    WaveformDetail(WaveformDetail),
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(pre_assert(header.kind == ContentKind::WaveformColorPreview))]
    WaveformColorPreview(WaveformColorPreview),
    /// Variable-width large colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(pre_assert(header.kind == ContentKind::WaveformColorDetail))]
    WaveformColorDetail(WaveformColorDetail),
    /// Describes the structure of a sond (Intro, Chrous, Verse, etc.).
    ///
    /// Used in `.EXT` files.
    #[brw(pre_assert(header.kind == ContentKind::SongStructure))]
    SongStructure(SongStructure),
    /// Unknown content.
    ///
    /// This allows handling files that contain unknown section types and allows to access later
    /// sections in the file that have a known type instead of failing to parse the whole file.
    #[brw(pre_assert(matches!(header.kind, ContentKind::Unknown(_))))]
    Unknown(#[br(args(header.clone()))] Unknown),
}

/// All beats in the track.
#[derive(Debug, PartialEq)]
#[binrw]
pub struct BeatGrid {
    /// Unknown field.
    unknown1: u32,
    /// Unknown field.
    ///
    /// According to [@flesniak](https://github.com/flesniak), this is always `00800000`.
    unknown2: u32,
    /// Number of beats in this beatgrid.
    len_beats: u32,
    /// Beats in this beatgrid.
    #[br(count = len_beats)]
    pub beats: Vec<Beat>,
}

/// List of cue points or loops (either hot cues or memory cues).
#[derive(Debug, PartialEq)]
#[binrw]
pub struct CueList {
    /// The types of cues (memory or hot) that this list contains.
    pub list_type: CueListType,
    /// Unknown field
    unknown: u16,
    /// Number of cues.
    len_cues: u16,
    /// Unknown field.
    memory_count: u32,
    /// Cues
    #[br(count = len_cues)]
    pub cues: Vec<Cue>,
}

/// List of cue points or loops (either hot cues or memory cues, extended version).
///
/// Variation of the original `CueList` that also adds support for more metadata such as
/// comments and colors. Introduces with the Nexus 2 series players.
#[derive(Debug, PartialEq)]
#[binrw]
pub struct ExtendedCueList {
    /// The types of cues (memory or hot) that this list contains.
    pub list_type: CueListType,
    /// Number of cues.
    len_cues: u16,
    /// Unknown field
    #[br(assert(unknown == 0))]
    unknown: u16,
    /// Cues
    #[br(count = len_cues)]
    pub cues: Vec<ExtendedCue>,
}

/// Path of the audio file that this analysis belongs to.
#[derive(Debug, PartialEq)]
#[binrw]
#[br(import(header: Header))]
pub struct Path {
    /// Length of the path field in bytes.
    len_path: u32,
    #[br(assert(len_path == header.content_size()))]
    /// Path of the audio file.
    #[br(assert((path.len() as u32 + 1) * 2 == len_path))]
    pub path: NullWideString,
}

/// Seek information for variable bitrate files (probably).
#[derive(Debug, PartialEq)]
#[binrw]
#[br(import(header: Header))]
pub struct VBR {
    /// Unknown field.
    unknown1: u32,
    /// Unknown data.
    #[br(count = header.content_size())]
    unknown2: Vec<u8>,
}

/// Fixed-width monochrome preview of the track waveform.
#[derive(Debug, PartialEq)]
#[binrw]
#[br(import(header: Header))]
pub struct WaveformPreview {
    /// Unknown field.
    len_preview: u32,
    /// Unknown field (apparently always `0x00100000`)
    unknown: u32,
    /// Waveform preview column data.
    #[br(count = header.content_size())]
    pub data: Vec<WaveformPreviewColumn>,
}

/// Smaller version of the fixed-width monochrome preview of the track waveform.
#[derive(Debug, PartialEq)]
#[binrw]
#[br(import(header: Header))]
pub struct TinyWaveformPreview {
    /// Unknown field.
    len_preview: u32,
    /// Unknown field (apparently always `0x00100000`)
    unknown: u32,
    /// Waveform preview column data.
    #[br(count = header.content_size())]
    pub data: Vec<TinyWaveformPreviewColumn>,
}

/// Variable-width large monochrome version of the track waveform.
///
/// Used in `.EXT` files.
#[derive(Debug, PartialEq)]
#[binrw]
pub struct WaveformDetail {
    /// Size of a single entry, always 1.
    #[br(assert(len_entry_bytes == 1))]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    len_entries: u32,
    /// Unknown field (apparently always `0x00960000`)
    #[br(assert(unknown == 0x00960000))]
    unknown: u32,
    /// Waveform preview column data.
    ///
    /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
    /// so for each second of track audio there are 150 waveform detail entries.
    #[br(count = len_entries)]
    pub data: Vec<WaveformPreviewColumn>,
}

/// Variable-width large monochrome version of the track waveform.
///
/// Used in `.EXT` files.
#[derive(Debug, PartialEq)]
#[binrw]
pub struct WaveformColorPreview {
    /// Size of a single entry, always 6.
    #[br(assert(len_entry_bytes == 6))]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    len_entries: u32,
    /// Unknown field.
    unknown: u32,
    /// Waveform preview column data.
    ///
    /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
    /// so for each second of track audio there are 150 waveform detail entries.
    #[br(count = len_entries)]
    pub data: Vec<WaveformColorPreviewColumn>,
}

/// Variable-width large colored version of the track waveform.
///
/// Used in `.EXT` files.
#[derive(Debug, PartialEq)]
#[binrw]
pub struct WaveformColorDetail {
    /// Size of a single entry, always 2.
    #[br(assert(len_entry_bytes == 2))]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    len_entries: u32,
    /// Unknown field.
    unknown: u32,
    /// Waveform detail column data.
    #[br(count = len_entries)]
    pub data: Vec<WaveformColorDetailColumn>,
}

/// Describes the structure of a sond (Intro, Chrous, Verse, etc.).
///
/// Used in `.EXT` files.
#[derive(Debug, PartialEq)]
#[binrw]
pub struct SongStructure {
    /// Size of a single entry, always 24.
    #[br(assert(len_entry_bytes == 24))]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    len_entries: u16,
    /// Overall type of phrase structure.
    pub mood: Mood,
    /// Unknown field.
    unknown1: u32,
    /// Unknown field.
    unknown2: u16,
    /// Number of the beat at which the last recognized phrase ends.
    pub end_beat: u16,
    /// Unknown field.
    unknown3: u16,
    /// Stylistic bank assigned in Lightning Mode.
    pub bank: Bank,
    /// Unknown field.
    unknown4: u8,
    /// Phrase entry data.
    #[br(count = len_entries)]
    pub data: Vec<Phrase>,
}

/// Unknown content.
#[derive(Debug, PartialEq)]
#[binrw]
#[br(import(header: Header))]
pub struct Unknown {
    /// Unknown header data.
    #[br(count = header.remaining_size())]
    header_data: Vec<u8>,
    /// Unknown content data.
    #[br(count = header.content_size())]
    content_data: Vec<u8>,
}

/// ANLZ Section.
#[derive(Debug, PartialEq)]
#[binrw]
pub struct Section {
    /// The header.
    pub header: Header,
    /// The section content.
    #[br(args(header.clone()))]
    pub content: Content,
}

/// ANLZ file section.
///
/// The actual contents are not part of this struct and can parsed on-the-fly by iterating over the
/// `ANLZ::sections()` method.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(big)]
pub struct ANLZ {
    /// The file header.
    #[br(assert(header.kind == ContentKind::File))]
    pub header: Header,
    /// The header data.
    #[br(count = header.remaining_size())]
    pub header_data: Vec<u8>,
    /// The content sections.
    #[br(parse_with = Self::parse_sections, args(header.content_size()))]
    pub sections: Vec<Section>,
}

impl ANLZ {
    fn parse_sections<R: Read + Seek>(
        reader: &mut R,
        ro: &ReadOptions,
        args: (u32,),
    ) -> BinResult<Vec<Section>> {
        let (content_size,) = args;
        let final_position = reader.stream_position()? + u64::from(content_size);

        let mut sections: Vec<Section> = vec![];
        while reader.stream_position()? < final_position {
            let section = Section::read_options(reader, ro, ())?;
            sections.push(section);
        }

        Ok(sections)
    }
}
