// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::BinRead;
use clap::{Parser, Subcommand};
use rekordcrate::anlz::ANLZ;
use rekordcrate::pdb::{DatabaseType, Header, PageContent, PlainPageType, PlainRow, Row, Track, TrackId};
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
    /// Export device export to Rekordbox XML format.
    ExportXML {
        /// Path to the device export directory.
        #[arg(value_name = "EXPORT_PATH")]
        path: PathBuf,
        /// Output XML file path.
        #[arg(value_name = "OUTPUT_FILE")]
        output: Option<PathBuf>,
    },
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

fn export_xml(path: &Path, output_path: Option<&PathBuf>) -> rekordcrate::Result<()> {
    use rekordcrate::pdb::PlainRow;
    use rekordcrate::xml::{PositionMark, Tempo};

    // Load the device export
    let mut export = rekordcrate::DeviceExport::new(path.into());
    export.load_pdb()?;
    export.load_anlz()?;

    let pdb = export.pdb().ok_or(rekordcrate::Error::NotLoadedError)?;

    // Create XML document
    let mut doc = Document::new();

    for track in pdb.get_tracks() {
        let track_id = track.id.0 as i32;
        let title = track.offsets.title.clone().into_string().ok();
        let artist = track.artist_id.0;
        let album_id = track.album_id.0;
        let genre_id = track.genre_id.0;
        let duration = track.duration;
        let year = track.year;
        let tempo = track.tempo;
        let file_path = track.offsets.file_path.clone().into_string().unwrap_or_default();
        let analyze_path = track.offsets.analyze_path.clone().into_string().ok().unwrap_or_default();

        // Look up artist name
        let artist_name = if artist > 0 {
            pdb.get_rows_by_page_type(PlainPageType::Artists)
                .filter_map(|row| {
                    if let Row::Plain(PlainRow::Artist(a)) = row {
                        Some(a)
                    } else {
                        None
                    }
                })
                .find(|a| a.id.0 == artist)
                .and_then(|a| a.offsets.name.clone().into_string().ok())
        } else {
            None
        };

        // Look up album name
        let album_name = if album_id > 0 {
            pdb.get_rows_by_page_type(PlainPageType::Albums)
                .filter_map(|row| {
                    if let Row::Plain(PlainRow::Album(a)) = row {
                        Some(a)
                    } else {
                        None
                    }
                })
                .find(|a| a.id.0 == album_id)
                .and_then(|a| a.offsets.name.clone().into_string().ok())
        } else {
            None
        };

        // Look up genre name
        let genre_name = if genre_id > 0 {
            pdb.get_rows_by_page_type(PlainPageType::Genres)
                .filter_map(|row| {
                    if let Row::Plain(PlainRow::Genre(g)) = row {
                        Some(g)
                    } else {
                        None
                    }
                })
                .find(|g| g.id.0 == genre_id)
                .and_then(|g| g.name.clone().into_string().ok())
        } else {
            None
        };

        // Get track analysis (hot cues, BPM)
        let analysis = export.get_track_analysis(&analyze_path);

        // Create track
        let mut xml_track = rekordcrate::xml::Track::from_pdb_track(
            track_id,
            title,
            artist_name,
            album_name,
            genre_name,
            duration,
            year,
            tempo,
            file_path,
        );

        // Add tempos from analysis
        if let Some(analysis) = analysis {
            for tempo_entry in &analysis.tempos {
                xml_track.add_tempo(Tempo {
                    inizio: tempo_entry.start,
                    bpm: tempo_entry.bpm,
                    metro: "4/4".to_string(),
                    battito: 1,
                });
            }

            // Add hot cues as position marks
            for hot_cue in &analysis.hot_cues {
                let mark_type = if hot_cue.is_loop { 4 } else { 0 };
                xml_track.add_position_mark(PositionMark {
                    name: hot_cue.name.clone(),
                    mark_type,
                    start: hot_cue.start,
                    end: hot_cue.end,
                    num: (hot_cue.number as i32) - 1, // Convert to 0-indexed
                });
            }
        }

        doc.collection.track.push(xml_track);
    }

    doc.collection.entries = doc.collection.track.len() as i32;

    // Serialize to XML
    let xml_output = quick_xml::se::to_string(&doc).expect("failed to serialize XML");

    // Write to file or stdout
    match output_path {
        Some(output) => {
            std::fs::write(output, &xml_output)?;
            println!("Exported XML to: {}", output.display());
        }
        None => {
            println!("{}", xml_output);
        }
    }

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
        Commands::ExportXML { path, output } => export_xml(path, output.as_ref()),
    }
}
