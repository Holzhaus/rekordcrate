// Copyright (c) 2023 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! High-level API for working with Rekordbox device exports.

use crate::{
    pdb::{Header, Page, PageType, PlaylistTreeNode, PlaylistTreeNodeId, Row, Track, TrackId},
    setting,
    setting::Setting,
};
use binrw::{BinRead, ReadOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a Rekordbox device export.
#[derive(Debug, PartialEq)]
pub struct DeviceExport {
    path: PathBuf,
    pdb: Option<Pdb>,
    devsetting: Option<Setting>,
    djmmysetting: Option<Setting>,
    mysetting: Option<Setting>,
    mysetting2: Option<Setting>,
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

    /// Load setting files.
    pub fn load_settings(&mut self) -> crate::Result<()> {
        let path = self.path.join("PIONEER");
        self.devsetting = Some(Self::read_setting_file(&path.join("DEVSETTING.DAT"))?);
        self.djmmysetting = Some(Self::read_setting_file(&path.join("DJMMYSETTING.DAT"))?);
        self.mysetting = Some(Self::read_setting_file(&path.join("MYSETTING.DAT"))?);
        self.mysetting2 = Some(Self::read_setting_file(&path.join("MYSETTING2.DAT"))?);

        Ok(())
    }

    fn read_pdb_file(path: &PathBuf) -> crate::Result<Pdb> {
        let mut reader = std::fs::File::open(path)?;
        let header = Header::read(&mut reader)?;
        let pages = header
            .tables
            .iter()
            .flat_map(|table| {
                header
                    .read_pages(
                        &mut reader,
                        &ReadOptions::new(binrw::Endian::NATIVE),
                        (&table.first_page, &table.last_page),
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

    /// Get the playlists tree.
    pub fn get_playlists(&self) -> crate::Result<Vec<PlaylistNode>> {
        match &self.pdb {
            Some(pdb) => pdb.get_playlists(),
            None => Err(crate::Error::NotLoadedError),
        }
    }

    /// Get the entries for a single playlist.
    pub fn get_playlist_entries(
        &self,
        id: PlaylistTreeNodeId,
    ) -> crate::Result<impl Iterator<Item = crate::Result<(u32, TrackId)>> + '_> {
        match &self.pdb {
            Some(pdb) => Ok(pdb.get_playlist_entries(id)),
            None => Err(crate::Error::NotLoadedError),
        }
    }

    /// Get the tracks.
    pub fn get_tracks(&self) -> crate::Result<impl Iterator<Item = crate::Result<Track>> + '_> {
        match &self.pdb {
            Some(pdb) => Ok(pdb.get_tracks()),
            None => Err(crate::Error::NotLoadedError),
        }
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
    pub fn new_from_path(path: &PathBuf) -> crate::Result<Self> {
        let mut reader = std::fs::File::open(path)?;
        let header = Header::read(&mut reader)?;
        let pages = header
            .tables
            .iter()
            .flat_map(|table| {
                header
                    .read_pages(
                        &mut reader,
                        &ReadOptions::new(binrw::Endian::NATIVE),
                        (&table.first_page, &table.last_page),
                    )
                    .into_iter()
            })
            .flatten()
            .collect::<Vec<Page>>();

        let pdb = Pdb { header, pages };

        Ok(pdb)
    }

    fn get_rows_by_page_type(&self, page_type: PageType) -> impl Iterator<Item = &Row> + '_ {
        self.pages
            .iter()
            .filter(move |page| page.page_type == page_type)
            .flat_map(|page| page.row_groups.iter())
            .flat_map(|row_group| row_group.present_rows())
    }

    /// Get playlist tree.
    pub fn get_playlists(&self) -> crate::Result<Vec<PlaylistNode>> {
        let mut playlists: HashMap<PlaylistTreeNodeId, Vec<&PlaylistTreeNode>> = HashMap::new();
        self.get_rows_by_page_type(PageType::PlaylistTree)
            .map(|row| {
                if let Row::PlaylistTreeNode(playlist_tree) = row {
                    Ok(playlist_tree)
                } else {
                    Err(crate::Error::IntegrityError(
                        "encountered non-playlist tree row in playlist table",
                    ))
                }
            })
            .try_for_each(|row| {
                row.map(|node| playlists.entry(node.parent_id).or_default().push(node))
            })?;

        fn get_child_nodes<'a>(
            playlists: &'a HashMap<PlaylistTreeNodeId, Vec<&PlaylistTreeNode>>,
            id: PlaylistTreeNodeId,
        ) -> impl Iterator<Item = crate::Result<PlaylistNode>> + 'a {
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
    ) -> impl Iterator<Item = crate::Result<(u32, TrackId)>> + '_ {
        self.get_rows_by_page_type(PageType::PlaylistEntries)
            .filter_map(move |row| {
                if let Row::PlaylistEntry(entry) = row {
                    if entry.playlist_id == playlist_id {
                        Some(Ok((entry.entry_index, entry.track_id)))
                    } else {
                        None
                    }
                } else {
                    Some(Err(crate::Error::IntegrityError(
                        "encountered non-playlist tree row in playlist table",
                    )))
                }
            })
    }

    /// Get tracks.
    pub fn get_tracks(&self) -> impl Iterator<Item = crate::Result<Track>> + '_ {
        self.get_rows_by_page_type(PageType::Tracks).map(|row| {
            if let Row::Track(track) = row {
                Ok(track.clone())
            } else {
                Err(crate::Error::IntegrityError(
                    "encountered non-track row in track table",
                ))
            }
        })
    }
}
