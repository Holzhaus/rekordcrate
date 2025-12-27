// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::BinRead;
use clap::{Parser, Subcommand};
use fallible_iterator::FallibleIterator;
use rekordcrate::pdb::io::Database;
use rekordcrate::pdb::*;
use rekordcrate::setting::{Setting, SettingType};
use rekordcrate::xml::Document;
use rekordcrate::{anlz::ANLZ, util::TableIndex};
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
        /// Setting type.
        #[arg(long, value_name = "SETTING_TYPE", value_parser = ["devsetting", "djmmysetting", "mysetting", "mysetting2"])]
        setting_type: Option<String>,
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
    let mut db = Database::open_non_persistent(&mut reader, DatabaseType::Plain)?;

    let mut playlist_tree: HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>> = HashMap::new();
    let mut playlist_entries: HashMap<PlaylistTreeNodeId, BTreeMap<u32, TrackId>> = HashMap::new();
    let mut artists: HashMap<ArtistId, Artist> = HashMap::new();
    let mut tracks: HashMap<TrackId, Track> = HashMap::new();

    db.iter_rows::<PlaylistTreeNode>()?.for_each(|tree_node| {
        playlist_tree
            .entry(tree_node.parent_id)
            .or_default()
            .push(tree_node.clone());
        Ok(())
    })?;

    db.iter_rows::<Artist>()?.for_each(|artist| {
        artists.insert(artist.id, artist.clone());
        Ok(())
    })?;

    db.iter_rows::<Track>()?.for_each(|track| {
        tracks.insert(track.id, track.clone());
        Ok(())
    })?;

    db.iter_rows::<PlaylistEntry>()?.for_each(|entry| {
        playlist_entries
            .entry(entry.playlist_id)
            .or_default()
            .insert(entry.entry_index, entry.track_id);
        Ok(())
    })?;

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

fn dump_anlz(path: &PathBuf) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(path)?;
    let anlz = ANLZ::read(&mut reader)?;
    println!("{:#?}", anlz);

    Ok(())
}

fn dump_pdb(path: &PathBuf, typ: DatabaseType) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(path)?;
    let mut db = Database::open_non_persistent(&mut reader, typ)?;

    println!("{:#?}", db.get_header());

    let tables = db.get_header().tables.clone();
    for (i, table) in tables.iter().enumerate() {
        let id = TableIndex::from(i);
        println!("Table {:?}: {:?}", id, table.page_type);
        let mut page_iter = db.iter_pages_for_table(id)?;
        while let Some(page) = page_iter.next()? {
            match &page.content {
                PageContent::Data(data_content) => {
                    for row in data_content.rows.values() {
                        println!("      {:?}", row);
                    }
                }
                PageContent::Index(index_content) => {
                    println!("    {:?}", index_content);
                    for entry in index_content.entries.iter() {
                        println!("      {:?}", entry);
                    }
                }
            }
        }
    }

    Ok(())
}

fn dump_setting(path: &PathBuf, setting_type: SettingType) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(path)?;
    let setting = Setting::read_args(&mut reader, (setting_type,))?;

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

fn guess_setting_type(path: &Path, setting_type: Option<&str>) -> Option<SettingType> {
    let setting_type_cli = setting_type.map(|str| match str {
        "devsetting" => SettingType::DevSetting,
        "djmmysetting" => SettingType::DJMMySetting,
        "mysetting" => SettingType::MySetting,
        "mysetting2" => SettingType::MySetting2,
        invalid => {
            unreachable!("invalid flag {invalid}, should have already been checked by clap")
        }
    });
    let file_name = match path.file_name() {
        None => {
            eprintln!("{} not a file!", path.display());
            return None; // TODO: turn into proper error
        }
        Some(file_name) => file_name,
    };
    let setting_type_file = SettingType::from_filename(file_name);
    let setting_type = match (setting_type_cli, setting_type_file) {
        (None, None) => {
            eprintln!("no SETTING_TYPE supplied nor could it be guessed!");
            return None; // TODO: turn into proper error
        }
        (None, Some(guess)) | (Some(guess), None) => guess,
        (Some(setting_type_cli), Some(setting_type_file))
            if setting_type_cli == setting_type_file =>
        {
            setting_type_cli
        }
        (Some(setting_type_cli), Some(setting_type_file)) => {
            eprintln!("Warning: passed {setting_type_cli:?}, but found {setting_type_file:?} from file name, using {setting_type_cli:?}!");
            setting_type_cli
        }
    };
    Some(setting_type)
}

fn main() -> rekordcrate::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ListPlaylists { path } => list_playlists(path),
        Commands::DumpPDB { path, db_type } => {
            let db_type = match guess_db_type(path, db_type.as_deref()) {
                Some(db_type) => db_type,
                None => return Ok(()), // TODO(Swiftb0y): turn into proper error;
            };
            dump_pdb(path, db_type)
        }
        Commands::DumpANLZ { path } => dump_anlz(path),
        Commands::DumpSetting { path, setting_type } => {
            let setting_type = match guess_setting_type(path, setting_type.as_deref()) {
                Some(setting_type) => setting_type,
                None => return Ok(()), // TODO: turn into proper error
            };
            dump_setting(path, setting_type)
        }
        Commands::DumpXML { path } => dump_xml(path),
    }
}
