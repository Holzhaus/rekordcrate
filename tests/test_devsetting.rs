// Copyright (c) 2022 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{io::Cursor, BinRead};
use rekordcrate::setting::*;

macro_rules! read_devsetting {
    ($path:literal) => {{
        let data = include_bytes!($path);
        println!("Setting file: {}", $path);
        let mut reader = Cursor::new(data);
        let setting = Setting::read(&mut reader).expect("failed to parse setting file");
        let result = match setting.data {
            SettingData::DevSetting(x) => Some(x),
            _ => None,
        };
        result.expect("failed to match data section")
    }};
}

#[test]
fn read_devsetting_default() {
    let data = read_devsetting!("../data/complete_export/empty/PIONEER/DEVSETTING.DAT");
    assert_eq!(data, DevSetting::default());
}
