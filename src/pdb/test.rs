// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0
use super::ext::*;
use super::*;
use crate::util::testing::{test_roundtrip, test_roundtrip_with_args};
use std::num::NonZero;

#[test]
fn empty_header() {
    let header = Header {
        page_size: 4096,
        next_unused_page: PageIndex(1),
        unknown: 0,
        sequence: 1,
        tables: vec![],
    };
    test_roundtrip(
        &[
            0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
        ],
        header,
    );
}

#[test]
fn demo_tracks_header() {
    let header = Header {
        page_size: 4096,
        next_unused_page: PageIndex(51),
        unknown: 5,
        sequence: 34,
        tables: [
            Table {
                page_type: PageType::Plain(PlainPageType::Tracks),
                empty_candidate: 47,
                first_page: PageIndex(1),
                last_page: PageIndex(2),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Genres),
                empty_candidate: 4,
                first_page: PageIndex(3),
                last_page: PageIndex(3),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Artists),
                empty_candidate: 49,
                first_page: PageIndex(5),
                last_page: PageIndex(6),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Albums),
                empty_candidate: 8,
                first_page: PageIndex(7),
                last_page: PageIndex(7),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Labels),
                empty_candidate: 50,
                first_page: PageIndex(9),
                last_page: PageIndex(10),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Keys),
                empty_candidate: 46,
                first_page: PageIndex(11),
                last_page: PageIndex(12),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Colors),
                empty_candidate: 42,
                first_page: PageIndex(13),
                last_page: PageIndex(14),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::PlaylistTree),
                empty_candidate: 16,
                first_page: PageIndex(15),
                last_page: PageIndex(15),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::PlaylistEntries),
                empty_candidate: 18,
                first_page: PageIndex(17),
                last_page: PageIndex(17),
            },
            Table {
                page_type: PageType::Unknown(9),
                empty_candidate: 20,
                first_page: PageIndex(19),
                last_page: PageIndex(19),
            },
            Table {
                page_type: PageType::Unknown(10),
                empty_candidate: 22,
                first_page: PageIndex(21),
                last_page: PageIndex(21),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::HistoryPlaylists),
                empty_candidate: 24,
                first_page: PageIndex(23),
                last_page: PageIndex(23),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::HistoryEntries),
                empty_candidate: 26,
                first_page: PageIndex(25),
                last_page: PageIndex(25),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Artwork),
                empty_candidate: 28,
                first_page: PageIndex(27),
                last_page: PageIndex(27),
            },
            Table {
                page_type: PageType::Unknown(14),
                empty_candidate: 30,
                first_page: PageIndex(29),
                last_page: PageIndex(29),
            },
            Table {
                page_type: PageType::Unknown(15),
                empty_candidate: 32,
                first_page: PageIndex(31),
                last_page: PageIndex(31),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Columns),
                empty_candidate: 43,
                first_page: PageIndex(33),
                last_page: PageIndex(34),
            },
            Table {
                page_type: PageType::Unknown(17),
                empty_candidate: 44,
                first_page: PageIndex(35),
                last_page: PageIndex(36),
            },
            Table {
                page_type: PageType::Unknown(18),
                empty_candidate: 45,
                first_page: PageIndex(37),
                last_page: PageIndex(38),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::History),
                empty_candidate: 48,
                first_page: PageIndex(39),
                last_page: PageIndex(41),
            },
        ]
        .to_vec(),
    };

    test_roundtrip(
        &[
            0, 0, 0, 0, 0, 16, 0, 0, 20, 0, 0, 0, 51, 0, 0, 0, 5, 0, 0, 0, 34, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 47, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 3, 0, 0, 0, 3,
            0, 0, 0, 2, 0, 0, 0, 49, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 3, 0, 0, 0, 8, 0, 0, 0, 7, 0,
            0, 0, 7, 0, 0, 0, 4, 0, 0, 0, 50, 0, 0, 0, 9, 0, 0, 0, 10, 0, 0, 0, 5, 0, 0, 0, 46, 0,
            0, 0, 11, 0, 0, 0, 12, 0, 0, 0, 6, 0, 0, 0, 42, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 7,
            0, 0, 0, 16, 0, 0, 0, 15, 0, 0, 0, 15, 0, 0, 0, 8, 0, 0, 0, 18, 0, 0, 0, 17, 0, 0, 0,
            17, 0, 0, 0, 9, 0, 0, 0, 20, 0, 0, 0, 19, 0, 0, 0, 19, 0, 0, 0, 10, 0, 0, 0, 22, 0, 0,
            0, 21, 0, 0, 0, 21, 0, 0, 0, 11, 0, 0, 0, 24, 0, 0, 0, 23, 0, 0, 0, 23, 0, 0, 0, 12, 0,
            0, 0, 26, 0, 0, 0, 25, 0, 0, 0, 25, 0, 0, 0, 13, 0, 0, 0, 28, 0, 0, 0, 27, 0, 0, 0, 27,
            0, 0, 0, 14, 0, 0, 0, 30, 0, 0, 0, 29, 0, 0, 0, 29, 0, 0, 0, 15, 0, 0, 0, 32, 0, 0, 0,
            31, 0, 0, 0, 31, 0, 0, 0, 16, 0, 0, 0, 43, 0, 0, 0, 33, 0, 0, 0, 34, 0, 0, 0, 17, 0, 0,
            0, 44, 0, 0, 0, 35, 0, 0, 0, 36, 0, 0, 0, 18, 0, 0, 0, 45, 0, 0, 0, 37, 0, 0, 0, 38, 0,
            0, 0, 19, 0, 0, 0, 48, 0, 0, 0, 39, 0, 0, 0, 41, 0, 0, 0,
        ],
        header,
    );
}

#[test]
fn track_row() {
    let row = Track {
        subtype: Subtype(0x24),
        index_shift: 160,
        bitmask: 788224,
        sample_rate: 44100,
        composer_id: ArtistId(0),
        file_size: 6899624,
        unknown2: 214020570,
        unknown3: 64128,
        unknown4: 1511,
        artwork_id: ArtworkId(0),
        key_id: KeyId(5),
        orig_artist_id: ArtistId(0),
        label_id: LabelId(1),
        remixer_id: ArtistId(0),
        bitrate: 320,
        track_number: 0,
        tempo: 12800,
        genre_id: GenreId(0),
        album_id: AlbumId(0),
        artist_id: ArtistId(1),
        id: TrackId(1),
        disc_number: 0,
        play_count: 0,
        year: 0,
        sample_depth: 16,
        duration: 172,
        unknown5: 41,
        color: ColorIndex::None,
        rating: 0,
        offsets: OffsetArrayContainer {
            offsets: [
                1u16, 3, 136, 137, 138, 140, 142, 143, 144, 145, 148, 149, 150, 161, 162, 163, 164,
                208, 219, 249, 262, 263, 280,
            ]
            .into(),
            inner: TrackStrings {
                isrc: DeviceSQLString::new_isrc("".to_string()).unwrap(),
                unknown_string1: DeviceSQLString::empty(),
                unknown_string2: "3".parse().unwrap(),
                unknown_string3: "3".parse().unwrap(),
                unknown_string4: DeviceSQLString::empty(),
                message: DeviceSQLString::empty(),
                kuvo_public: DeviceSQLString::empty(),
                autoload_hotcues: "ON".parse().unwrap(),
                unknown_string5: DeviceSQLString::empty(),
                unknown_string6: DeviceSQLString::empty(),
                date_added: "2018-05-25".parse().unwrap(),
                release_date: DeviceSQLString::empty(),
                mix_name: DeviceSQLString::empty(),
                unknown_string7: DeviceSQLString::empty(),
                analyze_path: "/PIONEER/USBANLZ/P016/0000875E/ANLZ0000.DAT"
                    .parse()
                    .unwrap(),
                analyze_date: "2022-02-02".parse().unwrap(),
                comment: "Tracks by www.loopmasters.com".parse().unwrap(),
                title: "Demo Track 1".parse().unwrap(),
                unknown_string8: DeviceSQLString::empty(),
                filename: "Demo Track 1.mp3".parse().unwrap(),
                file_path: "/Contents/Loopmasters/UnknownAlbum/Demo Track 1.mp3"
                    .parse()
                    .unwrap(),
            },
        },
        padding: 0.into(),
    };
    test_roundtrip(
        &[
            36, 0, 160, 0, 0, 7, 12, 0, 68, 172, 0, 0, 0, 0, 0, 0, 168, 71, 105, 0, 218, 177, 193,
            12, 128, 250, 231, 5, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 64,
            1, 0, 0, 0, 0, 0, 0, 0, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 16, 0, 172, 0, 41, 0, 0, 0, 1, 0, 3, 0, 136, 0, 137, 0, 138, 0, 140, 0,
            142, 0, 143, 0, 144, 0, 145, 0, 148, 0, 149, 0, 150, 0, 161, 0, 162, 0, 163, 0, 164, 0,
            208, 0, 219, 0, 249, 0, 6, 1, 7, 1, 24, 1, 3, 3, 5, 51, 5, 51, 3, 3, 3, 7, 79, 78, 3,
            3, 23, 50, 48, 49, 56, 45, 48, 53, 45, 50, 53, 3, 3, 3, 89, 47, 80, 73, 79, 78, 69, 69,
            82, 47, 85, 83, 66, 65, 78, 76, 90, 47, 80, 48, 49, 54, 47, 48, 48, 48, 48, 56, 55, 53,
            69, 47, 65, 78, 76, 90, 48, 48, 48, 48, 46, 68, 65, 84, 23, 50, 48, 50, 50, 45, 48, 50,
            45, 48, 50, 61, 84, 114, 97, 99, 107, 115, 32, 98, 121, 32, 119, 119, 119, 46, 108,
            111, 111, 112, 109, 97, 115, 116, 101, 114, 115, 46, 99, 111, 109, 27, 68, 101, 109,
            111, 32, 84, 114, 97, 99, 107, 32, 49, 3, 35, 68, 101, 109, 111, 32, 84, 114, 97, 99,
            107, 32, 49, 46, 109, 112, 51, 105, 47, 67, 111, 110, 116, 101, 110, 116, 115, 47, 76,
            111, 111, 112, 109, 97, 115, 116, 101, 114, 115, 47, 85, 110, 107, 110, 111, 119, 110,
            65, 108, 98, 117, 109, 47, 68, 101, 109, 111, 32, 84, 114, 97, 99, 107, 32, 49, 46,
            109, 112, 51,
        ],
        row,
    );
}

#[test]
fn artist_row() {
    let row = Artist {
        subtype: Subtype(0x60),
        index_shift: 32,
        id: ArtistId(1),
        offsets: OffsetArrayContainer {
            offsets: [3u8, 10u8].into(),
            inner: TrailingName {
                name: "Loopmasters".parse().unwrap(),
            },
        },
        padding: 0.into(),
    };
    test_roundtrip(
        &[
            96, 0, 32, 0, 1, 0, 0, 0, 3, 10, 25, 76, 111, 111, 112, 109, 97, 115, 116, 101, 114,
            115,
        ],
        row,
    );
}

#[test]
fn album_row() {
    let row1 = Album {
        subtype: Subtype(0x80),
        index_shift: 32,
        unknown2: 0,
        artist_id: ArtistId(2),
        id: AlbumId(2),
        unknown3: 0,
        offsets: OffsetArrayContainer {
            offsets: [3u8, 0x16u8].into(),
            inner: TrailingName {
                name: "GOOD LUCK".parse().unwrap(),
            },
        },
        padding: 0.into(),
    };

    test_roundtrip(
        &[
            0x80, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x16, 0x15, 0x47, 0x4f, 0x4f, 0x44, 0x20,
            0x4c, 0x55, 0x43, 0x4b,
        ],
        row1,
    );
    let row2 = Album {
        subtype: Subtype(0x80),
        index_shift: 64,
        unknown2: 0,
        artist_id: ArtistId(0),
        id: AlbumId(3),
        unknown3: 0,
        offsets: OffsetArrayContainer {
            offsets: [3u8, 0x16u8].into(),
            inner: TrailingName {
                name: "Techno Rave 2023".parse().unwrap(),
            },
        },
        padding: 0.into(),
    };

    test_roundtrip(
        &[
            0x80, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x16, 0x23, 0x54, 0x65, 0x63, 0x68, 0x6e,
            0x6f, 0x20, 0x52, 0x61, 0x76, 0x65, 0x20, 0x32, 0x30, 0x32, 0x33,
        ],
        row2,
    );
}

