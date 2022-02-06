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
