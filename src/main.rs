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
use std::path::PathBuf;

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
    },
    /// Parse and dump a Pioneer Settings (`*SETTING.DAT`) file.
    DumpSetting {
        /// File to parse.
        #[arg(value_name = "SETTING_FILE")]
        path: PathBuf,
    },
}

fn list_playlists(path: &PathBuf) {
    use rekordcrate::pdb::PlaylistTreeNode;
    use std::collections::HashMap;

    fn print_children_of(tree: &HashMap<u32, Vec<PlaylistTreeNode>>, id: u32, level: usize) {
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

    let mut reader = std::fs::File::open(&path).expect("failed to open file");
    let header = Header::read(&mut reader).expect("failed to parse pdb file");

    let mut tree: HashMap<u32, Vec<PlaylistTreeNode>> = HashMap::new();

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

    print_children_of(&tree, 0, 0);
}

fn dump_anlz(path: &PathBuf) {
    let mut reader = std::fs::File::open(&path).expect("failed to open file");
    let anlz = ANLZ::read(&mut reader).expect("failed to parse setting file");
    println!("{:#?}", anlz);
}

fn dump_pdb(path: &PathBuf) {
    let mut reader = std::fs::File::open(&path).expect("failed to open file");
    let header = Header::read(&mut reader).expect("failed to parse pdb file");

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
}

fn dump_setting(path: &PathBuf) {
    let mut reader = std::fs::File::open(&path).expect("failed to open file");
    let setting = Setting::read(&mut reader).expect("failed to parse setting file");

    println!("{:#04x?}", setting);
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ListPlaylists { path } => list_playlists(path),
        Commands::DumpPDB { path } => dump_pdb(path),
        Commands::DumpANLZ { path } => dump_anlz(path),
        Commands::DumpSetting { path } => dump_setting(path),
    }
}
