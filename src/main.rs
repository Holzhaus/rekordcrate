// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::BinRead;
use binrw::BinWrite;
use clap::{Parser, Subcommand, ValueEnum};
use fallible_iterator::FallibleIterator;
use rekordcrate::device::{get_playlists, DeviceExportWriter, ExportLayout, Track as DeviceTrack};
use rekordcrate::edb::Library;
use rekordcrate::pdb::io::Database;
use rekordcrate::pdb::*;
use rekordcrate::setting::{Setting, SettingType};
use rekordcrate::{anlz::ANLZ, util::TableIndex};
#[cfg(feature = "json")]
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use thiserror::Error;

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Output format for the dump commands. The `Json` variant is only available when the
/// `json` feature is enabled; clap's derive drops it from `--format`'s allowed values when absent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
enum DumpFormat {
    #[default]
    #[value(name = "debug")]
    Debug,
    #[cfg(feature = "json")]
    #[value(name = "json")]
    Json,
}

#[derive(Debug, Error)]
enum CliError {
    #[error(transparent)]
    Rekordcrate(#[from] rekordcrate::Error),

    #[error("{0} is not a file!")]
    NotAFile(PathBuf),

    #[error("no DB_TYPE supplied nor could it be guessed for {0}!")]
    CouldNotInferDatabaseType(PathBuf),

    #[error("no SETTING_TYPE supplied nor could it be guessed for {0}!")]
    CouldNotInferSettingType(PathBuf),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("missing {0}")]
    MissingArgument(&'static str),

    #[error("the library holds no single playlist named {0:?}")]
    NoSuchPlaylist(String),
}

type CliResult<T> = Result<T, CliError>;

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
        /// Attempt to parse unknown table types instead of skipping them.
        #[arg(long)]
        parse_unknown_tables: bool,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value_t = DumpFormat::Debug)]
        format: DumpFormat,
    },
    /// Parse and dump a Pioneer Settings (`*SETTING.DAT`) file.
    DumpSetting {
        /// File to parse.
        #[arg(value_name = "SETTING_FILE")]
        path: PathBuf,
        /// Setting type.
        #[arg(long, value_name = "SETTING_TYPE", value_parser = ["devsetting", "djmmysetting", "mysetting", "mysetting2"])]
        setting_type: Option<String>,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value_t = DumpFormat::Debug)]
        format: DumpFormat,
    },
    /// List the playlists of a Rekordbox 5 library, or write a device export holding some of them.
    Library {
        /// Directory holding the library, i.e. the one with `datafile.edb` in it.
        #[arg(value_name = "LIBRARY_DIR")]
        path: PathBuf,
        /// List the playlists of the library instead of writing an export.
        #[arg(short = 'l', long)]
        list: bool,
        /// Playlist to put on the export. May be given more than once.
        #[arg(short = 'p', long = "playlist", value_name = "NAME")]
        playlists: Vec<String>,
        /// Directory to write the export to, which must not hold one yet.
        #[arg(value_name = "OUTPUT_DIR")]
        output: Option<PathBuf>,
    },
    /// Parse and dump a Pioneer XML (`*.xml`) file.
    #[cfg(feature = "xml")]
    DumpXML {
        /// File to parse.
        #[arg(value_name = "XML_FILE")]
        path: PathBuf,
    },
}

