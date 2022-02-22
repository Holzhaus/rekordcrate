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
    DJMMySetting {
        /// Unknown field.
        unknown1: Vec<u8>,
        /// "CH FADER CURVE" setting.
        channel_fader_curve: ChannelFaderCurve,
        /// "CROSSFADER CURVE" setting.
        crossfader_curve: CrossfaderCurve,
        /// Unknown field.
        unknown2: Vec<u8>,
        /// "CH FADER CURVE (LONG FADER)" setting.
        channel_fader_curve_long_fader: ChannelFaderCurveLongFader,
    },
    /// Payload of a `MYSETTING.DAT` file (40 bytes).
    MySetting {
        /// Unknown field.
        unknown1: Vec<u8>,
        /// "ON AIR DISPLAY" setting.
        on_air_display: OnAirDisplay,
        /// "LCD BRIGHTNESS" setting.
        lcd_brightness: LCDBrightness,
        /// "QUANTIZE" setting.
        quantize: Quantize,
        /// "AUTO CUE LEVEL" setting.
        auto_cue_level: AutoCueLevel,
        /// "LANGUAGE" setting.
        language: Language,
        /// Unknown field.
        unknown2: u8,
        /// "JOG RING BRIGHTNESS" setting.
        jog_ring_brightness: JogRingBrightness,
        /// "JOG RING INDICATOR" setting.
        jog_ring_indicator: JogRingIndicator,
        /// "SLIP FLASHING" setting.
        slip_flashing: SlipFlashing,
        /// Unknown field.
        unknown3: Vec<u8>,
        /// "DISC SLOT ILLUMINATION" setting.
        disc_slot_illumination: DiscSlotIllumination,
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
        unknown4: u16,
        /// "NEEDLE LOCK" setting.
        needle_lock: NeedleLock,
        /// Unknown field (apparently always 0).
        unknown5: u16,
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
        unknown6: u16,
    },
    /// Payload of a `MYSETTING2.DAT` file (40 bytes).
    MySetting2 {
        /// "VINYL SPEED ADJUST" setting.
        vinyl_speed_adjust: VinylSpeedAdjust,
        /// "JOG DISPLAY MODE" setting.
        jog_display_mode: JogDisplayMode,
        /// "PAD/BUTTON BRIGHTNESS" setting.
        pad_button_brightness: PadButtonBrightness,
        /// "JOG LCD BRIGHTNESS" setting.
        jog_lcd_brightness: JogLCDBrightness,
        /// "WAVEFORM DIVISIONS" setting.
        waveform_divisions: WaveformDivisions,
        /// "WAVEFORM / PHASE METER" setting.
        waveform: Waveform,
        /// Unknown field.
        unknown: u8,
        /// "BEAT JUMP BEAT VALUE" setting.
        beat_jump_beat_value: BeatJumpBeatValue,
    },
    /// Payload of an unknown setting file.
    Unknown(Vec<u8>),
}

impl SettingData {
    fn parse(input: &[u8], len_data: u32) -> IResult<&[u8], Self> {
        match len_data {
            40 => nom::branch::alt((Self::parse_mysetting2, Self::parse_mysetting))(input),
            52 => Self::parse_djmmysetting(input),
            _ => {
                let data_size = usize::try_from(len_data)
                    .map_err(|_| nom_input_error_with_kind(input, ErrorKind::TooLarge))?;
                let (input, data) = nom::bytes::complete::take(data_size)(input)?;
                Ok((input, Self::Unknown(data.to_vec())))
            }
        }
    }

