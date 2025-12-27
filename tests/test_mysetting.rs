// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{io::Cursor, BinRead};
use rekordcrate::setting::*;

macro_rules! read_mysetting {
    ($path:literal) => {{
        let data = include_bytes!($path);
        println!("Setting file: {}", $path);
        let mut reader = Cursor::new(data);
        let setting = Setting::read_args(&mut reader, (SettingType::MySetting,))
            .expect("failed to parse setting file");
        let result = match setting.data {
            SettingData::MySetting(x) => Some(x),
            _ => None,
        };
        result.expect("failed to match data section")
    }};
}

#[test]
fn read_mysetting_default() {
    let data = read_mysetting!("../data/complete_export/empty/PIONEER/MYSETTING.DAT");
    assert_eq!(data, MySetting::default());
}

#[test]
fn read_mysetting_autocuelevel_36() {
    let data = read_mysetting!("../data/mysetting/autocuelevel_36/MYSETTING.DAT");
    assert_eq!(data.auto_cue_level, AutoCueLevel::Minus36dB);
}

#[test]
fn read_mysetting_autocuelevel_42() {
    let data = read_mysetting!("../data/mysetting/autocuelevel_42/MYSETTING.DAT");
    assert_eq!(data.auto_cue_level, AutoCueLevel::Minus42dB);
}

#[test]
fn read_mysetting_autocuelevel_72() {
    let data = read_mysetting!("../data/mysetting/autocuelevel_72/MYSETTING.DAT");
    assert_eq!(data.auto_cue_level, AutoCueLevel::Minus72dB);
}

#[test]
fn read_mysetting_autocuelevel_78() {
    let data = read_mysetting!("../data/mysetting/autocuelevel_78/MYSETTING.DAT");
    assert_eq!(data.auto_cue_level, AutoCueLevel::Minus78dB);
}

#[test]
fn read_mysetting_autocue_off() {
    let data = read_mysetting!("../data/mysetting/autocue_off/MYSETTING.DAT");
    assert_eq!(data.auto_cue, AutoCue::Off);
}

#[test]
fn read_mysetting_discslotillumination_dark() {
    let data = read_mysetting!("../data/mysetting/discslotillumination_dark/MYSETTING.DAT");
    assert_eq!(data.disc_slot_illumination, DiscSlotIllumination::Dark);
}

#[test]
fn read_mysetting_discslotillumination_off() {
    let data = read_mysetting!("../data/mysetting/discslotillumination_off/MYSETTING.DAT");
    assert_eq!(data.disc_slot_illumination, DiscSlotIllumination::Off);
}

#[test]
fn read_mysetting_hotcueautoload_off() {
    let data = read_mysetting!("../data/mysetting/hotcueautoload_off/MYSETTING.DAT");
    assert_eq!(data.hotcue_autoload, HotCueAutoLoad::Off);
}

#[test]
fn read_mysetting_hotcueautoload_rekordboxsetting() {
    let data = read_mysetting!("../data/mysetting/hotcueautoload_rekordboxsetting/MYSETTING.DAT");
    assert_eq!(data.hotcue_autoload, HotCueAutoLoad::RekordboxSetting);
}

#[test]
fn read_mysetting_hotcuecolor_on() {
    let data = read_mysetting!("../data/mysetting/hotcuecolor_on/MYSETTING.DAT");
    assert_eq!(data.hotcue_color, HotCueColor::On);
}

#[test]
fn read_mysetting_jogmode_cdj() {
    let data = read_mysetting!("../data/mysetting/jogmode_cdj/MYSETTING.DAT");
    assert_eq!(data.jog_mode, JogMode::CDJ);
}

#[test]
fn read_mysetting_jogringbrightness_dark() {
    let data = read_mysetting!("../data/mysetting/jogringbrightness_dark/MYSETTING.DAT");
    assert_eq!(data.jog_ring_brightness, JogRingBrightness::Dark);
}

#[test]
fn read_mysetting_jogringbrightness_off() {
    let data = read_mysetting!("../data/mysetting/jogringbrightness_off/MYSETTING.DAT");
    assert_eq!(data.jog_ring_brightness, JogRingBrightness::Off);
}

#[test]
fn read_mysetting_jogringindicator_off() {
    let data = read_mysetting!("../data/mysetting/jogringindicator_off/MYSETTING.DAT");
    assert_eq!(data.jog_ring_indicator, JogRingIndicator::Off);
}

