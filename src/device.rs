// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! High-level API for working with Rekordbox device exports.

use crate::{
    pdb::io::Database,
    pdb::{DatabaseType, PlaylistTreeNode, PlaylistTreeNodeId},
    setting,
    setting::{Setting, SettingType},
};
use binrw::BinRead;
use fallible_iterator::FallibleIterator;
use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

/// Represents a Rekordbox device export.
#[derive(Debug, PartialEq)]
pub struct DeviceExportLoader(PathBuf);

impl DeviceExportLoader {
    /// Load device export from the given path.
    ///
    /// The path should contain a `PIONEER` directory.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Get the device path.
    #[must_use]
    pub fn get_path(&self) -> &Path {
        &self.0
    }

    fn read_setting_file(path: &PathBuf, setting_type: SettingType) -> crate::Result<Setting> {
        let mut reader = std::fs::File::open(path)?;
        let setting = Setting::read_args(&mut reader, (setting_type,))?;
        Ok(setting)
    }

    /// Load setting files. If a file is missing or cannot be read,
    /// a warning will be printed.
    #[must_use]
    pub fn load_settings(&self) -> Settings {
        let path = self.0.join("PIONEER");

        let load_setting = |filename: &str, setting_type: SettingType| -> Option<Setting> {
            let file_path = path.join(filename);
            match Self::read_setting_file(&file_path, setting_type) {
                Ok(setting) => Some(setting),
                Err(e) => {
                    eprintln!("Warning: Could not load {}: {}", file_path.display(), e);
                    None
                }
            }
        };

        let mut settings = Settings::default();
        if let Some(devsetting) = load_setting("DEVSETTING.DAT", SettingType::DevSetting) {
            settings.set_devsetting(devsetting.data.as_dev_setting().unwrap());
        }
        if let Some(djmmysetting) = load_setting("DJMMYSETTING.DAT", SettingType::DJMMySetting) {
            settings.set_djmmysetting(djmmysetting.data.as_djm_my_setting().unwrap());
        }
        if let Some(mysetting) = load_setting("MYSETTING.DAT", SettingType::MySetting) {
            settings.set_mysetting(mysetting.data.as_my_setting().unwrap());
        }
        if let Some(mysetting2) = load_setting("MYSETTING2.DAT", SettingType::MySetting2) {
            settings.set_mysetting2(mysetting2.data.as_my_setting2().unwrap());
        }

        settings
    }

    /// Open a PDB database without persistence back to disk.
    /// Still allows modifying data in memory.
    pub fn open_pdb_non_persistent(&self) -> crate::Result<Database<std::fs::File>> {
        let path = self.0.join("PIONEER").join("rekordbox").join("export.pdb");
        Database::open_non_persistent(std::fs::File::open(path)?, DatabaseType::Plain)
    }
}

