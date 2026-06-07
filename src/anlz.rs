// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
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

use crate::{util::ColorIndex, xor::XorStream};
use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian, NullWideString,
};
use modular_bitfield::prelude::*;

/// The kind of section.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
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
    WaveformBluePreview,
    /// Smaller version of the fixed-width monochrome preview of the track waveform (for the
    /// CDJ-900).
    #[brw(magic = b"PWV2")]
    WaveformBlueTinyPreview,
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PWV3")]
    WaveformBlueDetail,
    /// Fixed-width colored preview of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PWV4")]
    WaveformRGBPreview,
    /// Variable-width large colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PWV5")]
    WaveformRGBDetail,
    /// Fixed-width 3-band preview of the track waveform.
    ///
    /// Used in `.2EX` files.
    #[brw(magic = b"PWV6")]
    Waveform3BandPreview,
    /// Variable-width large 3-band version of the track waveform.
    ///
    /// Used in `.2EX` files.
    #[brw(magic = b"PWV7")]
    Waveform3BandDetail,
    /// Per-band gain calibration for the 3-band player waveform.
    ///
    /// Used in `.2EX` files.
    #[brw(magic = b"PWVC")]
    Waveform3BandCalibration,
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
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
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
#[binrw]
#[derive(Debug, PartialEq, Eq)]
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
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big, repr = u32)]
pub enum CueListType {
    /// Memory cues or loops.
    MemoryCues = 0,
    /// Hot cues or loops.
    HotCues = 1,
}

/// Indicates if the cue is point or a loop.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(repr = u8)]
pub enum CueType {
    /// Cue is a single point.
    Point = 1,
    /// Cue is a loop.
    Loop = 2,
}

/// A memory or hot cue (or loop).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
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
    unknown1: u32,
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
    unknown2: u8,
    /// Unknown field. Seems always have the value `0x03E8` (= decimal 1000).
    unknown3: u16,
    /// Time in milliseconds after which this cue would occur (at normal playback speed).
    pub time: u32,
    /// Time in milliseconds after which this the loop would jump back to `time` (at normal playback speed).
    pub loop_time: u32,
    /// Unknown field.
    unknown4: u32,
    /// Unknown field.
    unknown5: u32,
    /// Unknown field.
    unknown6: u32,
    /// Unknown field.
    unknown7: u32,
}

/// A length-prefixed wide (UTF-16BE) string.
///
/// The binary representation is a `u32` length (in bytes, including the trailing
/// NUL terminator) followed by that many bytes of UTF-16BE encoded text. The
/// trailing NUL is stripped on read and appended on write.
/// Its binary structure can be visualized as follows:
///
/// ```text
/// | <length> (u32) | UTF-16BE encoded text | 0x0000 |
///                   <------------------------------>
///                            <length> bytes
/// ```
/// Used for the `comment` field in the `ExtendedCue` section.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LenPrefixedWideString(pub String);

impl LenPrefixedWideString {
    /// Returns the number of bytes required to encode this string as UTF-16BE
    /// with a trailing NUL terminator (excluding the length prefix).
    #[must_use]
    pub fn byte_len(&self) -> u32 {
        if self.0.is_empty() {
            0
        } else {
            (self.0.encode_utf16().count() as u32 + 1) * 2
        }
    }
}

impl From<String> for LenPrefixedWideString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for LenPrefixedWideString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::ops::Deref for LenPrefixedWideString {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl BinRead for LenPrefixedWideString {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let len: u32 = u32::read_options(reader, endian, ())?;
        let len = len as usize;
        if len == 0 {
            return Ok(Self(String::new()));
        }
        let mut bytes = vec![0u8; len];
        reader.read_exact(&mut bytes)?;
        let code_units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        let s = String::from_utf16(&code_units)
            .map(|s| s.trim_end_matches('\0').to_string())
            .map_err(|e| binrw::Error::Custom {
                pos: reader.stream_position().unwrap_or(0),
                err: Box::new(e),
            })?;
        Ok(Self(s))
    }
}

impl BinWrite for LenPrefixedWideString {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        let bl = self.byte_len();
        bl.write_options(writer, endian, ())?;
        if bl > 0 {
            let mut bytes: Vec<u8> = Vec::new();
            for cu in self.0.encode_utf16() {
                bytes.extend_from_slice(&cu.to_be_bytes());
            }
            bytes.extend_from_slice(&[0, 0]);
            writer.write_all(&bytes)?;
        }
        Ok(())
    }
}

