// Copyright (c) 2022 Nikolaus Einhauser
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! `binrw`-based implementation for DeviceSQLStrings capable of parsing and
//! serializing [`DeviceSQLString`]s
//!
//! See <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html#devicesql-strings>

use binrw::{binrw, NullString};
use std::fmt;

const MAX_SHORTSTR_SIZE: usize = ((u8::MAX >> 1) - 1) as usize;

/// Error Objects occurring when dealing with [DeviceSQLString]'s
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[non_exhaustive]
pub enum StringError {
    /// String being handled was too long for DeviceSQL
    TooLong,
    /// Creating of ISRC String was unsuccessful because the string was not
    /// containing a valid ISRC string specifier
    InvalidISRC,
    /// String encoding Error (invalid UTF-8/16)
    Encoding,
}

/// Encapsulates the intrinsics of the format used by DeviceSQL strings
///
/// Once A [`DeviceSQLString`] has been constructed, there is no way to change it.
///
/// ```
/// # pub fn main() -> binrw::BinResult<()> {
/// use rekordcrate::pdb::string::DeviceSQLString;
/// use binrw::{BinWrite, BinRead};
/// let string = DeviceSQLString::new("foo".to_owned()).unwrap();
/// let binary = vec![0x9, 0x66, 0x6F, 0x6F];
///
/// let mut writer = binrw::io::Cursor::new(vec![]);
/// string.write_to(&mut writer)?;
/// assert_eq!(&binary, writer.get_ref());
///
/// let mut reader = binrw::io::Cursor::new(binary);
/// let parsed = DeviceSQLString::read(&mut reader)?;
/// assert_eq!(parsed, string);
/// # Ok(())
/// # }
/// ```
#[derive(PartialEq, Clone)]
#[binrw]
pub struct DeviceSQLString(DeviceSQLStringImpl);
impl DeviceSQLString {
    /// Initializes a [`DeviceSQLString`] from a plain Rust [`std::string::String`]
    pub fn new(string: String) -> Result<Self, StringError> {
        let len = string.len();
        let only_ascii = string.is_ascii();
        if only_ascii && len <= MAX_SHORTSTR_SIZE {
            Ok(Self(DeviceSQLStringImpl::ShortASCII {
                content: string.into_bytes(),
            }))
        } else if len <= (i16::MAX as usize) {
            if only_ascii {
                Ok(Self(DeviceSQLStringImpl::Long {
                    content: LongBody::Ascii(string.into_bytes()),
                }))
            } else {
                Ok(Self(DeviceSQLStringImpl::Long {
                    // note: The DeviceSQL database may only support UCS-2 so
                    // we might need to do some additional filtering here
                    content: LongBody::Ucs2le(string.encode_utf16().collect()),
                }))
            }
        } else {
            Err(StringError::TooLong)
        }
    }

    /// Creates a [`DeviceSQLString`] containing an ISRC instead of an expected string
    ///
    /// The DeviceSQL Database as used by rekordbox has a strange oddity
    /// where it serializes strings containing a tracks ISRC (International
    /// Standard Recording Code) in an unexpected format, if this is desired,
    /// use this constructor function instead of [`DeviceSQLString::new`].
    pub fn new_isrc(string: String) -> Result<Self, StringError> {
        if string.is_empty() {
            return Ok(Self::empty());
        }
        // basic validation taken from
        // https://isrc.ifpi.org/downloads/ISRC_Bulletin-2015-01.pdf
        if string.len() != 12 || !string.is_ascii() {
            return Err(StringError::InvalidISRC);
        }
        Ok(Self(DeviceSQLStringImpl::Long {
            content: LongBody::Isrc(NullString::from_string(string)),
        }))
    }

