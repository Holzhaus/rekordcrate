// Copyright (c) 2023 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{io::Cursor, BinRead};
use rekordcrate::setting::*;

macro_rules! read_mysetting2 {
    ($path:literal) => {{
        let data = include_bytes!($path);
        println!("Setting file: {}", $path);
        let mut reader = Cursor::new(data);
        let setting = Setting::read(&mut reader).expect("failed to parse setting file");
        let result = match setting.data {
            SettingData::MySetting2(x) => Some(x),
            _ => None,
        };
        result.expect("failed to match data section")
    }};
}

#[test]
fn read_mysetting2_default() {
    let data = read_mysetting2!("../data/complete_export/empty/PIONEER/MYSETTING2.DAT");
    assert_eq!(data, MySetting2::default());
}

#[test]
fn read_mysetting2_beatjumpbeatvalue_64() {
    let data = read_mysetting2!("../data/mysetting2/beatjumpbeatvalue_64/MYSETTING2.DAT");
    assert_eq!(data.beat_jump_beat_value, BeatJumpBeatValue::SixtyfourBeat);
}

#[test]
fn read_mysetting2_beatjumpbeatvalue_half() {
    let data = read_mysetting2!("../data/mysetting2/beatjumpbeatvalue_half/MYSETTING2.DAT");
    assert_eq!(data.beat_jump_beat_value, BeatJumpBeatValue::HalfBeat);
}

#[test]
fn read_mysetting2_beatjumpbeatvalue_one() {
    let data = read_mysetting2!("../data/mysetting2/beatjumpbeatvalue_one/MYSETTING2.DAT");
    assert_eq!(data.beat_jump_beat_value, BeatJumpBeatValue::OneBeat);
}

#[test]
fn read_mysetting2_jogdisplaymode_artwork() {
    let data = read_mysetting2!("../data/mysetting2/jogdisplaymode_artwork/MYSETTING2.DAT");
    assert_eq!(data.jog_display_mode, JogDisplayMode::Artwork);
}

#[test]
fn read_mysetting2_jogdisplaymode_info() {
    let data = read_mysetting2!("../data/mysetting2/jogdisplaymode_info/MYSETTING2.DAT");
    assert_eq!(data.jog_display_mode, JogDisplayMode::Info);
}

#[test]
fn read_mysetting2_jogdisplaymode_simple() {
    let data = read_mysetting2!("../data/mysetting2/jogdisplaymode_simple/MYSETTING2.DAT");
    assert_eq!(data.jog_display_mode, JogDisplayMode::Simple);
}

#[test]
fn read_mysetting2_joglcdbrightness_1() {
    let data = read_mysetting2!("../data/mysetting2/joglcdbrightness_1/MYSETTING2.DAT");
    assert_eq!(data.jog_lcd_brightness, JogLCDBrightness::One);
}

#[test]
fn read_mysetting2_joglcdbrightness_5() {
    let data = read_mysetting2!("../data/mysetting2/joglcdbrightness_5/MYSETTING2.DAT");
    assert_eq!(data.jog_lcd_brightness, JogLCDBrightness::Five);
}

#[test]
fn read_mysetting2_padbuttonbrightness_1() {
    let data = read_mysetting2!("../data/mysetting2/padbuttonbrightness_1/MYSETTING2.DAT");
    assert_eq!(data.pad_button_brightness, PadButtonBrightness::One);
}

#[test]
fn read_mysetting2_padbuttonbrightness_5() {
    let data = read_mysetting2!("../data/mysetting2/padbuttonbrightness_4/MYSETTING2.DAT");
    assert_eq!(data.pad_button_brightness, PadButtonBrightness::Four);
}

#[test]
fn read_mysetting2_vinylspeedadjust_release() {
    let data = read_mysetting2!("../data/mysetting2/vinylspeedadjust_release/MYSETTING2.DAT");
    assert_eq!(data.vinyl_speed_adjust, VinylSpeedAdjust::Release);
}

#[test]
fn read_mysetting2_vinylspeedadjust_touchrelease() {
    let data = read_mysetting2!("../data/mysetting2/vinylspeedadjust_touchrelease/MYSETTING2.DAT");
    assert_eq!(data.vinyl_speed_adjust, VinylSpeedAdjust::TouchRelease);
}

#[test]
fn read_mysetting2_waveformdivisions_timescale() {
    let data = read_mysetting2!("../data/mysetting2/waveformdivisions_timescale/MYSETTING2.DAT");
    assert_eq!(data.waveform_divisions, WaveformDivisions::TimeScale);
}

#[test]
fn read_mysetting2_waveform_phasemeter() {
    let data = read_mysetting2!("../data/mysetting2/waveform_phasemeter/MYSETTING2.DAT");
    assert_eq!(data.waveform, Waveform::PhaseMeter);
}