/// A memory or hot cue (or loop).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
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
    unknown1: u8,
    /// Unknown field. Seems always have the value `0x03E8` (= decimal 1000).
    unknown2: u16,
    /// Time in milliseconds after which this cue would occur (at normal playback speed).
    pub time: u32,
    /// Time in milliseconds after which this the loop would jump back to `time` (at normal playback speed).
    pub loop_time: u32,
    /// Color assigned to this cue.
    ///
    /// Only used by memory cues, hot cues use a different value (see below).
    pub color: ColorIndex,
    /// Unknown field.
    unknown3: u8,
    /// Unknown field.
    unknown4: u16,
    /// Unknown field.
    unknown5: u32,
    /// Represents the loop size numerator (if this is a quantized loop).
    pub loop_numerator: u16,
    /// Represents the loop size denominator (if this is a quantized loop).
    pub loop_denominator: u16,
    /// An UTF-16BE encoded string with a leading `u32` length prefix and a
    /// trailing NUL (`0x0000`).
    pub comment: LenPrefixedWideString,
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
    /// Rekordbox hot cue color RGB value.
    ///
    /// This color is similar but not identical to the color that Rekordbox displays, and possibly
    /// used to illuminate the RGB LEDs in a player that has loaded the cue. If not color is
    /// associated with this hot cue, the value is `(0, 0, 0)`.
    pub hot_cue_color_rgb: (u8, u8, u8),
    /// Unknown field.
    unknown6: u32,
    /// Unknown field.
    unknown7: u32,
    /// Unknown field.
    unknown8: u32,
    /// Unknown field.
    unknown9: u32,
    /// Unknown field.
    unknown10: u32,
    /// Trailing unknown bytes after `unknown10` to the end of the entry.
    ///
    /// Per the Kaitai spec, the entry may contain extra data beyond the known fields;
    /// any remaining bytes are captured here.
    #[br(count = header.total_size.saturating_sub(68 + comment.byte_len()) as usize)]
    pub trailing: Vec<u8>,
}

impl Default for WaveformBlueColumn {
    fn default() -> Self {
        Self::new()
    }
}

/// Single Column value in a blue waveform.
///
/// See these the documentation for details:
/// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#waveform-preview-tag>
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(big, map = Self::from_bytes)]
#[bw(big, map = |x: &WaveformBlueColumn| x.into_bytes())]
pub struct WaveformBlueColumn {
    /// Height of the Column in pixels.
    pub height: B5,
    /// Shade of white.
    pub whiteness: B3,
}

impl Default for WaveformBlueTinyPreviewColumn {
    fn default() -> Self {
        Self::new()
    }
}

/// Single Column value in a tiny blue waveform preview.
///
/// See these the documentation for details:
/// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#tiny-preview>
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(big, map = Self::from_bytes)]
#[bw(big, map = |x: &WaveformBlueTinyPreviewColumn| x.into_bytes())]
pub struct WaveformBlueTinyPreviewColumn {
    /// Height of the Column in pixels.
    pub height: B4,
    #[allow(dead_code)]
    unknown: B4,
}

/// Single column value in an RGB waveform preview.
///
/// See these the documentation for details:
/// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#color-preview>
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big)]
pub struct WaveformRGBPreviewColumn {
    /// Auxiliary byte in RGB preview entries.
    ///
    /// Across the current fixture corpus this usually pairs with `unknown2` so their sum is near
    /// `255` or `256`, but its exact meaning is still unknown.
    pub unknown1: u8,
    /// Auxiliary byte in RGB preview entries.
    ///
    /// Across the current fixture corpus this usually pairs with `unknown1` so their sum is near
    /// `255` or `256`, but its exact meaning is still unknown.
    pub unknown2: u8,
    /// Auxiliary byte in RGB preview entries.
    ///
    /// This varies across the full `0..=127` range and seems related to overall preview intensity,
    /// but its exact meaning is still unknown.
    pub unknown3: u8,
    /// Red channel intensity, corresponding to bass.
    pub red_bass: u8,
    /// Green channel intensity, corresponding to mids.
    pub green_mids: u8,
    /// Blue channel intensity, corresponding to highs.
    pub blue_highs: u8,
}

impl Default for WaveformRGBDetailColumn {
    fn default() -> Self {
        Self::new()
    }
}