    fn parse_djmmysetting(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, unknown1) = nom::bytes::complete::take(12usize)(input)?;
        let unknown1 = unknown1.to_vec();
        let (input, channel_fader_curve) = ChannelFaderCurve::parse(input)?;
        let (input, crossfader_curve) = CrossfaderCurve::parse(input)?;
        let (input, unknown2) = nom::bytes::complete::take(10usize)(input)?;
        let unknown2 = unknown2.to_vec();
        let (input, channel_fader_curve_long_fader) = ChannelFaderCurveLongFader::parse(input)?;
        let (input, _) = nom::bytes::complete::tag(&[0; 27])(input)?;
        let data = Self::DJMMySetting {
            unknown1,
            channel_fader_curve,
            crossfader_curve,
            unknown2,
            channel_fader_curve_long_fader,
        };
        Ok((input, data))
    }

    fn parse_mysetting2(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, vinyl_speed_adjust) = VinylSpeedAdjust::parse(input)?;
        let (input, jog_display_mode) = JogDisplayMode::parse(input)?;
        let (input, pad_button_brightness) = PadButtonBrightness::parse(input)?;
        let (input, jog_lcd_brightness) = JogLCDBrightness::parse(input)?;
        let (input, waveform_divisions) = WaveformDivisions::parse(input)?;
        let (input, _) = nom::bytes::complete::tag(&[0; 5])(input)?;
        let (input, waveform) = Waveform::parse(input)?;
        let (input, unknown) = nom::number::complete::u8(input)?;
        let (input, beat_jump_beat_value) = BeatJumpBeatValue::parse(input)?;
        let (input, _) = nom::bytes::complete::tag(&[0; 27])(input)?;

        Ok((
            input,
            Self::MySetting2 {
                vinyl_speed_adjust,
                jog_display_mode,
                pad_button_brightness,
                jog_lcd_brightness,
                waveform_divisions,
                waveform,
                unknown,
                beat_jump_beat_value,
            },
        ))
    }

    fn parse_mysetting(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, unknown1) = nom::bytes::complete::take(8usize)(input)?;
        let unknown1 = unknown1.to_vec();
        let (input, on_air_display) = OnAirDisplay::parse(input)?;
        let (input, lcd_brightness) = LCDBrightness::parse(input)?;
        let (input, quantize) = Quantize::parse(input)?;
        let (input, auto_cue_level) = AutoCueLevel::parse(input)?;
        let (input, language) = Language::parse(input)?;
        let (input, unknown2) = nom::number::complete::u8(input)?;
        let (input, jog_ring_brightness) = JogRingBrightness::parse(input)?;
        let (input, jog_ring_indicator) = JogRingIndicator::parse(input)?;
        let (input, slip_flashing) = SlipFlashing::parse(input)?;
        let (input, unknown3) = nom::bytes::complete::take(3usize)(input)?;
        let unknown3 = unknown3.to_vec();
        let (input, disc_slot_illumination) = DiscSlotIllumination::parse(input)?;
        let (input, eject_lock) = EjectLock::parse(input)?;
        let (input, sync) = Sync::parse(input)?;
        let (input, play_mode) = PlayMode::parse(input)?;
        let (input, quantize_beat_value) = QuantizeBeatValue::parse(input)?;
        let (input, hotcue_autoload) = HotCueAutoLoad::parse(input)?;
        let (input, hotcue_color) = HotCueColor::parse(input)?;
        let (input, unknown4) = nom::number::complete::be_u16(input)?;
        let (input, needle_lock) = NeedleLock::parse(input)?;
        let (input, unknown5) = nom::number::complete::be_u16(input)?;
        let (input, time_mode) = TimeMode::parse(input)?;
        let (input, jog_mode) = JogMode::parse(input)?;
        let (input, auto_cue) = AutoCue::parse(input)?;
        let (input, master_tempo) = MasterTempo::parse(input)?;
        let (input, tempo_range) = TempoRange::parse(input)?;
        let (input, phase_meter) = PhaseMeter::parse(input)?;
        let (input, unknown6) = nom::number::complete::be_u16(input)?;
        let data = Self::MySetting {
            unknown1,
            on_air_display,
            lcd_brightness,
            quantize,
            auto_cue_level,
            language,
            unknown2,
            jog_ring_brightness,
            jog_ring_indicator,
            slip_flashing,
            unknown3,
            disc_slot_illumination,
            eject_lock,
            sync,
            play_mode,
            quantize_beat_value,
            hotcue_autoload,
            hotcue_color,
            unknown4,
            needle_lock,
            unknown5,
            time_mode,
            jog_mode,
            auto_cue,
            master_tempo,
            tempo_range,
            phase_meter,
            unknown6,
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

/// Found at "PLAYER > DJ SETTING > WAVEFORM / PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum Waveform {
    /// Named "WAVEFORM" in the Rekordbox preferences.
    Waveform,
    /// Named "PHASE METER" in the Rekordbox preferences.
    PhaseMeter,
    /// Unknown value.
    Unknown(u8),
}

