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
    };
    pub fn test_roundtrip<T>(bin: &[u8], obj: T)
    where
        <T as binrw::BinRead>::Args: Default,
        <T as binrw::BinWrite>::Args: Default,
        T: BinRead + BinWrite + PartialEq + core::fmt::Debug + ReadEndian + WriteEndian,
    {
        // T->binary
        let mut writer = binrw::io::Cursor::new(Vec::with_capacity(bin.len()));
        obj.write(&mut writer).unwrap();
        assert_eq!(bin, writer.get_ref());
        // T->binary->T
        writer.set_position(0);
        let parsed = T::read(&mut writer).unwrap();
        assert_eq!(obj, parsed);
        // binary->T
        let mut cursor = binrw::io::Cursor::new(bin);
        let parsed = T::read(&mut cursor).unwrap();
        assert_eq!(obj, parsed);
        // binary->T->binary
        writer.set_position(0);
        parsed.write(&mut writer).unwrap();
        assert_eq!(bin, writer.get_ref());
    }
}
