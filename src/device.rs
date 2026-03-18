// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! High-level API for working with Rekordbox device exports.

use crate::{
    anlz::{ANLZ, Content, CueListType},
    pdb::{
        DatabaseType, Header, Page, PageContent, PageType, PlainPageType, PlainRow,
        PlaylistTreeNode, PlaylistTreeNodeId, Row, Track, TrackId,
    },
    setting,
    setting::Setting,
};
use binrw::BinRead;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Represents analysis data for a track (hot cues, BPM, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct TrackAnalysis {
    /// Hot cues (if any)
    pub hot_cues: Vec<HotCue>,
    /// Beat grid entries (BPM changes)
    pub tempos: Vec<Tempo>,
}

/// Represents a hot cue
#[derive(Debug, Clone, PartialEq)]
pub struct HotCue {
    /// Hot cue number (0-9)
    pub number: u32,
    /// Name of the hot cue
    pub name: String,
    /// Start position in seconds
    pub start: f64,
    /// End position (for loops) in seconds
    pub end: Option<f64>,
    /// Color index
    pub color: Option<String>,
    /// Is this a loop?
    pub is_loop: bool,
}

/// Represents a tempo/BPM entry
#[derive(Debug, Clone, PartialEq)]
pub struct Tempo {
    /// Start position in seconds
    pub start: f64,
    /// BPM value
    pub bpm: f64,
}

/// Represents a Rekordbox device export.
#[derive(Debug, PartialEq)]
pub struct DeviceExport {
    path: PathBuf,
    pdb: Option<Pdb>,
    devsetting: Option<Setting>,
    djmmysetting: Option<Setting>,
    mysetting: Option<Setting>,
    mysetting2: Option<Setting>,
    /// ANLZ analysis files, keyed by the analyze path
    anlz_files: HashMap<String, ANLZ>,
}

impl DeviceExport {
    /// Load device export from the given path.
    ///
    /// The path should contain a `PIONEER` directory.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            pdb: None,
            devsetting: None,
            djmmysetting: None,
            mysetting: None,
            mysetting2: None,
            anlz_files: HashMap::new(),
        }
    }

    /// Get the device path.
    #[must_use]
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    fn read_setting_file(path: &PathBuf) -> crate::Result<Setting> {
        let mut reader = std::fs::File::open(path)?;
        let setting = Setting::read(&mut reader)?;
        Ok(setting)
    }

    /// Load setting files. If a file is missing or cannot be read, the
    /// corresponding setting will be `None` and a warning will be printed.
    pub fn load_settings(&mut self) {
        let path = self.path.join("PIONEER");

        let load_setting = |filename: &str| -> Option<Setting> {
            let file_path = path.join(filename);
            match Self::read_setting_file(&file_path) {
                Ok(setting) => Some(setting),
                Err(e) => {
                    eprintln!("Warning: Could not load {}: {}", file_path.display(), e);
                    None
                }
            }
        };

        self.devsetting = load_setting("DEVSETTING.DAT");
        self.djmmysetting = load_setting("DJMMYSETTING.DAT");
        self.mysetting = load_setting("MYSETTING.DAT");
        self.mysetting2 = load_setting("MYSETTING2.DAT");
    }

    fn read_pdb_file(path: &PathBuf) -> crate::Result<Pdb> {
        let mut reader = std::fs::File::open(path)?;
        let header = Header::read_args(&mut reader, (DatabaseType::Plain,))?;
        let pages = header
            .tables
            .iter()
            .flat_map(|table| {
                header
                    .read_pages(
                        &mut reader,
                        binrw::Endian::NATIVE,
                        (&table.first_page, &table.last_page, DatabaseType::Plain),
                    )
                    .into_iter()
            })
            .flatten()
            .collect::<Vec<Page>>();

        let pdb = Pdb { header, pages };
        Ok(pdb)
    }

    /// Load PDB file.
    pub fn load_pdb(&mut self) -> crate::Result<()> {
        let path = self
            .path
            .join("PIONEER")
            .join("rekordbox")
            .join("export.pdb");
        self.pdb = Some(Self::read_pdb_file(&path)?);
        Ok(())
    }

    /// Load ANLZ analysis files from the USBANLZ directory.
    /// This loads all analysis files found in the export.
    pub fn load_anlz(&mut self) -> crate::Result<()> {
        let usbanlz_path = self.path.join("PIONEER").join("USBANLZ");
        if !usbanlz_path.exists() {
            return Ok(()); // No analysis files, that's fine
        }

        // Walk through all .DAT, .EXT, and .2EX files
        for entry in walkdir::WalkDir::new(&usbanlz_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if ext_lower == "dat" || ext_lower == "ext" || ext_lower == "2ex" {
                    if let Ok(anlz) = Self::read_anlz_file(&path.into()) {
                        // Store with a simple key based on the file path
                        let key = path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        self.anlz_files.insert(key, anlz);
                    }
                }
            }
        }
        Ok(())
    }

    fn read_anlz_file(path: &PathBuf) -> crate::Result<ANLZ> {
        let mut reader = std::fs::File::open(path)?;
        let anlz = ANLZ::read(&mut reader)?;
        Ok(anlz)
    }

    /// Get analysis data for a specific track by its analyze path.
    pub fn get_track_analysis(&self, analyze_path: &str) -> Option<TrackAnalysis> {
        // Try to find the ANLZ file based on the analyze path
        // The analyze_path typically looks like /PIONEER/USBANLZ/P016/0000875E/ANLZ0000
        // We need to extract the relevant part to find the file

        // Try to find a matching ANLZ file
        for (key, anlz) in &self.anlz_files {
            if analyze_path.contains(key) || key.contains("ANLZ") {
                return Some(self.parse_anlz(anlz));
            }
        }

        // If no match found, try to get the first ANLZ file as a fallback
        if let Some(anlz) = self.anlz_files.values().next() {
            return Some(self.parse_anlz(anlz));
        }

        None
    }

    /// Parse ANLZ data into TrackAnalysis
    fn parse_anlz(&self, anlz: &ANLZ) -> TrackAnalysis {
        let mut hot_cues = Vec::new();
        let mut tempos = Vec::new();

        for section in &anlz.sections {
            match &section.content {
                Content::CueList(cue_list) => {
                    if cue_list.list_type == CueListType::HotCues {
                        for cue in &cue_list.cues {
                            if cue.hot_cue > 0 {
                                hot_cues.push(HotCue {
                                    number: cue.hot_cue,
                                    name: format!("Hot Cue {}", char::from(b'A' + (cue.hot_cue - 1) as u8)),
                                    start: cue.time as f64 / 1000.0,
                                    end: if cue.cue_type == crate::anlz::CueType::Loop {
                                        Some(cue.loop_time as f64 / 1000.0)
                                    } else {
                                        None
                                    },
                                    color: None,
                                    is_loop: cue.cue_type == crate::anlz::CueType::Loop,
                                });
                            }
                        }
                    }
                }
                Content::ExtendedCueList(ext_cue_list) => {
                    if ext_cue_list.list_type == CueListType::HotCues {
                        for cue in &ext_cue_list.cues {
                            if cue.hot_cue > 0 {
                                hot_cues.push(HotCue {
                                    number: cue.hot_cue,
                                    name: cue.comment.to_string(),
                                    start: cue.time as f64 / 1000.0,
                                    end: if cue.cue_type == crate::anlz::CueType::Loop {
                                        Some(cue.loop_time as f64 / 1000.0)
                                    } else {
                                        None
                                    },
                                    color: Some(format!("{:?}", cue.hot_cue_color_index)),
                                    is_loop: cue.cue_type == crate::anlz::CueType::Loop,
                                });
                            }
                        }
                    }
                }
                Content::BeatGrid(beatgrid) => {
                    for beat in &beatgrid.beats {
                        // Only add tempo entries at beat 1 of each bar (where beat_number == 1)
                        if beat.beat_number == 1 {
                            tempos.push(Tempo {
                                start: beat.time as f64 / 1000.0,
                                bpm: beat.tempo as f64 / 100.0,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        TrackAnalysis { hot_cues, tempos }
    }

    /// Get the settings from this export.
    #[must_use]
    pub fn get_settings(&self) -> Settings {
        let mut settings = Settings::default();
        [
            &self.mysetting,
            &self.mysetting2,
            &self.djmmysetting,
            &self.devsetting,
        ]
        .into_iter()
        .flatten()
        .for_each(|setting| match &setting.data {
            setting::SettingData::MySetting(data) => {
                settings.set_mysetting(data);
            }
            setting::SettingData::MySetting2(data) => {
                settings.set_mysetting2(data);
            }
            setting::SettingData::DJMMySetting(data) => {
                settings.set_djmmysetting(data);
            }
            setting::SettingData::DevSetting(data) => {
                settings.set_devsetting(data);
            }
        });

        settings
    }

    /// Get a reference to the PDB if loaded.
    #[must_use]
    pub fn pdb(&self) -> Option<&Pdb> {
        self.pdb.as_ref()
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

/// Represent a PDB file.
#[derive(Debug, PartialEq)]
pub struct Pdb {
    header: Header,
    pages: Vec<Page>,
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

impl Pdb {
    /// Create a new `Pdb` object by reading the PDB file at the given path.
    pub fn open_from_path(path: &PathBuf) -> crate::Result<Self> {
        let mut reader = std::fs::File::open(path)?;
        let header = Header::read_args(&mut reader, (DatabaseType::Plain,))?;
        let pages = header
            .tables
            .iter()
            .flat_map(|table| {
                header
                    .read_pages(
                        &mut reader,
                        binrw::Endian::NATIVE,
                        (&table.first_page, &table.last_page, DatabaseType::Plain),
                    )
                    .into_iter()
            })
            .flatten()
            .collect::<Vec<Page>>();

        let pdb = Pdb { header, pages };

        Ok(pdb)
    }

    pub fn get_rows_by_page_type(&self, page_type: PlainPageType) -> impl Iterator<Item = &Row> + '_ {
        self.pages
            .iter()
            .filter(move |page| page.header.page_type == PageType::Plain(page_type))
            .filter_map(|page| match &page.content {
                PageContent::Data(data) => Some(data),
                _ => None,
            })
            .flat_map(|data| data.rows.values())
    }

    /// Get playlist tree.
    pub fn get_playlists(&self) -> crate::Result<Vec<PlaylistNode>> {
        let mut playlists: HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>> = HashMap::new();
        self.get_rows_by_page_type(PlainPageType::PlaylistTree)
            .filter_map(|row| {
                if let Row::Plain(PlainRow::PlaylistTreeNode(playlist_tree)) = row {
                    Some(playlist_tree.clone())
                } else {
                    None
                }
            })
            .for_each(|node| {
                playlists.entry(node.parent_id).or_default().push(node);
            });

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

        get_child_nodes(&playlists, PlaylistTreeNodeId(0))
            .collect::<crate::Result<Vec<PlaylistNode>>>()
    }

    /// Get playlist entries.
    pub fn get_playlist_entries(
        &self,
        playlist_id: PlaylistTreeNodeId,
    ) -> impl Iterator<Item = (u32, TrackId)> + '_ {
        self.get_rows_by_page_type(PlainPageType::PlaylistEntries)
            .filter_map(move |row| {
                if let Row::Plain(PlainRow::PlaylistEntry(entry)) = row {
                    if entry.playlist_id == playlist_id {
                        Some((entry.entry_index, entry.track_id))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }

    /// Get tracks.
    pub fn get_tracks(&self) -> impl Iterator<Item = &Track> + '_ {
        self.get_rows_by_page_type(PlainPageType::Tracks)
            .filter_map(|row| {
                if let Row::Plain(PlainRow::Track(track)) = row {
                    Some(track)
                } else {
                    None
                }
            })
    }
}
