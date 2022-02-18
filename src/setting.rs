// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Parser for Rekordbox `*SETTING.DAT` files.
//!
//! These files either in the `PIONEER` directory of a USB drive (device exports), but are also
//! present for on local installations of Rekordbox 6 in `%APPDATA%\Pioneer\rekordbox6`.
//!
//! The settings files store the settings found on the "DJ System" > "My Settings" page of the
//! Rekordbox preferences. These includes language, LCD brightness, tempo fader range, crossfader
//! curve, etc.
//!
//! The exact format still has to be reverse-engineered.

use crate::util::nom_input_error_with_kind;
use nom::error::ErrorKind;
use nom::IResult;

#[derive(Debug)]
/// Represents a setting file.
pub struct Setting {
    /// Size of the string data field (should be always 96).
    pub len_stringdata: u32,
    /// Name of the company ("PIONEER").
    pub company: String,
    /// Name of the software ("rekordbox").
    pub software: String,
    /// Some kind of version number.
    pub version: String,
    /// Size of the `data` data in bytes.
    pub len_data: u32,
    /// Unknown field.
    pub data: SettingData,
    /// CRC16 XMODEM checksum. The checksum is calculated over the contents of the `data`
    /// field, except for `DJMSETTING.DAT` files where the checksum is calculated over all
    /// preceding bytes including the length fields.
    ///
    /// See <https://reveng.sourceforge.io/crc-catalogue/all.htm#crc.cat.crc-16-xmodem> for
    /// details.
    pub checksum: u16,
    /// Unknown field (apparently always `0000`).
    pub unknown: u16,
}

impl Setting {
    /// Parses the Setting file and returns the structure.
    pub fn parse(orig_input: &[u8]) -> IResult<&[u8], Self> {
        let (input, len_stringdata) = nom::number::complete::le_u32(orig_input)?;
        let stringdata_size = usize::try_from(len_stringdata)
            .map_err(|_| nom_input_error_with_kind(input, ErrorKind::TooLarge))?;
        let stringdatasection_size = stringdata_size / 3;
        let (input, company) = nom::bytes::complete::take(stringdatasection_size)(input)?;
        let company = std::str::from_utf8(company)
            .unwrap()
            .trim_end_matches('\0')
            .to_owned();
        let (input, software) = nom::bytes::complete::take(stringdatasection_size)(input)?;
        let software = std::str::from_utf8(software)
            .unwrap()
            .trim_end_matches('\0')
            .to_owned();
        let (input, version) = nom::bytes::complete::take(stringdatasection_size)(input)?;
        let version = std::str::from_utf8(version)
            .unwrap()
            .trim_end_matches('\0')
            .to_owned();

        let (input, len_data) = nom::number::complete::le_u32(input)?;
        let (input, data) = SettingData::parse(input, len_data)?;
        let (input, checksum) = nom::number::complete::le_u16(input)?;
        let (input, unknown) = nom::number::complete::le_u16(input)?;
        if !input.is_empty() {
            return Err(nom_input_error_with_kind(input, ErrorKind::Complete));
        }

        Ok((
            input,
            Self {
                len_stringdata,
                company,
                software,
                version,
                len_data,
                data,
                checksum,
                unknown,
            },
        ))
    }
}

/// Data section of a `*SETTING.DAT` file.
#[derive(Debug)]
pub enum SettingData {
    /// Payload of a `DEVSETTING.DAT` file (32 bytes).
    DevSetting(Vec<u8>),
    /// Payload of a `DJMMYSETTING.DAT` file (52 bytes).
    DJMMySetting(Vec<u8>),
    /// Payload of a `MYSETTING.DAT` file (40 bytes).
    MySetting(Vec<u8>),
    /// Payload of a `MYSETTING2.DAT` file (40 bytes).
    MySetting2(Vec<u8>),
    /// Payload of an unknown setting file.
    Unknown(Vec<u8>),
}

impl SettingData {
    fn parse(input: &[u8], len_data: u32) -> IResult<&[u8], Self> {
        let data_size = usize::try_from(len_data)
            .map_err(|_| nom_input_error_with_kind(input, ErrorKind::TooLarge))?;
        let (input, data) = nom::bytes::complete::take(data_size)(input)?;
        Ok((input, Self::Unknown(data.to_vec())))
    }
}