/// Settings object containing for all device settings.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Settings {
    // MYSETTING.DAT
    /// "ON AIR DISPLAY" setting.
    pub on_air_display: Option<setting::OnAirDisplay>,
    /// "LCD BRIGHTNESS" setting.
    pub lcd_brightness: Option<setting::LCDBrightness>,
    /// "QUANTIZE" setting.
    pub quantize: Option<setting::Quantize>,
    /// "AUTO CUE LEVEL" setting.
    pub auto_cue_level: Option<setting::AutoCueLevel>,
    /// "LANGUAGE" setting.
    pub language: Option<setting::Language>,
    /// "JOG RING BRIGHTNESS" setting.
    pub jog_ring_brightness: Option<setting::JogRingBrightness>,
    /// "JOG RING INDICATOR" setting.
    pub jog_ring_indicator: Option<setting::JogRingIndicator>,
    /// "SLIP FLASHING" setting.
    pub slip_flashing: Option<setting::SlipFlashing>,
    /// "DISC SLOT ILLUMINATION" setting.
    pub disc_slot_illumination: Option<setting::DiscSlotIllumination>,
    /// "EJECT/LOAD LOCK" setting.
    pub eject_lock: Option<setting::EjectLock>,
    /// "SYNC" setting.
    pub sync: Option<setting::Sync>,
    /// "PLAY MODE / AUTO PLAY MODE" setting.
    pub play_mode: Option<setting::PlayMode>,
    /// Quantize Beat Value setting.
    pub quantize_beat_value: Option<setting::QuantizeBeatValue>,
    /// "HOT CUE AUTO LOAD" setting.
    pub hotcue_autoload: Option<setting::HotCueAutoLoad>,
    /// "HOT CUE COLOR" setting.
    pub hotcue_color: Option<setting::HotCueColor>,
    /// "NEEDLE LOCK" setting.
    pub needle_lock: Option<setting::NeedleLock>,
    /// "TIME MODE" setting.
    pub time_mode: Option<setting::TimeMode>,
    /// "TIME MODE" setting.
    pub jog_mode: Option<setting::JogMode>,
    /// "AUTO CUE" setting.
    pub auto_cue: Option<setting::AutoCue>,
    /// "MASTER TEMPO" setting.
    pub master_tempo: Option<setting::MasterTempo>,
    /// "TEMPO RANGE" setting.
    pub tempo_range: Option<setting::TempoRange>,
    /// "PHASE METER" setting.
    pub phase_meter: Option<setting::PhaseMeter>,

    // MYSETTING2.DAT
    /// "VINYL SPEED ADJUST" setting.
    pub vinyl_speed_adjust: Option<setting::VinylSpeedAdjust>,
    /// "JOG DISPLAY MODE" setting.
    pub jog_display_mode: Option<setting::JogDisplayMode>,
    /// "PAD/BUTTON BRIGHTNESS" setting.
    pub pad_button_brightness: Option<setting::PadButtonBrightness>,
    /// "JOG LCD BRIGHTNESS" setting.
    pub jog_lcd_brightness: Option<setting::JogLCDBrightness>,
    /// "WAVEFORM DIVISIONS" setting.
    pub waveform_divisions: Option<setting::WaveformDivisions>,
    /// "WAVEFORM / PHASE METER" setting.
    pub waveform: Option<setting::Waveform>,
    /// "BEAT JUMP BEAT VALUE" setting.
    pub beat_jump_beat_value: Option<setting::BeatJumpBeatValue>,

    // DJMSETTING.DAT
    /// "CH FADER CURVE" setting.
    pub channel_fader_curve: Option<setting::ChannelFaderCurve>,
    /// "CROSSFADER CURVE" setting.
    pub crossfader_curve: Option<setting::CrossfaderCurve>,
    /// "HEADPHONES PRE EQ" setting.
    pub headphones_pre_eq: Option<setting::HeadphonesPreEQ>,
    /// "HEADPHONES MONO SPLIT" setting.
    pub headphones_mono_split: Option<setting::HeadphonesMonoSplit>,
    /// "BEAT FX QUANTIZE" setting.
    pub beat_fx_quantize: Option<setting::BeatFXQuantize>,
    /// "MIC LOW CUT" setting.
    pub mic_low_cut: Option<setting::MicLowCut>,
    /// "TALK OVER MODE" setting.
    pub talk_over_mode: Option<setting::TalkOverMode>,
    /// "TALK OVER LEVEL" setting.
    pub talk_over_level: Option<setting::TalkOverLevel>,
    /// "MIDI CH" setting.
    pub midi_channel: Option<setting::MidiChannel>,
    /// "MIDI BUTTON TYPE" setting.
    pub midi_button_type: Option<setting::MidiButtonType>,
    /// "BRIGHTNESS > DISPLAY" setting.
    pub display_brightness: Option<setting::MixerDisplayBrightness>,
    /// "BRIGHTNESS > INDICATOR" setting.
    pub indicator_brightness: Option<setting::MixerIndicatorBrightness>,
    /// "CH FADER CURVE (LONG FADER)" setting.
    pub channel_fader_curve_long_fader: Option<setting::ChannelFaderCurveLongFader>,

    // DEVSETTING.DAT
    /// "Type of the overview Waveform" setting.
    pub overview_waveform_type: Option<setting::OverviewWaveformType>,
    /// "Waveform color" setting.
    pub waveform_color: Option<setting::WaveformColor>,
    /// "Key display format" setting.
    pub key_display_format: Option<setting::KeyDisplayFormat>,
    /// "Waveform Current Position" setting.
    pub waveform_current_position: Option<setting::WaveformCurrentPosition>,
}