    /// Extract the Rust string from the DeviceSQLString.
    ///
    /// Consumes itself in the process.
    pub fn into_string(self) -> Result<String, StringError> {
        match self.0 {
            DeviceSQLStringImpl::ShortASCII { content: vec, .. }
            | DeviceSQLStringImpl::Long {
                content: LongBody::Ascii(vec),
                ..
            } => String::from_utf8(vec).map_err(|_| StringError::Encoding),
            DeviceSQLStringImpl::Long {
                content: LongBody::Isrc(str),
                ..
            } => str
                .into_string_lossless()
                .map_err(|_| StringError::Encoding),
            DeviceSQLStringImpl::Long {
                content: LongBody::Ucs2le(vec),
            } => String::from_utf16(&vec).map_err(|_| StringError::Encoding),
        }
    }

    /// Create an empty [`DeviceSQLString`].
    ///
    /// Should be used to construct known empty strings.
    #[must_use]
    pub const fn empty() -> Self {
        Self(DeviceSQLStringImpl::ShortASCII {
            content: Vec::new(),
        })
    }
}

impl fmt::Debug for DeviceSQLString {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self
            .clone()
            .into_string()
            .unwrap_or_else(|_| "<string error>".to_string());
        fmt.debug_tuple("DeviceSQLString").field(&value).finish()
    }
}

/// A String encapsulating how DeviceSQL VARCHAR's are structured
///
/// This implementation forces them to be immutable for now
#[binrw]
#[derive(Debug, PartialEq, Clone)]
enum DeviceSQLStringImpl {
    /// Short-String optimization case
    ShortASCII {
        // To differentiate between the two variants, we test the LSB of the
        // first byte (`header` here, `flags` in Long). If its set, the
        // string being parsed is a of the ShortASCII kind, if its not, its
        // the Long form.
        #[br(temp, assert(header & 0b1 == 1))]
        #[bw(calc = (((content.len() + 1) << 1) | 1) as u8)]
        header: u8,

        #[br(count = (header >> 1) - 1)]
        content: Vec<u8>,
    },
    /// Regular long form strings, containing possibly different encodings
    Long {
        #[br(temp)]
        #[br(assert(flags & 0b1 == 0))]
        #[bw(calc = content.flags())]
        flags: u8,

        #[br(temp)]
        #[bw(calc = content.byte_count().unwrap() + 4)]
        length: u16,

        #[brw(magic(0u8))] // padding
        #[br(args(flags, length - 4))]
        content: LongBody,
    },
}

#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[br(import(flags: u8, len: u16))]
enum LongBody {
    // Ordering is important otherwise, UCS2LE strings could be parsed
    // instead of the stricter ISRC

