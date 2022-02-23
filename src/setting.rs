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

use binrw::{binrw, NullString};

#[derive(Debug, PartialEq)]
#[binrw]
#[brw(little)]
/// Represents a setting file.
pub struct Setting {
    /// Size of the string data field (should be always 96).
    #[br(assert(len_stringdata == 0x60))]
    pub len_stringdata: u32,
    /// Name of the company ("PIONEER").
    #[brw(pad_size_to = 0x20)]
    pub company: NullString,
    /// Name of the software ("rekordbox").
    #[brw(pad_size_to = 0x20)]
    pub software: NullString,
    /// Some kind of version number.
    #[brw(pad_size_to = 0x20)]
    pub version: NullString,
    /// Size of the `data` data in bytes.
    pub len_data: u32,
    /// The actual settings data.
    #[br(args(len_data))]
    pub data: SettingData,
    /// CRC16 XMODEM checksum. The checksum is calculated over the contents of the `data`
    /// field, except for `DJMSETTING.DAT` files where the checksum is calculated over all
    /// preceding bytes including the length fields.
    ///
    /// See <https://reveng.sourceforge.io/crc-catalogue/all.htm#crc.cat.crc-16-xmodem> for
    /// details.
    pub checksum: u16,
    /// Unknown field (apparently always `0000`).
    #[br(assert(unknown == 0))]
    pub unknown: u16,
}

/// Data section of a `*SETTING.DAT` file.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(little)]
#[br(import(len: u32))]
pub enum SettingData {
    /// Payload of a `DEVSETTING.DAT` file (32 bytes).
    #[br(pre_assert(len == 32))]
    DevSetting(#[br(count = len)] Vec<u8>),
    /// Payload of a `DJMMYSETTING.DAT` file (52 bytes).
    #[br(pre_assert(len == 52))]
    DJMMySetting {
        /// Unknown field.
        unknown1: [u8; 12],
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
        /// Unknown field (apparently always 0).
        #[br(assert(unknown2 == [0; 27]))]
        unknown2: [u8; 27],
    },
    /// Payload of a `MYSETTING.DAT` file (40 bytes).
    #[br(pre_assert(len == 40))]
    MySetting {
        /// Unknown field.
        unknown1: [u8; 8],
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
        unknown3: [u8; 3],
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
        #[br(assert(unknown4 == 0))]
        unknown4: u16,
        /// "NEEDLE LOCK" setting.
        needle_lock: NeedleLock,
        /// Unknown field (apparently always 0).
        #[br(assert(unknown5 == 0))]
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
        #[br(assert(unknown6 == 0))]
        unknown6: u16,
    },
    /// Payload of a `MYSETTING2.DAT` file (40 bytes).
    #[br(pre_assert(len == 40))]
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
        /// Unknown field (apparently always 0).
        #[br(assert(unknown1 == [0; 5]))]
        unknown1: [u8; 5],
        /// "WAVEFORM / PHASE METER" setting.
        waveform: Waveform,
        /// Unknown field.
        unknown2: u8,
        /// "BEAT JUMP BEAT VALUE" setting.
        beat_jump_beat_value: BeatJumpBeatValue,
        /// Unknown field (apparently always 0).
        #[br(assert(unknown3 == [0; 27]))]
        unknown3: [u8; 27],
    },
}

/// Found at "PLAYER > DJ SETTING > PLAY MODE / AUTO PLAY MODE" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum PlayMode {
    /// Named "CONTINUE / ON" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Continue,
    /// Named "SINGLE / OFF" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Single,
}

/// Found at "PLAYER > DJ SETTING > EJECT/LOAD LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum EjectLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Unlock,
    /// Named "LOCK" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Lock,
}

/// Found at "PLAYER > DJ SETTING > NEEDLE LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum NeedleLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Unlock,
    /// Named "LOCK" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Lock,
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum QuantizeBeatValue {
    /// Named "1/8 Beat" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    EighthBeat,
    /// Named "1/4 Beat" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    QuarterBeat,
    /// Named "1/2 Beat" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    HalfBeat,
    /// Named "1 Beat" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    FullBeat,
}