/// Single column value in an RGB waveform detail section.
///
/// See these the documentation for details:
/// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#color-detail>
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct WaveformRGBDetailColumn {
    red_bass: u8,
    green_mids: u8,
    blue_highs: u8,
    height: u8,
    low_bits: u8,
}

impl BinRead for WaveformRGBDetailColumn {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let raw_word = u16::read_options(reader, endian, ())?;
        Ok(Self::from_packed_word(raw_word))
    }
}

impl BinWrite for WaveformRGBDetailColumn {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.to_packed_word().write_options(writer, endian, ())
    }
}

impl WaveformRGBDetailColumn {
    const RED_BASS_SHIFT: u16 = 13;
    const GREEN_MIDS_SHIFT: u16 = 10;
    const BLUE_HIGHS_SHIFT: u16 = 7;
    const HEIGHT_SHIFT: u16 = 2;
    const CHANNEL_MASK: u16 = 0x7;
    const HEIGHT_MASK: u16 = 0x1f;
    const LOW_BITS_MASK: u16 = 0x3;

    /// Create an empty RGB waveform detail word.
    pub const fn new() -> Self {
        Self {
            red_bass: 0,
            green_mids: 0,
            blue_highs: 0,
            height: 0,
            low_bits: 0,
        }
    }

    /// Decode an RGB waveform detail word from its packed 16-bit representation.
    pub const fn from_packed_word(raw_word: u16) -> Self {
        Self {
            red_bass: ((raw_word >> Self::RED_BASS_SHIFT) & Self::CHANNEL_MASK) as u8,
            green_mids: ((raw_word >> Self::GREEN_MIDS_SHIFT) & Self::CHANNEL_MASK) as u8,
            blue_highs: ((raw_word >> Self::BLUE_HIGHS_SHIFT) & Self::CHANNEL_MASK) as u8,
            height: ((raw_word >> Self::HEIGHT_SHIFT) & Self::HEIGHT_MASK) as u8,
            low_bits: (raw_word & Self::LOW_BITS_MASK) as u8,
        }
    }

    const fn to_packed_word(self) -> u16 {
        ((self.red_bass as u16 & Self::CHANNEL_MASK) << Self::RED_BASS_SHIFT)
            | ((self.green_mids as u16 & Self::CHANNEL_MASK) << Self::GREEN_MIDS_SHIFT)
            | ((self.blue_highs as u16 & Self::CHANNEL_MASK) << Self::BLUE_HIGHS_SHIFT)
            | ((self.height as u16 & Self::HEIGHT_MASK) << Self::HEIGHT_SHIFT)
            | (self.low_bits as u16 & Self::LOW_BITS_MASK)
    }

    /// Red channel intensity, corresponding to bass.
    pub const fn red_bass(&self) -> u8 {
        self.red_bass
    }

    /// Green channel intensity, corresponding to mids.
    pub const fn green_mids(&self) -> u8 {
        self.green_mids
    }

    /// Blue channel intensity, corresponding to highs.
    pub const fn blue_highs(&self) -> u8 {
        self.blue_highs
    }

    /// Coarse 5-bit column height.
    pub const fn height(&self) -> u8 {
        self.height
    }

    /// Raw low-order fine-height bits in on-disk order.
    pub const fn low_bits(&self) -> u8 {
        self.low_bits
    }

    /// Fine-height sub-step with the observed significance ordering.
    pub const fn fine_height_substep(&self) -> u8 {
        let low = self.low_bits();
        ((low & 1) << 1) | ((low >> 1) & 1)
    }

    /// Full 7-bit height formed from the coarse height plus reordered fine-height bits.
    pub const fn full_height(&self) -> u8 {
        (self.height() << 2) | self.fine_height_substep()
    }

    /// Return a copy with the red / bass intensity updated.
    pub fn with_red_bass(mut self, value: u8) -> Self {
        self.red_bass = value.min(7);
        self
    }

    /// Return a copy with the green / mids intensity updated.
    pub fn with_green_mids(mut self, value: u8) -> Self {
        self.green_mids = value.min(7);
        self
    }

    /// Return a copy with the blue / highs intensity updated.
    pub fn with_blue_highs(mut self, value: u8) -> Self {
        self.blue_highs = value.min(7);
        self
    }

    /// Return a copy with the coarse height updated.
    pub fn with_height(mut self, value: u8) -> Self {
        self.height = value.min(31);
        self
    }

    /// Return a copy with the raw fine-height bits updated.
    pub fn with_low_bits(mut self, value: u8) -> Self {
        self.low_bits = value.min(3);
        self
    }
}

