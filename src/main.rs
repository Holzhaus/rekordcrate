// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::BinRead;
use clap::{Parser, Subcommand};
use rekordcrate::anlz::{Content, CueListType, CueType, ANLZ};
use rekordcrate::device::{Pdb, PlaylistNode};
use rekordcrate::pdb::{ArtistId, DatabaseType, Header, PageContent, Track, TrackId};
use rekordcrate::setting::Setting;
use rekordcrate::util::FileType;
use rekordcrate::xml::{
    Collection, Document, PlaylistFolderNode, PlaylistGenericNode, PlaylistPlaylistNode,
    PlaylistTrack, Playlists, PositionMark, Product, Tempo,
};
use std::collections::HashMap;
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
    /// Export a Rekordbox device export to Rekordbox XML.
    ExportXML {
        /// Path to a Rekordbox device export containing a PIONEER directory.
        #[arg(value_name = "EXPORT_PATH")]
        path: PathBuf,
        /// XML file to write.
        #[arg(value_name = "XML_FILE")]
        output_file: PathBuf,
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

fn export_xml(path: &Path, output_file: &Path) -> rekordcrate::Result<()> {
    use rekordcrate::DeviceExport;

    let mut export = DeviceExport::new(path.into());
    export.load_pdb()?;
    let pdb = export.pdb().ok_or(rekordcrate::Error::NotLoadedError)?;
    let document = build_xml_document(pdb, export.get_path())?;
    let xml = quick_xml::se::to_string(&document)?;
    std::fs::write(
        output_file,
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{xml}\n"),
    )?;

    Ok(())
}

fn build_xml_document(pdb: &Pdb, export_path: &Path) -> rekordcrate::Result<Document> {
    let artists = pdb
        .get_artists()
        .map(|artist| Ok((artist.id, artist.offsets.name.clone().into_string()?)))
        .collect::<rekordcrate::Result<HashMap<ArtistId, String>>>()?;
    let tracks = pdb
        .get_tracks()
        .map(|track| track_to_xml_track(track, &artists, export_path))
        .collect::<rekordcrate::Result<Vec<_>>>()?;
    let playlists = pdb
        .get_playlists()?
        .into_iter()
        .map(|node| playlist_node_to_xml(pdb, node))
        .collect::<rekordcrate::Result<Vec<_>>>()?;

    Ok(Document {
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
                nodes: playlists,
            },
        },
    })
}

fn track_to_xml_track(
    track: &Track,
    artists: &HashMap<ArtistId, String>,
    export_path: &Path,
) -> rekordcrate::Result<rekordcrate::xml::Track> {
    let title = non_empty(track.offsets.title.clone().into_string()?);
    let file_path = track.offsets.file_path.clone().into_string()?;
    let comment = non_empty(track.offsets.comment().clone().into_string()?);
    let analyses = load_analyses(export_path, track.offsets.analyze_path())?;
    let tempos = xml_tempos(&analyses, track.average_bpm());
    let position_marks = xml_position_marks(&analyses);

    Ok(rekordcrate::xml::Track {
        trackid: track.id.0 as i32,
        name: title,
        artist: artists.get(&track.artist_id).cloned().and_then(non_empty),
        composer: None,
        album: None,
        grouping: None,
        genre: None,
        kind: file_type_name(track.file_type()).map(String::from),
        size: (track.file_size() > 0).then_some(i64::from(track.file_size())),
        totaltime: (track.duration() > 0).then_some(f64::from(track.duration())),
        discnumber: (track.disc_number() > 0).then_some(i32::from(track.disc_number())),
        tracknumber: (track.track_number() > 0).then_some(track.track_number() as i32),
        year: (track.year() > 0).then_some(i32::from(track.year())),
        averagebpm: track.average_bpm(),
        datemodified: None,
        dateadded: None,
        bitrate: (track.bitrate() > 0).then_some(track.bitrate() as i32),
        samplerate: (track.sample_rate() > 0).then_some(f64::from(track.sample_rate())),
        comments: comment,
        playcount: (track.play_count() > 0).then_some(i32::from(track.play_count())),
        lastplayed: None,
        rating: None,
        location: location_uri(&file_path),
        remixer: None,
        tonality: None,
        label: None,
        mix: None,
        colour: None,
        tempos,
        position_marks,
    })
}

