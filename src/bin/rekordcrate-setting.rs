// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::BinRead;
use rekordcrate::setting::Setting;

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let mut reader = std::fs::File::open(&path).expect("failed to open file");
    let setting = Setting::read(&mut reader).expect("failed to parse setting file");

    println!("{:#04x?}", setting);
}
