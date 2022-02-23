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
use rekordcrate_derive::Parse;

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
        unknown: Vec<u8>,
        /// "CH FADER CURVE" setting.
        channel_fader_curve: ChannelFaderCurve,
        /// "CROSSFADER CURVE" setting.
        crossfader_curve: CrossfaderCurve,
        /// "HEADPHONES PRE EQ" setting.
        headphones_pre_eq: HeadphonesPreEQ,
        /// "HEADPHONES MONO SPLIT" setting.
        headphones_mono_split: HeadphonesMonoSplit,
        /// "BEAT FX QUANTIZE" setting.
        beat_fx_quantize: BeatFXQuantize,
        /// "MIC LOW CUT" setting.
        mic_low_cut: MicLowCut,
        /// "TALK OVER MODE" setting.
        talk_over_mode: TalkOverMode,
        /// "TALK OVER LEVEL" setting.
        talk_over_level: TalkOverLevel,
        /// "MIDI CH" setting.
        midi_channel: MidiChannel,
        /// "MIDI BUTTON TYPE" setting.
        midi_button_type: MidiButtonType,
        /// "BRIGHTNESS > DISPLAY" setting.
        display_brightness: MixerDisplayBrightness,
        /// "BRIGHTNESS > INDICATOR" setting.
        indicator_brightness: MixerIndicatorBrightness,
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
        let (input, unknown) = nom::bytes::complete::take(12usize)(input)?;
        let unknown = unknown.to_vec();
        let (input, channel_fader_curve) = ChannelFaderCurve::parse(input)?;
        let (input, crossfader_curve) = CrossfaderCurve::parse(input)?;
        let (input, headphones_pre_eq) = HeadphonesPreEQ::parse(input)?;
        let (input, headphones_mono_split) = HeadphonesMonoSplit::parse(input)?;
        let (input, beat_fx_quantize) = BeatFXQuantize::parse(input)?;
        let (input, mic_low_cut) = MicLowCut::parse(input)?;
        let (input, talk_over_mode) = TalkOverMode::parse(input)?;
        let (input, talk_over_level) = TalkOverLevel::parse(input)?;
        let (input, midi_channel) = MidiChannel::parse(input)?;
        let (input, midi_button_type) = MidiButtonType::parse(input)?;
        let (input, display_brightness) = MixerDisplayBrightness::parse(input)?;
        let (input, indicator_brightness) = MixerIndicatorBrightness::parse(input)?;
        let (input, channel_fader_curve_long_fader) = ChannelFaderCurveLongFader::parse(input)?;
        let (input, _) = nom::bytes::complete::tag(&[0; 27])(input)?;
        let data = Self::DJMMySetting {
            unknown,
            channel_fader_curve,
            crossfader_curve,
            headphones_pre_eq,
            headphones_mono_split,
            beat_fx_quantize,
            mic_low_cut,
            talk_over_mode,
            talk_over_level,
            midi_channel,
            midi_button_type,
            display_brightness,
            indicator_brightness,
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
#[derive(Debug, Parse)]
pub enum PlayMode {
    /// Named "CONTINUE / ON" in the Rekordbox preferences.
    Continue,
    /// Named "SINGLE / OFF" in the Rekordbox preferences.
    Single,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > EJECT/LOAD LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum EjectLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    Unlock,
    /// Named "LOCK" in the Rekordbox preferences.
    Lock,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > NEEDLE LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum NeedleLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    Unlock,
    /// Named "LOCK" in the Rekordbox preferences.
    Lock,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum QuantizeBeatValue {
    /// Named "1 Beat" in the Rekordbox preferences.
    FullBeat,
    /// Named "1/2 Beat" in the Rekordbox preferences.
    HalfBeat,
    /// Named "1/4 Beat" in the Rekordbox preferences.
    QuarterBeat,
    /// Named "1/8 Beat" in the Rekordbox preferences.
    EighthBeat,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > HOT CUE AUTO LOAD" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DJ SETTING > HOT CUE COLOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum HotCueColor {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "On" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum AutoCueLevel {
    /// Named "-36dB" in the Rekordbox preferences.
    Minus36dB,
    /// Named "-42dB" in the Rekordbox preferences.
    Minus42dB,
    /// Named "-48dB" in the Rekordbox preferences.
    Minus48dB,
    /// Named "-54dB" in the Rekordbox preferences.
    Minus54dB,
    /// Named "-60dB" in the Rekordbox preferences.
    Minus60dB,
    /// Named "-66dB" in the Rekordbox preferences.
    Minus66dB,
    /// Named "-72dB" in the Rekordbox preferences.
    Minus72dB,
    /// Named "-78dB" in the Rekordbox preferences.
    Minus78dB,
    /// Named "MEMORY" in the Rekordbox preferences.
    Memory,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > TIME MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum TimeMode {
    /// Named "Elapsed" in the Rekordbox preferences.
    Elapsed,
    /// Named "REMAIN" in the Rekordbox preferences.
    Remain,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum AutoCue {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > JOG MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum JogMode {
    /// Named "CDJ" in the Rekordbox preferences.
    CDJ,
    /// Named "VINYL" in the Rekordbox preferences.
    Vinyl,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > TEMPO RANGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DJ SETTING > MASTER TEMPO" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum MasterTempo {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum Quantize {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > SYNC" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum Sync {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum PhaseMeter {
    /// Named "TYPE 1" in the Rekordbox preferences.
    Type1,
    /// Named "TYPE 2" in the Rekordbox preferences.
    Type2,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM / PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum Waveform {
    /// Named "WAVEFORM" in the Rekordbox preferences.
    Waveform,
    /// Named "PHASE METER" in the Rekordbox preferences.
    PhaseMeter,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM DIVISIONS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum WaveformDivisions {
    /// Named "TIME SCALE" in the Rekordbox preferences.
    TimeScale,
    /// Named "PHRASE" in the Rekordbox preferences.
    Phrase,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DJ SETTING > VINYL SPEED ADJUST" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DJ SETTING > BEAT JUMP BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DISPLAY(LCD) > LANGUAGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DISPLAY(LCD) > LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DISPLAY(LCD) > JOG LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DISPLAY(LCD) > JOG DISPLAY MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DISPLAY(INDICATOR) > SLIP FLASHING" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum SlipFlashing {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > ON AIR DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum OnAirDisplay {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum JogRingIndicator {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > DISC SLOT ILLUMINATION" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "PLAYER > DISPLAY(INDICATOR) > PAD/BUTTON BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum PadButtonBrightness {
    /// Named "1" in the Rekordbox preferences.
    One,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "MIXER > DJ SETTING > CROSSFADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
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

/// Found at "MIXER > DJ SETTING > CH FADER CURVE (LONG FADER)" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, Parse)]
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

/// Found at "MIXER > DJ SETTING > HEADPHONES PRE EQ" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, Parse)]
pub enum HeadphonesPreEQ {
    /// Named "POST EQ" in the Rekordbox preferences.
    PostEQ,
    /// Named "PRE EQ" in the Rekordbox preferences.
    PreEQ,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "MIXER > DJ SETTING > HEADPHONES MONO SPLIT" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, Parse)]
pub enum HeadphonesMonoSplit {
    /// Named "MONO SPLIT" in the Rekordbox preferences.
    MonoSplit,
    /// Named "STEREO" in the Rekordbox preferences.
    Stereo,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "MIXER > DJ SETTING > BEAT FX QUANTIZE" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, Parse)]
pub enum BeatFXQuantize {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "MIXER > DJ SETTING > MIC LOW CUT" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, Parse)]
pub enum MicLowCut {
    /// Named "OFF" in the Rekordbox preferences.
    Off,
    /// Named "ON(for MC)" in the Rekordbox preferences.
    On,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "MIXER > DJ SETTING > TALK OVER MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum TalkOverMode {
    /// Named "ADVANCED" in the Rekordbox preferences.
    Advanced,
    /// Named "NORMAL" in the Rekordbox preferences.
    Normal,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "MIXER > DJ SETTING > TALK OVER LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum TalkOverLevel {
    /// Named "-24dB" in the Rekordbox preferences.
    Minus24dB,
    /// Named "-18dB" in the Rekordbox preferences.
    Minus18dB,
    /// Named "-12dB" in the Rekordbox preferences.
    Minus12dB,
    /// Named "-6dB" in the Rekordbox preferences.
    Minus6dB,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "MIXER > DJ SETTING > MIDI CH" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum MidiChannel {
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
    /// Named "6" in the Rekordbox preferences.
    Six,
    /// Named "7" in the Rekordbox preferences.
    Seven,
    /// Named "8" in the Rekordbox preferences.
    Eight,
    /// Named "9" in the Rekordbox preferences.
    Nine,
    /// Named "10" in the Rekordbox preferences.
    Ten,
    /// Named "11" in the Rekordbox preferences.
    Eleven,
    /// Named "12" in the Rekordbox preferences.
    Twelve,
    /// Named "13" in the Rekordbox preferences.
    Thirteen,
    /// Named "14" in the Rekordbox preferences.
    Fourteen,
    /// Named "15" in the Rekordbox preferences.
    Fifteen,
    /// Named "16" in the Rekordbox preferences.
    Sixteen,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "MIXER > DJ SETTING > MIDI BUTTON TYPE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum MidiButtonType {
    /// Named "TOGGLE" in the Rekordbox preferences.
    Toggle,
    /// Named "TRIGGER" in the Rekordbox preferences.
    Trigger,
    /// Unknown value.
    Unknown(u8),
}

/// Found at "MIXER > BRIGHTNESS > DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum MixerDisplayBrightness {
    /// Named "WHITE" in the Rekordbox preferences.
    White,
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

/// Found at "MIXER > BRIGHTNESS > INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, Parse)]
pub enum MixerIndicatorBrightness {
    /// Named "1" in the Rekordbox preferences.
    One,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
    /// Unknown value.
    Unknown(u8),
}