fn list_playlists(path: &Path) -> rekordcrate::Result<()> {
    use rekordcrate::pdb::{PlaylistTreeNode, PlaylistTreeNodeId};
    use std::collections::HashMap;

    let reader = File::open(path)?;
    let mut db = Database::open_non_persistent(reader, DatabaseType::Plain)?;

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
                    track.artist_id, track.offsets.title
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

fn export_playlists(path: &Path, output_dir: &Path) -> rekordcrate::Result<()> {
    use rekordcrate::device::{Playlist, PlaylistNode};
    use rekordcrate::DeviceExportLoader;
    use std::collections::HashMap;
    use std::io::Write;

    let loader = DeviceExportLoader::new(path.into());
    let export_path = loader.get_path();
    let mut db = loader.open_pdb_non_persistent()?;

    let playlists = get_playlists(&mut db)?;
    let playlist_entries = db
        .iter_rows::<PlaylistEntry>()?
        .map(|entry| Ok((entry.playlist_id, (entry.entry_index, entry.track_id))))
        .fold(
            HashMap::<PlaylistTreeNodeId, BTreeMap<u32, TrackId>>::new(),
            |mut acc, (playlist_id, entry)| {
                // BTreeMap keeps entries sorted by their index in the playlist.
                acc.entry(playlist_id).or_default().insert(entry.0, entry.1);
                Ok(acc)
            },
        )?;

    let tracks = db
        .iter_rows::<Track>()?
        .map(|track| Ok((track.id, track.offsets.file_path.clone().into_string()?)))
        .collect::<HashMap<_, _>>()?;

    fn walk_tree(
        node: PlaylistNode,
        path: &Path,
        visit: &impl Fn(Playlist, &Path) -> rekordcrate::Result<()>,
    ) -> rekordcrate::Result<()> {
        match node {
            PlaylistNode::Folder(folder) => folder
                .children
                .into_iter()
                .try_for_each(|child| walk_tree(child, &path.join(&folder.name), visit)),
            PlaylistNode::Playlist(playlist) => visit(playlist, path),
        }
    }

    playlists.into_iter().try_for_each(|node| {
        walk_tree(node, output_dir, &|playlist, path| {
            std::fs::create_dir_all(path)?;
            let playlist_path = path.join(format!("{}.m3u", playlist.name));

            println!("{}", playlist_path.display());
            let mut file = File::create(playlist_path)?;
            if let Some(entries) = playlist_entries.get(&playlist.id) {
                for track in entries.values().filter_map(|track_id| tracks.get(track_id)) {
                    writeln!(
                        &mut file,
                        "{}",
                        export_path
                            .canonicalize()?
                            .join(track.strip_prefix('/').unwrap_or(track))
                            .display(),
                    )?;
                }
            }
            Ok(())
        })
    })?;

    Ok(())
}

fn list_settings(path: &Path) -> rekordcrate::Result<()> {
    use rekordcrate::DeviceExportLoader;

    let loader = DeviceExportLoader::new(path.into());
    let settings = loader.load_settings();

    print!("{}", settings);

    Ok(())
}

fn dump_anlz(path: &Path, format: DumpFormat) -> rekordcrate::Result<()> {
    let mut reader = File::open(path)?;
    let anlz = ANLZ::read(&mut reader)?;
    match format {
        #[cfg(feature = "json")]
        DumpFormat::Json => println!("{}", serde_json::to_string_pretty(&anlz)?),
        DumpFormat::Debug => println!("{:#?}", anlz),
    }

    Ok(())
}

/// A table and all of its pages, for JSON serialization.
#[cfg(feature = "json")]
#[derive(Serialize)]
struct TableDump {
    page_type: PageType,
    pages: Vec<Page>,
}

/// The full PDB dump (header + tables), for JSON serialization.
#[cfg(feature = "json")]
#[derive(Serialize)]
struct PdbDump {
    header: Header,
    tables: Vec<TableDump>,
}

fn dump_pdb(
    path: &Path,
    typ: DatabaseType,
    parse_unknown_tables: bool,
    format: DumpFormat,
) -> rekordcrate::Result<()> {
    let reader = File::open(path)?;
    let mut db = Database::open_non_persistent(reader, typ)?;

    // `format` is only consulted under the json feature; bind it to avoid an unused-variable
    // warning when building with `cli` alone.
    #[cfg(not(feature = "json"))]
    let _ = format;

    #[cfg(feature = "json")]
    if matches!(format, DumpFormat::Json) {
        let header = db.get_header().clone();
        let mut tables = Vec::new();
        for (i, table) in header.tables.iter().enumerate() {
            let id = TableIndex::from(i);
            // Honor the same skip rule as the debug path so JSON output stays consistent.
            if matches!(table.page_type, PageType::Unknown(_)) && !parse_unknown_tables {
                continue;
            }
            let mut pages = Vec::new();
            let mut page_iter = db.iter_pages_for_table(id)?;
            while let Some(page) = page_iter.next()? {
                pages.push(page.clone());
            }
            tables.push(TableDump {
                page_type: table.page_type,
                pages,
            });
        }
        let dump = PdbDump { header, tables };
        println!("{}", serde_json::to_string_pretty(&dump)?);
        return Ok(());
    }

    println!("{:#?}", db.get_header());

    fn dump_table(db: &mut Database<File>, id: TableIndex) -> rekordcrate::Result<()> {
        let mut page_iter = db.iter_pages_for_table(id)?;
        while let Some(page) = page_iter.next()? {
            println!("  {:?}", page);
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
        Ok(())
    }

    let tables = db.get_header().tables.clone();
    for (i, table) in tables.iter().enumerate() {
        let id = TableIndex::from(i);
        println!("Table {:?}: {:?}", id, table.page_type);
        if matches!(table.page_type, PageType::Unknown(_)) && !parse_unknown_tables {
            println!(
                "  Skipping unknown table type, use --parse-unknown-tables to attempt parsing"
            );
            continue;
        }
        if let Err(e) = dump_table(&mut db, id) {
            eprintln!("Error dumping table {:?}: {e}", id);
        }
    }

    Ok(())
}

fn dump_setting(
    path: &Path,
    setting_type: SettingType,
    format: DumpFormat,
) -> rekordcrate::Result<()> {
    let mut reader = File::open(path)?;
    let setting = Setting::read_args(&mut reader, (setting_type,))?;

    match format {
        #[cfg(feature = "json")]
        DumpFormat::Json => println!("{}", serde_json::to_string_pretty(&setting)?),
        DumpFormat::Debug => println!("{:#04x?}", setting),
    }

    Ok(())
}

#[cfg(feature = "xml")]
fn dump_xml(path: &Path) -> rekordcrate::Result<()> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let document: rekordcrate::xml::Document = quick_xml::de::from_reader(reader)?;
    println!("{:#?}", document);

    Ok(())
}

fn guess_db_type(path: &Path, db_type: Option<&str>) -> CliResult<DatabaseType> {
    let db_type_cli = db_type.map(|str| match str {
        "plain" => DatabaseType::Plain,
        "ext" => DatabaseType::Ext,
        invalid => unreachable!("invalid flag {invalid}, should have already been checked by clap"),
    });
    let file_name = match path.file_name() {
        None => {
            return Err(CliError::NotAFile(path.to_path_buf()));
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
            return Err(CliError::CouldNotInferDatabaseType(path.to_path_buf()));
        }
        (None, Some(guess)) | (Some(guess), None) => guess,
        (Some(db_type_cli), Some(db_type_file)) if db_type_cli == db_type_file => db_type_cli,
        (Some(db_type_cli), Some(db_type_file)) => {
            eprintln!("Warning: passed {db_type_cli:?}, but found {db_type_file:?} from file name, using {db_type_cli:?}!");
            db_type_cli
        }
    };
    Ok(db_type)
}

fn guess_setting_type(path: &Path, setting_type: Option<&str>) -> CliResult<SettingType> {
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
            return Err(CliError::NotAFile(path.to_path_buf()));
        }
        Some(file_name) => file_name,
    };
    let setting_type_file = SettingType::from_filename(file_name);
    let setting_type = match (setting_type_cli, setting_type_file) {
        (None, None) => {
            return Err(CliError::CouldNotInferSettingType(path.to_path_buf()));
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
    Ok(setting_type)
}

/// Today's date, in the `YYYY-MM-DD` form that an export records.
fn today() -> String {
    let days = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_secs() / 86_400)
            .unwrap_or(0),
    )
    .unwrap_or(0);
    // Days since the epoch to a date, shifting the year to start in March so that the leap day
    // falls at the end of it.
    let era_days = days + 719_468;
    let era = era_days.div_euclid(146_097);
    let day_of_era = era_days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Lists the playlists of a library, or writes an export holding the named ones.
