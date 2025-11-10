// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Common types used in multiple modules.

use crate::pdb::string::StringError;
use binrw::binrw;
use thiserror::Error;

/// Enumerates errors returned by this library.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum RekordcrateError {
    /// Represents a failure to decode a DeviceSQL string.
    #[error(transparent)]
    StringError(#[from] StringError),

    /// Represents a failure to parse input.
    #[error(transparent)]
    ParseError(#[from] binrw::Error),

    /// Represents an `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// Represents an `quick_xml::DeError`.
    #[cfg(feature = "xml")]
    #[error(transparent)]
    XmlDeserializationFailed(#[from] quick_xml::DeError),
}

/// Type alias for results where the error is a `RekordcrateError`.
pub type RekordcrateResult<T> = std::result::Result<T, RekordcrateError>;

/// Indexed Color identifiers used for memory cues and tracks.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ColorIndex {
    /// No color.
    #[brw(magic = 0u8)]
    None,
    /// Pink color.
    #[brw(magic = 1u8)]
    Pink,
    /// Red color.
    #[brw(magic = 2u8)]
    Red,
    /// Orange color.
    #[brw(magic = 3u8)]
    Orange,
    /// Yellow color.
    #[brw(magic = 4u8)]
    Yellow,
    /// Green color.
    #[brw(magic = 5u8)]
    Green,
    /// Aqua color.
    #[brw(magic = 6u8)]
    Aqua,
    /// Blue color.
    #[brw(magic = 7u8)]
    Blue,
    /// Purple color.
    #[brw(magic = 8u8)]
    Purple,
}

/// Track file type.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FileType {
    /// Unknown file type.
    #[brw(magic = 0x0u16)]
    Unknown,
    /// MP3.
    #[brw(magic = 0x1u16)]
    Mp3,
    /// M4A.
    #[brw(magic = 0x4u16)]
    M4a,
    /// FLAC.
    #[brw(magic = 0x5u16)]
    Flac,
    /// WAV.
    #[brw(magic = 0xbu16)]
    Wav,
    /// AIFF.
    #[brw(magic = 0xcu16)]
    Aiff,
    /// Value that we haven't seen before.
    Other(u16),
}

/// align given value to the alignment requirements by the given type.
#[must_use]
pub const fn align_by(alignment: u64, mut offset: u64) -> u64 {
    // This is technically dependent on the compile time ABI
    // but for x86 (which this is likely compiled on), we should be able
    // to assume that
    // the alignment of a type is just the size of its largest
    // member. That likely matches the assumptions made for the 32-bit
    // MCU (Renesas R8A77240D500BG) built into the different CDJ-2000 variants.
    // In either way, its better to overshoot the alignment
    // than to undershoot it. For CDJ-3000s, this assumption
    // is likely also correct since they use a 64-bit ARM CPU (Renesas R8A774C0HA01BG)
    if !offset.is_multiple_of(alignment) {
        offset += alignment - (offset % alignment);
    }
    offset
}

#[binrw(little)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[br(import(limit: usize))]
/// Represents explicit padding bytes in a binary structure.
pub struct ExplicitPadding(
    // use offset + a couple bytes as a limit (though this is rather a heuristic)
    #[br(parse_with = Self::guess_padding, args(limit))]
    #[bw(ignore)]
    #[bw(pad_after=self.0)]
    pub usize,
);

use binrw::{io::Read, BinResult};
impl ExplicitPadding {
    #[binrw::parser(reader)]
    fn guess_padding(limit: usize) -> BinResult<usize> {
        let before = reader.stream_position()?;
        let mut count = 0;
        // We never expect to read many bytes here, so this is fine
        #[allow(clippy::unbuffered_bytes)]
        for byte in reader.bytes() {
            if byte? != 0 {
                break;
            }
            // to avoid scanning an entire page after the last row,
            // we limit here
            // if the limit is reached, assume end of page and no padding
            // used
            if limit != 0 && count == limit {
                count = 0;
                break;
            }
            count += 1;
        }
        if reader.stream_position()? != before {
            // don't consume the non-zero byte we just read if we read anything at all
            reader.seek_relative(-1)?;
        }
        Ok(count)
    }
}

