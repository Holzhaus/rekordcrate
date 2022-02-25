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

const MAX_SHORTSTR_SIZE: usize = ((u8::MAX >> 1) - 1) as usize;

/// Encapsulates the intrinsics of the format used by DeviceSQL strings
///
/// Once A [`DeviceSQLString`] has been constructed, there is no way to change it.
///
/// ```
/// # pub fn main() -> binrw::BinResult<()> {
/// use rekordcrate::pdb::string::DeviceSQLString;
/// use binrw::{BinWrite, BinRead};
/// let string = DeviceSQLString::new("foo".to_owned());
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
#[derive(Debug, PartialEq)]
#[binrw]
pub struct DeviceSQLString(DeviceSQLStringImpl);
impl DeviceSQLString {
    /// Initializes a [`DeviceSQLString`] from a plain Rust [`std::string::String`]
    #[must_use]
    pub fn new(string: String) -> Self {
        let len = string.len();
        let only_ascii = string.is_ascii();
        if only_ascii && len <= MAX_SHORTSTR_SIZE {
            Self(DeviceSQLStringImpl::ShortASCII {
                content: string.into_bytes(),
            })
        } else if len <= (i16::MAX as usize) {
            if only_ascii {
                Self(DeviceSQLStringImpl::Long {
                    content: LongBody::Ascii(string.into_bytes()),
                })
            } else {
                Self(DeviceSQLStringImpl::Long {
                    // note: The DeviceSQL database may only support UCS-2 so
                    // we might need to do some additional filtering here
                    content: LongBody::Ucs2le(string.encode_utf16().collect()),
                })
            }
        } else {
            todo!()
        }
    }

    /// Creates a [`DeviceSQLString`] containing an ISRC instead of an expected string
    ///
    /// The DeviceSQL Database as used by rekordbox has a strange oddity
    /// where it serializes strings containing a tracks ISRC (International
    /// Standard Recording Code) in an unexpected format, if this is desired,
    /// use this constructor function instead of [`DeviceSQLString::new`].
    #[must_use]
    pub fn new_isrc(string: String) -> Self {
        if string.is_empty() {
            return Self::empty();
        }
        let len = string.len();
        // basic validation taken from
        // https://isrc.ifpi.org/downloads/ISRC_Bulletin-2015-01.pdf
        debug_assert!(string.is_ascii());
        debug_assert_eq!(len, 12);
        Self(DeviceSQLStringImpl::Long {
            content: LongBody::Isrc(NullString::from_string(string)),
        })
    }

    /// Extract the Rust string from the DeviceSQLString.
    ///
    /// Consumes itself in the process.
    #[must_use]
    pub fn into_string(self) -> String {
        match self.0 {
            DeviceSQLStringImpl::ShortASCII { content: vec, .. }
            | DeviceSQLStringImpl::Long {
                content: LongBody::Ascii(vec),
                ..
            } => String::from_utf8(vec).expect("invalid string"),
            DeviceSQLStringImpl::Long {
                content: LongBody::Isrc(str),
                ..
            } => str.into_string(),
            DeviceSQLStringImpl::Long {
                content: LongBody::Ucs2le(vec),
            } => String::from_utf16(&vec).expect("invalid UTF16 string"),
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

/// A String encapsulating how DeviceSQL VARCHAR's are structured
///
/// This implementation forces them to be immutable for now
#[binrw]
#[derive(Debug, PartialEq)]
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
        #[bw(calc = content.byte_count() + 4)]
        length: u16,

        #[brw(magic(0u8))] // padding
        #[br(args(flags, length - 4))]
        content: LongBody,
    },
}

#[binrw]
#[derive(Debug, PartialEq)]
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
    pub fn byte_count(&self) -> u16 {
        match self {
            // ISRC offset is compensating for trailing nullbyte + 0x3 magic byte.
            Self::Isrc(null_str) => null_str.len() + 2,
            Self::Ascii(buf) => buf.len(),
            Self::Ucs2le(buf) => (buf.len() * 2),
        }
        .try_into()
        .unwrap()
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
    fn default_string() {
        test_roundtrip(&[0x3], DeviceSQLString::default());
    }

    #[test]
    fn short_ascii_string() {
        test_roundtrip(
            &[0x9, 0x66, 0x6F, 0x6F],
            DeviceSQLString::new("foo".to_owned()),
        );
    }

    #[test]
    fn long_ascii_string() {
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
            DeviceSQLString::new(long_string.to_owned()),
        );
    }

    #[test]
    fn non_ascii() {
        let serialized = [
            0x90, 0x14, 0x00, 0x00, 0x49, 0x00, 0x20, 0x00, 0x64, 0x27, 0x20, 0x00, 0x52, 0x00,
            0x75, 0x00, 0x73, 0x00, 0x74, 0x00,
        ];
        test_roundtrip(&serialized, DeviceSQLString::new("I ❤ Rust".to_string()))
    }

    #[test]
    fn isrc_edge_case() {
        let serialized = [
            0x90, 0x12, 0x00, 0x00, 0x03, 0x47, 0x42, 0x41, 0x59, 0x45, 0x36, 0x37, 0x30, 0x30,
            0x31, 0x34, 0x39, 0x00,
        ];
        test_roundtrip(
            &serialized,
            DeviceSQLString::new_isrc("GBAYE6700149".to_string()),
        );
        test_roundtrip(&[0x3], DeviceSQLString::new_isrc("".to_string()));
    }
}