fn library(
    path: &Path,
    list: bool,
    playlist_names: &[String],
    output: Option<&Path>,
) -> CliResult<()> {
    let library = Library::open(path)?;

    if list {
        print_playlists(&library, 0, 0);
        return Ok(());
    }

    let Some(output) = output else {
        return Err(CliError::MissingArgument(
            "an output directory, or --list to see the playlists",
        ));
    };
    if playlist_names.is_empty() {
        return Err(CliError::MissingArgument("at least one --playlist"));
    }

    // Resolve the names first, so that nothing is written if one of them is wrong.
    let mut selected = Vec::new();
    for name in playlist_names {
        let playlist = library
            .find_playlist(name)
            .ok_or_else(|| CliError::NoSuchPlaylist(name.clone()))?;
        selected.push(playlist.clone());
    }

    // An export that is already there is added to, so that a playlist that has grown does not
    // mean copying everything again.
    let existing = ExportLayout::new(output.to_path_buf())
        .export_pdb()
        .is_file();
    let mut writer = if existing {
        println!("adding to the export already in {}", output.display());
        DeviceExportWriter::open(output)?
    } else {
        DeviceExportWriter::create(output, &today())?
    };
    let layout = writer.layout().clone();
    let contents = layout.root().join("Contents");
    std::fs::create_dir_all(&contents)?;

    let mut missing_audio = 0usize;
    let mut without_analysis = 0usize;
    let mut added = 0usize;
    let mut already_there = 0usize;
    let mut replaced = 0usize;
    for playlist in &selected {
        let tracks = library.playlist_tracks(playlist.id);
        // The folders the playlist sits in are put on the device as well, so that it is found
        // where it is in the library.
        let parent = create_folders_of(&mut writer, &library, playlist)?;
        let playlist_id = match writer.playlist_by_name(parent, &playlist.name) {
            Some(playlist_id) => playlist_id,
            None => writer.create_playlist(parent, &playlist.name)?,
        };
        println!("{}: {} tracks", playlist.name, tracks.len());

        for track in tracks {
            let source = Path::new(&track.file_path);
            if !source.is_file() {
                missing_audio += 1;
                eprintln!("  warning: no audio file at {}", track.file_path);
                continue;
            }

            let destination = contents.join(&track.filename);
            // A file that is already on the device is left alone, which is what makes adding to
            // an export cheap.
            let was_there = destination.is_file();
            if copy_unless_present(source, &destination)? && was_there {
                replaced += 1;
                eprintln!(
                    "  warning: copied {} again, as the file on the device had another size",
                    track.filename
                );
            }

            // The path the track has on the device, which is also what decides where its
            // analysis files belong. The name is taken from the device rather than from the
            // library, as a file system does not always store a name as it was given.
            let device_path = format!("/Contents/{}", stored_name(&destination)?);

            // The artwork keeps the place it has in the library, so that the path recorded for
            // it is the one the player finds it at.
            let artwork_path = copy_artwork(&library, track, &layout)?;

            let analysis = library.analysis_files(track);
            if analysis.is_empty() {
                without_analysis += 1;
            }
            // The paths of two tracks can lead to the same directory, so the name within it is
            // chosen rather than assumed.
            let analysis_dir = layout.anlz_dir_for_audio(&device_path);
            let stem = choose_analysis_stem(&analysis_dir, &device_path)?;
            for file in analysis {
                let extension = file
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .unwrap_or("DAT");
                let destination = analysis_dir.join(format!("{stem}.{extension}"));
                copy_analysis_file(&file, &destination, &device_path)?;
            }

            let track_id = writer.add_track(&DeviceTrack {
                file_path: device_path.clone(),
                title: track.title.clone(),
                artist: track.artist.clone(),
                album: track.album.clone(),
                genre: track.genre.clone(),
                label: track.label.clone(),
                comment: track.comment.clone(),
                date_added: track.date_added.clone(),
                analyze_date: track.analyze_date.clone(),
                artwork_path,
                analyze_path: analysis_dir
                    .join(format!("{stem}.DAT"))
                    .strip_prefix(layout.root())
                    .map(|path| format!("/{}", path.display()))
                    .unwrap_or_default(),
                tempo: track.tempo_bpm(),
                duration: track.duration,
                bitrate: track.bitrate,
                sample_rate: track.sample_rate,
                sample_depth: track.sample_depth,
                file_size: track.file_size,
                track_number: track.track_number,
                disc_number: track.disc_number,
                year: track.year,
                rating: track.rating,
                color: track.color.clone(),
                file_type: track.file_type.clone(),
                ..Default::default()
            })?;
            if writer.add_track_to_playlist(playlist_id, track_id)? {
                added += 1;
            } else {
                already_there += 1;
            }
        }
    }
    writer.close()?;

    println!("wrote an export to {}", output.display());
    println!("{added} tracks added, {already_there} already on the device");
    if replaced > 0 {
        println!("{replaced} audio files were copied again, as the ones on the device differed");
    }
    if missing_audio > 0 {
        println!("{missing_audio} tracks were left out, as their audio file was not there");
    }
    if without_analysis > 0 {
        println!("{without_analysis} tracks have no analysis files in the library");
    }
    Ok(())
}

