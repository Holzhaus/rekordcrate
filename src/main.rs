// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use rekordcrate::pdb::Header;

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let data = std::fs::read(&path).expect("failed to read file");
    let (_, header) = Header::parse(&data).expect("failed to parse header");

    println!("{:#?}", header);
}
