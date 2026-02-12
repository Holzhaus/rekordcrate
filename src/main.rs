// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::BinRead;
use clap::{Parser, Subcommand, ValueEnum};
use rekordcrate::anlz::ANLZ;
use rekordcrate::pdb::{DatabaseType, Header, Page, PageContent, PageType, Track, TrackId};
use rekordcrate::setting::Setting;
use rekordcrate::xml::Document;
use serde::Serialize;
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
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value_t = DumpFormat::Debug)]
        format: DumpFormat,
    },
    /// Parse and dump a Pioneer Database (`.PDB`) file.
    DumpPDB {
        /// File to parse.
        #[arg(value_name = "PDB_FILE")]
        path: PathBuf,
        /// Database type: "plain" (export.pdb) or "ext" (exportExt.pdb). Tries to guess based on file name of not specified.
        #[arg(long, value_name = "DB_TYPE", value_parser = ["plain", "ext"])]
        db_type: Option<String>,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value_t = DumpFormat::Debug)]
        format: DumpFormat,
    },
    /// Parse and dump a Pioneer Settings (`*SETTING.DAT`) file.
    DumpSetting {
        /// File to parse.
        #[arg(value_name = "SETTING_FILE")]
        path: PathBuf,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value_t = DumpFormat::Debug)]
        format: DumpFormat,
    },
    /// Parse and dump a Pioneer XML (`*.xml`) file.
    DumpXML {
        /// File to parse.
        #[arg(value_name = "XML_FILE")]
        path: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
enum DumpFormat {
    #[default]
    Debug,
    Json,
}

#[derive(Serialize)]
struct TableDump {
    page_type: PageType,
    pages: Vec<Page>,
}

#[derive(Serialize)]
struct PdbDump {
    header: Header,
    tables: Vec<TableDump>,
}

fn list_playlists(path: &PathBuf) -> rekordcrate::Result<()> {
    use rekordcrate::device::{Pdb, PlaylistNode};
    use std::collections::HashMap;

    let pdb = Pdb::open_from_path(path)?;
    let playlists = pdb.get_playlists()?;
    let tracks: HashMap<_, _> = pdb.get_tracks().map(|t| (t.id, t)).collect();

    fn print_node(pdb: &Pdb, tracks: &HashMap<TrackId, &Track>, node: &PlaylistNode, level: usize) {
        let indentation = "    ".repeat(level);
        match node {
            PlaylistNode::Folder(folder) => {
                println!("{}🗀 {}", indentation, folder.name);
                for child in &folder.children {
                    print_node(pdb, tracks, child, level + 1);
                }
            }
            PlaylistNode::Playlist(playlist) => {
                println!("{}🗎 {}", indentation, playlist.name);
                let mut entries: Vec<_> = pdb.get_playlist_entries(playlist.id).collect();
                entries.sort_by_key(|(index, _)| *index);
                for (index, track_id) in entries {
                    if let Some(track) = tracks.get(&track_id) {
                        println!("{}  ♫ {}: {}", indentation, index, track.offsets.title);
                    } else {
                        println!(
                            "{}  ♫ {}: <Track for {:?} not found>",
                            indentation, index, track_id
                        );
                    }
                }
            }
        }
    }

    for node in &playlists {
        print_node(&pdb, &tracks, node, 0);
    }

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
        tracks: &HashMap<TrackId, &Track>,
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
    export.load_settings();
    let settings = export.get_settings();

    print!("{}", settings);

    Ok(())
}

fn dump_anlz(path: &PathBuf, format: DumpFormat) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(path)?;
    let anlz = ANLZ::read(&mut reader)?;
    match format {
        DumpFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&anlz).expect("failed to serialize ANLZ")
        ),
        DumpFormat::Debug => println!("{:#?}", anlz),
    }

    Ok(())
}

fn dump_pdb(path: &PathBuf, typ: DatabaseType, format: DumpFormat) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(path)?;
    let header = Header::read_args(&mut reader, (typ,))?;

    match format {
        DumpFormat::Json => {
            let mut tables = Vec::new();
            for table in &header.tables {
                let mut pages = Vec::new();
                for page in header
                    .read_pages(
                        &mut reader,
                        binrw::Endian::NATIVE,
                        (&table.first_page, &table.last_page, typ),
                    )
                    .unwrap()
                    .into_iter()
                {
                    pages.push(page);
                }
                tables.push(TableDump {
                    page_type: table.page_type,
                    pages,
                });
            }
            let dump = PdbDump { header, tables };
            println!(
                "{}",
                serde_json::to_string_pretty(&dump).expect("failed to serialize PDB output")
            );
        }
        DumpFormat::Debug => {
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
        }
    }

    Ok(())
}

fn dump_setting(path: &PathBuf, format: DumpFormat) -> rekordcrate::Result<()> {
    let mut reader = std::fs::File::open(path)?;
    let setting = Setting::read(&mut reader)?;

    match format {
        DumpFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&setting).expect("failed to serialize Setting")
        ),
        DumpFormat::Debug => println!("{:#04x?}", setting),
    }

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
        Commands::DumpPDB {
            path,
            db_type,
            format,
        } => {
            let db_type = match guess_db_type(path, db_type.as_deref()) {
                Some(db_type) => db_type,
                None => return Ok(()), // TODO(Swiftb0y): turn into proper error;
            };
            dump_pdb(path, db_type, *format)
        }
        Commands::DumpANLZ { path, format } => dump_anlz(path, *format),
        Commands::DumpSetting { path, format } => dump_setting(path, *format),
        Commands::DumpXML { path } => dump_xml(path),
    }
}