/// The name that the device holds a file under, which is not always the name it was given.
///
/// A name holding a character outside ASCII can be stored in more than one form, and a file
/// system is free to pick one: macOS writes the decomposed form to a FAT file system, so a file
/// given a name holding "ö" is stored under a name holding "o" followed by a combining mark
/// instead. Both forms find the file again from the machine that wrote it, but a player compares
/// the name in the directory against the path the database holds, and those have to be the same.
///
/// The name is found by looking for the file the path leads to in the directory holding it, so
/// that whatever the file system stored is what the export records.
fn stored_name(path: &Path) -> CliResult<String> {
    let given = || {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned()
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let Some(parent) = path.parent() else {
            return Ok(given());
        };
        let inode = path.metadata()?.ino();
        for entry in std::fs::read_dir(parent)? {
            let entry = entry?;
            if entry.metadata().is_ok_and(|data| data.ino() == inode) {
                if let Some(name) = entry.file_name().to_str() {
                    return Ok(name.to_owned());
                }
            }
        }
    }
    Ok(given())
}

/// Copies a file to a device unless the device already holds it, and says whether it copied.
///
/// A file of the same size is taken to be the same file. Nothing in a library holds a checksum of
/// an audio file, so size is as far as this can go, but it does catch the case that matters: a
/// file left half written by a copy that was interrupted is copied again rather than left as it
/// is.
fn copy_unless_present(source: &Path, destination: &Path) -> CliResult<bool> {
    if let (Ok(source_data), Ok(destination_data)) = (source.metadata(), destination.metadata()) {
        if source_data.len() == destination_data.len() {
            return Ok(false);
        }
    }
    if let Some(dir) = destination.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::copy(source, destination)?;
    Ok(true)
}