fn load_analyses(
    export_path: &Path,
    analyze_path: &rekordcrate::pdb::string::DeviceSQLString,
) -> rekordcrate::Result<Vec<ANLZ>> {
    let analyze_path = analyze_path.clone().into_string()?;
    if analyze_path.is_empty() {
        return Ok(Vec::new());
    }

    analysis_paths(&export_path.join(analyze_path.trim_start_matches('/')))
        .into_iter()
        .map(|path| match std::fs::File::open(path) {
            Ok(mut reader) => Ok(Some(ANLZ::read(&mut reader)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        })
        .filter_map(Result::transpose)
        .collect()
}

fn analysis_paths(dat_path: &Path) -> Vec<PathBuf> {
    ["DAT", "EXT", "2EX"]
        .into_iter()
        .map(|extension| dat_path.with_extension(extension))
        .collect()
}

fn xml_tempos(analyses: &[ANLZ], average_bpm: Option<f64>) -> Vec<Tempo> {
    let tempos = analyses
        .iter()
        .flat_map(|analysis| {
            analysis
                .sections
                .iter()
                .filter_map(|section| {
                    if let Content::BeatGrid(beatgrid) = &section.content {
                        Some(
                            beatgrid
                                .beats
                                .iter()
                                .map(|beat| Tempo {
                                    inizio: f64::from(beat.time) / 1000.0,
                                    bpm: f64::from(beat.tempo) / 100.0,
                                    metro: "4/4".to_string(),
                                    battito: i32::from(beat.beat_number),
                                })
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    }
                })
                .flatten()
        })
        .collect::<Vec<_>>();
    if !tempos.is_empty() {
        return tempos;
    }

    average_bpm
        .map(|bpm| {
            vec![Tempo {
                inizio: 0.0,
                bpm,
                metro: "4/4".to_string(),
                battito: 1,
            }]
        })
        .unwrap_or_default()
}

fn xml_position_marks(analyses: &[ANLZ]) -> Vec<PositionMark> {
    let mut position_marks = analyses
        .iter()
        .flat_map(|analysis| {
            analysis
                .sections
                .iter()
                .flat_map(|section| match &section.content {
                    Content::CueList(cue_list) if cue_list.list_type == CueListType::HotCues => {
                        cue_list
                            .cues
                            .iter()
                            .filter_map(|cue| {
                                hot_cue_position_mark(
                                    cue.hot_cue,
                                    &cue.cue_type,
                                    cue.time,
                                    cue.loop_time,
                                )
                            })
                            .collect::<Vec<_>>()
                    }
                    Content::ExtendedCueList(cue_list)
                        if cue_list.list_type == CueListType::HotCues =>
                    {
                        cue_list
                            .cues
                            .iter()
                            .filter_map(|cue| {
                                hot_cue_position_mark(
                                    cue.hot_cue,
                                    &cue.cue_type,
                                    cue.time,
                                    cue.loop_time,
                                )
                            })
                            .collect::<Vec<_>>()
                    }
                    _ => Vec::new(),
                })
        })
        .collect::<Vec<_>>();
    position_marks.sort_by(|a, b| {
        a.num
            .cmp(&b.num)
            .then_with(|| a.start.total_cmp(&b.start))
            .then_with(|| a.mark_type.cmp(&b.mark_type))
    });
    position_marks.dedup_by(|a, b| {
        a.num == b.num && a.mark_type == b.mark_type && a.start == b.start && a.end == b.end
    });
    position_marks
}

fn hot_cue_position_mark(
    hot_cue: u32,
    cue_type: &CueType,
    time: u32,
    loop_time: u32,
) -> Option<PositionMark> {
    if hot_cue == 0 {
        return None;
    }

    let is_loop = *cue_type == CueType::Loop;
    Some(PositionMark {
        name: hot_cue_name(hot_cue),
        mark_type: if is_loop { 4 } else { 0 },
        start: f64::from(time) / 1000.0,
        end: (is_loop && loop_time > time).then_some(f64::from(loop_time) / 1000.0),
        num: hot_cue.saturating_sub(1) as i32,
    })
}

fn playlist_node_to_xml(pdb: &Pdb, node: PlaylistNode) -> rekordcrate::Result<PlaylistGenericNode> {
    match node {
        PlaylistNode::Folder(folder) => Ok(PlaylistGenericNode::Folder(PlaylistFolderNode {
            name: folder.name,
            nodes: folder
                .children
                .into_iter()
                .map(|child| playlist_node_to_xml(pdb, child))
                .collect::<rekordcrate::Result<Vec<_>>>()?,
        })),
        PlaylistNode::Playlist(playlist) => {
            let mut entries = pdb.get_playlist_entries(playlist.id).collect::<Vec<_>>();
            entries.sort_by_key(|(index, _)| *index);
            Ok(PlaylistGenericNode::Playlist(PlaylistPlaylistNode {
                name: playlist.name,
                keytype: "0".to_string(),
                tracks: entries
                    .into_iter()
                    .map(|(_index, track_id)| PlaylistTrack {
                        key: track_id.0 as i32,
                    })
                    .collect(),
            }))
        }
    }
}

fn location_uri(file_path: &str) -> String {
    let path = percent_encode_path(file_path);
    if path.starts_with('/') {
        format!("file://localhost{path}")
    } else {
        format!("file://localhost/{path}")
    }
}

fn percent_encode_path(path: &str) -> String {
    path.bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'-' | b'_' | b'~' => {
                vec![char::from(byte)]
            }
            other => format!("%{other:02X}").chars().collect(),
        })
        .collect()
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn file_type_name(file_type: &FileType) -> Option<&'static str> {
    match file_type {
        FileType::Unknown => None,
        FileType::Mp3 => Some("MP3 File"),
        FileType::M4a => Some("M4A File"),
        FileType::Flac => Some("FLAC File"),
        FileType::Wav => Some("WAV File"),
        FileType::Aiff => Some("AIFF File"),
        FileType::Other(_) => None,
    }
}

fn hot_cue_name(hot_cue: u32) -> String {
    if (1..=26).contains(&hot_cue) {
        char::from(b'A' + (hot_cue as u8) - 1).to_string()
    } else {
        hot_cue.to_string()
    }
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
        Commands::ExportXML { path, output_file } => export_xml(path, output_file),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_uri_percent_encodes_spaces() {
        assert_eq!(
            location_uri("/Contents/My Tracks/demo track.mp3"),
            "file://localhost/Contents/My%20Tracks/demo%20track.mp3"
        );
    }

    #[test]
    fn hot_cue_position_mark_converts_loop_metadata() {
        let mark = hot_cue_position_mark(2, &CueType::Loop, 1_000, 2_500)
            .expect("hot cue should become position mark");

        assert_eq!(mark.name, "B");
        assert_eq!(mark.mark_type, 4);
        assert_eq!(mark.start, 1.0);
        assert_eq!(mark.end, Some(2.5));
        assert_eq!(mark.num, 1);
    }
}