impl Settings {
    fn set_mysetting(&mut self, data: &setting::MySetting) {
        self.on_air_display = data.on_air_display.into();
        self.lcd_brightness = data.lcd_brightness.into();
        self.quantize = data.quantize.into();
        self.auto_cue_level = data.auto_cue_level.into();
        self.language = data.language.into();
        self.jog_ring_brightness = data.jog_ring_brightness.into();
        self.jog_ring_indicator = data.jog_ring_indicator.into();
        self.slip_flashing = data.slip_flashing.into();
        self.disc_slot_illumination = data.disc_slot_illumination.into();
        self.eject_lock = data.eject_lock.into();
        self.sync = data.sync.into();
        self.play_mode = data.play_mode.into();
        self.quantize_beat_value = data.quantize_beat_value.into();
        self.hotcue_autoload = data.hotcue_autoload.into();
        self.hotcue_color = data.hotcue_color.into();
        self.needle_lock = data.needle_lock.into();
        self.time_mode = data.time_mode.into();
        self.jog_mode = data.jog_mode.into();
        self.auto_cue = data.auto_cue.into();
        self.master_tempo = data.master_tempo.into();
        self.tempo_range = data.tempo_range.into();
        self.phase_meter = data.phase_meter.into();
    }

    fn set_mysetting2(&mut self, data: &setting::MySetting2) {
        self.vinyl_speed_adjust = data.vinyl_speed_adjust.into();
        self.jog_display_mode = data.jog_display_mode.into();
        self.pad_button_brightness = data.pad_button_brightness.into();
        self.jog_lcd_brightness = data.jog_lcd_brightness.into();
        self.waveform_divisions = data.waveform_divisions.into();
        self.waveform = data.waveform.into();
        self.beat_jump_beat_value = data.beat_jump_beat_value.into();
    }

    fn set_djmmysetting(&mut self, data: &setting::DJMMySetting) {
        self.channel_fader_curve = data.channel_fader_curve.into();
        self.crossfader_curve = data.crossfader_curve.into();
        self.headphones_pre_eq = data.headphones_pre_eq.into();
        self.headphones_mono_split = data.headphones_mono_split.into();
        self.beat_fx_quantize = data.beat_fx_quantize.into();
        self.mic_low_cut = data.mic_low_cut.into();
        self.talk_over_mode = data.talk_over_mode.into();
        self.talk_over_level = data.talk_over_level.into();
        self.midi_channel = data.midi_channel.into();
        self.midi_button_type = data.midi_button_type.into();
        self.display_brightness = data.display_brightness.into();
        self.indicator_brightness = data.indicator_brightness.into();
        self.channel_fader_curve_long_fader = data.channel_fader_curve_long_fader.into();
    }