#[test]
fn label_row() {
    let row = Label {
        id: LabelId(1),
        name: "Loopmasters".parse().unwrap(),
    };
    test_roundtrip(
        &[
            1, 0, 0, 0, 25, 76, 111, 111, 112, 109, 97, 115, 116, 101, 114, 115,
        ],
        row,
    );
}

#[test]
fn key_row() {
    let row = Key {
        id: KeyId(1),
        id2: 1,
        name: "Dm".parse().unwrap(),
    };
    test_roundtrip(&[1, 0, 0, 0, 1, 0, 0, 0, 7, 68, 109], row);
}

#[test]
fn color_row() {
    let row = Color {
        unknown1: 0,
        unknown2: 1,
        color: ColorIndex::Pink,
        unknown3: 0,
        name: "Pink".parse().unwrap(),
    };
    test_roundtrip(&[0, 0, 0, 0, 1, 1, 0, 0, 11, 80, 105, 110, 107], row);
}

#[test]
fn playlist_tree_row() {
    let row = PlaylistTreeNode {
        parent_id: PlaylistTreeNodeId(0),
        unknown: 0,
        sort_order: 0,
        id: PlaylistTreeNodeId(1),
        node_is_folder: 1,
        name: "current set 2021 reduced".parse().unwrap(),
    };

    test_roundtrip(
        &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0x33, 0x63, 0x75, 0x72,
            0x72, 0x65, 0x6e, 0x74, 0x20, 0x73, 0x65, 0x74, 0x20, 0x32, 0x30, 0x32, 0x31, 0x20,
            0x72, 0x65, 0x64, 0x75, 0x63, 0x65, 0x64,
        ],
        row,
    );
}

#[test]
fn playlist_entry_row() {
    let row = PlaylistEntry {
        entry_index: 1,
        track_id: TrackId(1),
        playlist_id: PlaylistTreeNodeId(6),
    };

    test_roundtrip(&[1, 0, 0, 0, 1, 0, 0, 0, 6, 0, 0, 0], row);
}

#[test]
fn column_entry() {
    let row = ColumnEntry {
        id: 1,
        unknown0: 128,
        column_name: "\u{fffa}GENRE\u{fffb}".parse().unwrap(),
    };
    let bin = &[
        0x01, 0x00, 0x80, 0x00, 0x90, 0x12, 0x00, 0x00, 0xfa, 0xff, 0x47, 0x00, 0x45, 0x00, 0x4e,
        0x00, 0x52, 0x00, 0x45, 0x00, 0xfb, 0xff,
    ];
    test_roundtrip(bin, row);
}

