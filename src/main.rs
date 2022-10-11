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
use rekordcrate::pdb::{Header, PageType, Row};
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

fn list_playlists(path: &PathBuf) -> rekordcrate::Result<()> {
    use rekordcrate::pdb::{PlaylistTreeNode, PlaylistTreeNodeId};
    use std::collections::HashMap;

    fn print_children_of(
        tree: &HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>>,
        id: PlaylistTreeNodeId,
        level: usize,
    ) {
        tree.get(&id)
            .iter()
            .flat_map(|nodes| nodes.iter())
            .for_each(|node| {
                println!(
                    "{}{} {}",
                    "    ".repeat(level),
                    if node.is_folder() { "🗀" } else { "🗎" },
                    node.name.clone().into_string().unwrap(),
                );
                print_children_of(tree, node.id, level + 1);
            });
    }

    let mut reader = std::fs::File::open(&path)?;
    let header = Header::read(&mut reader)?;

    let mut tree: HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>> = HashMap::new();

    header
        .tables
        .iter()
        .filter(|table| table.page_type == PageType::PlaylistTree)
        .flat_map(|table| {
            header
                .read_pages(
                    &mut reader,
                    &ReadOptions::new(binrw::Endian::NATIVE),
                    (&table.first_page, &table.last_page),
                )
                .unwrap()
                .into_iter()
                .flat_map(|page| page.row_groups.into_iter())
                .flat_map(|row_group| {
                    row_group
                        .present_rows()
                        .map(|row| {
                            if let Row::PlaylistTreeNode(playlist_tree) = row {
                                playlist_tree
                            } else {
                                unreachable!("encountered non-playlist tree row in playlist table");
                            }
                        })
                        .cloned()
                        .collect::<Vec<PlaylistTreeNode>>()
                        .into_iter()
                })
        })
        .for_each(|row| tree.entry(row.parent_id).or_default().push(row));

    print_children_of(&tree, PlaylistTreeNodeId(0), 0);

    Ok(())
}

fn list_settings(path: &Path) -> rekordcrate::Result<()> {
    use rekordcrate::DeviceExport;

    let mut export = DeviceExport::default();
    export.load(path)?;
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
        Commands::DumpPDB { path } => dump_pdb(path),
        Commands::DumpANLZ { path } => dump_anlz(path),
        Commands::DumpSetting { path } => dump_setting(path),
    }
}
