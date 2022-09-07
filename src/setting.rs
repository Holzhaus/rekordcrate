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
//!
//! # Defaults
//!
//! The `SettingData` structs implement the `Default` trait and allows you to create objects that
//! use the same default values as found in Rekordbox 6.6.1.

use binrw::{binrw, io::Cursor, BinWrite, Endian, NullString, WriteOptions};

#[binrw]
#[derive(Debug, PartialEq)]
#[brw(little)]
#[bw(import(no_checksum: bool))]
/// Represents a setting file.
pub struct Setting {
    /// Size of the string data field (should be always 96).
    #[br(temp, assert(len_stringdata == 0x60))]
    #[bw(calc = 0x60)]
    len_stringdata: u32,
    /// Name of the brand.
    ///
    /// The value seems to depend on the kind of file:
    ///
    /// | File               | Value        |
    /// | ------------------ | ------------ |
    /// | `DEVSETTING.DAT`   | `PIONEER DJ` |
    /// | `DJMMYSETTING.DAT` | `PioneerDJ`  |
    /// | `MYSETTING.DAT`    | `PIONEER`    |
    /// | `MYSETTING2.DAT`   | `PIONEER`    |
    #[brw(pad_size_to = 0x20, assert(brand.len() <= (0x20 - 1)))]
    pub brand: NullString,
    /// Name of the software ("rekordbox").
    #[brw(pad_size_to = 0x20, assert(software.len() <= (0x20 - 1)))]
    pub software: NullString,
    /// Some kind of version number.
    #[brw(pad_size_to = 0x20, assert(version.len() <= (0x20 - 1)))]
    pub version: NullString,
    /// Size of the `data` data in bytes.
    #[br(temp)]
    #[bw(calc = data.size())]
    len_data: u32,
    /// The actual settings data.
    #[br(args(len_data))]
    pub data: SettingData,
    /// CRC16 XMODEM checksum. The checksum is calculated over the contents of the `data`
    /// field, except for `DJMSETTING.DAT` files where the checksum is calculated over all
    /// preceding bytes including the length fields.
    ///
    /// See <https://reveng.sourceforge.io/crc-catalogue/all.htm#crc.cat.crc-16-xmodem> for
    /// details.
    #[br(temp)]
    #[bw(calc = no_checksum.then(|| 0).unwrap_or_else(|| self.calculate_checksum()))]
    checksum: u16,
    /// Unknown field (apparently always `0000`).
    #[br(assert(unknown == 0))]
    unknown: u16,
}

impl Setting
where
    Setting: BinWrite,
{
    /// Calculate the CRC16 checksum.
    ///
    /// This is horribly inefficient and basically serializes the whole data structure twice, but
    /// there seems to be no other way to archieve this.
    ///
    /// Upstream issue: https://github.com/jam1garner/binrw/issues/102
    fn calculate_checksum(&self) -> u16 {
        let mut data = Vec::<u8>::with_capacity(156);
        let mut writer = Cursor::new(&mut data);
        self.write_options(&mut writer, &WriteOptions::new(Endian::Little), (true,))
            .unwrap();
        let start = match self.data {
            // In `DJMMYSETTING.DAT`, the checksum is calculated over all previous bytes, including
            // the section lengths and string data.
            SettingData::DJMMySetting(_) => 0,
            // In all other files`, the checksum is calculated just over the data section which
            // starts at offset 104,
            _ => 104,
        };

        let end = data.len() - 4;
        crc16::State::<crc16::XMODEM>::calculate(&data[start..end])
    }
}

/// Data section of a `*SETTING.DAT` file.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(little)]
#[br(import(len: u32))]
pub enum SettingData {
    /// Payload of a `DEVSETTING.DAT` file (32 bytes).
    #[br(pre_assert(len == 32))]
    DevSetting(DevSetting),
    /// Payload of a `DJMMYSETTING.DAT` file (52 bytes).
    #[br(pre_assert(len == 52))]
    DJMMySetting(DJMMySetting),
    /// Payload of a `MYSETTING.DAT` file (40 bytes).
    #[br(pre_assert(len == 40))]
    MySetting(MySetting),
    /// Payload of a `MYSETTING2.DAT` file (40 bytes).
    #[br(pre_assert(len == 40))]
    MySetting2(MySetting2),
}

impl SettingData {
    fn size(&self) -> u32 {
        match &self {
            Self::DevSetting(_) => 32,
            Self::DJMMySetting(_) => 52,
            Self::MySetting(_) => 40,
            Self::MySetting2(_) => 40,
        }
    }
}

