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
use rekordcrate::pdb::{DatabaseType, Header, PageContent, PageType, PlainPageType, PlainRow, Row};
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

    let mut reader = std::fs::File::open(path)?;
    let header = Header::read_args(&mut reader, (DatabaseType::Plain,))?;

    let mut tree: HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>> = HashMap::new();

    header
        .tables
        .iter()
        .filter(|table| {
            matches!(
                table.page_type,
                PageType::Plain(PlainPageType::PlaylistTree)
            )
        })
        .flat_map(|table| {
            header
                .read_pages(
                    &mut reader,
                    binrw::Endian::NATIVE,
                    (&table.first_page, &table.last_page, DatabaseType::Plain),
                )
                .unwrap()
                .into_iter()
                .filter_map(|page| page.content.into_data())
                .flat_map(|data_content| data_content.row_groups.into_iter())
                .flat_map(|row_group| {
                    row_group
                        .iter()
                        .map(|row| {
                            if let Row::Plain(PlainRow::PlaylistTreeNode(playlist_tree)) = row {
                                playlist_tree.clone()
                            } else {
                                unreachable!("encountered non-playlist tree row in playlist table");
                            }
                        })
                        .collect::<Vec<PlaylistTreeNode>>()
                        .into_iter()
                })
        })
        .for_each(|row| tree.entry(row.parent_id).or_default().push(row));

    print_children_of(&tree, PlaylistTreeNodeId(0), 0);

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
                    data_content.row_groups.iter().for_each(|row_group| {
                        println!("    {:?}", row_group);
                        for row in row_group {
                            println!("      {:?}", row);
                        }
                    })
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