#[test]
fn track_page() {
    let mut row_groups = RowGroup {
        row_offsets: Default::default(),
        row_presence_flags: 0,
        unknown: 16,
        rows: vec![],
    };
    row_groups
        .add_row(Row::Plain(PlainRow::Track(Track {
            subtype: Subtype(0x24),
            index_shift: 0,
            bitmask: 788224,
            sample_rate: 44100,
            composer_id: ArtistId(0),
            file_size: 1382226,
            unknown2: 136659598,
            unknown3: 58465,
            unknown4: 11106,
            artwork_id: ArtworkId(0),
            key_id: KeyId(0),
            orig_artist_id: ArtistId(0),
            label_id: LabelId(0),
            remixer_id: ArtistId(0),
            bitrate: 2116,
            track_number: 0,
            tempo: 0,
            genre_id: GenreId(0),
            album_id: AlbumId(0),
            artist_id: ArtistId(0),
            id: TrackId(1),
            disc_number: 0,
            play_count: 0,
            year: 0,
            sample_depth: 24,
            duration: 5,
            unknown5: 41,
            color: ColorIndex::None,
            rating: 0,
            offsets: OffsetArrayContainer {
                offsets: [
                    0x0bu16, 0x03, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97,
                    0x98, 0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xde, 0xe4, 0xe5, 0xef,
                ]
                .into(),
                inner: TrackStrings {
                    isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                    unknown_string1: "".parse().unwrap(),
                    unknown_string2: "1".parse().unwrap(),
                    unknown_string3: "1".parse().unwrap(),
                    unknown_string4: "".parse().unwrap(),
                    message: "".parse().unwrap(),
                    kuvo_public: "ON".parse().unwrap(),
                    autoload_hotcues: "ON".parse().unwrap(),
                    unknown_string5: "".parse().unwrap(),
                    unknown_string6: "".parse().unwrap(),
                    date_added: "2015-09-07".parse().unwrap(),
                    release_date: "".parse().unwrap(),
                    mix_name: "".parse().unwrap(),
                    unknown_string7: "".parse().unwrap(),
                    analyze_path: "/PIONEER/USBANLZ/P019/00020AA9/ANLZ0000.DAT"
                        .parse()
                        .unwrap(),
                    analyze_date: "2025-09-01".parse().unwrap(),
                    comment: "".parse().unwrap(),
                    title: "NOISE".parse().unwrap(),
                    unknown_string8: "".parse().unwrap(),
                    filename: "NOISE.wav".parse().unwrap(),
                    file_path: "/Contents/UnknownArtist/UnknownAlbum/NOISE.wav"
                        .parse()
                        .unwrap(),
                },
            },
            padding: 0x32.into(),
        })))
        .unwrap();
    row_groups
        .add_row(Row::Plain(PlainRow::Track(Track {
            subtype: Subtype(0x24),
            index_shift: 32,
            bitmask: 788224,
            sample_rate: 44100,
            composer_id: ArtistId(0),
            file_size: 1515258,
            unknown2: 193378756,
            unknown3: 58465,
            unknown4: 11106,
            artwork_id: ArtworkId(0),
            key_id: KeyId(0),
            orig_artist_id: ArtistId(0),
            label_id: LabelId(0),
            remixer_id: ArtistId(0),
            bitrate: 2116,
            track_number: 0,
            tempo: 0,
            genre_id: GenreId(0),
            album_id: AlbumId(0),
            artist_id: ArtistId(0),
            id: TrackId(2),
            disc_number: 0,
            play_count: 0,
            year: 0,
            sample_depth: 24,
            duration: 5,
            unknown5: 41,
            color: ColorIndex::None,
            rating: 0,
            offsets: OffsetArrayContainer {
                offsets: [
                    0x0bu16, 0x03, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97,
                    0x98, 0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xde, 0xe7, 0xe8, 0xf5,
                ]
                .into(),
                inner: TrackStrings {
                    isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                    unknown_string1: "".parse().unwrap(),
                    unknown_string2: "1".parse().unwrap(),
                    unknown_string3: "1".parse().unwrap(),
                    unknown_string4: "".parse().unwrap(),
                    message: "".parse().unwrap(),
                    kuvo_public: "ON".parse().unwrap(),
                    autoload_hotcues: "ON".parse().unwrap(),
                    unknown_string5: "".parse().unwrap(),
                    unknown_string6: "".parse().unwrap(),
                    date_added: "2015-09-07".parse().unwrap(),
                    release_date: "".parse().unwrap(),
                    mix_name: "".parse().unwrap(),
                    unknown_string7: "".parse().unwrap(),
                    analyze_path: "/PIONEER/USBANLZ/P043/00011517/ANLZ0000.DAT"
                        .parse()
                        .unwrap(),
                    analyze_date: "2025-09-01".parse().unwrap(),
                    comment: "".parse().unwrap(),
                    title: "SINEWAVE".parse().unwrap(),
                    unknown_string8: "".parse().unwrap(),
                    filename: "SINEWAVE.wav".parse().unwrap(),
                    file_path: "/Contents/UnknownArtist/UnknownAlbum/SINEWAVE.wav"
                        .parse()
                        .unwrap(),
                },
            },
            padding: 0x35.into(),
        })))
        .unwrap();
    row_groups
        .add_row(Row::Plain(PlainRow::Track(Track {
            subtype: Subtype(0x24),
            index_shift: 64,
            bitmask: 788224,
            sample_rate: 44100,
            composer_id: ArtistId(0),
            file_size: 1941204,
            unknown2: 172751635,
            unknown3: 58465,
            unknown4: 11106,
            artwork_id: ArtworkId(0),
            key_id: KeyId(0),
            orig_artist_id: ArtistId(0),
            label_id: LabelId(0),
            remixer_id: ArtistId(0),
            bitrate: 2116,
            track_number: 0,
            tempo: 0,
            genre_id: GenreId(0),
            album_id: AlbumId(0),
            artist_id: ArtistId(0),
            id: TrackId(3),
            disc_number: 0,
            play_count: 0,
            year: 0,
            sample_depth: 24,
            duration: 7,
            unknown5: 41,
            color: ColorIndex::None,
            rating: 0,
            offsets: OffsetArrayContainer {
                offsets: [
                    0x0bu16, 0x03, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97,
                    0x98, 0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xde, 0xe4, 0xe5, 0xef,
                ]
                .into(),
                inner: TrackStrings {
                    isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                    unknown_string1: "".parse().unwrap(),
                    unknown_string2: "1".parse().unwrap(),
                    unknown_string3: "1".parse().unwrap(),
                    unknown_string4: "".parse().unwrap(),
                    message: "".parse().unwrap(),
                    kuvo_public: "ON".parse().unwrap(),
                    autoload_hotcues: "ON".parse().unwrap(),
                    unknown_string5: "".parse().unwrap(),
                    unknown_string6: "".parse().unwrap(),
                    date_added: "2015-09-07".parse().unwrap(),
                    release_date: "".parse().unwrap(),
                    mix_name: "".parse().unwrap(),
                    unknown_string7: "".parse().unwrap(),
                    analyze_path: "/PIONEER/USBANLZ/P017/00009B77/ANLZ0000.DAT"
                        .parse()
                        .unwrap(),
                    analyze_date: "2025-09-01".parse().unwrap(),
                    comment: "".parse().unwrap(),
                    title: "SIREN".parse().unwrap(),
                    unknown_string8: "".parse().unwrap(),
                    filename: "SIREN.wav".parse().unwrap(),
                    file_path: "/Contents/UnknownArtist/UnknownAlbum/SIREN.wav"
                        .parse()
                        .unwrap(),
                },
            },
            padding: 0x32.into(),
        })))
        .unwrap();
    row_groups
        .add_row(Row::Plain(PlainRow::Track(Track {
            subtype: Subtype(0x24),
            index_shift: 96,
            bitmask: 788224,
            sample_rate: 44100,
            composer_id: ArtistId(0),
            file_size: 2010816,
            unknown2: 60350957,
            unknown3: 58465,
            unknown4: 11106,
            artwork_id: ArtworkId(0),
            key_id: KeyId(0),
            orig_artist_id: ArtistId(0),
            label_id: LabelId(0),
            remixer_id: ArtistId(0),
            bitrate: 2116,
            track_number: 0,
            tempo: 0,
            genre_id: GenreId(0),
            album_id: AlbumId(0),
            artist_id: ArtistId(0),
            id: TrackId(4),
            disc_number: 0,
            play_count: 0,
            year: 0,
            sample_depth: 24,
            duration: 7,
            unknown5: 41,
            color: ColorIndex::None,
            rating: 0,
            offsets: OffsetArrayContainer {
                offsets: [
                    0x0bu16, 0x03, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97,
                    0x98, 0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xde, 0xe3, 0xe4, 0xed,
                ]
                .into(),
                inner: TrackStrings {
                    isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                    unknown_string1: "".parse().unwrap(),
                    unknown_string2: "1".parse().unwrap(),
                    unknown_string3: "1".parse().unwrap(),
                    unknown_string4: "".parse().unwrap(),
                    message: "".parse().unwrap(),
                    kuvo_public: "ON".parse().unwrap(),
                    autoload_hotcues: "ON".parse().unwrap(),
                    unknown_string5: "".parse().unwrap(),
                    unknown_string6: "".parse().unwrap(),
                    date_added: "2015-09-07".parse().unwrap(),
                    release_date: "".parse().unwrap(),
                    mix_name: "".parse().unwrap(),
                    unknown_string7: "".parse().unwrap(),
                    analyze_path: "/PIONEER/USBANLZ/P021/00006D2B/ANLZ0000.DAT"
                        .parse()
                        .unwrap(),
                    analyze_date: "2025-09-01".parse().unwrap(),
                    comment: "".parse().unwrap(),
                    title: "HORN".parse().unwrap(),
                    unknown_string8: "".parse().unwrap(),
                    filename: "HORN.wav".parse().unwrap(),
                    file_path: "/Contents/UnknownArtist/UnknownAlbum/HORN.wav"
                        .parse()
                        .unwrap(),
                },
            },
            padding: 0x35.into(),
        })))
        .unwrap();
    row_groups
        .add_row(Row::Plain(PlainRow::Track(Track {
            subtype: Subtype(0x24),
            index_shift: 128,
            bitmask: 788224,
            sample_rate: 44100,
            composer_id: ArtistId(0),
            file_size: 6899624,
            unknown2: 93475505,
            unknown3: 58465,
            unknown4: 11106,
            artwork_id: ArtworkId(0),
            key_id: KeyId(0),
            orig_artist_id: ArtistId(0),
            label_id: LabelId(1),
            remixer_id: ArtistId(0),
            bitrate: 320,
            track_number: 0,
            tempo: 12800,
            genre_id: GenreId(0),
            album_id: AlbumId(0),
            artist_id: ArtistId(1),
            id: TrackId(5),
            disc_number: 0,
            play_count: 0,
            year: 0,
            sample_depth: 16,
            duration: 172,
            unknown5: 41,
            color: ColorIndex::None,
            rating: 0,
            offsets: OffsetArrayContainer {
                offsets: [
                    0x01u16, 0x03, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97,
                    0x98, 0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xfb, 0x108, 0x109, 0x11a,
                ]
                .into(),
                inner: TrackStrings {
                    isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                    unknown_string1: "".parse().unwrap(),
                    unknown_string2: "1".parse().unwrap(),
                    unknown_string3: "1".parse().unwrap(),
                    unknown_string4: "".parse().unwrap(),
                    message: "".parse().unwrap(),
                    kuvo_public: "ON".parse().unwrap(),
                    autoload_hotcues: "ON".parse().unwrap(),
                    unknown_string5: "".parse().unwrap(),
                    unknown_string6: "".parse().unwrap(),
                    date_added: "2018-05-25".parse().unwrap(),
                    release_date: "".parse().unwrap(),
                    mix_name: "".parse().unwrap(),
                    unknown_string7: "".parse().unwrap(),
                    analyze_path: "/PIONEER/USBANLZ/P016/0000875E/ANLZ0000.DAT"
                        .parse()
                        .unwrap(),
                    analyze_date: "2025-09-01".parse().unwrap(),
                    comment: "Tracks by www.loopmasters.com".parse().unwrap(),
                    title: "Demo Track 1".parse().unwrap(),
                    unknown_string8: "".parse().unwrap(),
                    filename: "Demo Track 1.mp3".parse().unwrap(),
                    file_path: "/Contents/Loopmasters/UnknownAlbum/Demo Track 1.mp3"
                        .parse()
                        .unwrap(),
                },
            },
            padding: 0.into(), // TODO
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(2),
        page_type: PageType::Plain(PlainPageType::Tracks),
        next_page: PageIndex(46),
        unknown1: 12,
        unknown2: 0,
        num_rows_small: 5,
        unknown3: 160,
        unknown4: 0,
        page_flags: PageFlags(36),
        free_size: 2302,
        used_size: 1740,
        unknown5: 1,
        num_rows_large: 4,
        unknown6: 0,
        unknown7: 0,
        row_groups: vec![row_groups],
    };

    let page_size: u32 = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/track_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn genres_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0, // This is different from the usual
            rows: vec![],
        },
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 2, // This is different from the usual
            rows: vec![],
        },
    ];

    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(168),
            name: "#techno #deep #beatdown".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(169),
            name: "#broken #deep".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(170),
            name: "#deep #techno #beatdown".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(171),
            name: "#stepping #deep".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(172),
            name: "#deep #beatdown ".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(173),
            name: "#beatdown #stepping #deep".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(174),
            name: "#techno #dub #beatdown".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(175),
            name: "#techno #dub #deep".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(176),
            name: "#beatdown #oldschool".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(177),
            name: "#techno #beatin #deep".parse().unwrap(), // codespell:ignore beatin
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(178),
            name: "#beatdown #house #DEEP".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(179),
            name: "Minimal / Deep Tech".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(180),
            name: "#sunth #techno".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(181),
            name: "#classic #beatdown   ".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(182),
            name: "#beatdown)".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(183),
            name: "#house #oldschool".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(184),
            name: "#beatdown #synth ".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Genre(Genre {
            id: GenreId(185),
            name: "#techno #deep #intro".parse().unwrap(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(48),
        page_type: PageType::Plain(PlainPageType::Genres),
        next_page: PageIndex(449),
        unknown1: 14405,
        unknown2: 0,
        num_rows_small: 18,
        unknown3: 64,
        unknown4: 2,
        page_flags: PageFlags(36),
        free_size: 3560,
        used_size: 452,
        unknown5: 1,
        num_rows_large: 17,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/genres_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn artists_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            unknown: 0,
            row_presence_flags: 0,
            rows: vec![],
        },
        RowGroup {
            row_offsets: Default::default(),
            unknown: 0,
            row_presence_flags: 0,
            rows: vec![],
        },
        RowGroup {
            row_offsets: Default::default(),
            unknown: 0,
            row_presence_flags: 0,
            rows: vec![],
        },
        RowGroup {
            row_offsets: Default::default(),
            unknown: 0,
            row_presence_flags: 0,
            rows: vec![],
        },
        RowGroup {
            row_offsets: Default::default(),
            unknown: 0,
            row_presence_flags: 0,
            rows: vec![],
        },
        RowGroup {
            row_offsets: Default::default(),
            unknown: 0,
            row_presence_flags: 0,
            rows: vec![],
        },
        RowGroup {
            row_offsets: Default::default(),
            unknown: 0,
            row_presence_flags: 0,
            rows: vec![],
        },
        RowGroup {
            row_offsets: Default::default(),
            unknown: 128,
            row_presence_flags: 0,
            rows: vec![],
        },
    ];

    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 0,
            id: ArtistId(1),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Andreas Gehm".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 32,
            id: ArtistId(2),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "D'marc Cantu".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 64,
            id: ArtistId(3),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "DJ Plant Texture".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 96,
            id: ArtistId(4),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "DVS1".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 128,
            id: ArtistId(5),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Florian Kupfer".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 160,
            id: ArtistId(6),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Frak".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 192,
            id: ArtistId(7),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Frankie Knuckles".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 224,
            id: ArtistId(8),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "House Of Jezebel".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 256,
            id: ArtistId(9),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Innerspace Halflife".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 288,
            id: ArtistId(10),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "James T. Cotton".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 320,
            id: ArtistId(11),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "jozef k".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 352,
            id: ArtistId(12),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Juanpablo".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 384,
            id: ArtistId(13),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Juniper".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 416,
            id: ArtistId(14),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Kovyazin D".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 448,
            id: ArtistId(15),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Le Melange Inc. Ft China".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 480,
            id: ArtistId(16),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Louis Guilliaume".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();

    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 512,
            id: ArtistId(17),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Maxwell Church".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 544,
            id: ArtistId(18),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Various Artists".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 576,
            id: ArtistId(19),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Mutant Beat Dance".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 608,
            id: ArtistId(20),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Mutant beat dance".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 640,
            id: ArtistId(21),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Ron Trent".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 672,
            id: ArtistId(22),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Salvation REMIX".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 704,
            id: ArtistId(23),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Salvation".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 736,
            id: ArtistId(24),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Simoncino".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 768,
            id: ArtistId(25),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "HOTMIX RECORDS / NICK ANTHONY SIMONCINO".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 800,
            id: ArtistId(26),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Sneaker REMIX".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 832,
            id: ArtistId(27),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Tinman REMIX".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 864,
            id: ArtistId(28),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Alienata".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 896,
            id: ArtistId(29),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "AS1".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 928,
            id: ArtistId(30),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "DJ Hell".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 960,
            id: ArtistId(31),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Innershades & Robert D".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 992,
            id: ArtistId(32),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "intersterllar funk".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();

    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1024,
            id: ArtistId(33),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Kyle Hall, KMFH".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1056,
            id: ArtistId(34),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Luke's Anger".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1088,
            id: ArtistId(35),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 12u8].into(),
                inner: TrailingName {
                    name: "Manie Sans Délire".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1120,
            id: ArtistId(36),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Paul du Lac".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1152,
            id: ArtistId(37),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Ron Hardy".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1184,
            id: ArtistId(38),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Saturn V".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1216,
            id: ArtistId(39),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "VA".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1248,
            id: ArtistId(40),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "traxx   ".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1280,
            id: ArtistId(41),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "traxx feat Naughty wood".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1312,
            id: ArtistId(42),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Truncate ".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1344,
            id: ArtistId(43),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Ultrastation".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1376,
            id: ArtistId(44),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "2AM/FM".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1408,
            id: ArtistId(45),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Sepehr".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1440,
            id: ArtistId(46),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Cfade".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1472,
            id: ArtistId(47),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Miss Kittin & The Hacker".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1504,
            id: ArtistId(48),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Paul Du Lac".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();

    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1536,
            id: ArtistId(49),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Tyree Cooper".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1568,
            id: ArtistId(50),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Elbee Bad".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1600,
            id: ArtistId(51),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "The Prince of Dance".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1632,
            id: ArtistId(52),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Body Beat Ritual".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1664,
            id: ArtistId(53),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Nehuen".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1696,
            id: ArtistId(54),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "TRAXX Saturn V & X2".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1728,
            id: ArtistId(55),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Broken English Club".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1760,
            id: ArtistId(56),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "terrace".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1792,
            id: ArtistId(57),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Byron The Aquarius".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1824,
            id: ArtistId(58),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Konstantin Tschechow".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1856,
            id: ArtistId(59),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Romansoff".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1888,
            id: ArtistId(60),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "D'Marc Cantu".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1920,
            id: ArtistId(61),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "SvengalisGhost".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1952,
            id: ArtistId(62),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "X2".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 1984,
            id: ArtistId(63),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Cardopusher".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2016,
            id: ArtistId(64),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Steven Julien".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();

    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2048,
            id: ArtistId(65),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Advent".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2080,
            id: ArtistId(66),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Aleksi Perala".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2112,
            id: ArtistId(67),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Andre Kronert".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2144,
            id: ArtistId(68),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Andy Stott".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2176,
            id: ArtistId(69),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "ANOPOLIS".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2208,
            id: ArtistId(70),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Anthony Rother".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2240,
            id: ArtistId(71),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Anthony Rother UNRELEASED".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2272,
            id: ArtistId(72),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Area".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2304,
            id: ArtistId(73),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Aubrey".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2336,
            id: ArtistId(74),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Audion".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2368,
            id: ArtistId(75),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Audion - Black Strobe".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2400,
            id: ArtistId(76),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Cari Lekebusch & Jesper Dahlback".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2432,
            id: ArtistId(77),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Claro Intelecto".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2464,
            id: ArtistId(78),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Conforce".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2496,
            id: ArtistId(79),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "CT Trax".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2528,
            id: ArtistId(80),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "D-56m".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2560,
            id: ArtistId(81),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Deniro".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2592,
            id: ArtistId(82),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "DJ QU".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2624,
            id: ArtistId(83),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "DJ Qu REMIX".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2656,
            id: ArtistId(84),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Don williams remix".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2688,
            id: ArtistId(85),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Don Williams".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2720,
            id: ArtistId(86),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Dustmite".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2752,
            id: ArtistId(87),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "DVS1 ".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2784,
            id: ArtistId(88),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "DVS1 tesT".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2816,
            id: ArtistId(89),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Emmanuel Top".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2848,
            id: ArtistId(90),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Erika".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2880,
            id: ArtistId(91),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Jensen Interceptor REMIX ".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2912,
            id: ArtistId(92),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Jeroen Search".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2944,
            id: ArtistId(93),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Juho Kahilainen".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 2976,
            id: ArtistId(94),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Juxta Position".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3008,
            id: ArtistId(95),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Kenny Larkin".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3040,
            id: ArtistId(96),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Kirill Mamin".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3072,
            id: ArtistId(97),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "L.B. Dub Corp".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3104,
            id: ArtistId(98),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Levon Vincent".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3136,
            id: ArtistId(99),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "LEVON VINCENT".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3168,
            id: ArtistId(100),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Lil Tony".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3200,
            id: ArtistId(101),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Malin Genie".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3232,
            id: ArtistId(102),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Marcel Dettmann".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3264,
            id: ArtistId(103),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Marco Bernardi".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3296,
            id: ArtistId(104),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Mary Velo".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3328,
            id: ArtistId(105),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Mike Dearborn".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3360,
            id: ArtistId(106),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Mike Dunn JU EDIT".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3392,
            id: ArtistId(107),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Nina Kraviz".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3424,
            id: ArtistId(108),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Obsolete Music Technology".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3456,
            id: ArtistId(109),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Oliver Deutschmann REMIX".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3488,
            id: ArtistId(110),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Oliver Deutschmann".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3520,
            id: ArtistId(111),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Oliver Kapp".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3552,
            id: ArtistId(112),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Pacou".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3584,
            id: ArtistId(113),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Patrik Carrera".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3616,
            id: ArtistId(114),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Patrik Carrera (GER)".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3648,
            id: ArtistId(115),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Phil Kieran".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3680,
            id: ArtistId(116),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Planetary Assault Systems".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3712,
            id: ArtistId(117),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Planetary Assault Systems ".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3744,
            id: ArtistId(118),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Plastikman".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3776,
            id: ArtistId(119),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "QNA".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 3808,
            id: ArtistId(120),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Radial".parse().unwrap(),
                },
            },
            padding: 19.into(),
        })))
        .unwrap();
    let page = Page {
        page_index: PageIndex(6),
        page_type: PageType::Plain(PlainPageType::Artists),
        next_page: PageIndex(47),
        unknown1: 855,
        unknown2: 0,
        num_rows_small: 120,
        unknown3: 0,
        unknown4: 15,
        page_flags: PageFlags(36),
        free_size: 12,
        used_size: 3772,
        unknown5: 1,
        num_rows_large: 119,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/artists_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn artist_page_long() {
    use std::iter::repeat_n;
    let mut rowgroup = RowGroup {
        row_offsets: [0; RowGroup::MAX_ROW_COUNT],
        row_presence_flags: 7,
        unknown: 0x20,
        rows: Default::default(),
    };
    rowgroup
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x64),
            index_shift: 0,
            id: ArtistId(1),
            offsets: OffsetArrayContainer {
                offsets: [3u16, 12u16].into(),
                inner: TrailingName {
                    name: repeat_n('D', 256).collect::<String>().parse().unwrap(),
                },
            },
            padding: 4.into(),
        })))
        .unwrap();
    rowgroup
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 32,
            id: ArtistId(2),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Insert 2".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    rowgroup
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x64),
            index_shift: 64,
            id: ArtistId(3),
            offsets: OffsetArrayContainer {
                offsets: [3u16, 12u16].into(),
                inner: TrailingName {
                    name: repeat_n('C', 256).collect::<String>().parse().unwrap(),
                },
            },
            padding: 4.into(),
        })))
        .unwrap();
    rowgroup
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x60),
            index_shift: 96,
            id: ArtistId(4),
            offsets: OffsetArrayContainer {
                offsets: [3u8, 10u8].into(),
                inner: TrailingName {
                    name: "Insert 1".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    rowgroup
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x64),
            index_shift: 128,
            id: ArtistId(5),
            offsets: OffsetArrayContainer {
                offsets: [3u16, 12u16].into(),
                inner: TrailingName {
                    name: repeat_n('B', 254).collect::<String>().parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    rowgroup
        .add_row(Row::Plain(PlainRow::Artist(Artist {
            subtype: Subtype(0x64),
            index_shift: 160,
            id: ArtistId(6),
            offsets: OffsetArrayContainer {
                offsets: [3u16, 12u16].into(),
                inner: TrailingName {
                    name: repeat_n('❤', 256).collect::<String>().parse().unwrap(),
                },
            },
            padding: 0.into(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(6),
        page_type: PageType::Plain(PlainPageType::Artists),
        next_page: PageIndex(46),
        unknown1: 16,
        unknown2: 0,
        num_rows_small: 6,
        unknown3: 192,
        unknown4: 0,
        page_flags: PageFlags(36),
        free_size: 2624,
        used_size: 1416,
        unknown5: 1,
        num_rows_large: 5,
        unknown6: 0,
        unknown7: 0,

        row_groups: vec![rowgroup],
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/artist_page_long.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn albums_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        6
    ];
    row_groups.last_mut().unwrap().unknown = 0x02;
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 0,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(1),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "The Worst of Gehm".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 32,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(2),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 24u8].into(),
                inner: TrailingName {
                    name: "1ØPILLS003 MASTER MP3s".parse().unwrap(),
                },
            },
            padding: 4.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 64,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(3),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Love & Happiness".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 96,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(4),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Wind / Phazzled".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 128,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(5),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Spectral Sound Volume 3".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 160,
            unknown2: 0,
            artist_id: ArtistId(12),
            id: AlbumId(6),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "The Hideout (Mini-Lp)".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 192,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(7),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Sweet Dreams EP".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 224,
            unknown2: 0,
            artist_id: ArtistId(18),
            id: AlbumId(8),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Lab.our 05".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 256,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(9),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "PolyfonikDizko".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 288,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(10),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Altered States EP".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 320,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(11),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "My So Called Robot Life EP".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 352,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(12),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Deep Ep".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 384,
            unknown2: 0,
            artist_id: ArtistId(25),
            id: AlbumId(13),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 24u8].into(),
                inner: TrailingName {
                    name: "Simoncino \u{200e}– Mystic Adventures".parse().unwrap(),
                },
            },
            padding: 4.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 416,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(14),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Smu Is The Key EP".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 448,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(15),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "SOM Compilation Volume 2".parse().unwrap(), // codespell:ignore
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 480,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(16),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "NY Muscle".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 512,
            unknown2: 0,
            artist_id: ArtistId(31),
            id: AlbumId(17),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Point of No Return EP".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 544,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(18),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Tapes 08".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 576,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(19),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Like No One".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 608,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(20),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "The Boat Party".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 640,
            unknown2: 0,
            artist_id: ArtistId(34),
            id: AlbumId(21),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Raw & Unreleased".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 672,
            unknown2: 0,
            artist_id: ArtistId(36),
            id: AlbumId(22),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Living Low".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 704,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(23),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Muzic Box Classics #7".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 736,
            unknown2: 0,
            artist_id: ArtistId(39),
            id: AlbumId(24),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Stranger In The Strangest Of Lands".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 768,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(25),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Pt. 1".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 800,
            unknown2: 0,
            artist_id: ArtistId(45),
            id: AlbumId(26),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Body Mechanics EP".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 832,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(27),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "EAUX1091 ".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 864,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(28),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Lost Tracks, Vol. 2".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 896,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(29),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Dubbelbrein EP".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 928,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(30),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Vx, Vol. 1".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 960,
            unknown2: 0,
            artist_id: ArtistId(39),
            id: AlbumId(31),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "The Trax Records Anthology Compiled By Bill Brewster"
                        .parse()
                        .unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 992,
            unknown2: 0,
            artist_id: ArtistId(51),
            id: AlbumId(32),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "AfriOrker".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1024,
            unknown2: 0,
            artist_id: ArtistId(52),
            id: AlbumId(33),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Mortal Sin EP".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1056,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(34),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Psyops part one EP".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1088,
            unknown2: 0,
            artist_id: ArtistId(55),
            id: AlbumId(35),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "White Rats III".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1120,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(36),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "far from reality".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1152,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(37),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "EP1".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1184,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(38),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Alpha Omega".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1216,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(39),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Decay".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1248,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(40),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "[LIES 009] Mind Control 320".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1280,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(41),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Nation".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1312,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(42),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Split 02".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1344,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(43),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Another Number".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1376,
            unknown2: 0,
            artist_id: ArtistId(64),
            id: AlbumId(44),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "8 Ball".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1408,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(45),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "H-Productions presents_Mutations 101 (HPX60)"
                        .parse()
                        .unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1440,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(46),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "CBS024X".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1472,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(47),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Ben Sims pres Tribology".parse().unwrap(), // codespell:ignore
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1504,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(48),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Night Jewel".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1536,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(49),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "AKROPOLEOS".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1568,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(50),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Mistress 12".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1600,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(51),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Mistress 12.5".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1632,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(52),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Remember Each Moment Of Freedom".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1664,
            unknown2: 0,
            artist_id: ArtistId(29),
            id: AlbumId(53),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Mood Sequences".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1696,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(54),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Death Is Nothing To Fear 1".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1728,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(55),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "I'm A Man".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1760,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(56),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Cari Lekebusch & Jesper Dahlback - Hands on experience"
                        .parse()
                        .unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1792,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(57),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "New Life EP".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1824,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(58),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "State of Mind EP".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1856,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(59),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "DABJ Allstars".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1888,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(60),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Mendoza".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1920,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(61),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "CD Thirteen".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1952,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(62),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Raw 7".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 1984,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(63),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Endurance - UNDERGROUND QUALITY".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2016,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(64),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "EFDEMIN - DECAY VERSIONS PT.2".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2048,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(65),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "HUSH 03".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2080,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(66),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Mistress 20".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2112,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(67),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Love under pressure ".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2144,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(68),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Release".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2176,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(69),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Hexagon Cloud".parse().unwrap(),
                },
            },
            padding: 8.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2208,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(70),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "NRDR 011".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2240,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(71),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Diptych".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2272,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(72),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Seven Days".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2304,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(73),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Unknown Origin".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2336,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(74),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Arpeggiator".parse().unwrap(),
                },
            },
            padding: 6.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2368,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(75),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "DECONSTRUCT MUSIC DEC-02".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2400,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(76),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Basement Tracks EP".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2432,
            unknown2: 0,
            artist_id: ArtistId(101),
            id: AlbumId(77),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Corpse Grinder".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2464,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(78),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Kamm / Plain".parse().unwrap(),
                },
            },
            padding: 9.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2496,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(79),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Fluxus_Digital_006".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2528,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(80),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Minutes In Ice".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2560,
            unknown2: 0,
            artist_id: ArtistId(0),
            id: AlbumId(81),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "TRP001".parse().unwrap(),
                },
            },
            padding: 7.into(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Album(Album {
            subtype: Subtype(0x80),
            index_shift: 2592,
            unknown2: 0,
            artist_id: ArtistId(108),
            id: AlbumId(82),
            unknown3: 0,
            offsets: OffsetArrayContainer {
                offsets: [3u8, 22u8].into(),
                inner: TrailingName {
                    name: "Mmmmmusic".parse().unwrap(),
                },
            },
            padding: 44.into(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(8),
        page_type: PageType::Plain(PlainPageType::Albums),
        next_page: PageIndex(49),
        unknown1: 772,
        unknown2: 0,
        num_rows_small: 82,
        unknown3: 64,
        unknown4: 10,
        page_flags: PageFlags(36),
        free_size: 36,
        used_size: 3832,
        unknown5: 1,
        num_rows_large: 81,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size: u32 = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/albums_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn labels_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        12
    ];
    row_groups.last_mut().unwrap().unknown = 1;

    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(1),
            name: "Solar One Music".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(2),
            name: "Spectral Sound".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(3),
            name: "TENG".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(4),
            name: "Prescription Classic Recordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(5),
            name: "Mathematics".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(6),
            name: "Rawax".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(7),
            name: "&nd".parse().unwrap(), //codespell:ignore nd
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(8),
            name: "Wild Oats Music".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(9),
            name: "Creme Organization".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(10),
            name: "Nobody's Bizzness".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(11),
            name: "ADD".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(12),
            name: "Footage Series".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(13),
            name: "Lone Romantic".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(14),
            name: "Clone Jack For Daze".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(15),
            name: "Rat Life".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(16),
            name: "Classicworks".parse().unwrap(),
        })))
        .unwrap();

    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(17),
            name: "Machine".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(18),
            name: "Modern Love".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(19),
            name: "Transient Force".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(20),
            name: "Playlouderecordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(21),
            name: "International DeeJay Gigolo Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(22),
            name: "Strength Music Recordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(23),
            name: "Dial Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(24),
            name: "Interdimensional Transmissions".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(25),
            name: "Subself Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(26),
            name: "Art of Dance".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(27),
            name: "Ostgut Ton (Germany)".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(28),
            name: "Innervisions".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(29),
            name: "Beatstreet".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(30),
            name: "трип".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(31),
            name: "Caduceus Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(32),
            name: "Stockholm LTD".parse().unwrap(),
        })))
        .unwrap();

    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(33),
            name: "Planet Rhythm".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(34),
            name: "Paranoid Dancer".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(35),
            name: "Snork Enterprises".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(36),
            name: "PKR".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(37),
            name: "Mord".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(38),
            name: "Circus Company".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(39),
            name: "Baalsaal Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(40),
            name: "Mephyst".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(41),
            name: "Peacefrog".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(42),
            name: "Kanzleramt".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(43),
            name: "Clone Jack For Daze Series".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(44),
            name: "Chronocircle".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(45),
            name: "Unknown To The Unknown".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(46),
            name: "Super Rhythm Trax".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(47),
            name: "LABEL".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(48),
            name: "Wilson Records".parse().unwrap(),
        })))
        .unwrap();

    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(49),
            name: "Houndstooth".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(50),
            name: "WRKTRX".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(51),
            name: "Apotek Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(52),
            name: "Figure SPC".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(53),
            name: "Rush Hour".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(54),
            name: "Wagon Repair".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(55),
            name: "Nada Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(56),
            name: "Plus 8 Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(57),
            name: "Ostgut Ton".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(58),
            name: "Scion Versions".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(59),
            name: "Phil Kieran Recordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(60),
            name: "Enemy Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(61),
            name: "UNCAGE".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(62),
            name: "RMR".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(63),
            name: "Warok Music".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(64),
            name: "Axis Records".parse().unwrap(),
        })))
        .unwrap();

    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(65),
            name: "Rekids".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(66),
            name: "GND Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(67),
            name: "© Evod Music".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(68),
            name: "Equalized".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(69),
            name: "H-Productions".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(70),
            name: "Drumcode".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(71),
            name: "Eclectic Limited".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(72),
            name: "Subject Detroit".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(73),
            name: "Ultra".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(74),
            name: "Chiwax".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(75),
            name: "Supervoid Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(76),
            name: "Soleil Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(77),
            name: "Intacto".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(78),
            name: "AYCB".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(79),
            name: "Token".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(80),
            name: "Purpose Maker".parse().unwrap(),
        })))
        .unwrap();

    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(81),
            name: "R&S Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(82),
            name: "Odd Even".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(83),
            name: "F Communications".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(84),
            name: "430 West Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(85),
            name: "From Another Mind".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(86),
            name: "100% Pure".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(87),
            name: "© Dynamic Reflection 2015".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(88),
            name: "Subsist Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(89),
            name: "SK BLACK".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(90),
            name: "Prologue".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(91),
            name: "SUB tl".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(92),
            name: "Granulart Recordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(93),
            name: "Voight".parse().unwrap(), // codespell:ignore
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(94),
            name: "Ahrpe Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(95),
            name: "Ovum Recordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(96),
            name: "Corpus".parse().unwrap(),
        })))
        .unwrap();

    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(97),
            name: "Indistinct Approach".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(98),
            name: "Counterchange".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(99),
            name: "Fanzine Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(100),
            name: "Sandwell District".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(101),
            name: "M_Rec Ltd".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(102),
            name: "Recode Musik".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(103),
            name: "Parabola".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(104),
            name: "Perc Trax Ltd.".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(105),
            name: "U.K Executes".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(106),
            name: "Cieli Di Orione".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(107),
            name: "Figure".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(108),
            name: "Illegal Alien LTD".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(109),
            name: "Next Week Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(110),
            name: "Labrynth".parse().unwrap(), // codespell:ignore
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(111),
            name: "Children Of Tomorrow".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(112),
            name: "Gynoid Audio".parse().unwrap(),
        })))
        .unwrap();

    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(113),
            name: "Devotion Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(114),
            name: "Gradient ".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(115),
            name: "Bunker Record".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(116),
            name: "Bio Rhythm".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(117),
            name: "Logistic Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(118),
            name: "BADs Label Larhon Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(119),
            name: "Be As One".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(120),
            name: "Cyclical Tracks".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(121),
            name: "SUB TL".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(122),
            name: "ANAOH".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(123),
            name: "Datapunk".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(124),
            name: "Lobster Theremin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(125),
            name: "Bass Agenda Recordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(126),
            name: "Clone West Coast Series".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(127),
            name: "Tresor Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(128),
            name: "Self Reflektion".parse().unwrap(),
        })))
        .unwrap();

    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(129),
            name: "Hotflush Recordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(130),
            name: "made of CONCRETE".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(131),
            name: "AKKOET LTD".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(132),
            name: "Zone".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(133),
            name: "Frigio Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(134),
            name: "International Deejay Gigolo Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(135),
            name: "Cod3 QR".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(136),
            name: "Central Processing Unit".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(137),
            name: "Transparent Sound".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(138),
            name: "Kneaded Pains".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(139),
            name: "The Third Room".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(140),
            name: "Allergy Season".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(141),
            name: "Mechatronica Music".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(142),
            name: "Minus".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(143),
            name: "Space Factory".parse().unwrap(),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(144),
            name: "Music Man Records".parse().unwrap(),
        })))
        .unwrap();

    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(145),
            name: "BCM Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(146),
            name: "Missing Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(147),
            name: "L.I.E.S.".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(148),
            name: "Sound Signature".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(149),
            name: "Mozaiku".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(150),
            name: "Boomstraat 1818".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(151),
            name: "TH Tar Hallow".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(152),
            name: "Rowan Underground".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(153),
            name: "Rekktor Music".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(154),
            name: "Nachtstrom Schallplatten".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(155),
            name: "N&N Records.".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(156),
            name: "Greta Recordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(157),
            name: "© Jerical Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(158),
            name: "Illegal Alien Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(159),
            name: "KR/LF Records ".parse().unwrap(),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(160),
            name: "Repetitive Rhythm Research".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(161),
            name: "Fides Tempo".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(162),
            name: "Starwork sas".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(163),
            name: "Blueprint".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(164),
            name: "Mirage".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(165),
            name: "EPMmusic (V-Series)".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(166),
            name: "© West Rules".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(167),
            name: "Copyright Control".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(168),
            name: "© Aquae Sextiae".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(169),
            name: "Tsunami Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(170),
            name: "Amelie Records".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(171),
            name: "Hivemind".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(172),
            name: "4 Track Recordings".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(173),
            name: "Exekutive Funktionen".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(174),
            name: "© Evod Music".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(175),
            name: "© Consumed Music".parse().unwrap(),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(176),
            name: "© EP Digital Music".parse().unwrap(),
        })))
        .unwrap();

    row_groups[11]
        .add_row(Row::Plain(PlainRow::Label(Label {
            id: LabelId(177),
            name: "Symbolism".parse().unwrap(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(10),
        page_type: PageType::Plain(PlainPageType::Labels),
        next_page: PageIndex(50),
        unknown1: 4627,
        unknown2: 0,
        num_rows_small: 177,
        unknown3: 32,
        unknown4: 22,
        page_flags: PageFlags(36),
        free_size: 2,
        used_size: 3652,
        unknown5: 1,
        num_rows_large: 176,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/labels_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn keys_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        4
    ];

    row_groups[3].unknown = 8;

    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(1),
            id2: 1,
            name: "Emin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(2),
            id2: 2,
            name: "Fmaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(3),
            id2: 3,
            name: "E".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(4),
            id2: 4,
            name: "Amin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(5),
            id2: 5,
            name: "2d".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(6),
            id2: 6,
            name: "Bmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(7),
            id2: 7,
            name: "Cmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(8),
            id2: 8,
            name: "Cmaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(9),
            id2: 9,
            name: "Abmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(10),
            id2: 10,
            name: "Dmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(11),
            id2: 11,
            name: "Gmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(12),
            id2: 12,
            name: "Dm".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(13),
            id2: 13,
            name: "Am".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(14),
            id2: 14,
            name: "A#".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(15),
            id2: 15,
            name: "G#min".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(16),
            id2: 16,
            name: "A#min".parse().unwrap(),
        })))
        .unwrap();

    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(17),
            id2: 17,
            name: "Amaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(18),
            id2: 18,
            name: "Gmaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(19),
            id2: 19,
            name: "D#min".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(20),
            id2: 20,
            name: "Dmaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(21),
            id2: 21,
            name: "Ebmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(22),
            id2: 22,
            name: "Emaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(23),
            id2: 23,
            name: "F#min".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(24),
            id2: 24,
            name: "A#maj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(25),
            id2: 25,
            name: "D#".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(26),
            id2: 26,
            name: "Gbmaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(27),
            id2: 27,
            name: "D#maj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(28),
            id2: 28,
            name: "Bmaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(29),
            id2: 29,
            name: "7m".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(30),
            id2: 30,
            name: "C#min".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(31),
            id2: 31,
            name: "5m".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(32),
            id2: 32,
            name: "Dbmaj".parse().unwrap(),
        })))
        .unwrap();

    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(33),
            id2: 33,
            name: "Bbmaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(34),
            id2: 34,
            name: "12m".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(35),
            id2: 35,
            name: "Bbmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(36),
            id2: 36,
            name: "Fmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(37),
            id2: 37,
            name: "F#maj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(38),
            id2: 38,
            name: "10m".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(39),
            id2: 39,
            name: "A".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(40),
            id2: 40,
            name: "Bbm".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(41),
            id2: 41,
            name: "C".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(42),
            id2: 42,
            name: "Dbmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(43),
            id2: 43,
            name: "Gm".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(44),
            id2: 44,
            name: "Gbmin".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(45),
            id2: 45,
            name: "A m".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(46),
            id2: 46,
            name: "3d".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(47),
            id2: 47,
            name: "7d".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(48),
            id2: 48,
            name: "F#m".parse().unwrap(),
        })))
        .unwrap();

    row_groups[3]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(49),
            id2: 49,
            name: "Unknown".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(50),
            id2: 50,
            name: "Em".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(51),
            id2: 51,
            name: "Bm".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Key(Key {
            id: KeyId(52),
            id2: 52,
            name: "Ab".parse().unwrap(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(12),
        page_type: PageType::Plain(PlainPageType::Keys),
        next_page: PageIndex(51),
        unknown1: 13484,
        unknown2: 0,
        num_rows_small: 52,
        unknown3: 128,
        unknown4: 6,
        page_flags: PageFlags(36),
        free_size: 3188,
        used_size: 748,
        unknown5: 1,
        num_rows_large: 51,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/keys_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn colors_page() {
    let mut row_groups = vec![RowGroup {
        row_offsets: Default::default(),
        row_presence_flags: 0,
        unknown: 255,
        rows: vec![],
    }];

    row_groups[0]
        .add_row(Row::Plain(PlainRow::Color(Color {
            unknown1: 0,
            unknown2: 1,
            color: ColorIndex::Pink,
            unknown3: 0,
            name: "Pink".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Color(Color {
            unknown1: 0,
            unknown2: 2,
            color: ColorIndex::Red,
            unknown3: 0,
            name: "Red".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Color(Color {
            unknown1: 0,
            unknown2: 3,
            color: ColorIndex::Orange,
            unknown3: 0,
            name: "Orange".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Color(Color {
            unknown1: 0,
            unknown2: 4,
            color: ColorIndex::Yellow,
            unknown3: 0,
            name: "Yellow".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Color(Color {
            unknown1: 0,
            unknown2: 5,
            color: ColorIndex::Green,
            unknown3: 0,
            name: "Green".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Color(Color {
            unknown1: 0,
            unknown2: 6,
            color: ColorIndex::Aqua,
            unknown3: 0,
            name: "Aqua".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Color(Color {
            unknown1: 0,
            unknown2: 7,
            color: ColorIndex::Blue,
            unknown3: 0,
            name: "Blue".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Color(Color {
            unknown1: 0,
            unknown2: 8,
            color: ColorIndex::Purple,
            unknown3: 0,
            name: "Purple".parse().unwrap(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(14),
        page_type: PageType::Plain(PlainPageType::Colors),
        next_page: PageIndex(42),
        unknown1: 2,
        unknown2: 0,
        num_rows_small: 8,
        unknown3: 0,
        unknown4: 1,
        page_flags: PageFlags(36),
        free_size: 3912,
        used_size: 124,
        unknown5: 8,
        num_rows_large: 0,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/colors_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn playlist_tree_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        2
    ];

    row_groups[1].unknown = 1024;

    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(0),
            unknown: 0,
            sort_order: 0,
            id: PlaylistTreeNodeId(1),
            node_is_folder: 1,
            name: "folderb".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 0,
            id: PlaylistTreeNodeId(2),
            node_is_folder: 0,
            name: "listaz".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 2,
            id: PlaylistTreeNodeId(3),
            node_is_folder: 0,
            name: "listay".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 4,
            id: PlaylistTreeNodeId(4),
            node_is_folder: 0,
            name: "listax".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 6,
            id: PlaylistTreeNodeId(5),
            node_is_folder: 0,
            name: "listaw".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 8,
            id: PlaylistTreeNodeId(6),
            node_is_folder: 0,
            name: "listav".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 10,
            id: PlaylistTreeNodeId(7),
            node_is_folder: 0,
            name: "listau".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 12,
            id: PlaylistTreeNodeId(8),
            node_is_folder: 0,
            name: "listat".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 14,
            id: PlaylistTreeNodeId(9),
            node_is_folder: 0,
            name: "listas".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 16,
            id: PlaylistTreeNodeId(10),
            node_is_folder: 0,
            name: "listar".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 18,
            id: PlaylistTreeNodeId(11),
            node_is_folder: 0,
            name: "listaq".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 20,
            id: PlaylistTreeNodeId(12),
            node_is_folder: 0,
            name: "listap".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 22,
            id: PlaylistTreeNodeId(13),
            node_is_folder: 0,
            name: "listao".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 24,
            id: PlaylistTreeNodeId(14),
            node_is_folder: 0,
            name: "listan".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 26,
            id: PlaylistTreeNodeId(15),
            node_is_folder: 0,
            name: "listam".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 28,
            id: PlaylistTreeNodeId(16),
            node_is_folder: 0,
            name: "listak".parse().unwrap(),
        })))
        .unwrap();

    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 30,
            id: PlaylistTreeNodeId(17),
            node_is_folder: 0,
            name: "listal".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 32,
            id: PlaylistTreeNodeId(18),
            node_is_folder: 0,
            name: "listaj".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 34,
            id: PlaylistTreeNodeId(19),
            node_is_folder: 0,
            name: "listag".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 36,
            id: PlaylistTreeNodeId(20),
            node_is_folder: 0,
            name: "listai".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 38,
            id: PlaylistTreeNodeId(21),
            node_is_folder: 0,
            name: "listae".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 40,
            id: PlaylistTreeNodeId(22),
            node_is_folder: 0,
            name: "listaf".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 42,
            id: PlaylistTreeNodeId(23),
            node_is_folder: 0,
            name: "listah".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 44,
            id: PlaylistTreeNodeId(24),
            node_is_folder: 0,
            name: "listac".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 46,
            id: PlaylistTreeNodeId(25),
            node_is_folder: 0,
            name: "listad".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 48,
            id: PlaylistTreeNodeId(26),
            node_is_folder: 0,
            name: "listaa".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
            parent_id: PlaylistTreeNodeId(1),
            unknown: 0,
            sort_order: 50,
            id: PlaylistTreeNodeId(27),
            node_is_folder: 0,
            name: "listab".parse().unwrap(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(16),
        page_type: PageType::Plain(PlainPageType::PlaylistTree),
        next_page: PageIndex(46),
        unknown1: 36,
        unknown2: 0,
        num_rows_small: 27,
        unknown3: 96,
        unknown4: 3,
        page_flags: PageFlags(36),
        free_size: 3238,
        used_size: 756,
        unknown5: 1,
        num_rows_large: 26,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/playlist_tree_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn playlist_entries_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        18
    ];

    row_groups[17].unknown = 2048;

    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 1,
            track_id: TrackId(1),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 2,
            track_id: TrackId(2),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 3,
            track_id: TrackId(3),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 4,
            track_id: TrackId(4),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 5,
            track_id: TrackId(5),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 6,
            track_id: TrackId(6),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 7,
            track_id: TrackId(7),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 8,
            track_id: TrackId(8),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 9,
            track_id: TrackId(9),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 10,
            track_id: TrackId(10),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 11,
            track_id: TrackId(11),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 12,
            track_id: TrackId(12),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 13,
            track_id: TrackId(13),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 14,
            track_id: TrackId(14),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 15,
            track_id: TrackId(15),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 16,
            track_id: TrackId(16),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();

    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 17,
            track_id: TrackId(17),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 18,
            track_id: TrackId(18),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 19,
            track_id: TrackId(19),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 20,
            track_id: TrackId(20),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 21,
            track_id: TrackId(21),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 22,
            track_id: TrackId(22),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 23,
            track_id: TrackId(23),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 24,
            track_id: TrackId(24),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 25,
            track_id: TrackId(25),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 26,
            track_id: TrackId(26),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 27,
            track_id: TrackId(27),
            playlist_id: PlaylistTreeNodeId(6),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 1,
            track_id: TrackId(28),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 2,
            track_id: TrackId(29),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 3,
            track_id: TrackId(30),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 4,
            track_id: TrackId(31),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 5,
            track_id: TrackId(32),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();

    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 6,
            track_id: TrackId(33),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 7,
            track_id: TrackId(34),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 8,
            track_id: TrackId(35),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 9,
            track_id: TrackId(36),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 10,
            track_id: TrackId(37),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 11,
            track_id: TrackId(15),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 12,
            track_id: TrackId(38),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 13,
            track_id: TrackId(39),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 14,
            track_id: TrackId(40),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 15,
            track_id: TrackId(41),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 16,
            track_id: TrackId(42),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 17,
            track_id: TrackId(22),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 18,
            track_id: TrackId(43),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 19,
            track_id: TrackId(44),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 20,
            track_id: TrackId(45),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 21,
            track_id: TrackId(46),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();

    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 22,
            track_id: TrackId(47),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 23,
            track_id: TrackId(48),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 24,
            track_id: TrackId(49),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 25,
            track_id: TrackId(50),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 26,
            track_id: TrackId(51),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 27,
            track_id: TrackId(52),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 28,
            track_id: TrackId(53),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 29,
            track_id: TrackId(54),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 30,
            track_id: TrackId(55),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 31,
            track_id: TrackId(56),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 32,
            track_id: TrackId(57),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 33,
            track_id: TrackId(58),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 34,
            track_id: TrackId(59),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 35,
            track_id: TrackId(60),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 36,
            track_id: TrackId(61),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 37,
            track_id: TrackId(26),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();

    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 38,
            track_id: TrackId(4),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 39,
            track_id: TrackId(62),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 40,
            track_id: TrackId(63),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 41,
            track_id: TrackId(64),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 42,
            track_id: TrackId(65),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 43,
            track_id: TrackId(66),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 44,
            track_id: TrackId(67),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 45,
            track_id: TrackId(68),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 46,
            track_id: TrackId(69),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 47,
            track_id: TrackId(70),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 48,
            track_id: TrackId(71),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 49,
            track_id: TrackId(72),
            playlist_id: PlaylistTreeNodeId(7),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 1,
            track_id: TrackId(73),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 2,
            track_id: TrackId(74),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 3,
            track_id: TrackId(75),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 4,
            track_id: TrackId(76),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();

    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 5,
            track_id: TrackId(77),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 6,
            track_id: TrackId(78),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 7,
            track_id: TrackId(79),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 8,
            track_id: TrackId(80),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 9,
            track_id: TrackId(81),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 10,
            track_id: TrackId(82),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 11,
            track_id: TrackId(83),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 12,
            track_id: TrackId(84),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 13,
            track_id: TrackId(85),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 14,
            track_id: TrackId(86),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 15,
            track_id: TrackId(87),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 16,
            track_id: TrackId(88),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 17,
            track_id: TrackId(89),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 18,
            track_id: TrackId(90),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 19,
            track_id: TrackId(91),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 20,
            track_id: TrackId(92),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();

    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 21,
            track_id: TrackId(93),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 22,
            track_id: TrackId(94),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 23,
            track_id: TrackId(95),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 24,
            track_id: TrackId(96),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 25,
            track_id: TrackId(97),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 26,
            track_id: TrackId(98),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 27,
            track_id: TrackId(99),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 28,
            track_id: TrackId(100),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 29,
            track_id: TrackId(101),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 30,
            track_id: TrackId(102),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 31,
            track_id: TrackId(103),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 32,
            track_id: TrackId(4),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 33,
            track_id: TrackId(33),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 34,
            track_id: TrackId(104),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 35,
            track_id: TrackId(105),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 36,
            track_id: TrackId(106),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();

    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 37,
            track_id: TrackId(107),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 38,
            track_id: TrackId(108),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 39,
            track_id: TrackId(109),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 40,
            track_id: TrackId(110),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 41,
            track_id: TrackId(111),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 42,
            track_id: TrackId(112),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 43,
            track_id: TrackId(113),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 44,
            track_id: TrackId(114),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 45,
            track_id: TrackId(115),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 46,
            track_id: TrackId(116),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 47,
            track_id: TrackId(117),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 48,
            track_id: TrackId(118),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 49,
            track_id: TrackId(119),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 50,
            track_id: TrackId(120),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 51,
            track_id: TrackId(121),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[7]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 52,
            track_id: TrackId(122),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();

    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 53,
            track_id: TrackId(123),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 54,
            track_id: TrackId(124),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 55,
            track_id: TrackId(125),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 56,
            track_id: TrackId(126),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 57,
            track_id: TrackId(127),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 58,
            track_id: TrackId(128),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 59,
            track_id: TrackId(129),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 60,
            track_id: TrackId(130),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 61,
            track_id: TrackId(131),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 62,
            track_id: TrackId(132),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 63,
            track_id: TrackId(133),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 64,
            track_id: TrackId(134),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 65,
            track_id: TrackId(52),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 66,
            track_id: TrackId(135),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 67,
            track_id: TrackId(136),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[8]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 68,
            track_id: TrackId(137),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();

    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 69,
            track_id: TrackId(138),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 70,
            track_id: TrackId(139),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 71,
            track_id: TrackId(140),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 72,
            track_id: TrackId(141),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 73,
            track_id: TrackId(142),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 74,
            track_id: TrackId(143),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 75,
            track_id: TrackId(144),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 76,
            track_id: TrackId(145),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 77,
            track_id: TrackId(146),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 78,
            track_id: TrackId(66),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 79,
            track_id: TrackId(147),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 80,
            track_id: TrackId(148),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 81,
            track_id: TrackId(149),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 82,
            track_id: TrackId(150),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 83,
            track_id: TrackId(53),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[9]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 84,
            track_id: TrackId(151),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();

    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 85,
            track_id: TrackId(152),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 86,
            track_id: TrackId(153),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 87,
            track_id: TrackId(154),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 88,
            track_id: TrackId(155),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 89,
            track_id: TrackId(156),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 90,
            track_id: TrackId(157),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 91,
            track_id: TrackId(158),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 92,
            track_id: TrackId(159),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 93,
            track_id: TrackId(160),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 94,
            track_id: TrackId(161),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 95,
            track_id: TrackId(162),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 96,
            track_id: TrackId(163),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 97,
            track_id: TrackId(164),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 98,
            track_id: TrackId(165),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 99,
            track_id: TrackId(166),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[10]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 100,
            track_id: TrackId(167),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();

    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 101,
            track_id: TrackId(54),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 102,
            track_id: TrackId(47),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 103,
            track_id: TrackId(168),
            playlist_id: PlaylistTreeNodeId(8),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 1,
            track_id: TrackId(169),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 2,
            track_id: TrackId(170),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 3,
            track_id: TrackId(171),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 4,
            track_id: TrackId(57),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 5,
            track_id: TrackId(172),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 6,
            track_id: TrackId(173),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 7,
            track_id: TrackId(174),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 8,
            track_id: TrackId(175),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 9,
            track_id: TrackId(125),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 10,
            track_id: TrackId(176),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 11,
            track_id: TrackId(177),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 12,
            track_id: TrackId(178),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[11]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 13,
            track_id: TrackId(179),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();

    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 14,
            track_id: TrackId(180),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 15,
            track_id: TrackId(181),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 16,
            track_id: TrackId(182),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 17,
            track_id: TrackId(183),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 18,
            track_id: TrackId(166),
            playlist_id: PlaylistTreeNodeId(9),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 1,
            track_id: TrackId(184),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 2,
            track_id: TrackId(185),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 3,
            track_id: TrackId(77),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 4,
            track_id: TrackId(186),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 5,
            track_id: TrackId(187),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 6,
            track_id: TrackId(188),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 7,
            track_id: TrackId(189),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 8,
            track_id: TrackId(190),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 9,
            track_id: TrackId(191),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 10,
            track_id: TrackId(90),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[12]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 11,
            track_id: TrackId(192),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();

    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 12,
            track_id: TrackId(193),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 13,
            track_id: TrackId(194),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 14,
            track_id: TrackId(195),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 15,
            track_id: TrackId(101),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 16,
            track_id: TrackId(196),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 17,
            track_id: TrackId(197),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 18,
            track_id: TrackId(198),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 19,
            track_id: TrackId(199),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 20,
            track_id: TrackId(200),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 21,
            track_id: TrackId(201),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 22,
            track_id: TrackId(202),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 23,
            track_id: TrackId(203),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 24,
            track_id: TrackId(204),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 25,
            track_id: TrackId(205),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 26,
            track_id: TrackId(206),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[13]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 27,
            track_id: TrackId(207),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();

    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 28,
            track_id: TrackId(208),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 29,
            track_id: TrackId(209),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 30,
            track_id: TrackId(210),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 31,
            track_id: TrackId(211),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 32,
            track_id: TrackId(212),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 33,
            track_id: TrackId(213),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 34,
            track_id: TrackId(214),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 35,
            track_id: TrackId(215),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 36,
            track_id: TrackId(168),
            playlist_id: PlaylistTreeNodeId(10),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 1,
            track_id: TrackId(74),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 2,
            track_id: TrackId(79),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 3,
            track_id: TrackId(80),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 4,
            track_id: TrackId(81),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 5,
            track_id: TrackId(82),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 6,
            track_id: TrackId(87),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[14]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 7,
            track_id: TrackId(189),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();

    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 8,
            track_id: TrackId(216),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 9,
            track_id: TrackId(217),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 10,
            track_id: TrackId(218),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 11,
            track_id: TrackId(219),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 12,
            track_id: TrackId(220),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 13,
            track_id: TrackId(221),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 14,
            track_id: TrackId(222),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 15,
            track_id: TrackId(223),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 16,
            track_id: TrackId(195),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 17,
            track_id: TrackId(105),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 18,
            track_id: TrackId(224),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 19,
            track_id: TrackId(107),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 20,
            track_id: TrackId(225),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 21,
            track_id: TrackId(226),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 22,
            track_id: TrackId(227),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[15]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 23,
            track_id: TrackId(228),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();

    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 24,
            track_id: TrackId(229),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 25,
            track_id: TrackId(10),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 26,
            track_id: TrackId(230),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 27,
            track_id: TrackId(231),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 28,
            track_id: TrackId(232),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 29,
            track_id: TrackId(233),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 30,
            track_id: TrackId(234),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 31,
            track_id: TrackId(17),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 32,
            track_id: TrackId(235),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 33,
            track_id: TrackId(138),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 34,
            track_id: TrackId(236),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 35,
            track_id: TrackId(147),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 36,
            track_id: TrackId(237),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 37,
            track_id: TrackId(208),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 38,
            track_id: TrackId(238),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[16]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 39,
            track_id: TrackId(239),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();

    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 40,
            track_id: TrackId(240),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 41,
            track_id: TrackId(241),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 42,
            track_id: TrackId(242),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 43,
            track_id: TrackId(243),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 44,
            track_id: TrackId(244),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 45,
            track_id: TrackId(245),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 46,
            track_id: TrackId(120),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 47,
            track_id: TrackId(246),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 48,
            track_id: TrackId(247),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 49,
            track_id: TrackId(248),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 50,
            track_id: TrackId(249),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();
    row_groups[17]
        .add_row(Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
            entry_index: 51,
            track_id: TrackId(250),
            playlist_id: PlaylistTreeNodeId(11),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(18),
        page_type: PageType::Plain(PlainPageType::PlaylistEntries),
        next_page: PageIndex(54),
        unknown1: 1420,
        unknown2: 0,
        num_rows_small: 28,
        unknown3: 129,
        unknown4: 35,
        page_flags: PageFlags(36),
        free_size: 8,
        used_size: 3408,
        unknown5: 1,
        num_rows_large: 283,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/playlist_entries_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn artworks_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        7
    ];

    row_groups[6].unknown = 512;

    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(1),
            path: "/PIONEER/Artwork/00001/a1.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(2),
            path: "/PIONEER/Artwork/00001/a2.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(3),
            path: "/PIONEER/Artwork/00001/a3.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(4),
            path: "/PIONEER/Artwork/00001/a4.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(5),
            path: "/PIONEER/Artwork/00001/a5.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(6),
            path: "/PIONEER/Artwork/00001/a6.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(7),
            path: "/PIONEER/Artwork/00001/a7.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(8),
            path: "/PIONEER/Artwork/00001/a8.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(9),
            path: "/PIONEER/Artwork/00001/a9.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(10),
            path: "/PIONEER/Artwork/00001/a10.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(11),
            path: "/PIONEER/Artwork/00001/a11.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(12),
            path: "/PIONEER/Artwork/00001/a12.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(13),
            path: "/PIONEER/Artwork/00001/a13.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(14),
            path: "/PIONEER/Artwork/00001/a14.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(15),
            path: "/PIONEER/Artwork/00001/a15.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(16),
            path: "/PIONEER/Artwork/00001/a16.jpg".parse().unwrap(),
        })))
        .unwrap();

    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(17),
            path: "/PIONEER/Artwork/00001/a17.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(18),
            path: "/PIONEER/Artwork/00001/a18.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(19),
            path: "/PIONEER/Artwork/00001/a19.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(20),
            path: "/PIONEER/Artwork/00002/a20.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(21),
            path: "/PIONEER/Artwork/00002/a21.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(22),
            path: "/PIONEER/Artwork/00002/a22.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(23),
            path: "/PIONEER/Artwork/00002/a23.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(24),
            path: "/PIONEER/Artwork/00002/a24.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(25),
            path: "/PIONEER/Artwork/00002/a25.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(26),
            path: "/PIONEER/Artwork/00002/a26.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(27),
            path: "/PIONEER/Artwork/00002/a27.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(28),
            path: "/PIONEER/Artwork/00002/a28.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(29),
            path: "/PIONEER/Artwork/00002/a29.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(30),
            path: "/PIONEER/Artwork/00002/a30.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(31),
            path: "/PIONEER/Artwork/00002/a31.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(32),
            path: "/PIONEER/Artwork/00002/a32.jpg".parse().unwrap(),
        })))
        .unwrap();

    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(33),
            path: "/PIONEER/Artwork/00002/a33.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(34),
            path: "/PIONEER/Artwork/00002/a34.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(35),
            path: "/PIONEER/Artwork/00002/a35.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(36),
            path: "/PIONEER/Artwork/00002/a36.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(37),
            path: "/PIONEER/Artwork/00002/a37.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(38),
            path: "/PIONEER/Artwork/00002/a38.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(39),
            path: "/PIONEER/Artwork/00002/a39.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(40),
            path: "/PIONEER/Artwork/00003/a40.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(41),
            path: "/PIONEER/Artwork/00003/a41.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(42),
            path: "/PIONEER/Artwork/00003/a42.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(43),
            path: "/PIONEER/Artwork/00003/a43.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(44),
            path: "/PIONEER/Artwork/00003/a44.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(45),
            path: "/PIONEER/Artwork/00003/a45.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(46),
            path: "/PIONEER/Artwork/00003/a46.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(47),
            path: "/PIONEER/Artwork/00003/a47.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(48),
            path: "/PIONEER/Artwork/00003/a48.jpg".parse().unwrap(),
        })))
        .unwrap();

    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(49),
            path: "/PIONEER/Artwork/00003/a49.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(50),
            path: "/PIONEER/Artwork/00003/a50.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(51),
            path: "/PIONEER/Artwork/00003/a51.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(52),
            path: "/PIONEER/Artwork/00003/a52.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(53),
            path: "/PIONEER/Artwork/00003/a53.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(54),
            path: "/PIONEER/Artwork/00003/a54.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(55),
            path: "/PIONEER/Artwork/00003/a55.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(56),
            path: "/PIONEER/Artwork/00003/a56.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(57),
            path: "/PIONEER/Artwork/00003/a57.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(58),
            path: "/PIONEER/Artwork/00003/a58.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(59),
            path: "/PIONEER/Artwork/00003/a59.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(60),
            path: "/PIONEER/Artwork/00004/a60.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(61),
            path: "/PIONEER/Artwork/00004/a61.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(62),
            path: "/PIONEER/Artwork/00004/a62.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(63),
            path: "/PIONEER/Artwork/00004/a63.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(64),
            path: "/PIONEER/Artwork/00004/a64.jpg".parse().unwrap(),
        })))
        .unwrap();

    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(65),
            path: "/PIONEER/Artwork/00004/a65.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(66),
            path: "/PIONEER/Artwork/00004/a66.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(67),
            path: "/PIONEER/Artwork/00004/a67.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(68),
            path: "/PIONEER/Artwork/00004/a68.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(69),
            path: "/PIONEER/Artwork/00004/a69.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(70),
            path: "/PIONEER/Artwork/00004/a70.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(71),
            path: "/PIONEER/Artwork/00004/a71.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(72),
            path: "/PIONEER/Artwork/00004/a72.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(73),
            path: "/PIONEER/Artwork/00004/a73.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(74),
            path: "/PIONEER/Artwork/00004/a74.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(75),
            path: "/PIONEER/Artwork/00004/a75.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(76),
            path: "/PIONEER/Artwork/00004/a76.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(77),
            path: "/PIONEER/Artwork/00004/a77.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(78),
            path: "/PIONEER/Artwork/00004/a78.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(79),
            path: "/PIONEER/Artwork/00004/a79.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[4]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(80),
            path: "/PIONEER/Artwork/00005/a80.jpg".parse().unwrap(),
        })))
        .unwrap();

    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(81),
            path: "/PIONEER/Artwork/00005/a81.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(82),
            path: "/PIONEER/Artwork/00005/a82.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(83),
            path: "/PIONEER/Artwork/00005/a83.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(84),
            path: "/PIONEER/Artwork/00005/a84.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(85),
            path: "/PIONEER/Artwork/00005/a85.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(86),
            path: "/PIONEER/Artwork/00005/a86.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(87),
            path: "/PIONEER/Artwork/00005/a87.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(88),
            path: "/PIONEER/Artwork/00005/a88.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(89),
            path: "/PIONEER/Artwork/00005/a89.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(90),
            path: "/PIONEER/Artwork/00005/a90.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(91),
            path: "/PIONEER/Artwork/00005/a91.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(92),
            path: "/PIONEER/Artwork/00005/a92.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(93),
            path: "/PIONEER/Artwork/00005/a93.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(94),
            path: "/PIONEER/Artwork/00005/a94.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(95),
            path: "/PIONEER/Artwork/00005/a95.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[5]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(96),
            path: "/PIONEER/Artwork/00005/a96.jpg".parse().unwrap(),
        })))
        .unwrap();

    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(97),
            path: "/PIONEER/Artwork/00005/a97.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(98),
            path: "/PIONEER/Artwork/00005/a98.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(99),
            path: "/PIONEER/Artwork/00005/a99.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(100),
            path: "/PIONEER/Artwork/00006/a100.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(101),
            path: "/PIONEER/Artwork/00006/a101.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(102),
            path: "/PIONEER/Artwork/00006/a102.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(103),
            path: "/PIONEER/Artwork/00006/a103.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(104),
            path: "/PIONEER/Artwork/00006/a104.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(105),
            path: "/PIONEER/Artwork/00006/a105.jpg".parse().unwrap(),
        })))
        .unwrap();
    row_groups[6]
        .add_row(Row::Plain(PlainRow::Artwork(Artwork {
            id: ArtworkId(106),
            path: "/PIONEER/Artwork/00006/a106.jpg".parse().unwrap(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(28),
        page_type: PageType::Plain(PlainPageType::Artwork),
        next_page: PageIndex(53),
        unknown1: 1019,
        unknown2: 0,
        num_rows_small: 106,
        unknown3: 64,
        unknown4: 13,
        page_flags: PageFlags(36),
        free_size: 0,
        used_size: 3816,
        unknown5: 1,
        num_rows_large: 105,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/artworks_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

#[test]
fn tag_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        2
    ];

    row_groups[0].unknown = 65535; // interestingly, these are the same
    row_groups[1].unknown = 127; // as row_presence_flags in this page

    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 0,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(None),
            position: 0,
            id: TagId(1),
            raw_is_category: 16777216,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 44]),
                inner: TagOrCategoryStrings {
                    name: "TagCategory1".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 32,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(1).unwrap())),
            position: 0,
            id: TagId(3456350885),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag1Cat1".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 64,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(1).unwrap())),
            position: 1,
            id: TagId(246010797),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag2Cat1".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 96,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(None),
            position: 1,
            id: TagId(2),
            raw_is_category: 16777216,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 44]),
                inner: TagOrCategoryStrings {
                    name: "TagCategory2".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 128,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(2).unwrap())),
            position: 0,
            id: TagId(2923592519),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag1Cat2".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 160,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(2).unwrap())),
            position: 1,
            id: TagId(3518593467),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 48]),
                inner: TagOrCategoryStrings {
                    name: "Tag2Cat2LongName".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 192,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(2).unwrap())),
            position: 2,
            id: TagId(870902105),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag3Cat2".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 224,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(2).unwrap())),
            position: 3,
            id: TagId(3211624224),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag4Cat2".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 256,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(2).unwrap())),
            position: 4,
            id: TagId(3216792858),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag5Cat2".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 288,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(2).unwrap())),
            position: 5,
            id: TagId(712200756),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag6Cat2".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 320,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(2).unwrap())),
            position: 6,
            id: TagId(4166869272),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag7Cat2".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 352,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(2).unwrap())),
            position: 7,
            id: TagId(4052665282),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag8Cat2".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 384,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(None),
            position: 2,
            id: TagId(3),
            raw_is_category: 16777216,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 44]),
                inner: TagOrCategoryStrings {
                    name: "TagCategory3".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 416,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(3).unwrap())),
            position: 0,
            id: TagId(2498240426),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag1Cat3".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 448,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(3).unwrap())),
            position: 1,
            id: TagId(598441108),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag2Cat3".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 480,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(3).unwrap())),
            position: 2,
            id: TagId(4263562201),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag3Cat3".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();

    row_groups[1]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 512,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(3).unwrap())),
            position: 3,
            id: TagId(926017397),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag4Cat3".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 544,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(3).unwrap())),
            position: 4,
            id: TagId(707481115),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag5Cat3".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 576,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(3).unwrap())),
            position: 5,
            id: TagId(3043071597),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag6Cat3".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 608,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(3).unwrap())),
            position: 6,
            id: TagId(4026144338),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag7Cat3".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 640,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(3).unwrap())),
            position: 7,
            id: TagId(218937570),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 40]),
                inner: TagOrCategoryStrings {
                    name: "Tag8Cat3".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 672,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(None),
            position: 3,
            id: TagId(4),
            raw_is_category: 16777216,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 44]),
                inner: TagOrCategoryStrings {
                    name: "TagCategory4".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 11.into(),
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::Tag(TagOrCategory {
            subtype: Subtype(1664),
            index_shift: 704,
            unknown1: 0,
            unknown2: 0,
            parent_id: ParentId(Some(NonZero::new(4).unwrap())),
            position: 0,
            id: TagId(3074636465),
            raw_is_category: 0,
            offsets: OffsetArrayContainer {
                offsets: OffsetArray::U8([3, 31, 54]),
                inner: TagOrCategoryStrings {
                    name: "Tag1Cat4EvenLongerName".parse().unwrap(),
                    unknown: "".parse().unwrap(),
                },
            },
            padding: 0.into(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(8),
        page_type: PageType::Ext(ExtPageType::Tag),
        next_page: PageIndex(20),
        unknown1: 2,
        unknown2: 0,
        num_rows_small: 23,
        unknown3: 224,
        unknown4: 2,
        page_flags: PageFlags(36),
        free_size: 2770,
        used_size: 1232,
        unknown5: 23,
        num_rows_large: 0,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/tag_page.bin"),
        page,
        (page_size, DatabaseType::Ext),
        (page_size, DatabaseType::Ext),
    );
}

#[test]
fn track_tag_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        4
    ];

    row_groups[3].unknown = 8;

    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(1),
            tag_id: TagId(2498240426),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(2),
            tag_id: TagId(4052665282),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(2),
            tag_id: TagId(2498240426),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(3),
            tag_id: TagId(3456350885),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(3),
            tag_id: TagId(3518593467),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(3),
            tag_id: TagId(3074636465),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(4),
            tag_id: TagId(3456350885),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(4),
            tag_id: TagId(3518593467),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(4),
            tag_id: TagId(4026144338),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(4),
            tag_id: TagId(3074636465),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(5),
            tag_id: TagId(4052665282),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(5),
            tag_id: TagId(218937570),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(5),
            tag_id: TagId(3074636465),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(6),
            tag_id: TagId(3211624224),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(6),
            tag_id: TagId(3043071597),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(7),
            tag_id: TagId(2923592519),
            unknown_const: 3,
        })))
        .unwrap();

    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(7),
            tag_id: TagId(712200756),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(8),
            tag_id: TagId(3456350885),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(8),
            tag_id: TagId(4263562201),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(8),
            tag_id: TagId(3074636465),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(9),
            tag_id: TagId(4052665282),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(9),
            tag_id: TagId(3074636465),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(10),
            tag_id: TagId(3216792858),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(10),
            tag_id: TagId(4026144338),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(11),
            tag_id: TagId(3456350885),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(11),
            tag_id: TagId(598441108),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(11),
            tag_id: TagId(707481115),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(12),
            tag_id: TagId(2923592519),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(12),
            tag_id: TagId(3518593467),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(12),
            tag_id: TagId(926017397),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(13),
            tag_id: TagId(712200756),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[1]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(13),
            tag_id: TagId(4263562201),
            unknown_const: 3,
        })))
        .unwrap();

    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(14),
            tag_id: TagId(3211624224),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(14),
            tag_id: TagId(4026144338),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(15),
            tag_id: TagId(3456350885),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(15),
            tag_id: TagId(4166869272),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(16),
            tag_id: TagId(4052665282),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(16),
            tag_id: TagId(3043071597),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(17),
            tag_id: TagId(4166869272),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(17),
            tag_id: TagId(926017397),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(18),
            tag_id: TagId(3518593467),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(18),
            tag_id: TagId(870902105),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(19),
            tag_id: TagId(3456350885),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(19),
            tag_id: TagId(3211624224),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(20),
            tag_id: TagId(3456350885),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(21),
            tag_id: TagId(3456350885),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(22),
            tag_id: TagId(4166869272),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[2]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(24),
            tag_id: TagId(4166869272),
            unknown_const: 3,
        })))
        .unwrap();

    row_groups[3]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(25),
            tag_id: TagId(3211624224),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(25),
            tag_id: TagId(3216792858),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(29),
            tag_id: TagId(2498240426),
            unknown_const: 3,
        })))
        .unwrap();
    row_groups[3]
        .add_row(Row::Ext(ExtRow::TrackTag(TrackTag {
            track_id: TrackId(29),
            tag_id: TagId(598441108),
            unknown_const: 3,
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(10),
        page_type: PageType::Ext(ExtPageType::TrackTag),
        next_page: PageIndex(21),
        unknown1: 54,
        unknown2: 0,
        num_rows_small: 52,
        unknown3: 128,
        unknown4: 6,
        page_flags: PageFlags(36),
        free_size: 3104,
        used_size: 832,
        unknown5: 1,
        num_rows_large: 51,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/track_tag_page.bin"),
        page,
        (page_size, DatabaseType::Ext),
        (page_size, DatabaseType::Ext),
    );
}

/* TODO the CDJ-350 seems to create a HistoryPlaylists page for each row.
Find a player that properly fills a page and improve this test. */
#[test]
fn history_playlists_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        1
    ];
    row_groups[0].unknown = 1;

    row_groups[0]
        .add_row(Row::Plain(PlainRow::HistoryPlaylist(HistoryPlaylist {
            id: HistoryPlaylistId(1),
            name: "HISTORY 001".parse().unwrap(),
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(24),
        page_type: PageType::Plain(PlainPageType::HistoryPlaylists),
        next_page: PageIndex(59),
        unknown1: 240,
        unknown2: 0,
        num_rows_small: 1,
        unknown3: 32,
        unknown4: 0,
        page_flags: PageFlags(36),
        free_size: 4034,
        used_size: 16,
        unknown5: 1,
        num_rows_large: 0,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/history_playlists_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}

// TODO improve the test with a fuller page
#[test]
fn history_entries_page() {
    let mut row_groups = vec![
        RowGroup {
            row_offsets: Default::default(),
            row_presence_flags: 0,
            unknown: 0,
            rows: vec![],
        };
        1
    ];
    row_groups[0].unknown = 64;

    row_groups[0]
        .add_row(Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
            track_id: TrackId(35),
            playlist_id: HistoryPlaylistId(2),
            entry_index: 1,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
            track_id: TrackId(18),
            playlist_id: HistoryPlaylistId(2),
            entry_index: 2,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
            track_id: TrackId(25),
            playlist_id: HistoryPlaylistId(2),
            entry_index: 3,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
            track_id: TrackId(5),
            playlist_id: HistoryPlaylistId(2),
            entry_index: 4,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
            track_id: TrackId(12),
            playlist_id: HistoryPlaylistId(2),
            entry_index: 5,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
            track_id: TrackId(19),
            playlist_id: HistoryPlaylistId(2),
            entry_index: 6,
        })))
        .unwrap();
    row_groups[0]
        .add_row(Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
            track_id: TrackId(6),
            playlist_id: HistoryPlaylistId(2),
            entry_index: 7,
        })))
        .unwrap();

    let page = Page {
        page_index: PageIndex(60),
        page_type: PageType::Plain(PlainPageType::HistoryEntries),
        next_page: PageIndex(62),
        unknown1: 254,
        unknown2: 0,
        num_rows_small: 7,
        unknown3: 224,
        unknown4: 0,
        page_flags: PageFlags(36),
        free_size: 3954,
        used_size: 84,
        unknown5: 1,
        num_rows_large: 6,
        unknown6: 0,
        unknown7: 0,
        row_groups,
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/history_entries_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size, DatabaseType::Plain),
    );
}
