// Copyright (c) 2023 Jan Holthuis <jan.holthuis@rub.de>
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
    if offset % alignment != 0 {
        offset += alignment - (offset % alignment);
    }
    offset
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
        assert_eq!(bin.len(), writer.get_ref().len());
        assert_eq_hex!(&bin, &writer.get_ref());
        // T->binary->T
        writer.set_position(0);
        let parsed = T::read_options(&mut writer, endian, read_args.clone()).unwrap();
        assert_eq!(obj, parsed);
        // binary->T
        let mut cursor = binrw::io::Cursor::new(bin);
        let parsed = T::read_options(&mut cursor, endian, read_args.clone()).unwrap();
        assert_eq!(obj, parsed);
        // binary->T->binary
        writer.set_position(0);
        parsed
            .write_options(&mut writer, endian, write_args.clone())
            .unwrap();
        assert_eq!(bin.len(), writer.get_ref().len());
        assert_eq_hex!(&bin, &writer.get_ref());
    }

    pub fn test_roundtrip<'a, T>(bin: &[u8], obj: T)
    where
        <T as binrw::BinRead>::Args<'a>: Default + Clone,
        <T as binrw::BinWrite>::Args<'a>: Default + Clone,
        T: BinRead + BinWrite + PartialEq + core::fmt::Debug + ReadEndian + WriteEndian + Clone,
    {
        test_roundtrip_with_args(
            bin,
            obj,
            <T as binrw::BinRead>::Args::default(),
            <T as binrw::BinWrite>::Args::default(),
        );
    }
}
