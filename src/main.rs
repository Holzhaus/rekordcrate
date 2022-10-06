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
use rekordcrate::pdb::Header;
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
        Commands::DumpPDB { path } => dump_pdb(path),
        Commands::DumpANLZ { path } => dump_anlz(path),
        Commands::DumpSetting { path } => dump_setting(path),
    }
}