/// Payload of a `DEVSETTING.DAT` file (32 bytes).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(little)]
pub struct DevSetting {
    /// Unknown field.
    #[br(assert(unknown1 == [0x78, 0x56, 0x34, 0x12, 0x01, 0x00, 0x00, 0x00, 0x01]))]
    unknown1: [u8; 9],
    /// "Type of the overview Waveform" setting.
    pub overview_waveform_type: OverviewWaveformType,
    /// "Waveform color" setting.
    pub waveform_color: WaveformColor,
    /// Unknown field.
    #[br(assert(unknown2 == 0x01))]
    unknown2: u8,
    /// "Key display format" setting.
    pub key_display_format: KeyDisplayFormat,
    /// "Waveform Current Position" setting.
    pub waveform_current_position: WaveformCurrentPosition,
    /// Unknown field.
    #[br(assert(unknown3 == [0x00; 18]))]
    unknown3: [u8; 18],
}

impl Default for DevSetting {
    fn default() -> Self {
        Self {
            unknown1: [0x78, 0x56, 0x34, 0x12, 0x01, 0x00, 0x00, 0x00, 0x01],
            overview_waveform_type: OverviewWaveformType::default(),
            waveform_color: WaveformColor::default(),
            key_display_format: KeyDisplayFormat::default(),
            unknown2: 0x01,
            waveform_current_position: WaveformCurrentPosition::default(),
            unknown3: [0x00; 18],
        }
    }
}

/// Payload of a `DJMMYSETTING.DAT` file (52 bytes).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(little)]
pub struct DJMMySetting {
    /// Unknown field.
    unknown1: [u8; 12],
    /// "CH FADER CURVE" setting.
    pub channel_fader_curve: ChannelFaderCurve,
    /// "CROSSFADER CURVE" setting.
    pub crossfader_curve: CrossfaderCurve,
    /// "HEADPHONES PRE EQ" setting.
    pub headphones_pre_eq: HeadphonesPreEQ,
    /// "HEADPHONES MONO SPLIT" setting.
    pub headphones_mono_split: HeadphonesMonoSplit,
    /// "BEAT FX QUANTIZE" setting.
    pub beat_fx_quantize: BeatFXQuantize,
    /// "MIC LOW CUT" setting.
    pub mic_low_cut: MicLowCut,
    /// "TALK OVER MODE" setting.
    pub talk_over_mode: TalkOverMode,
    /// "TALK OVER LEVEL" setting.
    pub talk_over_level: TalkOverLevel,
    /// "MIDI CH" setting.
    pub midi_channel: MidiChannel,
    /// "MIDI BUTTON TYPE" setting.
    pub midi_button_type: MidiButtonType,
    /// "BRIGHTNESS > DISPLAY" setting.
    pub display_brightness: MixerDisplayBrightness,
    /// "BRIGHTNESS > INDICATOR" setting.
    pub indicator_brightness: MixerIndicatorBrightness,
    /// "CH FADER CURVE (LONG FADER)" setting.
    pub channel_fader_curve_long_fader: ChannelFaderCurveLongFader,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown2 == [0; 27]))]
    unknown2: [u8; 27],
}

impl Default for DJMMySetting {
    fn default() -> Self {
        Self {
            unknown1: [
                0x78, 0x56, 0x34, 0x12, 0x01, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00,
            ],
            channel_fader_curve: ChannelFaderCurve::default(),
            crossfader_curve: CrossfaderCurve::default(),
            headphones_pre_eq: HeadphonesPreEQ::default(),
            headphones_mono_split: HeadphonesMonoSplit::default(),
            beat_fx_quantize: BeatFXQuantize::default(),
            mic_low_cut: MicLowCut::default(),
            talk_over_mode: TalkOverMode::default(),
            talk_over_level: TalkOverLevel::default(),
            midi_channel: MidiChannel::default(),
            midi_button_type: MidiButtonType::default(),
            display_brightness: MixerDisplayBrightness::default(),
            indicator_brightness: MixerIndicatorBrightness::default(),
            channel_fader_curve_long_fader: ChannelFaderCurveLongFader::default(),
            unknown2: [0; 27],
        }
    }
}

