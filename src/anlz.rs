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
/// Section content which differs depending on the section type.
pub enum Content {
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
            _ => Self::parse_unknown(input, header),
        }
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
