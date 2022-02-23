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
    #[brw(pad_size_to = 0x20)]
    pub brand: NullString,
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
#[brw(repr = u8)]
pub enum PlayMode {
    /// Named "CONTINUE / ON" in the Rekordbox preferences.
    Continue = 0x80,
    /// Named "SINGLE / OFF" in the Rekordbox preferences.
    Single,
}

/// Found at "PLAYER > DJ SETTING > EJECT/LOAD LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum EjectLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    Unlock = 0x80,
    /// Named "LOCK" in the Rekordbox preferences.
    Lock,
}

/// Found at "PLAYER > DJ SETTING > NEEDLE LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum NeedleLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    Unlock = 0x80,
    /// Named "LOCK" in the Rekordbox preferences.
    Lock,
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum QuantizeBeatValue {
    /// Named "1/8 Beat" in the Rekordbox preferences.
    EighthBeat = 0x83,
    /// Named "1/4 Beat" in the Rekordbox preferences.
    QuarterBeat = 0x82,
    /// Named "1/2 Beat" in the Rekordbox preferences.
    HalfBeat = 0x81,
    /// Named "1 Beat" in the Rekordbox preferences.
    FullBeat = 0x80,
}

/// Found at "PLAYER > DJ SETTING > HOT CUE AUTO LOAD" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum HotCueAutoLoad {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "rekordbox SETTING" in the Rekordbox preferences.
    RekordboxSetting = 0x82,
    /// Named "On" in the Rekordbox preferences.
    On = 0x81,
}

/// Found at "PLAYER > DJ SETTING > HOT CUE COLOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum HotCueColor {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "On" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
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
    Memory = 0x88,
}

/// Found at "PLAYER > DJ SETTING > TIME MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum TimeMode {
    /// Named "Elapsed" in the Rekordbox preferences.
    Elapsed = 0x80,
    /// Named "REMAIN" in the Rekordbox preferences.
    Remain,
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum AutoCue {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DJ SETTING > JOG MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum JogMode {
    /// Named "VINYL" in the Rekordbox preferences.
    Vinyl = 0x80,
    /// Named "CDJ" in the Rekordbox preferences.
    CDJ = 0x81,
}

/// Found at "PLAYER > DJ SETTING > TEMPO RANGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum TempoRange {
    /// Named "±6" in the Rekordbox preferences.
    SixPercent = 0x80,
    /// Named "±10" in the Rekordbox preferences.
    TenPercent,
    /// Named "±16" in the Rekordbox preferences.
    SixteenPercent,
    /// Named "WIDE" in the Rekordbox preferences.
    Wide,
}

/// Found at "PLAYER > DJ SETTING > MASTER TEMPO" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum MasterTempo {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum Quantize {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DJ SETTING > SYNC" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum Sync {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DJ SETTING > PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum PhaseMeter {
    /// Named "TYPE 1" in the Rekordbox preferences.
    Type1 = 0x80,
    /// Named "TYPE 2" in the Rekordbox preferences.
    Type2,
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM / PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum Waveform {
    /// Named "WAVEFORM" in the Rekordbox preferences.
    Waveform = 0x80,
    /// Named "PHASE METER" in the Rekordbox preferences.
    PhaseMeter,
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM DIVISIONS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum WaveformDivisions {
    /// Named "TIME SCALE" in the Rekordbox preferences.
    TimeScale = 0x80,
    /// Named "PHRASE" in the Rekordbox preferences.
    Phrase,
}

/// Found at "PLAYER > DJ SETTING > VINYL SPEED ADJUST" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum VinylSpeedAdjust {
    /// Named "TOUCH & RELEASE" in the Rekordbox preferences.
    TouchRelease = 0x80,
    /// Named "TOUCH" in the Rekordbox preferences.
    Touch,
    /// Named "RELEASE" in the Rekordbox preferences.
    Release,
}

/// Found at "PLAYER > DJ SETTING > BEAT JUMP BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
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
    SixteenBeat,
    /// Named "32 BEAT" in the Rekordbox preferences.
    ThirtytwoBeat,
    /// Named "64 BEAT" in the Rekordbox preferences.
    SixtyfourBeat,
}