/// Payload of a `MYSETTING.DAT` file (40 bytes).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(little)]
pub struct MySetting {
    /// Unknown field.
    unknown1: [u8; 8],
    /// "ON AIR DISPLAY" setting.
    pub on_air_display: OnAirDisplay,
    /// "LCD BRIGHTNESS" setting.
    pub lcd_brightness: LCDBrightness,
    /// "QUANTIZE" setting.
    pub quantize: Quantize,
    /// "AUTO CUE LEVEL" setting.
    pub auto_cue_level: AutoCueLevel,
    /// "LANGUAGE" setting.
    pub language: Language,
    /// Unknown field.
    unknown2: u8,
    /// "JOG RING BRIGHTNESS" setting.
    pub jog_ring_brightness: JogRingBrightness,
    /// "JOG RING INDICATOR" setting.
    pub jog_ring_indicator: JogRingIndicator,
    /// "SLIP FLASHING" setting.
    pub slip_flashing: SlipFlashing,
    /// Unknown field.
    unknown3: [u8; 3],
    /// "DISC SLOT ILLUMINATION" setting.
    pub disc_slot_illumination: DiscSlotIllumination,
    /// "EJECT/LOAD LOCK" setting.
    pub eject_lock: EjectLock,
    /// "SYNC" setting.
    pub sync: Sync,
    /// "PLAY MODE / AUTO PLAY MODE" setting.
    pub play_mode: PlayMode,
    /// Quantize Beat Value setting.
    pub quantize_beat_value: QuantizeBeatValue,
    /// "HOT CUE AUTO LOAD" setting.
    pub hotcue_autoload: HotCueAutoLoad,
    /// "HOT CUE COLOR" setting.
    pub hotcue_color: HotCueColor,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown4 == 0))]
    unknown4: u16,
    /// "NEEDLE LOCK" setting.
    pub needle_lock: NeedleLock,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown5 == 0))]
    unknown5: u16,
    /// "TIME MODE" setting.
    pub time_mode: TimeMode,
    /// "TIME MODE" setting.
    pub jog_mode: JogMode,
    /// "AUTO CUE" setting.
    pub auto_cue: AutoCue,
    /// "MASTER TEMPO" setting.
    pub master_tempo: MasterTempo,
    /// "TEMPO RANGE" setting.
    pub tempo_range: TempoRange,
    /// "PHASE METER" setting.
    pub phase_meter: PhaseMeter,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown6 == 0))]
    unknown6: u16,
}

impl Default for MySetting {
    fn default() -> Self {
        Self {
            unknown1: [0x78, 0x56, 0x34, 0x12, 0x02, 0x00, 0x00, 0x00],
            on_air_display: OnAirDisplay::default(),
            lcd_brightness: LCDBrightness::default(),
            quantize: Quantize::default(),
            auto_cue_level: AutoCueLevel::default(),
            language: Language::default(),
            unknown2: 0x01,
            jog_ring_brightness: JogRingBrightness::default(),
            jog_ring_indicator: JogRingIndicator::default(),
            slip_flashing: SlipFlashing::default(),
            unknown3: [0x01, 0x01, 0x01],
            disc_slot_illumination: DiscSlotIllumination::default(),
            eject_lock: EjectLock::default(),
            sync: Sync::default(),
            play_mode: PlayMode::default(),
            quantize_beat_value: QuantizeBeatValue::default(),
            hotcue_autoload: HotCueAutoLoad::default(),
            hotcue_color: HotCueColor::default(),
            unknown4: 0x0000,
            needle_lock: NeedleLock::default(),
            unknown5: 0x0000,
            time_mode: TimeMode::default(),
            jog_mode: JogMode::default(),
            auto_cue: AutoCue::default(),
            master_tempo: MasterTempo::default(),
            tempo_range: TempoRange::default(),
            phase_meter: PhaseMeter::default(),
            unknown6: 0x0000,
        }
    }
}

/// Payload of a `MYSETTING2.DAT` file (40 bytes).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(little)]
pub struct MySetting2 {
    /// "VINYL SPEED ADJUST" setting.
    pub vinyl_speed_adjust: VinylSpeedAdjust,
    /// "JOG DISPLAY MODE" setting.
    pub jog_display_mode: JogDisplayMode,
    /// "PAD/BUTTON BRIGHTNESS" setting.
    pub pad_button_brightness: PadButtonBrightness,
    /// "JOG LCD BRIGHTNESS" setting.
    pub jog_lcd_brightness: JogLCDBrightness,
    /// "WAVEFORM DIVISIONS" setting.
    pub waveform_divisions: WaveformDivisions,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown1 == [0; 5]))]
    unknown1: [u8; 5],
    /// "WAVEFORM / PHASE METER" setting.
    pub waveform: Waveform,
    /// Unknown field.
    unknown2: u8,
    /// "BEAT JUMP BEAT VALUE" setting.
    pub beat_jump_beat_value: BeatJumpBeatValue,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown3 == [0; 27]))]
    unknown3: [u8; 27],
}