/// Found at "PLAYER > DJ SETTING > HOT CUE AUTO LOAD" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum HotCueAutoLoad {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "rekordbox SETTING" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    RekordboxSetting,
    /// Named "On" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "PLAYER > DJ SETTING > HOT CUE COLOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum HotCueColor {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "On" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum AutoCueLevel {
    /// Named "-78dB" in the Rekordbox preferences.
    #[brw(magic = 0x87u8)]
    Minus78dB,
    /// Named "-72dB" in the Rekordbox preferences.
    #[brw(magic = 0x86u8)]
    Minus72dB,
    /// Named "-66dB" in the Rekordbox preferences.
    #[brw(magic = 0x85u8)]
    Minus66dB,
    /// Named "-60dB" in the Rekordbox preferences.
    #[brw(magic = 0x84u8)]
    Minus60dB,
    /// Named "-54dB" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    Minus54dB,
    /// Named "-48dB" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Minus48dB,
    /// Named "-42dB" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Minus42dB,
    /// Named "-36dB" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Minus36dB,
    /// Named "MEMORY" in the Rekordbox preferences.
    #[brw(magic = 0x88u8)]
    Memory,
}

/// Found at "PLAYER > DJ SETTING > TIME MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum TimeMode {
    /// Named "Elapsed" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Elapsed,
    /// Named "REMAIN" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Remain,
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum AutoCue {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "ON" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "PLAYER > DJ SETTING > JOG MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum JogMode {
    /// Named "VINYL" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Vinyl,
    /// Named "CDJ" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    CDJ,
}

/// Found at "PLAYER > DJ SETTING > TEMPO RANGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum TempoRange {
    /// Named "±6" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    SixPercent,
    /// Named "±10" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    TenPercent,
    /// Named "±16" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    SixteenPercent,
    /// Named "WIDE" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    Wide,
}

/// Found at "PLAYER > DJ SETTING > MASTER TEMPO" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum MasterTempo {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "ON" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum Quantize {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "ON" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "PLAYER > DJ SETTING > SYNC" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum Sync {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "ON" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "PLAYER > DJ SETTING > PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum PhaseMeter {
    /// Named "TYPE 1" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Type1,
    /// Named "TYPE 2" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Type2,
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM / PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum Waveform {
    /// Named "WAVEFORM" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Waveform,
    /// Named "PHASE METER" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    PhaseMeter,
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM DIVISIONS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum WaveformDivisions {
    /// Named "TIME SCALE" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    TimeScale,
    /// Named "PHRASE" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Phrase,
}

/// Found at "PLAYER > DJ SETTING > VINYL SPEED ADJUST" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum VinylSpeedAdjust {
    /// Named "TOUCH & RELEASE" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    TouchRelease,
    /// Named "TOUCH" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Touch,
    /// Named "RELEASE" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Release,
}

/// Found at "PLAYER > DJ SETTING > BEAT JUMP BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum BeatJumpBeatValue {
    /// Named "1/2 BEAT" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    HalfBeat,
    /// Named "1 BEAT" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    OneBeat,
    /// Named "2 BEAT" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    TwoBeat,
    /// Named "4 BEAT" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    FourBeat,
    /// Named "8 BEAT" in the Rekordbox preferences.
    #[brw(magic = 0x84u8)]
    EightBeat,
    /// Named "16 BEAT" in the Rekordbox preferences.
    #[brw(magic = 0x85u8)]
    SixteenBeat,
    /// Named "32 BEAT" in the Rekordbox preferences.
    #[brw(magic = 0x86u8)]
    ThirtytwoBeat,
    /// Named "64 BEAT" in the Rekordbox preferences.
    #[brw(magic = 0x87u8)]
    SixtyfourBeat,
}

