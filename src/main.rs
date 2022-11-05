// Copyright (c) 2022 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{BinRead, ReadOptions};
use clap::{Parser, Subcommand};
use rekordcrate::anlz::ANLZ;
use rekordcrate::pdb::Header;
use rekordcrate::setting::Setting;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List the playlist tree from a Pioneer Database (`.PDB`) file.
    ListPlaylists {
        /// File to parse.
        #[arg(value_name = "PDB_FILE")]
        path: PathBuf,
    },
    /// Display settings from a Rekordbox device export.
    ListSettings {
        /// Path to parse.
        #[arg(value_name = "EXPORT_PATH")]
        path: PathBuf,
    },
    /// Export the playlists from a Pioneer Database (`.PDB`) file to M3U files.
    ExportPlaylists {
        /// File to parse.
        #[arg(value_name = "EXPORT_PATH")]
        path: PathBuf,
        /// Output directory to write M3U files to.
        #[arg(value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,
    },
    /// Parse and dump a Rekordbox Analysis (`ANLZXXXX.DAT`) file.
    DumpANLZ {
        /// File to parse.
        #[arg(value_name = "ANLZ_FILE")]
        path: PathBuf,
    },
    /// Parse and dump a Pioneer Database (`.PDB`) file.
    DumpPDB {
        /// File to parse.
        #[arg(value_name = "PDB_FILE")]
        path: PathBuf,
    },
    /// Parse and dump a Pioneer Settings (`*SETTING.DAT`) file.
    DumpSetting {
        /// File to parse.
        #[arg(value_name = "SETTING_FILE")]
        path: PathBuf,
    },
}

fn list_playlists(path: &Path) -> rekordcrate::Result<()> {
    use rekordcrate::device::PlaylistNode;
    use rekordcrate::DeviceExport;

    let mut export = DeviceExport::new(path.into());
    export.load_pdb()?;
    let playlists = export.get_playlists()?;

    fn walk_tree(export: &DeviceExport, node: PlaylistNode, level: usize) {
        let indent = "    ".repeat(level);
        match node {
            PlaylistNode::Folder(folder) => {
                println!("{}🗀 {}", indent, folder.name);
                folder
                    .children
                    .into_iter()
                    .for_each(|child| walk_tree(export, child, level + 1));
            }
            PlaylistNode::Playlist(playlist) => {
                let num_tracks = export
                    .get_playlist_entries(playlist.id)
                    .expect("failed to get playlist entries")
                    .count();
                println!("{}🗎 {} ({} tracks)", indent, playlist.name, num_tracks)
            }
        };
    }
    playlists
        .into_iter()
        .for_each(|node| walk_tree(&export, node, 0));

    Ok(())
}

fn export_playlists(path: &Path, output_dir: &PathBuf) -> rekordcrate::Result<()> {
    use rekordcrate::device::PlaylistNode;
    use rekordcrate::pdb::{Track, TrackId};
    use rekordcrate::DeviceExport;
    use std::collections::HashMap;
    use std::io::Write;

    let mut export = DeviceExport::new(path.into());
    export.load_pdb()?;
    let playlists = export.get_playlists()?;
    let mut tracks: HashMap<TrackId, Track> = HashMap::new();
    export.get_tracks()?.try_for_each(|result| {
        if let Ok(track) = result {
            tracks.insert(track.id, track);
            Ok(())
        } else {
            result.map(|_| ())
        }
    })?;

    fn walk_tree(
        export: &DeviceExport,
        tracks: &HashMap<TrackId, Track>,
        node: PlaylistNode,
        path: &PathBuf,
    ) -> rekordcrate::Result<()> {
        match node {
            PlaylistNode::Folder(folder) => {
                folder.children.into_iter().try_for_each(|child| {
                    walk_tree(export, tracks, child, &path.join(&folder.name))
                })?;
            }
            PlaylistNode::Playlist(playlist) => {
                let mut playlist_entries = export
                    .get_playlist_entries(playlist.id)?
                    .collect::<rekordcrate::Result<Vec<(u32, TrackId)>>>()?;
                playlist_entries.sort_by_key(|entry| entry.0);

                std::fs::create_dir_all(path)?;
                let playlist_path = path.join(format!("{}.m3u", playlist.name));

                println!("{}", playlist_path.display());
                let mut file = std::fs::File::create(playlist_path)?;
                playlist_entries
                    .into_iter()
                    .filter_map(|(_index, id)| tracks.get(&id))
                    .try_for_each(|track| -> rekordcrate::Result<()> {
                        let track_path = track.file_path.clone().into_string()?;
                        Ok(writeln!(
                            &mut file,
                            "{}",
                            export
                                .get_path()
                                .canonicalize()?
                                .join(track_path.strip_prefix('/').unwrap_or(&track_path))
                                .display(),
                        )?)
                    })?;
            }
        };

        Ok(())
    }

    playlists
        .into_iter()
        .try_for_each(|node| walk_tree(&export, &tracks, node, output_dir))?;

    Ok(())
}