impl Default for MySetting2 {
    fn default() -> Self {
        Self {
            vinyl_speed_adjust: VinylSpeedAdjust::default(),
            jog_display_mode: JogDisplayMode::default(),
            pad_button_brightness: PadButtonBrightness::default(),
            jog_lcd_brightness: JogLCDBrightness::default(),
            waveform_divisions: WaveformDivisions::default(),
            unknown1: [0; 5],
            waveform: Waveform::default(),
            unknown2: 0x81,
            beat_jump_beat_value: BeatJumpBeatValue::default(),
            unknown3: [0; 27],
        }
    }
}

/// Found at "PLAYER > DJ SETTING > PLAY MODE / AUTO PLAY MODE" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum PlayMode {
    /// Named "CONTINUE / ON" in the Rekordbox preferences.
    Continue = 0x80,
    /// Named "SINGLE / OFF" in the Rekordbox preferences.
    #[default]
    Single,
}

/// Found at "PLAYER > DJ SETTING > EJECT/LOAD LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum EjectLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    #[default]
    Unlock = 0x80,
    /// Named "LOCK" in the Rekordbox preferences.
    Lock,
}

/// Found at "PLAYER > DJ SETTING > NEEDLE LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum NeedleLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    Unlock = 0x80,
    /// Named "LOCK" in the Rekordbox preferences.
    #[default]
    Lock,
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum QuantizeBeatValue {
    /// Named "1/8 Beat" in the Rekordbox preferences.
    EighthBeat = 0x83,
    /// Named "1/4 Beat" in the Rekordbox preferences.
    QuarterBeat = 0x82,
    /// Named "1/2 Beat" in the Rekordbox preferences.
    HalfBeat = 0x81,
    /// Named "1 Beat" in the Rekordbox preferences.
    #[default]
    FullBeat = 0x80,
}

/// Found at "PLAYER > DJ SETTING > HOT CUE AUTO LOAD" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum HotCueAutoLoad {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "rekordbox SETTING" in the Rekordbox preferences.
    RekordboxSetting = 0x82,
    /// Named "On" in the Rekordbox preferences.
    #[default]
    On = 0x81,
}

/// Found at "PLAYER > DJ SETTING > HOT CUE COLOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum HotCueColor {
    /// Named "OFF" in the Rekordbox preferences.
    #[default]
    Off = 0x80,
    /// Named "On" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum AutoCueLevel {
    /// Named "-78dB" in the Rekordbox preferences.
    Minus78dB = 0x87,
    /// Named "-72dB" in the Rekordbox preferences.
    Minus72dB = 0x86,
    /// Named "-66dB" in the Rekordbox preferences.
    Minus66dB = 0x85,
    /// Named "-60dB" in the Rekordbox preferences.
    Minus60dB = 0x84,
    /// Named "-54dB" in the Rekordbox preferences.
    Minus54dB = 0x83,
    /// Named "-48dB" in the Rekordbox preferences.
    Minus48dB = 0x82,
    /// Named "-42dB" in the Rekordbox preferences.
    Minus42dB = 0x81,
    /// Named "-36dB" in the Rekordbox preferences.
    Minus36dB = 0x80,
    /// Named "MEMORY" in the Rekordbox preferences.
    #[default]
    Memory = 0x88,
}

/// Found at "PLAYER > DJ SETTING > TIME MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum TimeMode {
    /// Named "Elapsed" in the Rekordbox preferences.
    Elapsed = 0x80,
    /// Named "REMAIN" in the Rekordbox preferences.
    #[default]
    Remain,
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum AutoCue {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

/// Found at "PLAYER > DJ SETTING > JOG MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum JogMode {
    /// Named "VINYL" in the Rekordbox preferences.
    #[default]
    Vinyl = 0x81,
    /// Named "CDJ" in the Rekordbox preferences.
    CDJ = 0x80,
}

/// Found at "PLAYER > DJ SETTING > TEMPO RANGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum TempoRange {
    /// Named "±6" in the Rekordbox preferences.
    SixPercent = 0x80,
    /// Named "±10" in the Rekordbox preferences.
    #[default]
    TenPercent,
    /// Named "±16" in the Rekordbox preferences.
    SixteenPercent,
    /// Named "WIDE" in the Rekordbox preferences.
    Wide,
}