/// Found at "PLAYER > DISPLAY(LCD) > LANGUAGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum Language {
    /// Named "English" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    English,
    /// Named "Français" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    French,
    /// Named "Deutsch" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    German,
    /// Named "Italiano" in the Rekordbox preferences.
    #[brw(magic = 0x84u8)]
    Italian,
    /// Named "Nederlands" in the Rekordbox preferences.
    #[brw(magic = 0x85u8)]
    Dutch,
    /// Named "Español" in the Rekordbox preferences.
    #[brw(magic = 0x86u8)]
    Spanish,
    /// Named "Русский" in the Rekordbox preferences.
    #[brw(magic = 0x87u8)]
    Russian,
    /// Named "한국어" in the Rekordbox preferences.
    #[brw(magic = 0x88u8)]
    Korean,
    /// Named "简体中文" in the Rekordbox preferences.
    #[brw(magic = 0x89u8)]
    ChineseSimplified,
    /// Named "繁體中文" in the Rekordbox preferences.
    #[brw(magic = 0x8au8)]
    ChineseTraditional,
    /// Named "日本語" in the Rekordbox preferences.
    #[brw(magic = 0x8bu8)]
    Japanese,
    /// Named "Português" in the Rekordbox preferences.
    #[brw(magic = 0x8cu8)]
    Portuguese,
    /// Named "Svenska" in the Rekordbox preferences.
    #[brw(magic = 0x8du8)]
    Swedish,
    /// Named "Čeština" in the Rekordbox preferences.
    #[brw(magic = 0x8eu8)]
    Czech,
    /// Named "Magyar" in the Rekordbox preferences.
    #[brw(magic = 0x8fu8)]
    Hungarian,
    /// Named "Dansk" in the Rekordbox preferences.
    #[brw(magic = 0x90u8)]
    Danish,
    /// Named "Ελληνικά" in the Rekordbox preferences.
    #[brw(magic = 0x91u8)]
    Greek,
    /// Named "Türkçe" in the Rekordbox preferences.
    #[brw(magic = 0x92u8)]
    Turkish,
}

/// Found at "PLAYER > DISPLAY(LCD) > LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum LCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    One,
    /// Named "2" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[brw(magic = 0x84u8)]
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[brw(magic = 0x85u8)]
    Five,
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum JogLCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    One,
    /// Named "2" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[brw(magic = 0x84u8)]
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[brw(magic = 0x85u8)]
    Five,
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG DISPLAY MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum JogDisplayMode {
    /// Named "AUTO" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Auto,
    /// Named "INFO" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Info,
    /// Named "SIMPLE" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Simple,
    /// Named "ARTWORK" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    Artwork,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > SLIP FLASHING" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum SlipFlashing {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "ON" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > ON AIR DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum OnAirDisplay {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "ON" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum JogRingBrightness {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Bright,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum JogRingIndicator {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "ON" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > DISC SLOT ILLUMINATION" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum DiscSlotIllumination {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Bright,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > PAD/BUTTON BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum PadButtonBrightness {
    /// Named "1" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    One,
    /// Named "2" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[brw(magic = 0x84u8)]
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[brw(magic = 0x85u8)]
    Five,
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum ChannelFaderCurve {
    /// Steep volume raise when the fader is moved near the top.
    #[brw(magic = 0x80u8)]
    SteepTop,
    /// Linear volume raise when the fader is moved.
    #[brw(magic = 0x81u8)]
    Linear,
    /// Steep volume raise when the fader is moved near the bottom.
    #[brw(magic = 0x82u8)]
    SteepBottom,
}

/// Found at "MIXER > DJ SETTING > CROSSFADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum CrossfaderCurve {
    /// Logarithmic volume raise of the other channel near the edges of the fader.
    #[brw(magic = 0x80u8)]
    ConstantPower,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    #[brw(magic = 0x81u8)]
    SlowCut,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    #[brw(magic = 0x82u8)]
    FastCut,
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE (LONG FADER)" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum ChannelFaderCurveLongFader {
    /// Very steep volume raise when the fader is moved the near the top (e.g. y = x⁵).
    #[brw(magic = 0x80u8)]
    Exponential,
    /// Steep volume raise when the fader is moved the near the top (e.g. y = x²).
    #[brw(magic = 0x81u8)]
    Smooth,
    /// Linear volume raise when the fader is moved (e.g. y = k * x).
    #[brw(magic = 0x82u8)]
    Linear,
}

