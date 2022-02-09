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

        let str_length = usize::try_from(len_comment).unwrap() / 2 - 1;
        let (input, str_data) =
            nom::multi::count(nom::number::complete::be_u16, str_length)(input)?;
        let (input, _) = nom::bytes::complete::tag(b"\x00\x00")(input)?;
        let comment = match String::from_utf16(&str_data) {
            Ok(x) => x,
            Err(_) => {
                return Err(Err::Error(nom::error::Error::from_error_kind(
                    input,
                    ErrorKind::Char,
                )));
            }
        };

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
        let (input, memory_count) = nom::number::complete::be_u32(input)?;
        let (input, cues) = nom::multi::count(Cue::parse, len_cues.try_into().unwrap())(input)?;

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
        let (input, _) = nom::bytes::complete::tag(b"00")(input)?;
        let (input, cues) =
            nom::multi::count(ExtendedCue::parse, len_cues.try_into().unwrap())(input)?;

        Ok((input, Content::ExtendedCueList { list_type, cues }))
    }

    fn parse_path<'a>(input: &'a [u8], _header: &Header) -> IResult<&'a [u8], Self> {
        let (input, len_path) = nom::number::complete::be_u32(input)?;
        let str_length = usize::try_from(len_path).unwrap() / 2 - 1;
        let (input, str_data) =
            nom::multi::count(nom::number::complete::be_u16, str_length)(input)?;
        let (input, _) = nom::bytes::complete::tag(b"\x00\x00")(input)?;
        let path = match String::from_utf16(&str_data) {
            Ok(x) => x,
            Err(_) => {
                return Err(Err::Error(nom::error::Error::from_error_kind(
                    input,
                    ErrorKind::Char,
                )));
            }
        };

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