    fn set_devsetting(&mut self, data: &setting::DevSetting) {
        self.overview_waveform_type = data.overview_waveform_type.into();
        self.waveform_color = data.waveform_color.into();
        self.key_display_format = data.key_display_format.into();
        self.waveform_current_position = data.waveform_current_position.into();
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_option<T: fmt::Display>(opt: Option<T>) -> String {
            opt.map(|v| v.to_string())
                .unwrap_or_else(|| "<missing>".to_string())
        }

        writeln!(
            f,
            "On Air Display:                 {}",
            format_option(self.on_air_display)
        )?;
        writeln!(
            f,
            "LCD Brightness:                 {}",
            format_option(self.lcd_brightness)
        )?;
        writeln!(
            f,
            "Quantize:                       {}",
            format_option(self.quantize)
        )?;
        writeln!(
            f,
            "Auto Cue Level:                 {}",
            format_option(self.auto_cue_level)
        )?;
        writeln!(
            f,
            "Language:                       {}",
            format_option(self.language)
        )?;
        writeln!(
            f,
            "Jog Ring Brightness:            {}",
            format_option(self.jog_ring_brightness)
        )?;
        writeln!(
            f,
            "Jog Ring Indicator:             {}",
            format_option(self.jog_ring_indicator)
        )?;
        writeln!(
            f,
            "Slip Flashing:                  {}",
            format_option(self.slip_flashing)
        )?;
        writeln!(
            f,
            "Disc Slot Illumination:         {}",
            format_option(self.disc_slot_illumination)
        )?;
        writeln!(
            f,
            "Eject Lock:                     {}",
            format_option(self.eject_lock)
        )?;
        writeln!(
            f,
            "Sync:                           {}",
            format_option(self.sync)
        )?;
        writeln!(
            f,
            "Play Mode:                      {}",
            format_option(self.play_mode)
        )?;
        writeln!(
            f,
            "Quantize Beat Value:            {}",
            format_option(self.quantize_beat_value)
        )?;
        writeln!(
            f,
            "Hotcue Autoload:                {}",
            format_option(self.hotcue_autoload)
        )?;
        writeln!(
            f,
            "Hotcue Color:                   {}",
            format_option(self.hotcue_color)
        )?;
        writeln!(
            f,
            "Needle Lock:                    {}",
            format_option(self.needle_lock)
        )?;
        writeln!(
            f,
            "Time Mode:                      {}",
            format_option(self.time_mode)
        )?;
        writeln!(
            f,
            "Jog Mode:                       {}",
            format_option(self.jog_mode)
        )?;
        writeln!(
            f,
            "Auto Cue:                       {}",
            format_option(self.auto_cue)
        )?;
        writeln!(
            f,
            "Master Tempo:                   {}",
            format_option(self.master_tempo)
        )?;
        writeln!(
            f,
            "Tempo Range:                    {}",
            format_option(self.tempo_range)
        )?;
        writeln!(
            f,
            "Phase Meter:                    {}",
            format_option(self.phase_meter)
        )?;
        writeln!(
            f,
            "Vinyl Speed Adjust:             {}",
            format_option(self.vinyl_speed_adjust)
        )?;
        writeln!(
            f,
            "Jog Display Mode:               {}",
            format_option(self.jog_display_mode)
        )?;
        writeln!(
            f,
            "Pad Button Brightness:          {}",
            format_option(self.pad_button_brightness)
        )?;
        writeln!(
            f,
            "Jog LCD Brightness:             {}",
            format_option(self.jog_lcd_brightness)
        )?;
        writeln!(
            f,
            "Waveform Divisions:             {}",
            format_option(self.waveform_divisions)
        )?;
        writeln!(
            f,
            "Waveform:                       {}",
            format_option(self.waveform)
        )?;
        writeln!(
            f,
            "Beat Jump Beat Value:           {}",
            format_option(self.beat_jump_beat_value)
        )?;
        writeln!(
            f,
            "Channel Fader Curve:            {}",
            format_option(self.channel_fader_curve)
        )?;
        writeln!(
            f,
            "Crossfader Curve:               {}",
            format_option(self.crossfader_curve)
        )?;
        writeln!(
            f,
            "Headphones Pre Eq:              {}",
            format_option(self.headphones_pre_eq)
        )?;
        writeln!(
            f,
            "Headphones Mono Split:          {}",
            format_option(self.headphones_mono_split)
        )?;
        writeln!(
            f,
            "Beat FX Quantize:               {}",
            format_option(self.beat_fx_quantize)
        )?;
        writeln!(
            f,
            "Mic Low Cut:                    {}",
            format_option(self.mic_low_cut)
        )?;
        writeln!(
            f,
            "Talk Over Mode:                 {}",
            format_option(self.talk_over_mode)
        )?;
        writeln!(
            f,
            "Talk Over Level:                {}",
            format_option(self.talk_over_level)
        )?;
        writeln!(
            f,
            "MIDI Channel:                   {}",
            format_option(self.midi_channel)
        )?;
        writeln!(
            f,
            "MIDI Button Type:               {}",
            format_option(self.midi_button_type)
        )?;
        writeln!(
            f,
            "Display Brightness:             {}",
            format_option(self.display_brightness)
        )?;
        writeln!(
            f,
            "Indicator Brightness:           {}",
            format_option(self.indicator_brightness)
        )?;
        writeln!(
            f,
            "Channel Fader Curve Long Fader: {}",
            format_option(self.channel_fader_curve_long_fader)
        )?;
        writeln!(
            f,
            "Overview Waveform Type:         {}",
            format_option(self.overview_waveform_type)
        )?;
        writeln!(
            f,
            "Waveform Color:                 {}",
            format_option(self.waveform_color)
        )?;
        writeln!(
            f,
            "Key Display Format:             {}",
            format_option(self.key_display_format)
        )?;
        write!(
            f,
            "Waveform Current Position:      {}",
            format_option(self.waveform_current_position)
        )
    }
}