/// Found at "PLAYER > DJ SETTING > MASTER TEMPO" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum MasterTempo {
    /// Named "OFF" in the Rekordbox preferences.
    #[default]
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum Quantize {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

/// Found at "PLAYER > DJ SETTING > SYNC" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum Sync {
    /// Named "OFF" in the Rekordbox preferences.
    #[default]
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DJ SETTING > PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum PhaseMeter {
    /// Named "TYPE 1" in the Rekordbox preferences.
    #[default]
    Type1 = 0x80,
    /// Named "TYPE 2" in the Rekordbox preferences.
    Type2,
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM / PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum Waveform {
    /// Named "WAVEFORM" in the Rekordbox preferences.
    #[default]
    Waveform = 0x80,
    /// Named "PHASE METER" in the Rekordbox preferences.
    PhaseMeter,
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM DIVISIONS" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum WaveformDivisions {
    /// Named "TIME SCALE" in the Rekordbox preferences.
    TimeScale = 0x80,
    /// Named "PHRASE" in the Rekordbox preferences.
    #[default]
    Phrase,
}

/// Found at "PLAYER > DJ SETTING > VINYL SPEED ADJUST" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum VinylSpeedAdjust {
    /// Named "TOUCH & RELEASE" in the Rekordbox preferences.
    TouchRelease = 0x80,
    /// Named "TOUCH" in the Rekordbox preferences.
    #[default]
    Touch,
    /// Named "RELEASE" in the Rekordbox preferences.
    Release,
}

/// Found at "PLAYER > DJ SETTING > BEAT JUMP BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum BeatJumpBeatValue {
    /// Named "1/2 BEAT" in the Rekordbox preferences.
    HalfBeat = 0x80,
    /// Named "1 BEAT" in the Rekordbox preferences.
    OneBeat,
    /// Named "2 BEAT" in the Rekordbox preferences.
    TwoBeat,
    /// Named "4 BEAT" in the Rekordbox preferences.
    FourBeat,
    /// Named "8 BEAT" in the Rekordbox preferences.
    EightBeat,
    /// Named "16 BEAT" in the Rekordbox preferences.
    #[default]
    SixteenBeat,
    /// Named "32 BEAT" in the Rekordbox preferences.
    ThirtytwoBeat,
    /// Named "64 BEAT" in the Rekordbox preferences.
    SixtyfourBeat,
}

/// Found at "PLAYER > DISPLAY(LCD) > LANGUAGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum Language {
    /// Named "English" in the Rekordbox preferences.
    #[default]
    English = 0x81,
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
}

/// Found at "PLAYER > DISPLAY(LCD) > LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum LCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    One = 0x81,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[default]
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Named "5" in the Rekordbox preferences.
    Five,
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum JogLCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    One = 0x81,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[default]
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Named "5" in the Rekordbox preferences.
    Five,
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG DISPLAY MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum JogDisplayMode {
    /// Named "AUTO" in the Rekordbox preferences.
    #[default]
    Auto = 0x80,
    /// Named "INFO" in the Rekordbox preferences.
    Info,
    /// Named "SIMPLE" in the Rekordbox preferences.
    Simple,
    /// Named "ARTWORK" in the Rekordbox preferences.
    Artwork,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > SLIP FLASHING" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum SlipFlashing {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > ON AIR DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum OnAirDisplay {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum JogRingBrightness {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    #[default]
    Bright,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum JogRingIndicator {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > DISC SLOT ILLUMINATION" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum DiscSlotIllumination {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    #[default]
    Bright,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > PAD/BUTTON BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum PadButtonBrightness {
    /// Named "1" in the Rekordbox preferences.
    One = 0x81,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[default]
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum ChannelFaderCurve {
    /// Steep volume raise when the fader is moved near the top.
    SteepTop = 0x80,
    /// Linear volume raise when the fader is moved.
    #[default]
    Linear,
    /// Steep volume raise when the fader is moved near the bottom.
    SteepBottom,
}

/// Found at "MIXER > DJ SETTING > CROSSFADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum CrossfaderCurve {
    /// Logarithmic volume raise of the other channel near the edges of the fader.
    ConstantPower = 0x80,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    SlowCut,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    #[default]
    FastCut,
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE (LONG FADER)" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum ChannelFaderCurveLongFader {
    /// Very steep volume raise when the fader is moved the near the top (e.g. y = x⁵).
    #[default]
    Exponential = 0x80,
    /// Steep volume raise when the fader is moved the near the top (e.g. y = x²).
    Smooth,
    /// Linear volume raise when the fader is moved (e.g. y = k * x).
    Linear,
}

/// Found at "MIXER > DJ SETTING > HEADPHONES PRE EQ" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum HeadphonesPreEQ {
    /// Named "POST EQ" in the Rekordbox preferences.
    #[default]
    PostEQ = 0x80,
    /// Named "PRE EQ" in the Rekordbox preferences.
    PreEQ,
}

/// Found at "MIXER > DJ SETTING > HEADPHONES MONO SPLIT" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum HeadphonesMonoSplit {
    /// Named "MONO SPLIT" in the Rekordbox preferences.
    MonoSplit = 0x81,
    /// Named "STEREO" in the Rekordbox preferences.
    #[default]
    Stereo = 0x80,
}

