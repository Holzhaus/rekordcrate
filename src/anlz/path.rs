// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Addressing for analysis files inside a device export's `PIONEER/USBANLZ` directory.
//!
//! Rekordbox derives an analysis directory from the audio file's exact device path. Hardware tests
//! confirm that the XDJ-XZ recomputes this directory instead of relying on the `analyze_path`
//! stored in `export.pdb`; whether other player models do the same is not established. Exporters
//! should therefore reproduce the calculated directory and keep the PDB field consistent with it.
//! When both locations agree, the player's lookup strategy is immaterial to compatibility.
//!
//! The path is processed as UTF-16 code units by a custom rolling hash, reduced modulo 200003,
//! then selected bits from that hash form the `P` directory component.
//!
//! The algorithm and test vectors were contributed by AnnoyingTechnology from [interoperability
//! research][research] published in December 2025 and validated against rekordbox exports.
//!
//! [research]: https://github.com/AnnoyingTechnology/rhythmbox-to-pioneer-xdj-exporter/blob/master/posterity/WAVEFORMS.md

const FIRST_MULTIPLIER: u32 = 0x5BC9;
const SECOND_MULTIPLIER: u32 = 0x93B5;
const HASH_MODULUS: u32 = 200_003;

/// Calculate the `(P value, hash value)` used to address an audio file's analysis directory.
///
/// `audio_path` must be the exact device path stored for the track, including its leading slash,
/// for example `/Contents/Artist/Album/Track.flac`. Case, separators, and Unicode are significant;
/// this function deliberately performs no path normalization.
#[must_use]
pub fn hash(audio_path: &str) -> (u16, u32) {
    let mut hash = 0u32;

    for code_unit in audio_path.encode_utf16().map(u32::from) {
        let intermediate = hash.wrapping_mul(FIRST_MULTIPLIER).wrapping_add(code_unit);
        hash = intermediate
            .wrapping_mul(SECOND_MULTIPLIER)
            .wrapping_add(code_unit);
    }

    let hash = hash % HASH_MODULUS;
    let p_value = (hash & 0x01)
        | ((hash >> 1) & 0x02)
        | ((hash >> 4) & 0x04)
        | ((hash >> 4) & 0x08)
        | ((hash >> 5) & 0x10)
        | ((hash >> 8) & 0x20)
        | ((hash >> 10) & 0x40);

    (p_value as u16, hash)
}

/// Calculate the analysis directory relative to `PIONEER/USBANLZ`.
///
/// The returned path always has the form `Pxxx/yyyyyyyy`, using uppercase, zero-padded
/// hexadecimal components.
#[must_use]
pub fn directory(audio_path: &str) -> String {
    let (p_value, hash) = hash(audio_path);
    format!("P{p_value:03X}/{hash:08X}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_rekordbox_exports() {
        let cases = [
            (
                "/Contents/ARTISTTEST1/ALBUMTEST1/TITLETEST1.mp3",
                (0x051, 0x0001_D603),
            ),
            (
                "/Contents/ARTISTTEST2/ALBUMTEST2/TITLETEST2.mp3",
                (0x03C, 0x0000_A6CA),
            ),
            (
                "/Contents/ARTISTTEST3/ALBUMTEST3/TITLETEST3.mp3",
                (0x045, 0x0001_096B),
            ),
        ];

        for (audio_path, expected) in cases {
            assert_eq!(
                hash(audio_path),
                expected,
                "unexpected hash for {audio_path}"
            );
        }
    }

    #[test]
    fn formats_relative_directory() {
        assert_eq!(
            directory("/Contents/ARTISTTEST1/ALBUMTEST1/TITLETEST1.mp3"),
            "P051/0001D603"
        );
    }

    #[test]
    fn hashes_utf16_surrogate_pairs_as_two_code_units() {
        assert_eq!(directory("/Contents/Artist/🚀.flac"), "P074/00012258");
    }
}