/// Represents either a playlist folder or a playlist.
#[derive(Debug, PartialEq)]
pub enum PlaylistNode {
    /// Represents a playlist folder that contains `PlaylistNode`s.
    Folder(PlaylistFolder),
    /// Represents a playlist.
    Playlist(Playlist),
}

/// Represents a playlist folder that contains `PlaylistNode`s.
#[derive(Debug, PartialEq)]
pub struct PlaylistFolder {
    /// ID of this node in the playlist tree.
    pub id: PlaylistTreeNodeId,
    /// Name of the playlist folder.
    pub name: String,
    /// Child nodes of the playlist folder.
    pub children: Vec<PlaylistNode>,
}

/// Represents a playlist.
#[derive(Debug, PartialEq, Eq)]
pub struct Playlist {
    /// ID of this node in the playlist tree.
    pub id: PlaylistTreeNodeId,
    /// Name of the playlist.
    pub name: String,
}

/// Get playlist tree.
pub fn get_playlists<R: Read + Seek>(db: &mut Database<R>) -> crate::Result<Vec<PlaylistNode>> {
    let mut playlists: HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>> = HashMap::new();

    db.iter_rows::<PlaylistTreeNode>()?.for_each(|node| {
        playlists
            .entry(node.parent_id)
            .or_default()
            .push(node.clone());
        Ok(())
    })?;

    fn get_child_nodes(
        playlists: &HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>>,
        id: PlaylistTreeNodeId,
    ) -> impl Iterator<Item = crate::Result<PlaylistNode>> + '_ {
        playlists
            .get(&id)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
            .map(|node| -> crate::Result<PlaylistNode> {
                let child_node = if node.is_folder() {
                    let folder = PlaylistFolder {
                        id: node.id,
                        name: node.name.clone().into_string()?,
                        children: get_child_nodes(playlists, node.id)
                            .collect::<crate::Result<Vec<PlaylistNode>>>()?,
                    };
                    PlaylistNode::Folder(folder)
                } else {
                    let playlist = Playlist {
                        id: node.id,
                        name: node.name.clone().into_string()?,
                    };
                    PlaylistNode::Playlist(playlist)
                };
                Ok(child_node)
            })
    }

    get_child_nodes(&playlists, PlaylistTreeNodeId(0)).collect::<crate::Result<Vec<PlaylistNode>>>()
}