fn list_settings(path: &Path) -> rekordcrate::Result<()> {
    use rekordcrate::DeviceExport;

    let mut export = DeviceExport::new(path.into());
    export.load_settings()?;
    let settings = export.get_settings();

    println!(
        "On Air Display:                 {}",
        settings
            .on_air_display
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "LCD Brightness:                 {}",
        settings
            .lcd_brightness
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Quantize:                       {}",
        settings
            .quantize
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Auto Cue Level:                 {}",
        settings
            .auto_cue_level
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Language:                       {}",
        settings
            .language
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Jog Ring Brightness:            {}",
        settings
            .jog_ring_brightness
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Jog Ring Indicator:             {}",
        settings
            .jog_ring_indicator
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Slip Flashing:                  {}",
        settings
            .slip_flashing
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Disc Slot Illumination:         {}",
        settings
            .disc_slot_illumination
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Eject Lock:                     {}",
        settings
            .eject_lock
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Sync:                           {}",
        settings
            .sync
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Play Mode:                      {}",
        settings
            .play_mode
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Quantize Beat Value:            {}",
        settings
            .quantize_beat_value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Hotcue Autoload:                {}",
        settings
            .hotcue_autoload
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Hotcue Color:                   {}",
        settings
            .hotcue_color
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Needle Lock:                    {}",
        settings
            .needle_lock
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Time Mode:                      {}",
        settings
            .time_mode
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Jog Mode:                       {}",
        settings
            .jog_mode
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Auto Cue:                       {}",
        settings
            .auto_cue
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Master Tempo:                   {}",
        settings
            .master_tempo
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Tempo Range:                    {}",
        settings
            .tempo_range
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Phase Meter:                    {}",
        settings
            .phase_meter
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Vinyl Speed Adjust:             {}",
        settings
            .vinyl_speed_adjust
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Jog Display Mode:               {}",
        settings
            .jog_display_mode
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Pad Button Brightness:          {}",
        settings
            .pad_button_brightness
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Jog LCD Brightness:             {}",
        settings
            .jog_lcd_brightness
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Waveform Divisions:             {}",
        settings
            .waveform_divisions
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Waveform:                       {}",
        settings
            .waveform
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Beat Jump Beat Value:           {}",
        settings
            .beat_jump_beat_value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Channel Fader Curve:            {}",
        settings
            .channel_fader_curve
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Crossfader Curve:               {}",
        settings
            .crossfader_curve
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Headphones Pre Eq:              {}",
        settings
            .headphones_pre_eq
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Headphones Mono Split:          {}",
        settings
            .headphones_mono_split
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Beat FX Quantize:               {}",
        settings
            .beat_fx_quantize
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Mic Low Cut:                    {}",
        settings
            .mic_low_cut
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Talk Over Mode:                 {}",
        settings
            .talk_over_mode
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Talk Over Level:                {}",
        settings
            .talk_over_level
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "MIDI Channel:                   {}",
        settings
            .midi_channel
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "MIDI Button Type:               {}",
        settings
            .midi_button_type
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Display Brightness:             {}",
        settings
            .display_brightness
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Indicator Brightness:           {}",
        settings
            .indicator_brightness
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Channel Fader Curve Long Fader: {}",
        settings
            .channel_fader_curve_long_fader
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Overview Waveform Type:         {}",
        settings
            .overview_waveform_type
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Waveform Color:                 {}",
        settings
            .waveform_color
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Key Display Format:             {}",
        settings
            .key_display_format
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Waveform Current Position:      {}",
        settings
            .waveform_current_position
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    );

    Ok(())
}

fn dump_anlz(path: &PathBuf) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(&path)?;
    let anlz = ANLZ::read(&mut reader)?;
    println!("{:#?}", anlz);

    Ok(())
}

fn dump_pdb(path: &PathBuf) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(&path)?;
    let header = Header::read(&mut reader)?;

    println!("{:#?}", header);

    for (i, table) in header.tables.iter().enumerate() {
        println!("Table {}: {:?}", i, table.page_type);
        for page in header
            .read_pages(
                &mut reader,
                &ReadOptions::new(binrw::Endian::NATIVE),
                (&table.first_page, &table.last_page),
            )
            .unwrap()
            .into_iter()
        {
            println!("  {:?}", page);
            page.row_groups.iter().for_each(|row_group| {
                println!("    {:?}", row_group);
                for row in row_group.present_rows() {
                    println!("      {:?}", row);
                }
            })
        }
    }

    Ok(())
}

fn dump_setting(path: &PathBuf) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(&path)?;
    let setting = Setting::read(&mut reader)?;

    println!("{:#04x?}", setting);

    Ok(())
}

fn main() -> rekordcrate::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ListPlaylists { path } => list_playlists(path),
        Commands::ListSettings { path } => list_settings(path),
        Commands::ExportPlaylists { path, output_dir } => export_playlists(path, output_dir),
        Commands::DumpPDB { path } => dump_pdb(path),
        Commands::DumpANLZ { path } => dump_anlz(path),
        Commands::DumpSetting { path } => dump_setting(path),
    }
}