#[test]
fn read_mysetting_language_french() {
    let data = read_mysetting!("../data/mysetting/language_french/MYSETTING.DAT");
    assert_eq!(data.language, Language::French);
}

#[test]
fn read_mysetting_language_greek() {
    let data = read_mysetting!("../data/mysetting/language_greek/MYSETTING.DAT");
    assert_eq!(data.language, Language::Greek);
}
#[test]
fn read_mysetting_language_turkish() {
    let data = read_mysetting!("../data/mysetting/language_turkish/MYSETTING.DAT");
    assert_eq!(data.language, Language::Turkish);
}

#[test]
fn read_mysetting_lcdbrightness_1() {
    let data = read_mysetting!("../data/mysetting/lcdbrightness_1/MYSETTING.DAT");
    assert_eq!(data.lcd_brightness, LCDBrightness::One);
}

#[test]
fn read_mysetting_lcdbrightness_5() {
    let data = read_mysetting!("../data/mysetting/lcdbrightness_5/MYSETTING.DAT");
    assert_eq!(data.lcd_brightness, LCDBrightness::Five);
}

#[test]
fn read_mysetting_mastertempo_on() {
    let data = read_mysetting!("../data/mysetting/mastertempo_on/MYSETTING.DAT");
    assert_eq!(data.master_tempo, MasterTempo::On);
}

#[test]
fn read_mysetting_onairdisplay_off() {
    let data = read_mysetting!("../data/mysetting/onairdisplay_off/MYSETTING.DAT");
    assert_eq!(data.on_air_display, OnAirDisplay::Off);
}

#[test]
fn read_mysetting_phasemeter_type2() {
    let data = read_mysetting!("../data/mysetting/phasemeter_type2/MYSETTING.DAT");
    assert_eq!(data.phase_meter, PhaseMeter::Type2);
}

#[test]
fn read_mysetting_quantize_off() {
    let data = read_mysetting!("../data/mysetting/quantize_off/MYSETTING.DAT");
    assert_eq!(data.quantize, Quantize::Off);
}

#[test]
fn read_mysetting_quantize_eighth() {
    let data = read_mysetting!("../data/mysetting/quantize_eighth/MYSETTING.DAT");
    assert_eq!(data.quantize_beat_value, QuantizeBeatValue::EighthBeat);
}

#[test]
fn read_mysetting_quantize_half() {
    let data = read_mysetting!("../data/mysetting/quantize_half/MYSETTING.DAT");
    assert_eq!(data.quantize_beat_value, QuantizeBeatValue::HalfBeat);
}

#[test]
fn read_mysetting_quantize_quarter() {
    let data = read_mysetting!("../data/mysetting/quantize_quarter/MYSETTING.DAT");
    assert_eq!(data.quantize_beat_value, QuantizeBeatValue::QuarterBeat);
}

#[test]
fn read_mysetting_slipflashing_off() {
    let data = read_mysetting!("../data/mysetting/slipflashing_off/MYSETTING.DAT");
    assert_eq!(data.slip_flashing, SlipFlashing::Off);
}

#[test]
fn read_mysetting_sync_on() {
    let data = read_mysetting!("../data/mysetting/sync_on/MYSETTING.DAT");
    assert_eq!(data.sync, Sync::On);
}

#[test]
fn read_mysetting_temporange_16() {
    let data = read_mysetting!("../data/mysetting/temporange_16/MYSETTING.DAT");
    assert_eq!(data.tempo_range, TempoRange::SixteenPercent);
}

#[test]
fn read_mysetting_temporange_6() {
    let data = read_mysetting!("../data/mysetting/temporange_6/MYSETTING.DAT");
    assert_eq!(data.tempo_range, TempoRange::SixPercent);
}

#[test]
fn read_mysetting_temporange_wide() {
    let data = read_mysetting!("../data/mysetting/temporange_wide/MYSETTING.DAT");
    assert_eq!(data.tempo_range, TempoRange::Wide);
}

#[test]
fn read_mysetting_timemode_elapsed() {
    let data = read_mysetting!("../data/mysetting/timemode_elapsed/MYSETTING.DAT");
    assert_eq!(data.time_mode, TimeMode::Elapsed);
}
