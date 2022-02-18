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
    MySetting {
        /// Unknown field.
        unknown1: Vec<u8>,
        /// "QUANTIZE" setting.
        quantize: Quantize,
        /// "AUTO CUE LEVEL" setting.
        auto_cue_level: AutoCueLevel,
        /// Unknown field.
        unknown2: Vec<u8>,
        /// "EJECT/LOAD LOCK" setting.
        eject_lock: EjectLock,
        /// "SYNC" setting.
        sync: Sync,
        /// "PLAY MODE / AUTO PLAY MODE" setting.
        play_mode: PlayMode,
        /// Quantize Beat Value setting.
        quantize_beat_value: QuantizeBeatValue,
        /// "HOT CUE AUTO LOAD" setting.
        hotcue_autoload: HotCueAutoLoad,
        /// "HOT CUE COLOR" setting.
        hotcue_color: HotCueColor,
        /// Unknown field (apparently always 0).
        unknown3: u16,
        /// "NEEDLE LOCK" setting.
        needle_lock: NeedleLock,
        /// Unknown field (apparently always 0).
        unknown4: u16,
        /// "TIME MODE" setting.
        time_mode: TimeMode,
        /// "TIME MODE" setting.
        jog_mode: JogMode,
        /// "AUTO CUE" setting.
        auto_cue: AutoCue,
        /// "MASTER TEMPO" setting.
        master_tempo: MasterTempo,
        /// "TEMPO RANGE" setting.
        tempo_range: TempoRange,
        /// "PHASE METER" setting.
        phase_meter: PhaseMeter,
        /// Unknown field (apparently always 0).
        unknown5: u16,
    },
    /// Payload of a `MYSETTING2.DAT` file (40 bytes).
    MySetting2(Vec<u8>),
    /// Payload of an unknown setting file.
    Unknown(Vec<u8>),
}

impl SettingData {
    fn parse(input: &[u8], len_data: u32) -> IResult<&[u8], Self> {
        // TODO: Find a way to distinguish `MYSETTING.DAT` and `MYSETTING2.DAT` data fields (they
        // have the same size).
        match len_data {
            40 => Self::parse_mysetting(input),
            _ => {
                let data_size = usize::try_from(len_data)
                    .map_err(|_| nom_input_error_with_kind(input, ErrorKind::TooLarge))?;
                let (input, data) = nom::bytes::complete::take(data_size)(input)?;
                Ok((input, Self::Unknown(data.to_vec())))
            }
        }
    }

    fn parse_mysetting(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, unknown1) = nom::bytes::complete::take(10usize)(input)?;
        let unknown1 = unknown1.to_vec();
        let (input, quantize) = Quantize::parse(input)?;
        let (input, auto_cue_level) = AutoCueLevel::parse(input)?;
        let (input, unknown2) = nom::bytes::complete::take(9usize)(input)?;
        let unknown2 = unknown2.to_vec();
        let (input, eject_lock) = EjectLock::parse(input)?;
        let (input, sync) = Sync::parse(input)?;
        let (input, play_mode) = PlayMode::parse(input)?;
        let (input, quantize_beat_value) = QuantizeBeatValue::parse(input)?;
        let (input, hotcue_autoload) = HotCueAutoLoad::parse(input)?;
        let (input, hotcue_color) = HotCueColor::parse(input)?;
        let (input, unknown3) = nom::number::complete::be_u16(input)?;
        let (input, needle_lock) = NeedleLock::parse(input)?;
        let (input, unknown4) = nom::number::complete::be_u16(input)?;
        let (input, time_mode) = TimeMode::parse(input)?;
        let (input, jog_mode) = JogMode::parse(input)?;
        let (input, auto_cue) = AutoCue::parse(input)?;
        let (input, master_tempo) = MasterTempo::parse(input)?;
        let (input, tempo_range) = TempoRange::parse(input)?;
        let (input, phase_meter) = PhaseMeter::parse(input)?;
        let (input, unknown5) = nom::number::complete::be_u16(input)?;
        let data = Self::MySetting {
            unknown1,
            quantize,
            auto_cue_level,
            unknown2,
            eject_lock,
            sync,
            play_mode,
            quantize_beat_value,
            hotcue_autoload,
            hotcue_color,
            unknown3,
            needle_lock,
            unknown4,
            time_mode,
            jog_mode,
            auto_cue,
            master_tempo,
            tempo_range,
            phase_meter,
            unknown5,
        };
        Ok((input, data))
    }
}

/// Found at "PLAYER > DJ SETTING > PLAY MODE / AUTO PLAY MODE" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug)]
pub enum PlayMode {
    /// Named "CONTINUE / ON" in the Rekordbox preferences.
    Continue,
    /// Named "SINGLE / OFF" in the Rekordbox preferences.
    Single,
    /// Unknown value.
    Unknown(u8),
}