/// Found at "MIXER > DJ SETTING > HEADPHONES PRE EQ" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum HeadphonesPreEQ {
    /// Named "POST EQ" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    PostEQ,
    /// Named "PRE EQ" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    PreEQ,
}

/// Found at "MIXER > DJ SETTING > HEADPHONES MONO SPLIT" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum HeadphonesMonoSplit {
    /// Named "MONO SPLIT" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    MonoSplit,
    /// Named "STEREO" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Stereo,
}

/// Found at "MIXER > DJ SETTING > BEAT FX QUANTIZE" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum BeatFXQuantize {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "ON" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "MIXER > DJ SETTING > MIC LOW CUT" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum MicLowCut {
    /// Named "OFF" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Off,
    /// Named "ON(for MC)" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    On,
}

/// Found at "MIXER > DJ SETTING > TALK OVER MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum TalkOverMode {
    /// Named "ADVANCED" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Advanced,
    /// Named "NORMAL" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Normal,
}

/// Found at "MIXER > DJ SETTING > TALK OVER LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum TalkOverLevel {
    /// Named "-24dB" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Minus24dB,
    /// Named "-18dB" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Minus18dB,
    /// Named "-12dB" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Minus12dB,
    /// Named "-6dB" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    Minus6dB,
}

/// Found at "MIXER > DJ SETTING > MIDI CH" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum MidiChannel {
    /// Named "1" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    One,
    /// Named "2" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[brw(magic = 0x84u8)]
    Five,
    /// Named "6" in the Rekordbox preferences.
    #[brw(magic = 0x85u8)]
    Six,
    /// Named "7" in the Rekordbox preferences.
    #[brw(magic = 0x86u8)]
    Seven,
    /// Named "8" in the Rekordbox preferences.
    #[brw(magic = 0x87u8)]
    Eight,
    /// Named "9" in the Rekordbox preferences.
    #[brw(magic = 0x88u8)]
    Nine,
    /// Named "10" in the Rekordbox preferences.
    #[brw(magic = 0x89u8)]
    Ten,
    /// Named "11" in the Rekordbox preferences.
    #[brw(magic = 0x8au8)]
    Eleven,
    /// Named "12" in the Rekordbox preferences.
    #[brw(magic = 0x8bu8)]
    Twelve,
    /// Named "13" in the Rekordbox preferences.
    #[brw(magic = 0x8cu8)]
    Thirteen,
    /// Named "14" in the Rekordbox preferences.
    #[brw(magic = 0x8du8)]
    Fourteen,
    /// Named "15" in the Rekordbox preferences.
    #[brw(magic = 0x8eu8)]
    Fifteen,
    /// Named "16" in the Rekordbox preferences.
    #[brw(magic = 0x8fu8)]
    Sixteen,
}

/// Found at "MIXER > DJ SETTING > MIDI BUTTON TYPE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum MidiButtonType {
    /// Named "TOGGLE" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    Toggle,
    /// Named "TRIGGER" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Trigger,
}

/// Found at "MIXER > BRIGHTNESS > DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum MixerDisplayBrightness {
    /// Named "WHITE" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    White,
    /// Named "1" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    One,
    /// Named "2" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[brw(magic = 0x83u8)]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[brw(magic = 0x84u8)]
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[brw(magic = 0x85u8)]
    Five,
}

/// Found at "MIXER > BRIGHTNESS > INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
pub enum MixerIndicatorBrightness {
    /// Named "1" in the Rekordbox preferences.
    #[brw(magic = 0x80u8)]
    One,
    /// Named "2" in the Rekordbox preferences.
    #[brw(magic = 0x81u8)]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[brw(magic = 0x82u8)]
    Three,
}