/// Found at "PLAYER > DISPLAY(LCD) > LANGUAGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum Language {
    /// Named "English" in the Rekordbox preferences.
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
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum LCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    One = 0x81,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Named "5" in the Rekordbox preferences.
    Five,
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum JogLCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    One = 0x81,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Named "5" in the Rekordbox preferences.
    Five,
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG DISPLAY MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum JogDisplayMode {
    /// Named "AUTO" in the Rekordbox preferences.
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
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum SlipFlashing {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > ON AIR DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum OnAirDisplay {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum JogRingBrightness {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    Bright,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum JogRingIndicator {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > DISC SLOT ILLUMINATION" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum DiscSlotIllumination {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    Bright,
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > PAD/BUTTON BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum PadButtonBrightness {
    /// Named "1" in the Rekordbox preferences.
    One = 0x81,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
    /// Named "4" in the Rekordbox preferences.
    Four,
    /// Named "5" in the Rekordbox preferences.
    Five,
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum ChannelFaderCurve {
    /// Steep volume raise when the fader is moved near the top.
    SteepTop = 0x80,
    /// Linear volume raise when the fader is moved.
    Linear,
    /// Steep volume raise when the fader is moved near the bottom.
    SteepBottom,
}

/// Found at "MIXER > DJ SETTING > CROSSFADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum CrossfaderCurve {
    /// Logarithmic volume raise of the other channel near the edges of the fader.
    ConstantPower = 0x80,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    SlowCut,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    FastCut,
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE (LONG FADER)" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum ChannelFaderCurveLongFader {
    /// Very steep volume raise when the fader is moved the near the top (e.g. y = x⁵).
    Exponential = 0x80,
    /// Steep volume raise when the fader is moved the near the top (e.g. y = x²).
    Smooth,
    /// Linear volume raise when the fader is moved (e.g. y = k * x).
    Linear,
}

/// Found at "MIXER > DJ SETTING > HEADPHONES PRE EQ" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum HeadphonesPreEQ {
    /// Named "POST EQ" in the Rekordbox preferences.
    PostEQ = 0x80,
    /// Named "PRE EQ" in the Rekordbox preferences.
    PreEQ,
}

/// Found at "MIXER > DJ SETTING > HEADPHONES MONO SPLIT" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum HeadphonesMonoSplit {
    /// Named "MONO SPLIT" in the Rekordbox preferences.
    MonoSplit = 0x80,
    /// Named "STEREO" in the Rekordbox preferences.
    Stereo,
}

/// Found at "MIXER > DJ SETTING > BEAT FX QUANTIZE" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum BeatFXQuantize {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

/// Found at "MIXER > DJ SETTING > MIC LOW CUT" of the "My Settings" page in the
/// Rekordbox preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum MicLowCut {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON(for MC)" in the Rekordbox preferences.
    On,
}

/// Found at "MIXER > DJ SETTING > TALK OVER MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum TalkOverMode {
    /// Named "ADVANCED" in the Rekordbox preferences.
    Advanced = 0x80,
    /// Named "NORMAL" in the Rekordbox preferences.
    Normal,
}

/// Found at "MIXER > DJ SETTING > TALK OVER LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum TalkOverLevel {
    /// Named "-24dB" in the Rekordbox preferences.
    Minus24dB = 0x80,
    /// Named "-18dB" in the Rekordbox preferences.
    Minus18dB,
    /// Named "-12dB" in the Rekordbox preferences.
    Minus12dB,
    /// Named "-6dB" in the Rekordbox preferences.
    Minus6dB,
}

/// Found at "MIXER > DJ SETTING > MIDI CH" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum MidiChannel {
    /// Named "1" in the Rekordbox preferences.
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
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum MidiButtonType {
    /// Named "TOGGLE" in the Rekordbox preferences.
    Toggle = 0x80,
    /// Named "TRIGGER" in the Rekordbox preferences.
    Trigger,
}

/// Found at "MIXER > BRIGHTNESS > DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
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
    Five,
}

/// Found at "MIXER > BRIGHTNESS > INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[derive(Debug, PartialEq)]
#[binrw]
#[brw(repr = u8)]
pub enum MixerIndicatorBrightness {
    /// Named "1" in the Rekordbox preferences.
    One = 0x80,
    /// Named "2" in the Rekordbox preferences.
    Two,
    /// Named "3" in the Rekordbox preferences.
    Three,
}