/// Single Column value in a Waveform 3-Band Preview.
///
/// See these the documentation for details:
/// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#three-band-preview>
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big)]
pub struct Waveform3BandPreviewColumn {
    /// Low / bass band intensity.
    pub low: u8,
    /// Mid band intensity.
    pub mid: u8,
    /// High band intensity.
    pub high: u8,
}

/// Single Column value in a Waveform 3-Band Detail section.
///
/// See these the documentation for details:
/// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#three-band-detail>
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big)]
pub struct Waveform3BandDetailColumn {
    /// Low / bass band intensity.
    pub low: u8,
    /// Mid band intensity.
    pub mid: u8,
    /// High band intensity.
    pub high: u8,
}

/// Music classification that is used for Lightnight mode and based on rhythm, tempo kick drum and
/// sound density.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big, repr = u16)]
pub enum Mood {
    /// Phrase types consist of "Intro", "Up", "Down", "Chorus", and "Outro". Other values in each
    /// phrase entry cause the intro, chorus, and outro phrases to have their labels subdivided
    /// into styles "1" or "2" (for example, "Intro 1"), and "up" is subdivided into style "Up 1",
    /// "Up 2", or "Up 3".
    High = 1,
    /// Phrase types are labeled "Intro", "Verse 1" through "Verse 6", "Chorus", "Bridge", and
    /// "Outro".
    Mid,
    /// Phrase types are labeled "Intro", "Verse 1", "Verse 2", "Chorus", "Bridge", and "Outro".
    /// There are three different phrase type values for each of "Verse 1" and "Verse 2", but
    /// rekordbox makes no distinction between them.
    Low,
}

/// Stylistic track bank for Lightning mode.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(repr = u8)]
pub enum Bank {
    /// Default bank variant, treated as `Cool`.
    Default = 0,
    /// "Cool" bank variant.
    Cool,
    /// "Natural" bank variant.
    Natural,
    /// "Hot" bank variant.
    Hot,
    /// "Subtle" bank variant.
    Subtle,
    /// "Warm" bank variant.
    Warm,
    /// "Vivid" bank variant.
    Vivid,
    /// "Club 1" bank variant.
    Club1,
    /// "Club 2" bank variant.
    Club2,
}