/// Puts the folders that hold a playlist in the library on the device, and returns the one the
/// playlist belongs in.
///
/// A folder that is already on the device is used as it is, so that two playlists from the same
/// folder end up beside each other rather than in a folder each.
fn create_folders_of(
    writer: &mut DeviceExportWriter,
    library: &Library,
    playlist: &rekordcrate::edb::LibraryPlaylist,
) -> CliResult<Option<rekordcrate::pdb::PlaylistTreeNodeId>> {
    // Walk up to the root of the tree, which gives the folders from the innermost outwards.
    let mut names = Vec::new();
    let mut parent_id = playlist.parent_id;
    while parent_id != 0 {
        let Some(folder) = library.playlist_by_id(parent_id) else {
            break;
        };
        names.push(folder.name.clone());
        parent_id = folder.parent_id;
    }

    let mut parent = None;
    for name in names.into_iter().rev() {
        parent = Some(match writer.playlist_by_name(parent, &name) {
            Some(folder) => folder,
            None => writer.create_playlist_folder(parent, &name)?,
        });
    }
    Ok(parent)
}

/// Copies the artwork of a track to a device, and returns the path it has there.
///
/// Players look for the artwork of a track at the path the database records, alongside which they
/// expect the larger version that Rekordbox writes beside it, so both are copied to the place
/// below `PIONEER/Artwork` that they have in the library.
fn copy_artwork(
    library: &Library,
    track: &rekordcrate::edb::LibraryTrack,
    layout: &ExportLayout,
) -> CliResult<String> {
    let Some(source) = library.file_below_pioneer(&track.artwork_path) else {
        return Ok(String::new());
    };
    let Some(below_artwork) = track.artwork_path.split_once("PIONEER/Artwork/") else {
        return Ok(String::new());
    };
    let destination = layout.artwork_dir().join(below_artwork.1);
    copy_unless_present(&source, &destination)?;

    // Rekordbox keeps further versions of the same image beside it, which players look for by
    // name rather than through the database.
    for suffix in ["_m", "_s"] {
        let beside = |path: &Path| -> Option<PathBuf> {
            let stem = path.file_stem()?.to_str()?;
            let extension = path.extension()?.to_str()?;
            Some(path.with_file_name(format!("{stem}{suffix}.{extension}")))
        };
        if let (Some(from), Some(to)) = (beside(&source), beside(&destination)) {
            if from.is_file() {
                copy_unless_present(&from, &to)?;
            }
        }
    }

    Ok(format!("/PIONEER/Artwork/{}", below_artwork.1))
}

