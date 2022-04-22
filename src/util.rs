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

#[cfg(test)]
pub(crate) mod testing {
    use binrw::{
        meta::{ReadEndian, WriteEndian},
        prelude::*,
        Endian, ReadOptions, WriteOptions,
    };

    pub fn test_roundtrip_with_args<T>(
        bin: &[u8],
        obj: T,
        read_args: <T as binrw::BinRead>::Args,
        write_args: <T as binrw::BinWrite>::Args,
    ) where
        T: BinRead + BinWrite + PartialEq + core::fmt::Debug + ReadEndian + WriteEndian,
    {
        let write_opts = WriteOptions::new(Endian::NATIVE);
        let read_opts = ReadOptions::new(Endian::NATIVE);
        // T->binary
        let mut writer = binrw::io::Cursor::new(Vec::with_capacity(bin.len()));
        obj.write_options(&mut writer, &write_opts, write_args.clone())
            .unwrap();
        assert_eq!(bin.len(), writer.get_ref().len());
        assert_eq!(bin, writer.get_ref());
        // T->binary->T
        writer.set_position(0);
        let parsed = T::read_options(&mut writer, &read_opts, read_args.clone()).unwrap();
        assert_eq!(obj, parsed);
        // binary->T
        let mut cursor = binrw::io::Cursor::new(bin);
        let parsed = T::read_options(&mut cursor, &read_opts, read_args).unwrap();
        assert_eq!(obj, parsed);
        // binary->T->binary
        writer.set_position(0);
        parsed
            .write_options(&mut writer, &write_opts, write_args)
            .unwrap();
        assert_eq!(bin.len(), writer.get_ref().len());
        assert_eq!(bin, writer.get_ref());
    }

    pub fn test_roundtrip<T>(bin: &[u8], obj: T)
    where
        <T as binrw::BinRead>::Args: Default,
        <T as binrw::BinWrite>::Args: Default,
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