/// A song structure entry that represents a phrase in the track.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
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
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub enum Content {
    /// All beats in the track.
    #[br(pre_assert(header.kind == ContentKind::BeatGrid))]
    BeatGrid(BeatGrid),
    /// List of cue points or loops (either hot cues or memory cues).
    #[br(pre_assert(header.kind == ContentKind::CueList))]
    CueList(CueList),
    /// List of cue points or loops (either hot cues or memory cues, extended version).
    ///
    /// Variation of the original `CueList` that also adds support for more metadata such as
    /// comments and colors. Introduces with the Nexus 2 series players.
    #[br(pre_assert(header.kind == ContentKind::ExtendedCueList))]
    ExtendedCueList(ExtendedCueList),
    /// Path of the audio file that this analysis belongs to.
    #[br(pre_assert(header.kind == ContentKind::Path))]
    Path(#[br(args(header.clone()))] Path),
    /// Seek information for variable bitrate files (probably).
    #[br(pre_assert(header.kind == ContentKind::VBR))]
    VBR(#[br(args(header.clone()))] VBR),
    /// Fixed-width monochrome preview of the track waveform.
    #[br(pre_assert(header.kind == ContentKind::WaveformBluePreview))]
    WaveformBluePreview(#[br(args(header.clone()))] WaveformBluePreview),
    /// Smaller version of the fixed-width monochrome preview of the track waveform.
    #[br(pre_assert(header.kind == ContentKind::WaveformBlueTinyPreview))]
    WaveformBlueTinyPreview(#[br(args(header.clone()))] WaveformBlueTinyPreview),
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[br(pre_assert(header.kind == ContentKind::WaveformBlueDetail))]
    WaveformBlueDetail(#[br(args(header.clone()))] WaveformBlueDetail),
    /// Smaller version of the fixed-width colored preview of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[br(pre_assert(header.kind == ContentKind::WaveformRGBPreview))]
    WaveformRGBPreview(#[br(args(header.clone()))] WaveformRGBPreview),
    /// Variable-width large colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[br(pre_assert(header.kind == ContentKind::WaveformRGBDetail))]
    WaveformRGBDetail(#[br(args(header.clone()))] WaveformRGBDetail),
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.2EX` files.
    #[br(pre_assert(header.kind == ContentKind::Waveform3BandPreview))]
    Waveform3BandPreview(#[br(args(header.clone()))] Waveform3BandPreview),
    /// Variable-width large 3-band version of the track waveform.
    ///
    /// Used in `.2EX` files.
    #[br(pre_assert(header.kind == ContentKind::Waveform3BandDetail))]
    Waveform3BandDetail(#[br(args(header.clone()))] Waveform3BandDetail),
    /// Per-band gain calibration for the 3-band player waveform.
    ///
    /// Used in `.2EX` files.
    #[br(pre_assert(header.kind == ContentKind::Waveform3BandCalibration))]
    Waveform3BandCalibration(#[br(args(header.clone()))] Waveform3BandCalibration),
    /// Describes the structure of a sond (Intro, Chrous, Verse, etc.).
    ///
    /// Used in `.EXT` files.
    #[br(pre_assert(header.kind == ContentKind::SongStructure))]
    SongStructure(#[br(args(header.clone()))] SongStructure),
    /// Unknown content.
    ///
    /// This allows handling files that contain unknown section types and allows to access later
    /// sections in the file that have a known type instead of failing to parse the whole file.
    #[br(pre_assert(matches!(header.kind, ContentKind::Unknown(_))))]
    Unknown(#[br(args(header.clone()))] Unknown),
}

/// All beats in the track.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct BeatGrid {
    /// Unknown field.
    unknown1: u32,
    /// Unknown field.
    ///
    /// According to [@flesniak](https://github.com/flesniak), this is always `00800000`.
    unknown2: u32,
    /// Number of beats in this beatgrid.
    #[br(temp)]
    #[bw(calc = beats.len() as u32)]
    len_beats: u32,
    /// Beats in this beatgrid.
    #[br(count = len_beats)]
    pub beats: Vec<Beat>,
}

/// List of cue points or loops (either hot cues or memory cues).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct CueList {
    /// The types of cues (memory or hot) that this list contains.
    pub list_type: CueListType,
    /// Unknown field
    unknown: u16,
    /// Number of cues.
    #[br(temp)]
    #[bw(calc = cues.len() as u16)]
    len_cues: u16,
    /// Unknown field.
    memory_count: u32,
    /// Cues
    #[br(count = usize::from(len_cues))]
    pub cues: Vec<Cue>,
}

/// List of cue points or loops (either hot cues or memory cues, extended version).
///
/// Variation of the original `CueList` that also adds support for more metadata such as
/// comments and colors. Introduces with the Nexus 2 series players.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct ExtendedCueList {
    /// The types of cues (memory or hot) that this list contains.
    pub list_type: CueListType,
    /// Number of cues.
    #[br(temp)]
    #[bw(calc = cues.len() as u16)]
    len_cues: u16,
    /// Unknown field
    #[br(assert(unknown == 0))]
    unknown: u16,
    /// Cues
    #[br(count = usize::from(len_cues))]
    pub cues: Vec<ExtendedCue>,
}

/// Path of the audio file that this analysis belongs to.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct Path {
    /// Length of the path field in bytes.
    #[br(temp)]
    #[br(assert(len_path == header.content_size()))]
    #[bw(calc = ((path.len() as u32) + 1) * 2)]
    len_path: u32,
    /// Path of the audio file.
    #[br(assert(len_path == header.content_size()))]
    #[br(assert((path.len() as u32 + 1) * 2 == len_path))]
    pub path: NullWideString,
}

/// Seek information for variable bitrate files (probably).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct VBR {
    /// Unknown field.
    unknown1: u32,
    /// Unknown data.
    #[br(count = header.content_size())]
    unknown2: Vec<u8>,
}

/// Fixed-width monochrome preview of the track waveform.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct WaveformBluePreview {
    /// Number of preview columns.
    #[br(temp)]
    #[br(assert(len_preview == header.content_size()))]
    #[bw(calc = data.len() as u32)]
    len_preview: u32,
    /// Observed constant header field (`0x00010000` in all current fixtures).
    #[br(assert(unknown == 0x0001_0000))]
    #[bw(calc = 0x0001_0000u32)]
    unknown: u32,
    /// Waveform preview column data.
    #[br(count = len_preview)]
    pub data: Vec<WaveformBlueColumn>,
}

/// Smaller version of the fixed-width monochrome preview of the track waveform.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct WaveformBlueTinyPreview {
    /// Number of preview columns.
    #[br(temp)]
    #[br(assert(len_preview == header.content_size()))]
    #[bw(calc = data.len() as u32)]
    len_preview: u32,
    /// Observed constant header field (`0x00010000` in all current fixtures).
    #[br(assert(unknown == 0x0001_0000))]
    #[bw(calc = 0x0001_0000u32)]
    unknown: u32,
    /// Waveform preview column data.
    #[br(count = len_preview)]
    pub data: Vec<WaveformBlueTinyPreviewColumn>,
}

/// Variable-width large monochrome version of the track waveform.
///
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct WaveformBlueDetail {
    /// Size of a single entry, always 1.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 1))]
    #[bw(calc = 1u32)]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries)== header.content_size()))]
    len_entries: u32,
    /// Observed constant header field (`0x00960000` in all current fixtures).
    #[br(assert(unknown == 0x0096_0000))]
    #[bw(calc = 0x0096_0000u32)]
    unknown: u32,
    /// Waveform preview column data.
    ///
    /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
    /// so for each second of track audio there are 150 waveform detail entries.
    #[br(count = len_entries)]
    pub data: Vec<WaveformBlueColumn>,
}

/// Smaller version of the fixed-width colored preview of the track waveform.
///
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct WaveformRGBPreview {
    /// Size of a single entry, always 6.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 6))]
    #[bw(calc = 6u32)]
    len_entry_bytes: u32,
    /// Number of preview columns.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries) == header.content_size()))]
    len_entries: u32,
    /// Observed constant header field (`0x00000000` in all current fixtures).
    #[br(assert(unknown == 0))]
    #[bw(calc = 0u32)]
    unknown: u32,
    /// Waveform preview column data.
    #[br(count = len_entries)]
    pub data: Vec<WaveformRGBPreviewColumn>,
}

/// Variable-width large colored version of the track waveform.
///
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct WaveformRGBDetail {
    /// Size of a single entry, always 2.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 2))]
    #[bw(calc = 2u32)]
    len_entry_bytes: u32,
    /// Number of detail columns.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries) == header.content_size()))]
    len_entries: u32,
    /// Observed constant header field (`0x00960305` in all current fixtures).
    ///
    /// This likely describes the packed detail layout, but the exact meaning is still unknown.
    #[br(assert(unknown == 0x0096_0305))]
    #[bw(calc = 0x0096_0305u32)]
    unknown: u32,
    /// Waveform detail column data.
    ///
    /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
    /// so for each second of track audio there are 150 waveform detail entries.
    #[br(count = len_entries)]
    pub data: Vec<WaveformRGBDetailColumn>,
}

/// Smaller version of the fixed-width 3-band preview of the track waveform.
///
/// Used in `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct Waveform3BandPreview {
    /// Size of a single entry, always 3.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 3))]
    #[bw(calc = 3u32)]
    len_entry_bytes: u32,
    /// Number of preview columns.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries) == header.content_size()))]
    len_entries: u32,
    /// Waveform preview column data.
    #[br(count = len_entries)]
    pub data: Vec<Waveform3BandPreviewColumn>,
}

/// Variable-width large 3-band version of the track waveform.
///
/// Used in `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct Waveform3BandDetail {
    /// Size of a single entry, always 3.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 3))]
    #[bw(calc = 3u32)]
    len_entry_bytes: u32,
    /// Number of detail columns.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries) == header.content_size()))]
    len_entries: u32,
    /// Observed constant header field (`0x00960000` in all current fixtures).
    #[br(assert(unknown == 0x0096_0000))]
    #[bw(calc = 0x0096_0000u32)]
    unknown: u32,
    /// Waveform detail column data.
    ///
    /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
    /// so for each second of track audio there are 150 waveform detail entries.
    #[br(count = len_entries)]
    pub data: Vec<Waveform3BandDetailColumn>,
}

/// Per-band gain calibration for the 3-band player waveform.
///
/// Used in `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[br(import(header: Header))]
pub struct Waveform3BandCalibration {
    /// Observed constant header field (`0x0000` in all current fixtures).
    #[br(temp)]
    #[br(assert(header.remaining_size() == 2))]
    #[br(assert(unknown == 0))]
    #[bw(calc = 0u16)]
    pub unknown: u16,
    /// Gain applied to the low / blue waveform band.
    #[br(assert(header.content_size() == 6))]
    pub low_gain: u16,
    /// Gain applied to the mid / yellow waveform band.
    pub mid_gain: u16,
    /// Gain applied to the high / white waveform band.
    pub high_gain: u16,
}

/// Describes the structure of a song (Intro, Chrous, Verse, etc.).
///
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(header: Header))]
pub struct SongStructure {
    /// Size of a single entry, always 24.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 24))]
    #[bw(calc = 24u32)]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    #[br(temp)]
    #[br(assert((len_entry_bytes * (len_entries as u32)) == header.content_size()))]
    #[bw(calc = data.phrases.len() as u16)]
    len_entries: u16,
    /// Indicates if the remaining parts of the song structure section are encrypted.
    ///
    /// This is a virtual field and not actually present in the file.
    #[br(restore_position, map = |raw_mood: [u8; 2]| SongStructureData::check_if_encrypted(raw_mood, len_entries))]
    #[bw(ignore)]
    is_encrypted: bool,
    /// Song structure data.
    #[br(args(is_encrypted, len_entries), parse_with = SongStructureData::read_encrypted)]
    #[bw(args(*is_encrypted, len_entries), write_with = SongStructureData::write_encrypted)]
    data: SongStructureData,
}

/// The data part of the [`SongStructure`] section that may be encrypted (RB6+).
///
/// See the documentation for details:
/// - <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#song-structure-tag>
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[br(import(len_entries: u16))]
pub struct SongStructureData {
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
    #[br(count = usize::from(len_entries))]
    pub phrases: Vec<Phrase>,
}

impl SongStructureData {
    const KEY_DATA: [u8; 19] = [
        0xCB, 0xE1, 0xEE, 0xFA, 0xE5, 0xEE, 0xAD, 0xEE, 0xE9, 0xD2, 0xE9, 0xEB, 0xE1, 0xE9, 0xF3,
        0xE8, 0xE9, 0xF4, 0xE1,
    ];

    /// Returns an iterator over the key bytes (RB6+).
    fn get_key(len_entries: u16) -> impl Iterator<Item = u8> {
        Self::KEY_DATA.into_iter().map(move |x: u8| -> u8 {
            let value = u16::from(x) + len_entries;
            (value % 256) as u8
        })
    }

    /// Returns `true` if the [`SongStructureData`] is encrypted.
    ///
    /// The method tries to decrypt the `raw_mood` field and checking if the result is valid.
    fn check_if_encrypted(raw_mood: [u8; 2], len_entries: u16) -> bool {
        let buffer: Vec<u8> = raw_mood
            .iter()
            .zip(Self::get_key(len_entries).take(2))
            .map(|(byte, key)| byte ^ key)
            .collect();
        let mut reader = binrw::io::Cursor::new(buffer);
        Mood::read(&mut reader).is_ok()
    }

    /// Read a [`SongStructureData`] section that may be encrypted, depending on the `is_encrypted`
    /// value.
    fn read_encrypted<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (is_encrypted, len_entries): (bool, u16),
    ) -> BinResult<Self> {
        if is_encrypted {
            let key: Vec<u8> = Self::get_key(len_entries).collect();
            let mut xor_reader = XorStream::with_key(reader, key);
            Self::read_options(&mut xor_reader, endian, (len_entries,))
        } else {
            Self::read_options(reader, endian, (len_entries,))
        }
    }

    /// Write a [`SongStructureData`] section that may be encrypted, depending on the
    /// `is_encrypted` value.
    fn write_encrypted<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        (is_encrypted, len_entries): (bool, u16),
    ) -> BinResult<()> {
        if is_encrypted {
            let key: Vec<u8> = Self::get_key(len_entries).collect();
            let mut xor_writer = XorStream::with_key(writer, key);
            self.write_options(&mut xor_writer, endian, ())
        } else {
            self.write_options(writer, endian, ())
        }
    }
}

/// Unknown content.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
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
#[binrw]
#[derive(Debug, PartialEq, Eq)]
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
#[binrw]
#[derive(Debug, PartialEq, Eq)]
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
        endian: Endian,
        args: (u32,),
    ) -> BinResult<Vec<Section>> {
        let (content_size,) = args;
        let final_position = reader.stream_position()? + u64::from(content_size);

        let mut sections: Vec<Section> = vec![];
        while reader.stream_position()? < final_position {
            let section = Section::read_options(reader, endian, ())?;
            sections.push(section);
        }

        Ok(sections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::testing::test_roundtrip;
    use std::fs::File;

    #[test]
    fn extended_cue_empty_comment_roundtrip() {
        // Real ExtendedCue with an empty comment (len_comment=0), extracted
        // from a file provided by @FizzyApple12
        // This would have failed to parse before the fix that replaced
        // NullWideString with LenPrefixedWideString for the `comment` field.
        let raw = [
            0x50, 0x43, 0x50, 0x32, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x58, 0x00, 0x00,
            0x00, 0x04, 0x01, 0x00, 0x03, 0xe8, 0x00, 0x04, 0x62, 0xf7, 0xff, 0xff, 0xff, 0xff,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x4d, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc1, 0x70, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x30,
            0x77, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let cue = ExtendedCue {
            header: Header {
                kind: ContentKind::ExtendedCue,
                size: 16,
                total_size: 88,
            },
            hot_cue: 4,
            cue_type: CueType::Point,
            unknown1: 0,
            unknown2: 1000,
            time: 287479,
            loop_time: 4294967295,
            color: ColorIndex::None,
            unknown3: 1,
            unknown4: 0,
            unknown5: 0,
            loop_numerator: 0,
            loop_denominator: 0,
            comment: LenPrefixedWideString(String::new()),
            hot_cue_color_index: 0,
            hot_cue_color_rgb: (0x4d, 0x00, 0xff),
            unknown6: 0,
            unknown7: 0x00c17000,
            unknown8: 0,
            unknown9: 0,
            unknown10: 0,
            trailing: vec![
                0x02, 0x30, 0x77, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
        };

        test_roundtrip(&raw, cue);
    }

    #[test]
    fn parses_waveform_3band_calibration_section() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/complete_export/demo_tracks/PIONEER/USBANLZ/P016/0000875E/ANLZ0000.2EX");
        let mut reader = File::open(path).expect("fixture should open");
        let anlz = ANLZ::read(&mut reader).expect("fixture should parse");

        let calibration = anlz
            .sections
            .iter()
            .find_map(|section| match &section.content {
                Content::Waveform3BandCalibration(calibration) => Some(calibration),
                _ => None,
            })
            .expect("fixture should include PWVC calibration");

        assert!(calibration.low_gain > 0);
        assert!(calibration.mid_gain > 0);
        assert!(calibration.high_gain > 0);
    }

    #[test]
    fn waveform_rgb_detail_word_uses_expected_bit_layout() {
        let entry = WaveformRGBDetailColumn::from_packed_word(0x03fc);
        assert_eq!(entry.red_bass(), 0);
        assert_eq!(entry.green_mids(), 0);
        assert_eq!(entry.blue_highs(), 7);
        assert_eq!(entry.height(), 31);
        assert_eq!(entry.low_bits(), 0);
        assert_eq!(entry.fine_height_substep(), 0);
        assert_eq!(entry.full_height(), 124);

        let rebuilt = WaveformRGBDetailColumn::new()
            .with_red_bass(5)
            .with_green_mids(2)
            .with_blue_highs(3)
            .with_height(17)
            .with_low_bits(1);
        assert_eq!(rebuilt.to_packed_word(), 0xa9c5);
        assert_eq!(rebuilt.red_bass(), 5);
        assert_eq!(rebuilt.green_mids(), 2);
        assert_eq!(rebuilt.blue_highs(), 3);
        assert_eq!(rebuilt.height(), 17);
        assert_eq!(rebuilt.low_bits(), 1);
        assert_eq!(rebuilt.fine_height_substep(), 2);
        assert_eq!(rebuilt.full_height(), 70);
    }

    #[test]
    fn tiny_blue_waveform_preview_uses_low_nibble_for_height() {
        let sample = WaveformBlueTinyPreviewColumn::from_bytes([0x8f]);
        assert_eq!(sample.height(), 15);
        assert_eq!(sample.unknown(), 8);

        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/complete_export/demo_tracks/PIONEER/USBANLZ/P016/0000875E/ANLZ0000.DAT");
        let mut reader = File::open(path).expect("fixture should open");
        let anlz = ANLZ::read(&mut reader).expect("fixture should parse");

        let preview = anlz
            .sections
            .iter()
            .find_map(|section| match &section.content {
                Content::WaveformBlueTinyPreview(preview) => Some(preview),
                _ => None,
            })
            .expect("fixture should include PWV2");

        assert!(preview.data.iter().any(|entry| entry.height() > 0));
        assert!(preview.data.iter().all(|entry| entry.unknown() == 0));
    }
}