    // ISRC strings are a bug/flaw in pioneers implementation, this is technically
    // a semi-dirty workaround
    #[br(pre_assert(flags == 0x90))]
    Isrc(#[brw(magic = 0x3u8)] binrw::NullString),
    #[br(pre_assert(flags == 0x40))]
    Ascii(#[br(count = len)] Vec<u8>),
    #[br(pre_assert(flags == 0x90))]
    #[br(pre_assert(len % 2 == 0))]
    Ucs2le(#[br(count = len / 2)] Vec<u16>),
}

impl LongBody {
    pub fn byte_count(&self) -> Result<u16, StringError> {
        match self {
            // ISRC offset is compensating for trailing nullbyte + 0x3 magic byte.
            Self::Isrc(null_str) => null_str.len() + 2,
            Self::Ascii(buf) => buf.len(),
            Self::Ucs2le(buf) => buf.len() * 2,
        }
        .try_into()
        .map_err(|_| StringError::TooLong)
    }
    pub fn flags(&self) -> u8 {
        match self {
            Self::Ucs2le(_) | Self::Isrc(_) => 0x90,
            Self::Ascii(_) => 0x40,
        }
    }
}

impl Default for DeviceSQLString {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::testing::test_roundtrip;

    #[test]
    fn default_string() -> Result<(), StringError> {
        test_roundtrip(&[0x3], DeviceSQLString::default());
        Ok(())
    }

    #[test]
    fn short_ascii_string() -> Result<(), StringError> {
        test_roundtrip(
            &[0x9, 0x66, 0x6F, 0x6F],
            DeviceSQLString::new("foo".to_owned())?,
        );
        Ok(())
    }

    #[test]
    fn long_ascii_string() -> Result<(), StringError> {
        let long_string = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliqu";
        let long_string_serialized = [
            0x40, 0x83, 0x00, 0x00, 0x4C, 0x6F, 0x72, 0x65, 0x6D, 0x20, 0x69, 0x70, 0x73, 0x75,
            0x6D, 0x20, 0x64, 0x6F, 0x6C, 0x6F, 0x72, 0x20, 0x73, 0x69, 0x74, 0x20, 0x61, 0x6D,
            0x65, 0x74, 0x2C, 0x20, 0x63, 0x6F, 0x6E, 0x73, 0x65, 0x74, 0x65, 0x74, 0x75, 0x72,
            0x20, 0x73, 0x61, 0x64, 0x69, 0x70, 0x73, 0x63, 0x69, 0x6E, 0x67, 0x20, 0x65, 0x6C,
            0x69, 0x74, 0x72, 0x2C, 0x20, 0x73, 0x65, 0x64, 0x20, 0x64, 0x69, 0x61, 0x6D, 0x20,
            0x6E, 0x6F, 0x6E, 0x75, 0x6D, 0x79, 0x20, 0x65, 0x69, 0x72, 0x6D, 0x6F, 0x64, 0x20,
            0x74, 0x65, 0x6D, 0x70, 0x6F, 0x72, 0x20, 0x69, 0x6E, 0x76, 0x69, 0x64, 0x75, 0x6E,
            0x74, 0x20, 0x75, 0x74, 0x20, 0x6C, 0x61, 0x62, 0x6F, 0x72, 0x65, 0x20, 0x65, 0x74,
            0x20, 0x64, 0x6F, 0x6C, 0x6F, 0x72, 0x65, 0x20, 0x6D, 0x61, 0x67, 0x6E, 0x61, 0x20,
            0x61, 0x6C, 0x69, 0x71, 0x75,
        ];
        test_roundtrip(
            &long_string_serialized,
            DeviceSQLString::new(long_string.to_owned())?,
        );
        Ok(())
    }

    #[test]
    fn non_ascii() -> Result<(), StringError> {
        let serialized = [
            0x90, 0x14, 0x00, 0x00, 0x49, 0x00, 0x20, 0x00, 0x64, 0x27, 0x20, 0x00, 0x52, 0x00,
            0x75, 0x00, 0x73, 0x00, 0x74, 0x00,
        ];
        test_roundtrip(&serialized, DeviceSQLString::new("I ❤ Rust".to_string())?);
        Ok(())
    }

    #[test]
    fn too_long_string() {
        // construct super long string containing just "AAAAAAA"...
        const TOO_LARGE_STR_SIZE: usize = (u16::MAX as usize) + 1;
        const INIT_CHAR: char = 'A';
        const _: () = assert!(INIT_CHAR.is_ascii());
        const HUMONGOUS_ARRAY: [u8; TOO_LARGE_STR_SIZE] = [INIT_CHAR as u8; TOO_LARGE_STR_SIZE];

        // Since we already know that the string only contains ascii at compile time,
        // we could probably skip the validation, but that requires unsafe code which I'd consider overkill.
        let humongous_string = String::from_utf8(HUMONGOUS_ARRAY.to_vec()).unwrap();

        assert_eq!(
            DeviceSQLString::new(humongous_string).unwrap_err(),
            StringError::TooLong
        );
    }

    #[test]
    fn isrc_edge_case() -> Result<(), StringError> {
        let serialized = [
            0x90, 0x12, 0x00, 0x00, 0x03, 0x47, 0x42, 0x41, 0x59, 0x45, 0x36, 0x37, 0x30, 0x30,
            0x31, 0x34, 0x39, 0x00,
        ];
        test_roundtrip(
            &serialized,
            DeviceSQLString::new_isrc("GBAYE6700149".to_string())?,
        );
        test_roundtrip(&[0x3], DeviceSQLString::new_isrc("".to_string())?);

        assert_eq!(
            DeviceSQLString::new_isrc("non-conforming garbage".to_string()).unwrap_err(),
            StringError::InvalidISRC
        );

        Ok(())
    }
}
