// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::BinRead;
use clap::{Parser, Subcommand};
use rekordcrate::anlz::ANLZ;
use rekordcrate::pdb::{
    Artist, ArtistId, DatabaseType, Header, PageContent, PageType, PlainPageType, PlainRow, Row,
    Track, TrackId,
};
use rekordcrate::setting::Setting;
use rekordcrate::xml::Document;
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
        /// Database type: "plain" (export.pdb) or "ext" (exportExt.pdb). Tries to guess based on file name of not specified.
        #[arg(long, value_name = "DB_TYPE", value_parser = ["plain", "ext"])]
        db_type: Option<String>,
    },
    /// Parse and dump a Pioneer Settings (`*SETTING.DAT`) file.
    DumpSetting {
        /// File to parse.
        #[arg(value_name = "SETTING_FILE")]
        path: PathBuf,
    },
    /// Parse and dump a Pioneer XML (`*.xml`) file.
    DumpXML {
        /// File to parse.
        #[arg(value_name = "XML_FILE")]
        path: PathBuf,
    },
}

fn list_playlists(path: &PathBuf) -> rekordcrate::Result<()> {
    use rekordcrate::pdb::{PlaylistTreeNode, PlaylistTreeNodeId};
    use std::collections::{BTreeMap, HashMap};

    let mut reader = std::fs::File::open(path)?;
    let header = Header::read_args(&mut reader, (DatabaseType::Plain,))?;

    let mut playlist_tree: HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>> = HashMap::new();
    let mut playlist_entries: HashMap<PlaylistTreeNodeId, BTreeMap<u32, TrackId>> = HashMap::new();
    let mut artists: HashMap<ArtistId, Artist> = HashMap::new();
    let mut tracks: HashMap<TrackId, Track> = HashMap::new();

    header
        .tables
        .iter()
        .filter(|table| {
            matches!(
                table.page_type,
                PageType::Plain(
                    PlainPageType::PlaylistTree
                        | PlainPageType::Artists
                        | PlainPageType::Tracks
                        | PlainPageType::PlaylistEntries
                )
            )
        })
        .for_each(|table| {
            header
                .read_pages(
                    &mut reader,
                    binrw::Endian::NATIVE,
                    (&table.first_page, &table.last_page, DatabaseType::Plain),
                )
                .unwrap()
                .into_iter()
                .filter_map(|page| page.content.into_data())
                .flat_map(|data_content| data_content.rows.into_values())
                .for_each(|row| match row {
                    Row::Plain(PlainRow::PlaylistTreeNode(tree_node)) => {
                        playlist_tree
                            .entry(tree_node.parent_id)
                            .or_default()
                            .push(tree_node.clone());
                    }
                    Row::Plain(PlainRow::Artist(artist)) => {
                        artists.insert(artist.id, artist.clone());
                    }
                    Row::Plain(PlainRow::Track(track)) => {
                        tracks.insert(track.id, track.clone());
                    }
                    Row::Plain(PlainRow::PlaylistEntry(entry)) => {
                        playlist_entries
                            .entry(entry.playlist_id)
                            .or_default()
                            .insert(entry.entry_index, entry.track_id);
                    }
                    _ => unreachable!("encountered unexpected row type: {row:?}"),
                })
        });

    fn print_track(
        track_id: &TrackId,
        artists: &HashMap<ArtistId, Artist>,
        tracks: &HashMap<TrackId, Track>,
    ) {
        let track = match tracks.get(track_id) {
            Some(track) => track,
            None => {
                println!("<Track for {track_id:?} not found>");
                return;
            }
        };
        let artist = match artists.get(&track.artist_id) {
            Some(artist) => artist,
            None => {
                println!(
                    "<Artist for {:?} not found> - {}",
                    &track.artist_id, track.offsets.title
                );
                return;
            }
        };
        println!("{} - {}", artist.offsets.name, track.offsets.title)
    }
    fn print_children_of(
        tree: &HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>>,
        tree_entries: &HashMap<PlaylistTreeNodeId, BTreeMap<u32, TrackId>>,
        artists: &HashMap<ArtistId, Artist>,
        tracks: &HashMap<TrackId, Track>,
        id: PlaylistTreeNodeId,
        level: usize,
    ) {
        tree.get(&id)
            .iter()
            .flat_map(|nodes| nodes.iter())
            .for_each(|node| {
                let indentation = "    ".repeat(level);
                println!(
                    "{}{} {}",
                    indentation,
                    if node.is_folder() { "🗀" } else { "🗎" },
                    node.name,
                );
                if let Some(playlist_tracks) = tree_entries.get(&node.id) {
                    for (index, track_id) in playlist_tracks.iter() {
                        print!("{}  ♫ {}: ", indentation, index);
                        print_track(track_id, artists, tracks);
                    }
                }
                print_children_of(tree, tree_entries, artists, tracks, node.id, level + 1);
            });
    }

    print_children_of(
        &playlist_tree,
        &playlist_entries,
        &artists,
        &tracks,
        PlaylistTreeNodeId(0),
        0,
    );

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
    let pdb = export.pdb().ok_or(rekordcrate::Error::NotLoadedError)?;
    let playlists = pdb.get_playlists()?;
    let tracks = pdb
        .get_tracks()
        .map(|track| (track.id, track))
        .collect::<HashMap<_, _>>();

    fn walk_tree(
        pdb: &rekordcrate::device::Pdb,
        tracks: &HashMap<TrackId, Track>,
        node: PlaylistNode,
        path: &PathBuf,
        export_path: &Path,
    ) -> rekordcrate::Result<()> {
        match node {
            PlaylistNode::Folder(folder) => {
                folder.children.into_iter().try_for_each(|child| {
                    walk_tree(pdb, tracks, child, &path.join(&folder.name), export_path)
                })?;
            }
            PlaylistNode::Playlist(playlist) => {
                let mut playlist_entries: Vec<(u32, TrackId)> =
                    pdb.get_playlist_entries(playlist.id).collect();
                playlist_entries.sort_by_key(|entry| entry.0);

                std::fs::create_dir_all(path)?;
                let playlist_path = path.join(format!("{}.m3u", playlist.name));

                println!("{}", playlist_path.display());
                let mut file = std::fs::File::create(playlist_path)?;
                playlist_entries
                    .into_iter()
                    .filter_map(|(_index, id)| tracks.get(&id))
                    .try_for_each(|track| -> rekordcrate::Result<()> {
                        let track_path = track.offsets.file_path.clone().into_string()?;
                        Ok(writeln!(
                            &mut file,
                            "{}",
                            export_path
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
        .try_for_each(|node| walk_tree(pdb, &tracks, node, output_dir, export.get_path()))?;

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
    let mut reader = std::fs::File::open(path)?;
    let anlz = ANLZ::read(&mut reader)?;
    println!("{:#?}", anlz);

    Ok(())
}

fn dump_pdb(path: &PathBuf, typ: DatabaseType) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(path)?;
    let header = Header::read_args(&mut reader, (typ,))?;

    println!("{:#?}", header);

    for (i, table) in header.tables.iter().enumerate() {
        println!("Table {}: {:?}", i, table.page_type);
        for page in header
            .read_pages(
                &mut reader,
                binrw::Endian::NATIVE,
                (&table.first_page, &table.last_page, typ),
            )
            .unwrap()
            .into_iter()
        {
            println!("  {:?}", page);
            match page.content {
                PageContent::Data(data_content) => {
                    for (_, row) in data_content.rows {
                        println!("      {:?}", row);
                    }
                }
                PageContent::Index(index_content) => {
                    println!("    {:?}", index_content);
                    for entry in index_content.entries {
                        println!("      {:?}", entry);
                    }
                }
                PageContent::Unknown => (),
            }
        }
    }

    Ok(())
}

fn dump_setting(path: &PathBuf) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(path)?;
    let setting = Setting::read(&mut reader)?;

    println!("{:#04x?}", setting);

    Ok(())
}

fn dump_xml(path: &PathBuf) -> rekordcrate::Result<()> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let document: Document = quick_xml::de::from_reader(reader).expect("failed to deserialize XML");
    println!("{:#?}", document);

    Ok(())
}

fn guess_db_type(path: &Path, db_type: Option<&str>) -> Option<DatabaseType> {
    let db_type_cli = db_type.map(|str| match str {
        "plain" => DatabaseType::Plain,
        "ext" => DatabaseType::Ext,
        invalid => unreachable!("invalid flag {invalid}, should have already been checked by clap"),
    });
    let file_name = match path.file_name() {
        None => {
            eprintln!("{} not a file!", path.display());
            return None; // TODO(Swiftb0y): turn this into a proper error
        }
        Some(file_name) => file_name,
    };
    let db_type_file = if file_name == "export.pdb" {
        Some(DatabaseType::Plain)
    } else if file_name == "exportExt.pdb" {
        Some(DatabaseType::Ext)
    } else {
        None
    };
    let db_type = match (db_type_cli, db_type_file) {
        (None, None) => {
            eprintln!("no DB_TYPE supplied nor could it be guessed!");
            return None; // TODO(Swiftb0y): turn this into a proper error
        }
        (None, Some(guess)) | (Some(guess), None) => guess,
        (Some(db_type_cli), Some(db_type_file)) if db_type_cli == db_type_file => db_type_cli,
        (Some(db_type_cli), Some(db_type_file)) => {
            eprintln!("Warning: passed {db_type_cli:?}, but found {db_type_file:?} from file name, using {db_type_cli:?}!");
            db_type_cli
        }
    };
    Some(db_type)
}

fn main() -> rekordcrate::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ListPlaylists { path } => list_playlists(path),
        Commands::ListSettings { path } => list_settings(path),
        Commands::ExportPlaylists { path, output_dir } => export_playlists(path, output_dir),
        Commands::DumpPDB { path, db_type } => {
            let db_type = match guess_db_type(path, db_type.as_deref()) {
                Some(db_type) => db_type,
                None => return Ok(()), // TODO(Swiftb0y): turn into proper error;
            };
            dump_pdb(path, db_type)
        }
        Commands::DumpANLZ { path } => dump_anlz(path),
        Commands::DumpSetting { path } => dump_setting(path),
        Commands::DumpXML { path } => dump_xml(path),
    }
}
