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
use rekordcrate::pdb::{DatabaseType, Header, PageContent, Track, TrackId};
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
    /// Export track data from a Pioneer Database (`.PDB`) file to XML.
    ExportXML {
        /// File to parse.
        #[arg(value_name = "PDB_FILE")]
        path: PathBuf,
        /// Output file to write XML to (default: stdout).
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

/// Load hot cues from ANLZ files for a track
/// 
/// Searches for ANLZ files in the following locations:
/// 1. Same directory as the PDB file
/// 2. Parent directories (for typical rekordbox export structure)
/// 
/// ANLZ files are named ANLZ0001.DAT, ANLZ0002.DAT, etc.
fn load_hot_cues(pdb_dir: &Path, track_id: u32) -> rekordcrate::Result<Vec<rekordcrate::xml::PositionMark>> {
    use rekordcrate::anlz::{ANLZ, CueListType, CueType, Content};
    use rekordcrate::xml::PositionMark;
    use walkdir::WalkDir;
    
    let mut position_marks = Vec::new();
    
    // Format the ANLZ filename (e.g., ANLZ0001.DAT)
    let anlz_filename = format!("ANLZ{:04}.DAT", track_id);
    
    // Search for ANLZ files in PDB directory and parent directories
    let search_paths: Vec<PathBuf> = vec![
        pdb_dir.to_path_buf(),
        pdb_dir.parent().map(|p| p.to_path_buf()).unwrap_or_default(),
        pdb_dir.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or_default(),
    ];
    
    for search_dir in &search_paths {
        if search_dir.as_os_str().is_empty() {
            continue;
        }
        
        // Check if ANLZ file exists in this directory
        let anlz_path = search_dir.join(&anlz_filename);
        if anlz_path.exists() {
            // Found the ANLZ file, parse it
            if let Ok(mut reader) = std::fs::File::open(&anlz_path) {
                if let Ok(anlz) = ANLZ::read(&mut reader) {
                    // Find CueList sections and extract hot cues
                    for section in &anlz.sections {
                        if let Content::CueList(cue_list) = &section.content {
                            // Only process hot cues (not memory cues)
                            if cue_list.list_type == CueListType::HotCues {
                                for cue in &cue_list.cues {
                                    // Skip cues that are not hot cues (hot_cue == 0 means not a hot cue)
                                    if cue.hot_cue == 0 {
                                        continue;
                                    }
                                    
                                    // Convert cue time from milliseconds to seconds
                                    let start = cue.time as f64 / 1000.0;
                                    
                                    // Determine if this is a loop
                                    let (mark_type, end) = match cue.cue_type {
                                        CueType::Loop => (4, Some(start + 0.0)), // Loop type
                                        CueType::Point => (0, None), // Hot Cue type
                                    };
                                    
                                    let position_mark = PositionMark {
                                        name: format!("Hot Cue {}", (cue.hot_cue as i8).abs()),
                                        mark_type,
                                        start,
                                        end,
                                        num: cue.hot_cue as i32 - 1, // Convert to 0-based index
                                    };
                                    
                                    position_marks.push(position_mark);
                                }
                            }
                        } else if let Content::ExtendedCueList(extended_cue_list) = &section.content {
                            // Handle extended cue list (Nexus 2 series)
                            if extended_cue_list.list_type == CueListType::HotCues {
                                for cue in &extended_cue_list.cues {
                                    // Skip cues that are not hot cues
                                    if cue.hot_cue == 0 {
                                        continue;
                                    }
                                    
                                    // Convert cue time from milliseconds to seconds
                                    let start = cue.time as f64 / 1000.0;
                                    
                                    // Determine if this is a loop
                                    let (mark_type, end) = match cue.cue_type {
                                        CueType::Loop => (4, Some(start + 0.0)),
                                        CueType::Point => (0, None),
                                    };
                                    
                                    let position_mark = PositionMark {
                                        name: format!("Hot Cue {}", (cue.hot_cue as i8).abs()),
                                        mark_type,
                                        start,
                                        end,
                                        num: cue.hot_cue as i32 - 1,
                                    };
                                    
                                    position_marks.push(position_mark);
                                }
                            }
                        }
                    }
                    
                    // Found and processed the ANLZ file, no need to search further
                    break;
                }
            }
        }
        
        // Also search recursively in subdirectories (like PIONEER/USBANLZ)
        for entry in WalkDir::new(search_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name().map(|n| n.to_string_lossy() == anlz_filename).unwrap_or(false) {
                // Found the ANLZ file, parse it
                if let Ok(mut reader) = std::fs::File::open(path) {
                    if let Ok(anlz) = ANLZ::read(&mut reader) {
                        // Find CueList sections and extract hot cues
                        for section in &anlz.sections {
                            if let Content::CueList(cue_list) = &section.content {
                                if cue_list.list_type == CueListType::HotCues {
                                    for cue in &cue_list.cues {
                                        if cue.hot_cue == 0 {
                                            continue;
                                        }
                                        
                                        let start = cue.time as f64 / 1000.0;
                                        
                                        let (mark_type, end) = match cue.cue_type {
                                            CueType::Loop => (4, Some(start + 0.0)),
                                            CueType::Point => (0, None),
                                        };
                                        
                                        let position_mark = PositionMark {
                                            name: format!("Hot Cue {}", (cue.hot_cue as i8).abs()),
                                            mark_type,
                                            start,
                                            end,
                                            num: cue.hot_cue as i32 - 1,
                                        };
                                        
                                        position_marks.push(position_mark);
                                    }
                                }
                            } else if let Content::ExtendedCueList(extended_cue_list) = &section.content {
                                if extended_cue_list.list_type == CueListType::HotCues {
                                    for cue in &extended_cue_list.cues {
                                        if cue.hot_cue == 0 {
                                            continue;
                                        }
                                        
                                        let start = cue.time as f64 / 1000.0;
                                        
                                        let (mark_type, end) = match cue.cue_type {
                                            CueType::Loop => (4, Some(start + 0.0)),
                                            CueType::Point => (0, None),
                                        };
                                        
                                        let position_mark = PositionMark {
                                            name: format!("Hot Cue {}", (cue.hot_cue as i8).abs()),
                                            mark_type,
                                            start,
                                            end,
                                            num: cue.hot_cue as i32 - 1,
                                        };
                                        
                                        position_marks.push(position_mark);
                                    }
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    }
    
    Ok(position_marks)
}

fn export_xml(path: &Path, output: Option<&Path>) -> rekordcrate::Result<()> {
    use rekordcrate::pdb::{DatabaseType, Header, PageContent};
    use rekordcrate::xml::{Collection, Document, Playlists, PlaylistFolderNode, Product, Tempo, Track};
    use serde::Serialize;

    // Open and parse the PDB file
    let mut reader = std::fs::File::open(path)?;
    let db_type = DatabaseType::Plain;
    let header = Header::read_args(&mut reader, (db_type,))?;
    
    // Get the PDB file directory for finding ANLZ files
    let pdb_dir = path.parent().unwrap_or(Path::new("."));
    
    // Collect tracks from the PDB
    let mut tracks: Vec<Track> = Vec::new();
    
    for table in &header.tables {
        if matches!(table.page_type, rekordcrate::pdb::PageType::Plain(rekordcrate::pdb::PlainPageType::Tracks)) {
            let pages = header.read_pages(
                &mut reader,
                binrw::Endian::NATIVE,
                (&table.first_page, &table.last_page, db_type),
            )?;
            
            for page in pages {
                if let PageContent::Data(data_content) = page.content {
                    for (_, row) in data_content.rows {
                        if let rekordcrate::pdb::Row::Plain(rekordcrate::pdb::PlainRow::Track(track)) = row {
                            // Convert PDB track to XML track
                            let track_id = track.id.0 as i32;
                            let title = track.offsets.title.clone().into_string().ok();
                            let location = track.offsets.file_path.clone().into_string().ok().unwrap_or_default();
                            
                            // BPM is stored in centi-BPM, convert to actual BPM
                            let bpm = if track.tempo > 0 {
                                Some(track.tempo as f64 / 100.0)
                            } else {
                                None
                            };
                            
                            // Duration is in seconds
                            let duration = if track.duration > 0 {
                                Some(track.duration as f64)
                            } else {
                                None
                            };
                            
                            // File size
                            let file_size = if track.file_size > 0 {
                                Some(track.file_size as i64)
                            } else {
                                None
                            };
                            
                            // Load hot cues from ANLZ files
                            let position_marks = load_hot_cues(pdb_dir, track.id.0)?;
                            
                            // Create tempo element if BPM is available
                            let tempos: Vec<Tempo> = if let Some(bpm_val) = bpm {
                                vec![Tempo {
                                    inizio: 0.025,
                                    bpm: bpm_val,
                                    metro: "4/4".to_string(),
                                    battito: 1,
                                }]
                            } else {
                                vec![]
                            };
                            
                            let xml_track = Track {
                                trackid: track_id,
                                name: title,
                                artist: None,  // Would need to look up artist from artist ID
                                composer: None,
                                album: None,
                                grouping: None,
                                genre: None,
                                kind: Some("MP3 File".to_string()),
                                size: file_size,
                                totaltime: duration,
                                discnumber: if track.disc_number > 0 { Some(track.disc_number as i32) } else { None },
                                tracknumber: if track.track_number > 0 { Some(track.track_number as i32) } else { None },
                                year: if track.year > 0 { Some(track.year as i32) } else { None },
                                averagebpm: bpm,
                                datemodified: None,
                                dateadded: None,
                                bitrate: if track.bitrate > 0 { Some(track.bitrate as i32) } else { None },
                                samplerate: if track.sample_rate > 0 { Some(track.sample_rate as f64) } else { None },
                                comments: None,
                                playcount: if track.play_count > 0 { Some(track.play_count as i32) } else { None },
                                lastplayed: None,
                                rating: if track.rating > 0 { Some(track.rating as i32) } else { None },
                                location,
                                remixer: None,
                                tonality: None,
                                label: None,
                                mix: None,
                                colour: None,
                                tempos,
                                position_marks,
                            };
                            
                            tracks.push(xml_track);
                        }
                    }
                }
            }
        }
    }
    
    // Create the XML document
    let document = Document {
        version: "1.0.0".to_string(),
        product: Product {
            name: "rekordcrate".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            company: "rekordcrate".to_string(),
        },
        collection: Collection {
            entries: tracks.len() as i32,
            track: tracks,
        },
        playlists: Playlists {
            node: PlaylistFolderNode {
                name: "ROOT".to_string(),
                nodes: vec![],
            },
        },
    };
    
    // Serialize to XML using quick_xml with a writer
    let mut xml_output = String::new();
    {
        let mut serializer = quick_xml::se::Serializer::new(&mut xml_output);
        serializer.indent(' ', 2);
        document.serialize(serializer).map_err(|e| {
            rekordcrate::Error::XmlError(format!("Failed to serialize XML: {}", e))
        })?;
    }
    
    // Add XML declaration
    let xml_string = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml_output);
    
    // Write output
    if let Some(output_path) = output {
        std::fs::write(output_path, &xml_string)?;
        println!("Exported XML to: {}", output_path.display());
    } else {
        println!("{}", xml_string);
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
        Commands::ExportXML { path, output } => export_xml(&path, output.as_ref().map(|p| p.as_path())),
    }
}