/// Found at "MIXER > DJ SETTING > BEAT FX QUANTIZE" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum BeatFXQuantize {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

/// Found at "MIXER > DJ SETTING > MIC LOW CUT" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum MicLowCut {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON(for MC)" in the Rekordbox preferences.
    #[default]
    On,
}

/// Found at "MIXER > DJ SETTING > TALK OVER MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum TalkOverMode {
    /// Named "ADVANCED" in the Rekordbox preferences.
    #[default]
    Advanced = 0x80,
    /// Named "NORMAL" in the Rekordbox preferences.
    Normal,
}

/// Found at "MIXER > DJ SETTING > TALK OVER LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum TalkOverLevel {
    /// Named "-24dB" in the Rekordbox preferences.
    Minus24dB = 0x80,
    /// Named "-18dB" in the Rekordbox preferences.
    #[default]
    Minus18dB,
    /// Named "-12dB" in the Rekordbox preferences.
    Minus12dB,
    /// Named "-6dB" in the Rekordbox preferences.
    Minus6dB,
}

/// Found at "MIXER > DJ SETTING > MIDI CH" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum MidiChannel {
    /// Named "1" in the Rekordbox preferences.
    #[default]
    One = 0x80,
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
}

/// Found at "MIXER > DJ SETTING > MIDI BUTTON TYPE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum MidiButtonType {
    #[default]
    /// Named "TOGGLE" in the Rekordbox preferences.
    Toggle = 0x80,
    /// Named "TRIGGER" in the Rekordbox preferences.
    Trigger,
}

/// Found at "MIXER > BRIGHTNESS > DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum MixerDisplayBrightness {
    /// Named "WHITE" in the Rekordbox preferences.
    White = 0x80,
    /// Named "1" in the Rekordbox preferences.
    One,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[default]
    Five,
}

/// Found at "MIXER > BRIGHTNESS > INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum MixerIndicatorBrightness {
    /// Named "1" in the Rekordbox preferences.
    One = 0x80,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[default]
    Three,
}

/// Waveform color displayed on the CDJ.
///
/// Found on the "General" page in the Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum WaveformColor {
    /// Named "BLUE" in the Rekordbox preferences.
    #[default]
    Blue = 0x01,
    /// Named "RGB" in the Rekordbox preferences.
    Rgb = 0x03,
    /// Named "3Band" in the Rekordbox preferences.
    TriBand = 0x04,
}

/// Waveform Current Position displayed on the CDJ.
///
/// Found on the "General" page in the Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum WaveformCurrentPosition {
    /// Named "LEFT" in the Rekordbox preferences.
    Left = 0x02,
    /// Named "CENTER" in the Rekordbox preferences.
    #[default]
    Center = 0x01,
}

/// Type of the Overview Waveform displayed on the CDJ.
///
/// Found on the "General" page in the Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum OverviewWaveformType {
    /// Named "Half Waveform" in the Rekordbox preferences.
    #[default]
    HalfWaveform = 0x01,
    /// Named "Full Waveform" in the Rekordbox preferences.
    FullWaveform,
}

/// The key display format displayed on the CDJ.
///
/// Found on the "General" page in the Rekordbox preferences.
#[binrw]
#[derive(Debug, PartialEq, Eq, Default)]
#[brw(repr = u8)]
pub enum KeyDisplayFormat {
    /// Named "Classic" in the Rekordbox preferences.
    #[default]
    Classic = 0x01,
    /// Named "Alphanumeric" in the Rekordbox preferences.
    Alphanumeric,
}