impl Waveform {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Waveform,
            0x81 => Self::PhaseMeter,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM DIVISIONS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum WaveformDivisions {
    /// Named "TIME SCALE" in the Rekordbox preferences.
    TimeScale,
    /// Named "PHRASE" in the Rekordbox preferences.
    Phrase,
    /// Unknown value.
    Unknown(u8),
}

impl WaveformDivisions {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::TimeScale,
            0x81 => Self::Phrase,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > VINYL SPEED ADJUST" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum VinylSpeedAdjust {
    /// Named "TOUCH & RELEASE" in the Rekordbox preferences.
    TouchRelease,
    /// Named "TOUCH" in the Rekordbox preferences.
    Touch,
    /// Named "RELEASE" in the Rekordbox preferences.
    Release,
    /// Unknown value.
    Unknown(u8),
}

impl VinylSpeedAdjust {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::TouchRelease,
            0x81 => Self::Touch,
            0x82 => Self::Release,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DJ SETTING > BEAT JUMP BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum BeatJumpBeatValue {
    /// Named "1/2 BEAT" in the Rekordbox preferences.
    HalfBeat,
    /// Named "1 BEAT" in the Rekordbox preferences.
    OneBeat,
    /// Named "2 BEAT" in the Rekordbox preferences.
    TwoBeat,
    /// Named "4 BEAT" in the Rekordbox preferences.
    FourBeat,
    /// Named "8 BEAT" in the Rekordbox preferences.
    EightBeat,
    /// Named "16 BEAT" in the Rekordbox preferences.
    SixteenBeat,
    /// Named "32 BEAT" in the Rekordbox preferences.
    ThirtytwoBeat,
    /// Named "64 BEAT" in the Rekordbox preferences.
    SixtyfourBeat,
    /// Unknown value.
    Unknown(u8),
}

impl BeatJumpBeatValue {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::HalfBeat,
            0x81 => Self::OneBeat,
            0x82 => Self::TwoBeat,
            0x83 => Self::FourBeat,
            0x84 => Self::EightBeat,
            0x85 => Self::SixteenBeat,
            0x86 => Self::ThirtytwoBeat,
            0x87 => Self::SixtyfourBeat,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DISPLAY(LCD) > LANGUAGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum Language {
    /// Named "English" in the Rekordbox preferences.
    English,
    /// Named "Français" in the Rekordbox preferences.
    French,
    /// Named "Deutsch" in the Rekordbox preferences.
    German,
    /// Named "Italiano" in the Rekordbox preferences.
    Italian,
    /// Named "Nederlands" in the Rekordbox preferences.
    Dutch,
    /// Named "Español" in the Rekordbox preferences.
    Spanish,
    /// Named "Русский" in the Rekordbox preferences.
    Russian,
    /// Named "한국어" in the Rekordbox preferences.
    Korean,
    /// Named "简体中文" in the Rekordbox preferences.
    ChineseSimplified,
    /// Named "繁體中文" in the Rekordbox preferences.
    ChineseTraditional,
    /// Named "日本語" in the Rekordbox preferences.
    Japanese,
    /// Named "Português" in the Rekordbox preferences.
    Portuguese,
    /// Named "Svenska" in the Rekordbox preferences.
    Swedish,
    /// Named "Čeština" in the Rekordbox preferences.
    Czech,
    /// Named "Magyar" in the Rekordbox preferences.
    Hungarian,
    /// Named "Dansk" in the Rekordbox preferences.
    Danish,
    /// Named "Ελληνικά" in the Rekordbox preferences.
    Greek,
    /// Named "Türkçe" in the Rekordbox preferences.
    Turkish,
    /// Unknown value.
    Unknown(u8),
}

impl Language {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x81 => Self::English,
            0x82 => Self::French,
            0x83 => Self::German,
            0x84 => Self::Italian,
            0x85 => Self::Dutch,
            0x86 => Self::Spanish,
            0x87 => Self::Russian,
            0x88 => Self::Korean,
            0x89 => Self::ChineseSimplified,
            0x8a => Self::ChineseTraditional,
            0x8b => Self::Japanese,
            0x8c => Self::Portuguese,
            0x8d => Self::Swedish,
            0x8e => Self::Czech,
            0x8f => Self::Hungarian,
            0x90 => Self::Danish,
            0x91 => Self::Greek,
            0x92 => Self::Turkish,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DISPLAY(LCD) > LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum LCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    One,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Named "5" in the Rekordbox preferences.
    Five,
    /// Unknown value.
    Unknown(u8),
}

impl LCDBrightness {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x81 => Self::One,
            0x82 => Self::Two,
            0x83 => Self::Three,
            0x84 => Self::Four,
            0x85 => Self::Five,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum JogLCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    One,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Named "5" in the Rekordbox preferences.
    Five,
    /// Unknown value.
    Unknown(u8),
}

impl JogLCDBrightness {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x81 => Self::One,
            0x82 => Self::Two,
            0x83 => Self::Three,
            0x84 => Self::Four,
            0x85 => Self::Five,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG DISPLAY MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum JogDisplayMode {
    /// Named "AUTO" in the Rekordbox preferences.
    Auto,
    /// Named "INFO" in the Rekordbox preferences.
    Info,
    /// Named "SIMPLE" in the Rekordbox preferences.
    Simple,
    /// Named "ARTWORK" in the Rekordbox preferences.
    Artwork,
    /// Unknown value.
    Unknown(u8),
}

impl JogDisplayMode {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Auto,
            0x81 => Self::Info,
            0x82 => Self::Simple,
            0x83 => Self::Artwork,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > SLIP FLASHING" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum SlipFlashing {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

impl SlipFlashing {
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

/// Found at "PLAYER > DISPLAY(INDICATOR) > ON AIR DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum OnAirDisplay {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

impl OnAirDisplay {
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

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum JogRingBrightness {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    Bright,
    /// Unknown value.
    Unknown(u8),
}

impl JogRingBrightness {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Off,
            0x81 => Self::Dark,
            0x82 => Self::Bright,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum JogRingIndicator {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

impl JogRingIndicator {
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

/// Found at "PLAYER > DISPLAY(INDICATOR) > DISC SLOT ILLUMINATION" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum DiscSlotIllumination {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    Bright,
    /// Unknown value.
    Unknown(u8),
}

impl DiscSlotIllumination {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Off,
            0x81 => Self::Dark,
            0x82 => Self::Bright,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > PAD/BUTTON BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum PadButtonBrightness {
    /// Named "1" in the Rekordbox preferences.
    One,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Named "5" in the Rekordbox preferences.
    Five,
    /// Unknown value.
    Unknown(u8),
}

impl PadButtonBrightness {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x81 => Self::One,
            0x82 => Self::Two,
            0x83 => Self::Three,
            0x84 => Self::Four,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum ChannelFaderCurve {
    /// Steep volume raise when the fader is moved near the top.
    SteepTop,
    /// Linear volume raise when the fader is moved.
    Linear,
    /// Steep volume raise when the fader is moved near the bottom.
    SteepBottom,
    /// Unknown value.
    Unknown(u8),
}

impl ChannelFaderCurve {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::SteepTop,
            0x81 => Self::Linear,
            0x82 => Self::SteepBottom,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "MIXER > DJ SETTING > CROSSFADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug)]
pub enum CrossfaderCurve {
    /// Logarithmic volume raise of the other channel near the edges of the fader.
    ConstantPower,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    SlowCut,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    FastCut,
    /// Unknown value.
    Unknown(u8),
}

impl CrossfaderCurve {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::ConstantPower,
            0x81 => Self::SlowCut,
            0x82 => Self::FastCut,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE (LONG FADER)" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug)]
pub enum ChannelFaderCurveLongFader {
    /// Very steep volume raise when the fader is moved the near the top (e.g. y = x⁵).
    Exponential,
    /// Steep volume raise when the fader is moved the near the top (e.g. y = x²).
    Smooth,
    /// Linear volume raise when the fader is moved (e.g. y = k * x).
    Linear,
    /// Unknown value.
    Unknown(u8),
}

impl ChannelFaderCurveLongFader {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, value) = nom::number::complete::u8(input)?;
        let value = match value {
            0x80 => Self::Exponential,
            0x81 => Self::Smooth,
            0x82 => Self::Linear,
            _ => Self::Unknown(value),
        };
        Ok((input, value))
    }
}