/// The name to give the analysis files of a track in the directory they belong in.
///
/// The directory follows from the path of the audio file, and the paths of two tracks can lead to
/// the same one, which Rekordbox handles by counting the name up. A name already holding the
/// analysis of this track is used again, so that writing the same track twice does not leave a
/// second copy of it.
fn choose_analysis_stem(dir: &Path, audio_path: &str) -> CliResult<String> {
    for number in 0..10_000 {
        let stem = format!("ANLZ{number:04}");
        let candidate = dir.join(format!("{stem}.DAT"));
        if !candidate.is_file() {
            return Ok(stem);
        }
        // A file naming this track is the analysis of this track.
        if analysis_names(&candidate)?.as_deref() == Some(audio_path) {
            return Ok(stem);
        }
    }
    Err(CliError::MissingArgument(
        "a free name for the analysis files, as the directory is full",
    ))
}

/// The audio file that an analysis file names, if it names one.
fn analysis_names(path: &Path) -> CliResult<Option<String>> {
    let data = std::fs::read(path)?;
    let Ok(anlz) = ANLZ::read(&mut std::io::Cursor::new(data.as_slice())) else {
        return Ok(None);
    };
    Ok(anlz
        .sections
        .iter()
        .find_map(|section| match &section.content {
            rekordcrate::anlz::Content::Path(content) => Some(content.path.to_string()),
            _ => None,
        }))
}

/// Copies an analysis file to a device, naming the path the audio file has there.
///
/// An analysis file names the audio file it was made for, which on the device is somewhere else
/// than in the library it came from. A file that cannot be read is copied as it is, so that a
/// kind of analysis this crate does not know is still carried over.
fn copy_analysis_file(source: &Path, destination: &Path, audio_path: &str) -> CliResult<()> {
    let data = std::fs::read(source)?;
    let mut cursor = std::io::Cursor::new(data.as_slice());
    let Ok(mut anlz) = ANLZ::read(&mut cursor) else {
        copy_unless_present(source, destination)?;
        return Ok(());
    };
    if !anlz.set_audio_path(audio_path) {
        copy_unless_present(source, destination)?;
        return Ok(());
    }

    let mut written = std::io::Cursor::new(Vec::new());
    anlz.write(&mut written).map_err(rekordcrate::Error::from)?;
    let written = written.into_inner();
    // The file is rewritten rather than copied, so its size says nothing about the one it came
    // from, and what the device holds is compared against what would be written instead.
    if std::fs::read(destination).is_ok_and(|existing| existing == written) {
        return Ok(());
    }
    if let Some(dir) = destination.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(destination, written)?;
    Ok(())
}

/// Prints the playlists of a library as the tree they form.
fn print_playlists(library: &Library, parent_id: u32, depth: usize) {
    let mut nodes: Vec<_> = library
        .playlists()
        .iter()
        .filter(|node| node.parent_id == parent_id)
        .collect();
    nodes.sort_by_key(|node| (node.sort_order, node.id));

    for node in nodes {
        let indent = "  ".repeat(depth);
        if node.is_folder() {
            println!("{indent}{}/", node.name);
        } else {
            let count = library.playlist_tracks(node.id).len();
            println!("{indent}{} ({count} tracks)", node.name);
        }
        print_playlists(library, node.id, depth + 1);
    }
}

fn run() -> CliResult<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ListPlaylists { path } => list_playlists(path)?,
        Commands::Library {
            path,
            list,
            playlists,
            output,
        } => library(path, *list, playlists, output.as_deref())?,
        Commands::ListSettings { path } => list_settings(path)?,
        Commands::ExportPlaylists { path, output_dir } => export_playlists(path, output_dir)?,
        Commands::DumpPDB {
            path,
            db_type,
            parse_unknown_tables,
            format,
        } => {
            let db_type = guess_db_type(path, db_type.as_deref())?;
            dump_pdb(path, db_type, *parse_unknown_tables, *format)?
        }
        Commands::DumpANLZ { path, format } => dump_anlz(path, *format)?,
        Commands::DumpSetting {
            path,
            setting_type,
            format,
        } => {
            let setting_type = guess_setting_type(path, setting_type.as_deref())?;
            dump_setting(path, setting_type, *format)?
        }
        #[cfg(feature = "xml")]
        Commands::DumpXML { path } => dump_xml(path)?,
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