impl binrw::meta::ReadEndian for ExplicitPadding {
    const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::None;
}
impl binrw::meta::WriteEndian for ExplicitPadding {
    const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::None;
}

impl From<usize> for ExplicitPadding {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use binrw::{
        meta::{ReadEndian, WriteEndian},
        prelude::*,
        Endian,
    };
    use pretty_assertions::assert_eq;
    use pretty_hex::pretty_hex;

    macro_rules! assert_eq_hex {
        ($cond:expr, $expected:expr) => {
            assert_eq!(pretty_hex($cond), pretty_hex($expected));
        };
    }

    pub fn test_roundtrip_with_args<'a, T>(
        bin: &[u8],
        obj: T,
        read_args: <T as binrw::BinRead>::Args<'a>,
        write_args: <T as binrw::BinWrite>::Args<'a>,
    ) where
        <T as binrw::BinRead>::Args<'a>: Clone,
        <T as binrw::BinWrite>::Args<'a>: Clone,
        T: BinRead + BinWrite + PartialEq + core::fmt::Debug + ReadEndian + WriteEndian,
    {
        let endian = Endian::NATIVE;
        // T->binary
        let mut writer = binrw::io::Cursor::new(Vec::with_capacity(bin.len()));
        obj.write_options(&mut writer, endian, write_args.clone())
            .unwrap();
        assert_eq_hex!(&writer.get_ref(), &bin);
        // T->binary->T
        writer.set_position(0);
        let parsed = T::read_options(&mut writer, endian, read_args.clone()).unwrap();
        assert_eq!(parsed, obj);
        // binary->T
        let mut cursor = binrw::io::Cursor::new(bin);
        let parsed = T::read_options(&mut cursor, endian, read_args.clone()).unwrap();
        assert_eq!(parsed, obj);
        // binary->T->binary
        writer.set_position(0);
        parsed
            .write_options(&mut writer, endian, write_args.clone())
            .unwrap();
        assert_eq_hex!(&bin, &writer.get_ref());
    }

    pub fn test_roundtrip<'a, T>(bin: &[u8], obj: T)
    where
        <T as binrw::BinRead>::Args<'a>: Default + Clone,
        <T as binrw::BinWrite>::Args<'a>: Default + Clone,
        T: BinRead + BinWrite + PartialEq + core::fmt::Debug + ReadEndian + WriteEndian,
    {
        test_roundtrip_with_args(
            bin,
            obj,
            <T as binrw::BinRead>::Args::default(),
            <T as binrw::BinWrite>::Args::default(),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::testing::{test_roundtrip, test_roundtrip_with_args};
    mod explicit_padding {
        use super::*;

        #[test]
        fn empty() {
            test_roundtrip(&[], ExplicitPadding::default());
            test_roundtrip(&[], ExplicitPadding(0));
            test_roundtrip_with_args(&[], ExplicitPadding(0), (0,), ());
        }
        #[test]
        fn limit() {
            test_roundtrip_with_args(&[0x00], ExplicitPadding(1), (1,), ());
        }

        #[binrw(little)]
        #[brw(little)]
        #[derive(Debug, PartialEq, Clone)]
        #[br(import(limit: usize))]
        struct Something(u8, #[br(args(limit))] ExplicitPadding);
        #[test]
        fn non_empty() {
            test_roundtrip(&[0x00, 0x00], ExplicitPadding(2));

            let smth = Something(1, ExplicitPadding(2));
            test_roundtrip_with_args(&[0x01, 0x00, 0x00], smth, (0,), ());
        }
        #[test]
        fn multiple() {
            use binrw::VecArgs;
            let multiple = vec![Something(1, ExplicitPadding(1)); 2];
            test_roundtrip_with_args(
                &[0x01, 0x00, 0x01, 0x00],
                multiple,
                VecArgs {
                    count: 2,
                    inner: (0,),
                },
                (),
            );
        }
    }
}
