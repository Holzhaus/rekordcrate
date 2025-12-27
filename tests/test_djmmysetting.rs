// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{io::Cursor, BinRead};
use rekordcrate::setting::*;

macro_rules! read_djmmysetting {
    ($path:literal) => {{
        let data = include_bytes!($path);
        println!("Setting file: {}", $path);
        let mut reader = Cursor::new(data);
        let setting = Setting::read_args(&mut reader, (SettingType::DJMMySetting,))
            .expect("failed to parse setting file");
        let result = match setting.data {
            SettingData::DJMMySetting(x) => Some(x),
            _ => None,
        };
        result.expect("failed to match data section")
    }};
}

#[test]
fn read_djmmysetting_default() {
    let data = read_djmmysetting!("../data/complete_export/empty/PIONEER/DJMMYSETTING.DAT");
    assert_eq!(data, DJMMySetting::default());
}

#[test]
fn read_djmsetting_beatfxquantize_off() {
    let data = read_djmmysetting!("../data/djmmysetting/beatfxquantize_off/DJMMYSETTING.DAT");
    assert_eq!(data.beat_fx_quantize, BeatFXQuantize::Off);
}

#[test]
fn read_djmsetting_chfadercurvelong_exp2() {
    let data = read_djmmysetting!("../data/djmmysetting/chfadercurvelong_exp2/DJMMYSETTING.DAT");
    assert_eq!(
        data.channel_fader_curve_long_fader,
        ChannelFaderCurveLongFader::Smooth
    );
}

#[test]
fn read_djmsetting_chfadercurvelong_linear() {
    let data = read_djmmysetting!("../data/djmmysetting/chfadercurvelong_linear/DJMMYSETTING.DAT");
    assert_eq!(
        data.channel_fader_curve_long_fader,
        ChannelFaderCurveLongFader::Linear
    );
}

#[test]
fn read_djmsetting_chfadercurve_steepbottom() {
    let data = read_djmmysetting!("../data/djmmysetting/chfadercurve_steepbottom/DJMMYSETTING.DAT");
    assert_eq!(data.channel_fader_curve, ChannelFaderCurve::SteepBottom);
}

#[test]
fn read_djmsetting_chfadercurve_steeptop() {
    let data = read_djmmysetting!("../data/djmmysetting/chfadercurve_steeptop/DJMMYSETTING.DAT");
    assert_eq!(data.channel_fader_curve, ChannelFaderCurve::SteepTop);
}

#[test]
fn read_djmsetting_crossfadercurve_constantpower() {
    let data =
        read_djmmysetting!("../data/djmmysetting/crossfadercurve_constantpower/DJMMYSETTING.DAT");
    assert_eq!(data.crossfader_curve, CrossfaderCurve::ConstantPower);
}

#[test]
fn read_djmsetting_crossfadercurve_slowcut() {
    let data = read_djmmysetting!("../data/djmmysetting/crossfadercurve_slowcut/DJMMYSETTING.DAT");
    assert_eq!(data.crossfader_curve, CrossfaderCurve::SlowCut);
}

#[test]
fn read_djmsetting_headphones_monosplit_monosplit() {
    let data =
        read_djmmysetting!("../data/djmmysetting/headphones_monosplit_monosplit/DJMMYSETTING.DAT");
    assert_eq!(data.headphones_mono_split, HeadphonesMonoSplit::MonoSplit);
}

#[test]
fn read_djmsetting_headphones_preeq_preeq() {
    let data = read_djmmysetting!("../data/djmmysetting/headphones_preeq_preeq/DJMMYSETTING.DAT");
    assert_eq!(data.headphones_pre_eq, HeadphonesPreEQ::PreEQ);
}

#[test]
fn read_djmsetting_miclowcut_off() {
    let data = read_djmmysetting!("../data/djmmysetting/miclowcut_off/DJMMYSETTING.DAT");
    assert_eq!(data.mic_low_cut, MicLowCut::Off);
}

#[test]
fn read_djmsetting_midi_buttontype_trigger() {
    let data = read_djmmysetting!("../data/djmmysetting/midi_buttontype_trigger/DJMMYSETTING.DAT");
    assert_eq!(data.midi_button_type, MidiButtonType::Trigger);
}

#[test]
fn read_djmsetting_midi_ch_15() {
    let data = read_djmmysetting!("../data/djmmysetting/midi_ch_15/DJMMYSETTING.DAT");
    assert_eq!(data.midi_channel, MidiChannel::Fifteen);
}

#[test]
fn read_djmsetting_midi_ch_16() {
    let data = read_djmmysetting!("../data/djmmysetting/midi_ch_16/DJMMYSETTING.DAT");
    assert_eq!(data.midi_channel, MidiChannel::Sixteen);
}

#[test]
fn read_djmsetting_midi_ch_2() {
    let data = read_djmmysetting!("../data/djmmysetting/midi_ch_2/DJMMYSETTING.DAT");
    assert_eq!(data.midi_channel, MidiChannel::Two);
}

#[test]
fn read_djmsetting_mixer_brightness_display_1() {
    let data =
        read_djmmysetting!("../data/djmmysetting/mixer_brightness_display_1/DJMMYSETTING.DAT");
    assert_eq!(data.display_brightness, MixerDisplayBrightness::One);
}

#[test]
fn read_djmsetting_mixer_brightness_display_2() {
    let data =
        read_djmmysetting!("../data/djmmysetting/mixer_brightness_display_2/DJMMYSETTING.DAT");
    assert_eq!(data.display_brightness, MixerDisplayBrightness::Two);
}

#[test]
fn read_djmsetting_mixer_brightness_display_white() {
    let data =
        read_djmmysetting!("../data/djmmysetting/mixer_brightness_display_white/DJMMYSETTING.DAT");
    assert_eq!(data.display_brightness, MixerDisplayBrightness::White);
}

#[test]
fn read_djmsetting_mixer_brightness_indicator_1() {
    let data =
        read_djmmysetting!("../data/djmmysetting/mixer_brightness_indicator_1/DJMMYSETTING.DAT");
    assert_eq!(data.indicator_brightness, MixerIndicatorBrightness::One);
}

#[test]
fn read_djmsetting_mixer_brightness_indicator_2() {
    let data =
        read_djmmysetting!("../data/djmmysetting/mixer_brightness_indicator_2/DJMMYSETTING.DAT");
    assert_eq!(data.indicator_brightness, MixerIndicatorBrightness::Two);
}

#[test]
fn read_djmsetting_talkover_level_12() {
    let data = read_djmmysetting!("../data/djmmysetting/talkover_level_12/DJMMYSETTING.DAT");
    assert_eq!(data.talk_over_level, TalkOverLevel::Minus12dB);
}

#[test]
fn read_djmsetting_talkover_level_24() {
    let data = read_djmmysetting!("../data/djmmysetting/talkover_level_24/DJMMYSETTING.DAT");
    assert_eq!(data.talk_over_level, TalkOverLevel::Minus24dB);
}

#[test]
fn read_djmsetting_talkover_level_6() {
    let data = read_djmmysetting!("../data/djmmysetting/talkover_level_6/DJMMYSETTING.DAT");
    assert_eq!(data.talk_over_level, TalkOverLevel::Minus6dB);
}

#[test]
fn read_djmsetting_talkover_mode_normal() {
    let data = read_djmmysetting!("../data/djmmysetting/talkover_mode_normal/DJMMYSETTING.DAT");
    assert_eq!(data.talk_over_mode, TalkOverMode::Normal);
}
