// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Parser for Rekordbox analysis files, that can be found inside nested subdirectories of the
//! `PIONEER/USBANLZ` directory and can have the extensions `.DAT`, `.EXT` or `.2EX`.
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

use crate::util::ColorIndex;
use nom::error::{ErrorKind, ParseError};
use nom::Err;
use nom::IResult;

#[derive(Debug)]
/// The kind of section.
pub enum ContentKind {
    /// File section that contains all other sections.
    File,
    /// All beats found in the track.
    BeatGrid,
    /// Either memory points and loops or hotcues and hot loops of the track.
    ///
    /// *Note:* Since the release of the Nexus 2 series, there also exists the `ExtendedCueList`
    /// section which can carry additional information.
    CueList,
    /// Extended version of the `CueList` section (since Nexus 2 series).
    ExtendedCueList,
    /// Single cue entry inside a `ExtendedCueList` section.
    ExtendedCue,
    /// Single cue entry inside a `CueList` section.
    Cue,
    /// File path of the audio file.
    Path,
    /// Seek information for variable bitrate files.
    VBR,
    /// Fixed-width monochrome preview of the track waveform.
    WaveformPreview,
    /// Smaller version of the fixed-width monochrome preview of the track waveform (for the
    /// CDJ-900).
    TinyWaveformPreview,
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.EXT` files.
    WaveformDetail,
    /// Fixed-width colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    WaveformColorPreview,
    /// Variable-width large colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    WaveformColorDetail,
    /// Describes the structure of a sond (Intro, Chrous, Verse, etc.).
    ///
    /// Used in `.EXT` files.
    SongStructure,
    /// Unknown Kind.
    Unknown([u8; 4]),
}

impl ContentKind {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, kind_slice) = nom::bytes::complete::take(4usize)(input)?;
        let kind = match kind_slice {
            b"PMAI" => ContentKind::File,
            b"PCO2" => ContentKind::ExtendedCueList,
            b"PCOB" => ContentKind::CueList,
            b"PCP2" => ContentKind::ExtendedCue,
            b"PCPT" => ContentKind::Cue,
            b"PPTH" => ContentKind::Path,
            b"PQTZ" => ContentKind::BeatGrid,
            b"PVBR" => ContentKind::VBR,
            b"PWAV" => ContentKind::WaveformPreview,
            b"PWV2" => ContentKind::TinyWaveformPreview,
            b"PWV3" => ContentKind::WaveformDetail,
            b"PWV4" => ContentKind::WaveformColorPreview,
            b"PWV5" => ContentKind::WaveformColorDetail,
            b"PSSI" => ContentKind::SongStructure,
            unk => {
                let kind_buffer: [u8; 4] = unk.try_into().unwrap();
                Self::Unknown(kind_buffer)
            }
        };

        Ok((input, kind))
    }
}

#[derive(Debug)]
/// Header of a section that contains type and size information.
pub struct Header {
    /// Kind of content in this item.
    pub kind: ContentKind,
    /// Length of the header data (including `kind`, `size` and `total_size`).
    pub size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
}

impl Header {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, kind) = ContentKind::parse(input)?;
        let (input, size) = nom::number::complete::be_u32(input)?;
        let (input, total_size) = nom::number::complete::be_u32(input)?;

        Ok((
            input,
            Self {
                kind,
                size,
                total_size,
            },
        ))
    }

    fn remaining_size(&self) -> u32 {
        self.size - 12
    }

    fn content_size(&self) -> u32 {
        self.total_size - self.size
    }
}

#[derive(Debug)]
/// A single beat inside the beat grid.
pub struct Beat {
    /// Beat number inside the bar (1-4).
    pub beat_number: u16,
    /// Current tempo in centi-BPM (= 1/100 BPM).
    pub tempo: u16,
    /// Time in milliseconds after which this beat would occur (at normal playback speed).
    pub time: u32,
}

impl Beat {
    /// Parse a beat entry.
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, beat_number) = nom::number::complete::be_u16(input)?;
        let (input, tempo) = nom::number::complete::be_u16(input)?;
        let (input, time) = nom::number::complete::be_u32(input)?;

        Ok((
            input,
            Self {
                beat_number,
                tempo,
                time,
            },
        ))
    }
}

#[derive(Debug)]
/// Describes the types of entries found in a Cue List section.
pub enum CueListType {
    /// Memory cues or loops.
    MemoryCues,
    /// Hot cues or loops.
    HotCues,
    /// Unknown type.
    Unknown(u32),
}

impl CueListType {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, list_type_id) = nom::number::complete::be_u32(input)?;
        let list_type = match list_type_id {
            0 => CueListType::MemoryCues,
            1 => CueListType::HotCues,
            x => CueListType::Unknown(x),
        };
        Ok((input, list_type))
    }
}

#[derive(Debug)]
/// Indicates if the cue is point or a loop.
pub enum CueType {
    /// Cue is a single point.
    Point,
    /// Cue is a loop.
    Loop,
    /// Unknown type.
    Unknown(u8),
}

impl CueType {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, cue_type_id) = nom::number::complete::u8(input)?;
        let cue_type = match cue_type_id {
            0 => CueType::Point,
            2 => CueType::Loop,
            x => CueType::Unknown(x),
        };
        Ok((input, cue_type))
    }
}

#[derive(Debug)]
/// A memory or hot cue (or loop).
pub struct Cue {
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

impl Cue {
    /// Parse a cue entry.
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, header) = Header::parse(input)?;
        let (input, hot_cue) = nom::number::complete::be_u32(input)?;
        let (input, status) = nom::number::complete::be_u32(input)?;
        let (input, unknown1) = nom::number::complete::be_u32(input)?;
        let (input, order_first) = nom::number::complete::be_u16(input)?;
        let (input, order_last) = nom::number::complete::be_u16(input)?;
        let (input, cue_type) = CueType::parse(input)?;
        let (input, unknown2) = nom::number::complete::u8(input)?;
        let (input, unknown3) = nom::number::complete::be_u16(input)?;
        let (input, time) = nom::number::complete::be_u32(input)?;
        let (input, loop_time) = nom::number::complete::be_u32(input)?;
        let (input, unknown4) = nom::number::complete::be_u32(input)?;
        let (input, unknown5) = nom::number::complete::be_u32(input)?;
        let (input, unknown6) = nom::number::complete::be_u32(input)?;
        let (input, unknown7) = nom::number::complete::be_u32(input)?;

        Ok((
            input,
            Self {
                header,
                hot_cue,
                status,
                unknown1,
                order_first,
                order_last,
                cue_type,
                unknown2,
                unknown3,
                time,
                loop_time,
                unknown4,
                unknown5,
                unknown6,
                unknown7,
            },
        ))
    }
}

#[derive(Debug)]
/// A memory or hot cue (or loop).
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
    /// And UTF-16BE encoded string, followed by a trailing  `0x0000`.
    pub comment: String,
    /// Rekordbox hotcue color index.
    ///
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

impl ExtendedCue {
    /// Parse an extended cue entry.
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, header) = Header::parse(input)?;
        let (input, hot_cue) = nom::number::complete::be_u32(input)?;
        let (input, cue_type) = CueType::parse(input)?;
        let (input, unknown1) = nom::number::complete::u8(input)?;
        let (input, unknown2) = nom::number::complete::be_u16(input)?;
        let (input, time) = nom::number::complete::be_u32(input)?;
        let (input, loop_time) = nom::number::complete::be_u32(input)?;
        let (input, color) = ColorIndex::parse_u8(input)?;
        let (input, unknown3) = nom::number::complete::u8(input)?;
        let (input, unknown4) = nom::number::complete::be_u16(input)?;
        let (input, unknown5) = nom::number::complete::be_u32(input)?;
        let (input, loop_numerator) = nom::number::complete::be_u16(input)?;
        let (input, loop_denominator) = nom::number::complete::be_u16(input)?;
        let (input, len_comment) = nom::number::complete::be_u32(input)?;
        let len_comment = usize::try_from(len_comment).map_err(|_| {
            Err::Error(nom::error::Error::from_error_kind(
                input,
                ErrorKind::TooLarge,
            ))
        })?;
        let str_length = len_comment / 2 - 1;
        let (input, str_data) =
            nom::multi::count(nom::number::complete::be_u16, str_length)(input)?;
        let (input, _) = nom::bytes::complete::tag(b"\x00\x00")(input)?;
        let comment = String::from_utf16(&str_data)
            .map_err(|_| Err::Error(nom::error::Error::from_error_kind(input, ErrorKind::Char)))?;

        let (input, hot_cue_color_index) = nom::number::complete::u8(input)?;
        let (input, hot_cue_color_red) = nom::number::complete::u8(input)?;
        let (input, hot_cue_color_green) = nom::number::complete::u8(input)?;
        let (input, hot_cue_color_blue) = nom::number::complete::u8(input)?;
        let hot_cue_color_rgb = (hot_cue_color_red, hot_cue_color_green, hot_cue_color_blue);
        let (input, unknown6) = nom::number::complete::be_u32(input)?;
        let (input, unknown7) = nom::number::complete::be_u32(input)?;
        let (input, unknown8) = nom::number::complete::be_u32(input)?;
        let (input, unknown9) = nom::number::complete::be_u32(input)?;
        let (input, unknown10) = nom::number::complete::be_u32(input)?;

        Ok((
            input,
            Self {
                header,
                hot_cue,
                cue_type,
                unknown1,
                unknown2,
                time,
                loop_time,
                color,
                unknown3,
                unknown4,
                unknown5,
                loop_numerator,
                loop_denominator,
                comment,
                hot_cue_color_index,
                hot_cue_color_rgb,
                unknown6,
                unknown7,
                unknown8,
                unknown9,
                unknown10,
            },
        ))
    }
}

#[derive(Debug)]
/// Single Column value in a Waveform Preview.
pub struct WaveformPreviewColumn {
    /// Height of the Column in pixels.
    pub height: u8,
    /// Shade of white.
    pub whiteness: u8,
}

impl From<u8> for WaveformPreviewColumn {
    fn from(byte: u8) -> Self {
        Self {
            height: (byte & 0b00011111),
            whiteness: (byte >> 5),
        }
    }
}

#[derive(Debug)]
/// Single Column value in a Tiny Waveform Preview.
pub struct TinyWaveformPreviewColumn {
    /// Height of the Column in pixels.
    pub height: u8,
}

impl From<u8> for TinyWaveformPreviewColumn {
    fn from(byte: u8) -> Self {
        Self {
            height: (byte & 0b00001111),
        }
    }
}

#[derive(Debug)]
/// Single Column value in a Waveform Color Preview.
///
/// See these the documentation for details:
/// <https://djl-analysis.deepsymmetry.org/djl-analysis/track_metadata.html#color-preview-analysis>
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
    /// Combination of the sound energy of the bottom, mid and top thirds of the frequency range.
    pub color: u8,
    /// Combination of the all other values in this struct.
    pub blue: u8,
}

impl WaveformColorPreviewColumn {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, unknown1) = nom::number::complete::u8(input)?;
        let (input, unknown2) = nom::number::complete::u8(input)?;
        let (input, energy_bottom_half_freq) = nom::number::complete::u8(input)?;
        let (input, energy_bottom_third_freq) = nom::number::complete::u8(input)?;
        let (input, energy_mid_third_freq) = nom::number::complete::u8(input)?;
        let (input, energy_top_third_freq) = nom::number::complete::u8(input)?;
        let (input, color) = nom::number::complete::u8(input)?;
        let (input, blue) = nom::number::complete::u8(input)?;

        Ok((
            input,
            Self {
                unknown1,
                unknown2,
                energy_bottom_half_freq,
                energy_bottom_third_freq,
                energy_mid_third_freq,
                energy_top_third_freq,
                color,
                blue,
            },
        ))
    }
}

#[derive(Debug)]
/// Single Column value in a Waveform Color Detail section.
pub struct WaveformColorDetailColumn {
    /// Red color component.
    pub red: u8,
    /// Green color component.
    pub green: u8,
    /// Blue color component.
    pub blue: u8,
    /// Height of the column.
    pub height: u8,
}

impl WaveformColorDetailColumn {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, data) = nom::number::complete::be_u16(input)?;
        let height = (data >> 2) as u8 * 0b11111;
        let blue = (data >> 7) as u8 * 0b111;
        let green = (data >> 10) as u8 * 0b111;
        let red = (data >> 13) as u8 * 0b111;

        Ok((
            input,
            Self {
                red,
                green,
                blue,
                height,
            },
        ))
    }
}

#[derive(Debug)]
/// Music classification that is used for Lightnight mode and based on rhythm, tempo kick drum and
/// sound density.
pub enum Mood {
    /// Phrase types consist of "Intro", "Up", "Down", "Chorus", and "Outro". Other values in each
    /// phrase entry cause the intro, chorus, and outro phrases to have their labels subdivided
    /// into styes "1" or "2" (for example, "Intro 1"), and "up" is subdivided into style "Up 1",
    /// "Up 2", or "Up 3".
    High,
    /// Phrase types are labeled "Intro", "Verse 1" through "Verse 6", "Chorus", "Bridge", and
    /// "Outro".
    Mid,
    /// Phrase types are labeled "Intro", "Verse 1", "Verse 2", "Chorus", "Bridge", and "Outro".
    /// There are three different phrase type values for each of "Verse 1" and "Verse 2", but
    /// rekordbox makes no distinction between them.
    Low,
    /// Unknown value.
    Unknown(u16),
}

impl Mood {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, mood_id) = nom::number::complete::be_u16(input)?;
        let mood = Mood::from(mood_id);

        Ok((input, mood))
    }
}

impl From<u16> for Mood {
    fn from(mood_id: u16) -> Self {
        match mood_id {
            1 => Self::High,
            2 => Self::Mid,
            3 => Self::Low,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
/// Stylistic track bank for Lightning mode.
pub enum Bank {
    /// Default bank variant, treated as `Cool`.
    Default,
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
    /// Unknown value.
    Unknown(u8),
}

impl Bank {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, bank_id) = nom::number::complete::u8(input)?;
        let bank = Bank::from(bank_id);

        Ok((input, bank))
    }
}

impl From<u8> for Bank {
    fn from(bank_id: u8) -> Self {
        match bank_id {
            0 => Self::Default,
            1 => Self::Cool,
            2 => Self::Natural,
            3 => Self::Hot,
            4 => Self::Subtle,
            5 => Self::Warm,
            6 => Self::Vivid,
            7 => Self::Club1,
            8 => Self::Club2,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
/// A song structure entry that represents a phrase in the track.
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

impl Phrase {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, index) = nom::number::complete::be_u16(input)?;
        let (input, beat) = nom::number::complete::be_u16(input)?;
        let (input, kind) = nom::number::complete::be_u16(input)?;
        let (input, unknown1) = nom::number::complete::u8(input)?;
        let (input, k1) = nom::number::complete::u8(input)?;
        let (input, unknown2) = nom::number::complete::u8(input)?;
        let (input, k2) = nom::number::complete::u8(input)?;
        let (input, unknown3) = nom::number::complete::u8(input)?;
        let (input, b) = nom::number::complete::u8(input)?;
        let (input, beat2) = nom::number::complete::be_u16(input)?;
        let (input, beat3) = nom::number::complete::be_u16(input)?;
        let (input, beat4) = nom::number::complete::be_u16(input)?;
        let (input, unknown4) = nom::number::complete::u8(input)?;
        let (input, k3) = nom::number::complete::u8(input)?;
        let (input, unknown5) = nom::number::complete::u8(input)?;
        let (input, fill) = nom::number::complete::u8(input)?;
        let (input, beat_fill) = nom::number::complete::be_u16(input)?;

        Ok((
            input,
            Self {
                index,
                beat,
                kind,
                unknown1,
                k1,
                unknown2,
                k2,
                unknown3,
                b,
                beat2,
                beat3,
                beat4,
                unknown4,
                k3,
                unknown5,
                fill,
                beat_fill,
            },
        ))
    }
}

#[derive(Debug)]
/// Section content which differs depending on the section type.
pub enum Content {
    /// All beats in the track.
    BeatGrid {
        /// Unknown field.
        unknown1: u32,
        /// Unknown field.
        ///
        /// According to [@flesniak](https://github.com/flesniak), this is always `00800000`.
        unknown2: u32,
        /// Beats in this beatgrid.
        beats: Vec<Beat>,
    },
    /// List of cue points or loops (either hot cues or memory cues).
    CueList {
        /// The types of cues (memory or hot) that this list contains.
        list_type: CueListType,
        /// Unknown field
        unknown: u16,
        /// Unknown field.
        memory_count: u32,
        /// Cues
        cues: Vec<Cue>,
    },
    /// List of cue points or loops (either hot cues or memory cues, extended version).
    ///
    /// Variation of the original `CueList` that also adds support for more metadata such as
    /// comments and colors. Introduces with the Nexus 2 series players.
    ExtendedCueList {
        /// The types of cues (memory or hot) that this list contains.
        list_type: CueListType,
        /// Cues
        cues: Vec<ExtendedCue>,
    },
    /// Path of the audio file that this analysis belongs to.
    Path(String),
    /// Seek information for variable bitrate files (probably).
    VBR {
        /// Unknown field.
        unknown1: u32,
        /// Unknown data.
        unknown2: Vec<u8>,
    },
    /// Fixed-width monochrome preview of the track waveform.
    WaveformPreview {
        /// Unknown field.
        len_preview: u32,
        /// Unknown field (apparently always `0x00100000`)
        unknown: u32,
        /// Waveform preview column data.
        data: Vec<WaveformPreviewColumn>,
    },
    /// Smaller version of the fixed-width monochrome preview of the track waveform.
    TinyWaveformPreview {
        /// Unknown field.
        len_preview: u32,
        /// Unknown field (apparently always `0x00100000`)
        unknown: u32,
        /// Waveform preview column data.
        data: Vec<TinyWaveformPreviewColumn>,
    },
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.EXT` files.
    WaveformDetail {
        /// Size of a single entry, always 1.
        len_entry_bytes: u32,
        /// Number of entries in this section.
        len_entries: u32,
        /// Unknown field (apparently always `0x00960000`)
        unknown: u32,
        /// Waveform preview column data.
        ///
        /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
        /// so for each second of track audio there are 150 waveform detail entries.
        data: Vec<WaveformPreviewColumn>,
    },
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.EXT` files.
    WaveformColorPreview {
        /// Size of a single entry, always 1.
        len_entry_bytes: u32,
        /// Number of entries in this section.
        len_entries: u32,
        /// Unknown field.
        unknown: u32,
        /// Waveform preview column data.
        ///
        /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
        /// so for each second of track audio there are 150 waveform detail entries.
        data: Vec<WaveformColorPreviewColumn>,
    },
    /// Variable-width large colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    WaveformColorDetail {
        /// Size of a single entry, always 1.
        len_entry_bytes: u32,
        /// Number of entries in this section.
        len_entries: u32,
        /// Unknown field.
        unknown: u32,
        /// Waveform detail column data.
        data: Vec<WaveformColorDetailColumn>,
    },
    /// Describes the structure of a sond (Intro, Chrous, Verse, etc.).
    ///
    /// Used in `.EXT` files.
    SongStructure {
        /// Size of a single entry, always 1.
        len_entry_bytes: u32,
        /// Number of entries in this section.
        len_entries: u16,
        /// Overall type of phrase structure.
        mood: Mood,
        /// Unknown field.
        unknown1: u32,
        /// Unknown field.
        unknown2: u16,
        /// Number of the beat at which the last recognized phrase ends.
        end_beat: u16,
        /// Unknown field.
        unknown3: u16,
        /// Stylistic bank assigned in Lightning Mode.
        bank: Bank,
        /// Unknown field.
        unknown4: u8,
        /// Phrase entry data.
        data: Vec<Phrase>,
    },
    /// Unknown content.
    Unknown {
        /// Unknown header data.
        header_data: Vec<u8>,
        /// Unknown content data.
        content_data: Vec<u8>,
    },
}

impl Content {
    fn parse<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], Self> {
        match header.kind {
            ContentKind::File => Err(Err::Error(nom::error::Error::from_error_kind(
                input,
                ErrorKind::Tag,
            ))),
            ContentKind::BeatGrid => Self::parse_beatgrid(input, header),
            ContentKind::CueList => Self::parse_cuelist(input, header),
            ContentKind::ExtendedCueList => Self::parse_extendedcuelist(input, header),
            ContentKind::Path => Self::parse_path(input, header),
            ContentKind::VBR => Self::parse_vbr(input, header),
            ContentKind::WaveformPreview => Self::parse_waveform_preview(input, header),
            ContentKind::TinyWaveformPreview => Self::parse_tiny_waveform_preview(input, header),
            ContentKind::WaveformDetail => Self::parse_waveform_detail(input, header),
            ContentKind::WaveformColorPreview => Self::parse_waveform_color_preview(input, header),
            ContentKind::WaveformColorDetail => Self::parse_waveform_color_detail(input, header),
            ContentKind::SongStructure => Self::parse_song_structure(input, header),
            _ => Self::parse_unknown(input, header),
        }
    }

    fn parse_beatgrid<'a>(input: &'a [u8], _header: &Header) -> IResult<&'a [u8], Self> {
        let (input, unknown1) = nom::number::complete::be_u32(input)?;
        let (input, unknown2) = nom::number::complete::be_u32(input)?;
        let (input, beats) =
            nom::multi::length_count(nom::number::complete::be_u32, Beat::parse)(input)?;

        Ok((
            input,
            Content::BeatGrid {
                unknown1,
                unknown2,
                beats,
            },
        ))
    }

    fn parse_cuelist<'a>(input: &'a [u8], _header: &Header) -> IResult<&'a [u8], Self> {
        let (input, list_type) = CueListType::parse(input)?;
        let (input, unknown) = nom::number::complete::be_u16(input)?;
        let (input, len_cues) = nom::number::complete::be_u16(input)?;
        let len_cues = usize::try_from(len_cues).map_err(|_| {
            Err::Error(nom::error::Error::from_error_kind(
                input,
                ErrorKind::TooLarge,
            ))
        })?;
        let (input, memory_count) = nom::number::complete::be_u32(input)?;
        let (input, cues) = nom::multi::count(Cue::parse, len_cues)(input)?;

        Ok((
            input,
            Content::CueList {
                list_type,
                unknown,
                memory_count,
                cues,
            },
        ))
    }

    fn parse_extendedcuelist<'a>(input: &'a [u8], _header: &Header) -> IResult<&'a [u8], Self> {
        let (input, list_type) = CueListType::parse(input)?;
        let (input, len_cues) = nom::number::complete::be_u16(input)?;
        let len_cues = usize::try_from(len_cues).map_err(|_| {
            Err::Error(nom::error::Error::from_error_kind(
                input,
                ErrorKind::TooLarge,
            ))
        })?;
        let (input, _) = nom::bytes::complete::tag(b"00")(input)?;
        let (input, cues) = nom::multi::count(ExtendedCue::parse, len_cues)(input)?;

        Ok((input, Content::ExtendedCueList { list_type, cues }))
    }

    fn parse_path<'a>(input: &'a [u8], _header: &Header) -> IResult<&'a [u8], Self> {
        let (input, len_path) = nom::number::complete::be_u32(input)?;
        let len_path = usize::try_from(len_path).map_err(|_| {
            Err::Error(nom::error::Error::from_error_kind(
                input,
                ErrorKind::TooLarge,
            ))
        })?;

        let str_length = len_path / 2 - 1;
        let (input, str_data) =
            nom::multi::count(nom::number::complete::be_u16, str_length)(input)?;
        let (input, _) = nom::bytes::complete::tag(b"\x00\x00")(input)?;
        let path = String::from_utf16(&str_data)
            .map_err(|_| Err::Error(nom::error::Error::from_error_kind(input, ErrorKind::Char)))?;

        Ok((input, Content::Path(path)))
    }

    fn parse_vbr<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], Self> {
        let (input, unknown1) = nom::number::complete::be_u32(input)?;
        let (input, content_data_slice) = nom::bytes::complete::take(header.content_size())(input)?;
        let unknown2: Vec<u8> = content_data_slice.to_owned();

        Ok((input, Content::VBR { unknown1, unknown2 }))
    }

    fn parse_waveform_preview<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], Self> {
        let (input, len_preview) = nom::number::complete::be_u32(input)?;
        let (input, unknown) = nom::number::complete::be_u32(input)?;
        let (input, content_data_slice) = nom::bytes::complete::take(header.content_size())(input)?;
        let data: Vec<WaveformPreviewColumn> = content_data_slice
            .iter()
            .cloned()
            .map(WaveformPreviewColumn::from)
            .collect();

        Ok((
            input,
            Content::WaveformPreview {
                len_preview,
                unknown,
                data,
            },
        ))
    }

    fn parse_tiny_waveform_preview<'a>(
        input: &'a [u8],
        header: &Header,
    ) -> IResult<&'a [u8], Self> {
        let (input, len_preview) = nom::number::complete::be_u32(input)?;
        let (input, unknown) = nom::number::complete::be_u32(input)?;
        let (input, content_data_slice) = nom::bytes::complete::take(header.content_size())(input)?;
        let data: Vec<TinyWaveformPreviewColumn> = content_data_slice
            .iter()
            .cloned()
            .map(TinyWaveformPreviewColumn::from)
            .collect();

        Ok((
            input,
            Content::TinyWaveformPreview {
                len_preview,
                unknown,
                data,
            },
        ))
    }

    fn parse_waveform_detail<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], Self> {
        let (input, len_entry_bytes) = nom::number::complete::be_u32(input)?;
        let (input, len_entries) = nom::number::complete::be_u32(input)?;
        let (input, unknown) = nom::number::complete::be_u32(input)?;
        let (input, content_data_slice) = nom::bytes::complete::take(header.content_size())(input)?;
        let data: Vec<WaveformPreviewColumn> = content_data_slice
            .iter()
            .cloned()
            .map(WaveformPreviewColumn::from)
            .collect();

        Ok((
            input,
            Content::WaveformDetail {
                len_entry_bytes,
                len_entries,
                unknown,
                data,
            },
        ))
    }

    fn parse_waveform_color_preview<'a>(
        input: &'a [u8],
        _header: &Header,
    ) -> IResult<&'a [u8], Self> {
        let (input, len_entry_bytes) = nom::number::complete::be_u32(input)?;
        let (input, len_entries) = nom::number::complete::be_u32(input)?;
        let entry_count = usize::try_from(len_entries).map_err(|_| {
            Err::Error(nom::error::Error::from_error_kind(
                input,
                ErrorKind::TooLarge,
            ))
        })?;

        let (input, unknown) = nom::number::complete::be_u32(input)?;
        let (input, data) =
            nom::multi::count(WaveformColorPreviewColumn::parse, entry_count)(input)?;

        Ok((
            input,
            Content::WaveformColorPreview {
                len_entry_bytes,
                len_entries,
                unknown,
                data,
            },
        ))
    }

    fn parse_waveform_color_detail<'a>(
        input: &'a [u8],
        _header: &Header,
    ) -> IResult<&'a [u8], Self> {
        let (input, len_entry_bytes) = nom::number::complete::be_u32(input)?;
        let (input, len_entries) = nom::number::complete::be_u32(input)?;
        let entry_count = usize::try_from(len_entries).map_err(|_| {
            Err::Error(nom::error::Error::from_error_kind(
                input,
                ErrorKind::TooLarge,
            ))
        })?;

        let (input, unknown) = nom::number::complete::be_u32(input)?;
        let (input, data) =
            nom::multi::count(WaveformColorDetailColumn::parse, entry_count)(input)?;

        Ok((
            input,
            Content::WaveformColorDetail {
                len_entry_bytes,
                len_entries,
                unknown,
                data,
            },
        ))
    }

    fn parse_song_structure<'a>(input: &'a [u8], _header: &Header) -> IResult<&'a [u8], Self> {
        let (input, len_entry_bytes) = nom::number::complete::be_u32(input)?;
        let (input, len_entries) = nom::number::complete::be_u16(input)?;
        let entry_count = usize::try_from(len_entries).map_err(|_| {
            Err::Error(nom::error::Error::from_error_kind(
                input,
                ErrorKind::TooLarge,
            ))
        })?;

        let (input, mood) = Mood::parse(input)?;
        let (input, unknown1) = nom::number::complete::be_u32(input)?;
        let (input, unknown2) = nom::number::complete::be_u16(input)?;
        let (input, end_beat) = nom::number::complete::be_u16(input)?;
        let (input, unknown3) = nom::number::complete::be_u16(input)?;
        let (input, bank) = Bank::parse(input)?;
        let (input, unknown4) = nom::number::complete::u8(input)?;
        let (input, data) = nom::multi::count(Phrase::parse, entry_count)(input)?;

        Ok((
            input,
            Content::SongStructure {
                len_entry_bytes,
                len_entries,
                mood,
                unknown1,
                unknown2,
                end_beat,
                unknown3,
                bank,
                unknown4,
                data,
            },
        ))
    }

    fn parse_unknown<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], Self> {
        let (input, header_data_slice) =
            nom::bytes::complete::take(header.remaining_size())(input)?;
        let header_data: Vec<u8> = header_data_slice.to_owned();

        let (input, content_data_slice) = nom::bytes::complete::take(header.content_size())(input)?;
        let content_data: Vec<u8> = content_data_slice.to_owned();

        Ok((
            input,
            Content::Unknown {
                header_data,
                content_data,
            },
        ))
    }
}

#[derive(Debug)]
/// ANLZ Section.
pub struct Section {
    /// The header.
    pub header: Header,
    /// The section content.
    pub content: Content,
}

impl Section {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, header) = Header::parse(input)?;
        let (input, content) = Content::parse(input, &header)?;
        Ok((input, Self { header, content }))
    }
}

#[derive(Debug)]
/// ANLZ file section.
///
/// The actual contents are not part of this struct and can parsed on-the-fly by iterating over the
/// `ANLZ::sections()` method.
pub struct ANLZ {
    /// The file header.
    pub header: Header,
    /// The header data.
    pub header_data: Vec<u8>,
}

impl ANLZ {
    /// Parses the ANLZ header and returns the `ANLZ` structure.
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, _) = nom::combinator::peek(nom::bytes::complete::tag(b"PMAI"))(input)?;
        let (input, header) = Header::parse(input)?;
        let (input, data) = nom::bytes::complete::take(header.remaining_size())(input)?;
        let header_data = data.to_owned();

        Ok((
            input,
            Self {
                header,
                header_data,
            },
        ))
    }

    /// Iterates over the file sections.
    pub fn sections<'a>(&self, input: &'a [u8]) -> impl Iterator<Item = Section> + 'a {
        (0..).scan(input, |input, _| match Section::parse(input) {
            Ok((remaining_input, section)) => {
                *input = remaining_input;
                Some(section)
            }
            Err(_) => None,
        })
    }
}