impl PlayMode {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Continue,
            0x81 => Self::Single,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > EJECT/LOAD LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum EjectLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    Unlock,
    /// Named "LOCK" in the Rekordbox preferences.
    Lock,
    /// Unknown value.
    Unknown(u8),
}

impl EjectLock {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Unlock,
            0x81 => Self::Lock,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > NEEDLE LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum NeedleLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    Unlock,
    /// Named "LOCK" in the Rekordbox preferences.
    Lock,
    /// Unknown value.
    Unknown(u8),
}

impl NeedleLock {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Unlock,
            0x81 => Self::Lock,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum QuantizeBeatValue {
    /// Named "1/8 Beat" in the Rekordbox preferences.
    EighthBeat,
    /// Named "1/4 Beat" in the Rekordbox preferences.
    QuarterBeat,
    /// Named "1/2 Beat" in the Rekordbox preferences.
    HalfBeat,
    /// Named "1 Beat" in the Rekordbox preferences.
    FullBeat,
    /// Unknown value.
    Unknown(u8),
}

impl QuantizeBeatValue {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::FullBeat,
            0x81 => Self::HalfBeat,
            0x82 => Self::QuarterBeat,
            0x83 => Self::EighthBeat,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > HOT CUE AUTO LOAD" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum HotCueAutoLoad {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "rekordbox SETTING" in the Rekordbox preferences.
    RekordboxSetting,
    /// Named "On" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

impl HotCueAutoLoad {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Off,
            0x81 => Self::On,
            0x82 => Self::RekordboxSetting,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > HOT CUE COLOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum HotCueColor {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "On" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

impl HotCueColor {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Off,
            0x81 => Self::On,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum AutoCueLevel {
    /// Named "-78dB" in the Rekordbox preferences.
    Minus78dB,
    /// Named "-72dB" in the Rekordbox preferences.
    Minus72dB,
    /// Named "-66dB" in the Rekordbox preferences.
    Minus66dB,
    /// Named "-60dB" in the Rekordbox preferences.
    Minus60dB,
    /// Named "-54dB" in the Rekordbox preferences.
    Minus54dB,
    /// Named "-48dB" in the Rekordbox preferences.
    Minus48dB,
    /// Named "-42dB" in the Rekordbox preferences.
    Minus42dB,
    /// Named "-36dB" in the Rekordbox preferences.
    Minus36dB,
    /// Named "MEMORY" in the Rekordbox preferences.
    Memory,
    /// Unknown value.
    Unknown(u8),
}

impl AutoCueLevel {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Minus36dB,
            0x81 => Self::Minus42dB,
            0x82 => Self::Minus48dB,
            0x83 => Self::Minus54dB,
            0x84 => Self::Minus60dB,
            0x85 => Self::Minus66dB,
            0x86 => Self::Minus72dB,
            0x87 => Self::Minus78dB,
            0x88 => Self::Memory,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > TIME MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum TimeMode {
    /// Named "Elapsed" in the Rekordbox preferences.
    Elapsed,
    /// Named "REMAIN" in the Rekordbox preferences.
    Remain,
    /// Unknown value.
    Unknown(u8),
}

impl TimeMode {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Elapsed,
            0x81 => Self::Remain,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum AutoCue {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

impl AutoCue {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Off,
            0x81 => Self::On,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > JOG MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum JogMode {
    /// Named "VINYL" in the Rekordbox preferences.
    Vinyl,
    /// Named "CDJ" in the Rekordbox preferences.
    CDJ,
    /// Unknown value.
    Unknown(u8),
}

impl JogMode {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::CDJ,
            0x81 => Self::Vinyl,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > TEMPO RANGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum TempoRange {
    /// Named "±6" in the Rekordbox preferences.
    SixPercent,
    /// Named "±10" in the Rekordbox preferences.
    TenPercent,
    /// Named "±16" in the Rekordbox preferences.
    SixteenPercent,
    /// Named "WIDE" in the Rekordbox preferences.
    Wide,
    /// Unknown value.
    Unknown(u8),
}

impl TempoRange {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::SixPercent,
            0x81 => Self::TenPercent,
            0x82 => Self::SixteenPercent,
            0x83 => Self::Wide,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > MASTER TEMPO" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum MasterTempo {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

impl MasterTempo {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Off,
            0x81 => Self::On,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum Quantize {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

impl Quantize {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Off,
            0x81 => Self::On,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > SYNC" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum Sync {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

impl Sync {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Off,
            0x81 => Self::On,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum PhaseMeter {
    /// Named "TYPE 1" in the Rekordbox preferences.
    Type1,
    /// Named "TYPE 2" in the Rekordbox preferences.
    Type2,
    /// Unknown value.
    Unknown(u8),
}

impl PhaseMeter {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Type1,
            0x81 => Self::Type2,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}
