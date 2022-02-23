// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use rekordcrate::anlz::ANLZ;

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let data = std::fs::read(&path).expect("failed to read file");
    let (input, anlz) = ANLZ::parse(&data).expect("failed to parse header");
    println!("File Header: {:#?}", anlz);
    for (i, section) in anlz.sections(input).enumerate() {
        println!("#{}: {:#?}", i, section);
    }
}
