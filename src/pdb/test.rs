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
use crate::util::{ColorIndex, FileType};
use std::collections::BTreeMap;
use std::num::NonZero;

#[test]
fn empty_header() {
    let header = Header {
        page_size: 4096,
        num_tables: 0,
        next_unused_page: PageIndex::try_from(1).unwrap(),
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
        num_tables: 20,
        next_unused_page: PageIndex::try_from(51).unwrap(),
        unknown: 5,
        sequence: 34,
        tables: [
            Table {
                page_type: PageType::Plain(PlainPageType::Tracks),
                empty_candidate: 47,
                first_page: PageIndex::try_from(1).unwrap(),
                last_page: PageIndex::try_from(2).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Genres),
                empty_candidate: 4,
                first_page: PageIndex::try_from(3).unwrap(),
                last_page: PageIndex::try_from(3).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Artists),
                empty_candidate: 49,
                first_page: PageIndex::try_from(5).unwrap(),
                last_page: PageIndex::try_from(6).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Albums),
                empty_candidate: 8,
                first_page: PageIndex::try_from(7).unwrap(),
                last_page: PageIndex::try_from(7).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Labels),
                empty_candidate: 50,
                first_page: PageIndex::try_from(9).unwrap(),
                last_page: PageIndex::try_from(10).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Keys),
                empty_candidate: 46,
                first_page: PageIndex::try_from(11).unwrap(),
                last_page: PageIndex::try_from(12).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Colors),
                empty_candidate: 42,
                first_page: PageIndex::try_from(13).unwrap(),
                last_page: PageIndex::try_from(14).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::PlaylistTree),
                empty_candidate: 16,
                first_page: PageIndex::try_from(15).unwrap(),
                last_page: PageIndex::try_from(15).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::PlaylistEntries),
                empty_candidate: 18,
                first_page: PageIndex::try_from(17).unwrap(),
                last_page: PageIndex::try_from(17).unwrap(),
            },
            Table {
                page_type: PageType::Unknown(9),
                empty_candidate: 20,
                first_page: PageIndex::try_from(19).unwrap(),
                last_page: PageIndex::try_from(19).unwrap(),
            },
            Table {
                page_type: PageType::Unknown(10),
                empty_candidate: 22,
                first_page: PageIndex::try_from(21).unwrap(),
                last_page: PageIndex::try_from(21).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::HistoryPlaylists),
                empty_candidate: 24,
                first_page: PageIndex::try_from(23).unwrap(),
                last_page: PageIndex::try_from(23).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::HistoryEntries),
                empty_candidate: 26,
                first_page: PageIndex::try_from(25).unwrap(),
                last_page: PageIndex::try_from(25).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Artwork),
                empty_candidate: 28,
                first_page: PageIndex::try_from(27).unwrap(),
                last_page: PageIndex::try_from(27).unwrap(),
            },
            Table {
                page_type: PageType::Unknown(14),
                empty_candidate: 30,
                first_page: PageIndex::try_from(29).unwrap(),
                last_page: PageIndex::try_from(29).unwrap(),
            },
            Table {
                page_type: PageType::Unknown(15),
                empty_candidate: 32,
                first_page: PageIndex::try_from(31).unwrap(),
                last_page: PageIndex::try_from(31).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Columns),
                empty_candidate: 43,
                first_page: PageIndex::try_from(33).unwrap(),
                last_page: PageIndex::try_from(34).unwrap(),
            },
            Table {
                page_type: PageType::Plain(PlainPageType::Menu),
                empty_candidate: 44,
                first_page: PageIndex::try_from(35).unwrap(),
                last_page: PageIndex::try_from(36).unwrap(),
            },
            Table {
                page_type: PageType::Unknown(18),
                empty_candidate: 45,
                first_page: PageIndex::try_from(37).unwrap(),
                last_page: PageIndex::try_from(38).unwrap(),
            },
            Table {
                // page_type: PageType::Plain(PlainPageType::History),
                page_type: PageType::Unknown(19),
                empty_candidate: 48,
                first_page: PageIndex::try_from(39).unwrap(),
                last_page: PageIndex::try_from(41).unwrap(),
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
        index_shift: 0x0000,
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
        file_type: FileType::Mp3,
        offsets: OffsetArrayContainer {
            offsets: [
                3u16, 136, 137, 138, 140, 142, 143, 144, 145, 148, 149, 150, 161, 162, 163, 164,
                208, 219, 249, 262, 263, 280,
            ]
            .into(),
            inner: TrackStrings {
                isrc: DeviceSQLString::new_isrc("".to_string()).unwrap(),
                lyricist: DeviceSQLString::empty(),
                unknown_string2: "3".parse().unwrap(),
                unknown_string3: "3".parse().unwrap(),
                unknown_string4: DeviceSQLString::empty(),
                message: DeviceSQLString::empty(),
                publish_track_information: DeviceSQLString::empty(),
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
    };
    test_roundtrip(
        &[
            36, 0, 0, 0, 0, 7, 12, 0, 68, 172, 0, 0, 0, 0, 0, 0, 168, 71, 105, 0, 218, 177, 193,
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
        index_shift: 0x0000,
        id: ArtistId(1),
        offsets: OffsetArrayContainer {
            offsets: [3u8, 10u8].into(),
            inner: TrailingName {
                name: "Loopmasters".parse().unwrap(),
            },
        },
    };
    test_roundtrip(
        &[
            96, 0, 0, 0, 1, 0, 0, 0, 3, 10, 25, 76, 111, 111, 112, 109, 97, 115, 116, 101, 114, 115,
        ],
        row,
    );
}

#[test]
fn album_row() {
    let row1 = Album {
        subtype: Subtype(0x80),
        index_shift: 0x0000,
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
    };

    test_roundtrip(
        &[
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x16, 0x15, 0x47, 0x4f, 0x4f, 0x44, 0x20,
            0x4c, 0x55, 0x43, 0x4b,
        ],
        row1,
    );
    let row2 = Album {
        subtype: Subtype(0x80),
        index_shift: 0x0000,
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
    };

    test_roundtrip(
        &[
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00,
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
fn menu_row() {
    let row = Menu {
        category_id: 2,
        content_pointer: 2,
        unknown: 2,
        visibility: MenuVisibility::Visible,
        sort_order: 1,
    };
    let bin = &[0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x01, 0x00];
    test_roundtrip(bin, row);
}

#[test]
fn track_page() {
    let row_groups = vec![RowGroup {
        row_offsets: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x054c, 0x03fc, 0x02ac, 0x0150, 0x0000,
        ],
        row_presence_flags: 0x001f,
        unknown: 0x0010,
    }];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Track(Track {
                subtype: Subtype(0x24),
                index_shift: 0x0000,
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
                file_type: FileType::Wav,
                offsets: OffsetArrayContainer {
                    offsets: [
                        0x03u16, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97, 0x98,
                        0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xde, 0xe4, 0xe5, 0xef,
                    ]
                    .into(),
                    inner: TrackStrings {
                        isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                        lyricist: "".parse().unwrap(),
                        unknown_string2: "1".parse().unwrap(),
                        unknown_string3: "1".parse().unwrap(),
                        unknown_string4: "".parse().unwrap(),
                        message: "".parse().unwrap(),
                        publish_track_information: "ON".parse().unwrap(),
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
            })),
        ),
        (
            0x0150,
            Row::Plain(PlainRow::Track(Track {
                subtype: Subtype(0x24),
                index_shift: 0x0020,
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
                file_type: FileType::Wav,
                offsets: OffsetArrayContainer {
                    offsets: [
                        0x03u16, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97, 0x98,
                        0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xde, 0xe7, 0xe8, 0xf5,
                    ]
                    .into(),
                    inner: TrackStrings {
                        isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                        lyricist: "".parse().unwrap(),
                        unknown_string2: "1".parse().unwrap(),
                        unknown_string3: "1".parse().unwrap(),
                        unknown_string4: "".parse().unwrap(),
                        message: "".parse().unwrap(),
                        publish_track_information: "ON".parse().unwrap(),
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
            })),
        ),
        (
            0x02ac,
            Row::Plain(PlainRow::Track(Track {
                subtype: Subtype(0x24),
                index_shift: 0x0040,
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
                file_type: FileType::Wav,
                offsets: OffsetArrayContainer {
                    offsets: [
                        0x03u16, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97, 0x98,
                        0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xde, 0xe4, 0xe5, 0xef,
                    ]
                    .into(),
                    inner: TrackStrings {
                        isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                        lyricist: "".parse().unwrap(),
                        unknown_string2: "1".parse().unwrap(),
                        unknown_string3: "1".parse().unwrap(),
                        unknown_string4: "".parse().unwrap(),
                        message: "".parse().unwrap(),
                        publish_track_information: "ON".parse().unwrap(),
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
            })),
        ),
        (
            0x03fc,
            Row::Plain(PlainRow::Track(Track {
                subtype: Subtype(0x24),
                index_shift: 0x0060,
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
                file_type: FileType::Wav,
                offsets: OffsetArrayContainer {
                    offsets: [
                        0x03u16, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97, 0x98,
                        0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xde, 0xe3, 0xe4, 0xed,
                    ]
                    .into(),
                    inner: TrackStrings {
                        isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                        lyricist: "".parse().unwrap(),
                        unknown_string2: "1".parse().unwrap(),
                        unknown_string3: "1".parse().unwrap(),
                        unknown_string4: "".parse().unwrap(),
                        message: "".parse().unwrap(),
                        publish_track_information: "ON".parse().unwrap(),
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
            })),
        ),
        (
            0x054c,
            Row::Plain(PlainRow::Track(Track {
                subtype: Subtype(0x24),
                index_shift: 0x0080,
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
                file_type: FileType::Mp3,
                offsets: OffsetArrayContainer {
                    offsets: [
                        0x03u16, 0x88, 0x89, 0x8a, 0x8c, 0x8e, 0x8f, 0x90, 0x93, 0x96, 0x97, 0x98,
                        0xa3, 0xa4, 0xa5, 0xa6, 0xd2, 0xdd, 0xfb, 0x108, 0x109, 0x11a,
                    ]
                    .into(),
                    inner: TrackStrings {
                        isrc: DeviceSQLString::new_isrc(String::new()).unwrap(),
                        lyricist: "".parse().unwrap(),
                        unknown_string2: "1".parse().unwrap(),
                        unknown_string3: "1".parse().unwrap(),
                        unknown_string4: "".parse().unwrap(),
                        message: "".parse().unwrap(),
                        publish_track_information: "ON".parse().unwrap(),
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
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(2).unwrap(),
            page_type: PageType::Plain(PlainPageType::Tracks),
            next_page: PageIndex::try_from(46).unwrap(),
            unknown1: 12,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(5)
                .with_num_rows_valid(5),
            page_flags: PageFlags(36),
            free_size: 2302,
            used_size: 1740,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 4,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size: u32 = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/track_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn genres_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x0178, 0x0168, 0x014c, 0x0138, 0x0120, 0x0104, 0x00e8, 0x00cc, 0x00b4, 0x0098,
                0x0078, 0x0060, 0x004c, 0x0030, 0x001c, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01a8, 0x0190],
            row_presence_flags: 0x0003,
            unknown: 0x0002, // This is different from the usual
        },
    ];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(168),
                name: "#techno #deep #beatdown".parse().unwrap(),
            })),
        ),
        (
            0x001c,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(169),
                name: "#broken #deep".parse().unwrap(),
            })),
        ),
        (
            0x0030,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(170),
                name: "#deep #techno #beatdown".parse().unwrap(),
            })),
        ),
        (
            0x004c,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(171),
                name: "#stepping #deep".parse().unwrap(),
            })),
        ),
        (
            0x0060,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(172),
                name: "#deep #beatdown ".parse().unwrap(),
            })),
        ),
        (
            0x0078,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(173),
                name: "#beatdown #stepping #deep".parse().unwrap(),
            })),
        ),
        (
            0x0098,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(174),
                name: "#techno #dub #beatdown".parse().unwrap(),
            })),
        ),
        (
            0x00b4,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(175),
                name: "#techno #dub #deep".parse().unwrap(),
            })),
        ),
        (
            0x00cc,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(176),
                name: "#beatdown #oldschool".parse().unwrap(),
            })),
        ),
        (
            0x00e8,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(177),
                name: "#techno #beatin #deep".parse().unwrap(), // codespell:ignore beatin
            })),
        ),
        (
            0x0104,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(178),
                name: "#beatdown #house #DEEP".parse().unwrap(),
            })),
        ),
        (
            0x0120,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(179),
                name: "Minimal / Deep Tech".parse().unwrap(),
            })),
        ),
        (
            0x0138,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(180),
                name: "#sunth #techno".parse().unwrap(),
            })),
        ),
        (
            0x014c,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(181),
                name: "#classic #beatdown   ".parse().unwrap(),
            })),
        ),
        (
            0x0168,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(182),
                name: "#beatdown)".parse().unwrap(),
            })),
        ),
        (
            0x0178,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(183),
                name: "#house #oldschool".parse().unwrap(),
            })),
        ),
        (
            0x0190,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(184),
                name: "#beatdown #synth ".parse().unwrap(),
            })),
        ),
        (
            0x01a8,
            Row::Plain(PlainRow::Genre(Genre {
                id: GenreId(185),
                name: "#techno #deep #intro".parse().unwrap(),
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(48).unwrap(),
            page_type: PageType::Plain(PlainPageType::Genres),
            next_page: PageIndex::try_from(449).unwrap(),
            unknown1: 14405,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(18)
                .with_num_rows_valid(18),
            page_flags: PageFlags(36),
            free_size: 3560,
            used_size: 452,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 17,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/genres_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn artists_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x01d4, 0x01a8, 0x018c, 0x0174, 0x0158, 0x0140, 0x0120, 0x00fc, 0x00d8, 0x00b4,
                0x009c, 0x007c, 0x0064, 0x0040, 0x0020, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x03dc, 0x03b4, 0x039c, 0x0388, 0x036c, 0x034c, 0x032c, 0x02f4, 0x02d8, 0x02bc,
                0x029c, 0x0280, 0x025c, 0x0238, 0x0218, 0x01f8,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x05d4, 0x05a8, 0x0590, 0x0578, 0x0560, 0x0540, 0x0524, 0x04fc, 0x04e0, 0x04cc,
                0x04b0, 0x0494, 0x0478, 0x0440, 0x0420, 0x0400,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x07c4, 0x07a8, 0x0794, 0x0774, 0x0754, 0x0738, 0x0710, 0x06ec, 0x06d4, 0x06b0,
                0x068c, 0x0674, 0x0650, 0x062c, 0x0610, 0x05f0,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x09b8, 0x09a0, 0x0984, 0x0964, 0x0930, 0x0908, 0x08f0, 0x08d8, 0x08c0, 0x0894,
                0x0874, 0x0858, 0x083c, 0x081c, 0x07fc, 0x07e4,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0b94, 0x0b74, 0x0b54, 0x0b34, 0x0b14, 0x0ae8, 0x0ad0, 0x0ab0, 0x0a94, 0x0a7c,
                0x0a60, 0x0a40, 0x0a1c, 0x0a00, 0x09e8, 0x09d0,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0da0, 0x0d84, 0x0d60, 0x0d34, 0x0d08, 0x0cec, 0x0cc8, 0x0ca8, 0x0c8c, 0x0c6c,
                0x0c4c, 0x0c30, 0x0c14, 0x0bf4, 0x0bd4, 0x0bb4,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0, 0, 0, 0, 0, 0, 0, 0, 0x0ea4, 0x0e90, 0x0e74, 0x0e48, 0x0e1c, 0x0e00, 0x0dd8,
                0x0db8,
            ],
            row_presence_flags: 0x00ff,
            unknown: 0x0080,
        },
    ];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0000,
                id: ArtistId(1),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Andreas Gehm".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0020,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0020,
                id: ArtistId(2),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "D'marc Cantu".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0040,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0040,
                id: ArtistId(3),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "DJ Plant Texture".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0064,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0060,
                id: ArtistId(4),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "DVS1".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x007c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0080,
                id: ArtistId(5),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Florian Kupfer".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x009c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x00a0,
                id: ArtistId(6),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Frak".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x00b4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x00c0,
                id: ArtistId(7),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Frankie Knuckles".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x00d8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x00e0,
                id: ArtistId(8),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "House Of Jezebel".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x00fc,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0100,
                id: ArtistId(9),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Innerspace Halflife".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0120,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0120,
                id: ArtistId(10),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "James T. Cotton".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0140,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0140,
                id: ArtistId(11),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "jozef k".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0158,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0160,
                id: ArtistId(12),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Juanpablo".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0174,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0180,
                id: ArtistId(13),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Juniper".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x018c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x01a0,
                id: ArtistId(14),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Kovyazin D".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x01a8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x01c0,
                id: ArtistId(15),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Le Melange Inc. Ft China".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x01d4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x01e0,
                id: ArtistId(16),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Louis Guilliaume".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x01f8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0200,
                id: ArtistId(17),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Maxwell Church".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0218,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0220,
                id: ArtistId(18),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Various Artists".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0238,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0240,
                id: ArtistId(19),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Mutant Beat Dance".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x025c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0260,
                id: ArtistId(20),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Mutant beat dance".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0280,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0280,
                id: ArtistId(21),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Ron Trent".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x029c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x02a0,
                id: ArtistId(22),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Salvation REMIX".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x02bc,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x02c0,
                id: ArtistId(23),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Salvation".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x02d8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x02e0,
                id: ArtistId(24),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Simoncino".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x02f4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0300,
                id: ArtistId(25),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "HOTMIX RECORDS / NICK ANTHONY SIMONCINO".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x032c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0320,
                id: ArtistId(26),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Sneaker REMIX".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x034c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0340,
                id: ArtistId(27),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Tinman REMIX".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x036c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0360,
                id: ArtistId(28),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Alienata".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0388,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0380,
                id: ArtistId(29),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "AS1".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x039c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x03a0,
                id: ArtistId(30),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "DJ Hell".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x03b4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x03c0,
                id: ArtistId(31),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Innershades & Robert D".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x03dc,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x03e0,
                id: ArtistId(32),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "intersterllar funk".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0400,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0400,
                id: ArtistId(33),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Kyle Hall, KMFH".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0420,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0420,
                id: ArtistId(34),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Luke's Anger".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0440,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0440,
                id: ArtistId(35),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 12u8].into(),
                    inner: TrailingName {
                        name: "Manie Sans Délire".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0478,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0460,
                id: ArtistId(36),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Paul du Lac".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0494,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0480,
                id: ArtistId(37),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Ron Hardy".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x04b0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x04a0,
                id: ArtistId(38),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Saturn V".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x04cc,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x04c0,
                id: ArtistId(39),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "VA".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x04e0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x04e0,
                id: ArtistId(40),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "traxx   ".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x04fc,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0500,
                id: ArtistId(41),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "traxx feat Naughty wood".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0524,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0520,
                id: ArtistId(42),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Truncate ".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0540,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0540,
                id: ArtistId(43),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Ultrastation".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0560,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0560,
                id: ArtistId(44),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "2AM/FM".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0578,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0580,
                id: ArtistId(45),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Sepehr".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0590,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x05a0,
                id: ArtistId(46),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Cfade".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x05a8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x05c0,
                id: ArtistId(47),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Miss Kittin & The Hacker".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x05d4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x05e0,
                id: ArtistId(48),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Paul Du Lac".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x05f0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0600,
                id: ArtistId(49),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Tyree Cooper".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0610,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0620,
                id: ArtistId(50),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Elbee Bad".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x062c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0640,
                id: ArtistId(51),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "The Prince of Dance".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0650,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0660,
                id: ArtistId(52),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Body Beat Ritual".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0674,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0680,
                id: ArtistId(53),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Nehuen".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x068c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x06a0,
                id: ArtistId(54),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "TRAXX Saturn V & X2".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x06b0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x06c0,
                id: ArtistId(55),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Broken English Club".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x06d4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x06e0,
                id: ArtistId(56),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "terrace".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x06ec,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0700,
                id: ArtistId(57),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Byron The Aquarius".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0710,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0720,
                id: ArtistId(58),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Konstantin Tschechow".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0738,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0740,
                id: ArtistId(59),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Romansoff".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0754,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0760,
                id: ArtistId(60),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "D'Marc Cantu".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0774,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0780,
                id: ArtistId(61),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "SvengalisGhost".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0794,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x07a0,
                id: ArtistId(62),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "X2".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x07a8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x07c0,
                id: ArtistId(63),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Cardopusher".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x07c4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x07e0,
                id: ArtistId(64),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Steven Julien".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x07e4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0800,
                id: ArtistId(65),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Advent".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x07fc,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0820,
                id: ArtistId(66),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Aleksi Perala".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x081c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0840,
                id: ArtistId(67),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Andre Kronert".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x083c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0860,
                id: ArtistId(68),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Andy Stott".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0858,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0880,
                id: ArtistId(69),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "ANOPOLIS".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0874,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x08a0,
                id: ArtistId(70),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Anthony Rother".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0894,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x08c0,
                id: ArtistId(71),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Anthony Rother UNRELEASED".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x08c0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x08e0,
                id: ArtistId(72),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Area".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x08d8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0900,
                id: ArtistId(73),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Aubrey".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x08f0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0920,
                id: ArtistId(74),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Audion".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0908,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0940,
                id: ArtistId(75),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Audion - Black Strobe".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0930,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0960,
                id: ArtistId(76),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Cari Lekebusch & Jesper Dahlback".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0964,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0980,
                id: ArtistId(77),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Claro Intelecto".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0984,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x09a0,
                id: ArtistId(78),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Conforce".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x09a0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x09c0,
                id: ArtistId(79),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "CT Trax".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x09b8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x09e0,
                id: ArtistId(80),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "D-56m".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x09d0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0a00,
                id: ArtistId(81),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Deniro".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x09e8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0a20,
                id: ArtistId(82),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "DJ QU".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0a00,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0a40,
                id: ArtistId(83),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "DJ Qu REMIX".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0a1c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0a60,
                id: ArtistId(84),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Don williams remix".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0a40,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0a80,
                id: ArtistId(85),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Don Williams".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0a60,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0aa0,
                id: ArtistId(86),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Dustmite".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0a7c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0ac0,
                id: ArtistId(87),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "DVS1 ".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0a94,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0ae0,
                id: ArtistId(88),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "DVS1 tesT".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0ab0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0b00,
                id: ArtistId(89),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Emmanuel Top".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0ad0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0b20,
                id: ArtistId(90),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Erika".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0ae8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0b40,
                id: ArtistId(91),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Jensen Interceptor REMIX ".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0b14,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0b60,
                id: ArtistId(92),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Jeroen Search".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0b34,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0b80,
                id: ArtistId(93),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Juho Kahilainen".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0b54,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0ba0,
                id: ArtistId(94),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Juxta Position".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0b74,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0bc0,
                id: ArtistId(95),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Kenny Larkin".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0b94,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0be0,
                id: ArtistId(96),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Kirill Mamin".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0bb4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0c00,
                id: ArtistId(97),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "L.B. Dub Corp".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0bd4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0c20,
                id: ArtistId(98),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Levon Vincent".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0bf4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0c40,
                id: ArtistId(99),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "LEVON VINCENT".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0c14,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0c60,
                id: ArtistId(100),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Lil Tony".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0c30,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0c80,
                id: ArtistId(101),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Malin Genie".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0c4c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0ca0,
                id: ArtistId(102),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Marcel Dettmann".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0c6c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0cc0,
                id: ArtistId(103),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Marco Bernardi".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0c8c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0ce0,
                id: ArtistId(104),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Mary Velo".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0ca8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0d00,
                id: ArtistId(105),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Mike Dearborn".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0cc8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0d20,
                id: ArtistId(106),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Mike Dunn JU EDIT".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0cec,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0d40,
                id: ArtistId(107),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Nina Kraviz".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0d08,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0d60,
                id: ArtistId(108),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Obsolete Music Technology".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0d34,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0d80,
                id: ArtistId(109),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Oliver Deutschmann REMIX".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0d60,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0da0,
                id: ArtistId(110),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Oliver Deutschmann".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0d84,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0dc0,
                id: ArtistId(111),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Oliver Kapp".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0da0,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0de0,
                id: ArtistId(112),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Pacou".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0db8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0e00,
                id: ArtistId(113),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Patrik Carrera".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0dd8,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0e20,
                id: ArtistId(114),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Patrik Carrera (GER)".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0e00,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0e40,
                id: ArtistId(115),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Phil Kieran".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0e1c,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0e60,
                id: ArtistId(116),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Planetary Assault Systems".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0e48,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0e80,
                id: ArtistId(117),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Planetary Assault Systems ".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0e74,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0ea0,
                id: ArtistId(118),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Plastikman".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0e90,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0ec0,
                id: ArtistId(119),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "QNA".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0ea4,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0ee0,
                id: ArtistId(120),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Radial".parse().unwrap(),
                    },
                },
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(6).unwrap(),
            page_type: PageType::Plain(PlainPageType::Artists),
            next_page: PageIndex::try_from(47).unwrap(),
            unknown1: 855,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(120)
                .with_num_rows_valid(120),
            page_flags: PageFlags(36),
            free_size: 12,
            used_size: 3772,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 119,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/artists_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn artist_page_long() {
    use std::iter::repeat_n;
    let row_groups = vec![RowGroup {
        row_offsets: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0374, 0x0260, 0x0244, 0x0130, 0x0114, 0x0000,
        ],
        row_presence_flags: 0x003f,
        unknown: 0x0020,
    }];
    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x64),
                index_shift: 0x0000,
                id: ArtistId(1),
                offsets: OffsetArrayContainer {
                    offsets: [3u16, 12u16].into(),
                    inner: TrailingName {
                        name: repeat_n('D', 256).collect::<String>().parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0114,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0020,
                id: ArtistId(2),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Insert 2".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0130,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x64),
                index_shift: 0x0040,
                id: ArtistId(3),
                offsets: OffsetArrayContainer {
                    offsets: [3u16, 12u16].into(),
                    inner: TrailingName {
                        name: repeat_n('C', 256).collect::<String>().parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0244,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x60),
                index_shift: 0x0060,
                id: ArtistId(4),
                offsets: OffsetArrayContainer {
                    offsets: [3u8, 10u8].into(),
                    inner: TrailingName {
                        name: "Insert 1".parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0260,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x64),
                index_shift: 0x0080,
                id: ArtistId(5),
                offsets: OffsetArrayContainer {
                    offsets: [3u16, 12u16].into(),
                    inner: TrailingName {
                        name: repeat_n('B', 254).collect::<String>().parse().unwrap(),
                    },
                },
            })),
        ),
        (
            0x0374,
            Row::Plain(PlainRow::Artist(Artist {
                subtype: Subtype(0x64),
                index_shift: 0x00a0,
                id: ArtistId(6),
                offsets: OffsetArrayContainer {
                    offsets: [3u16, 12u16].into(),
                    inner: TrailingName {
                        name: repeat_n('❤', 256).collect::<String>().parse().unwrap(),
                    },
                },
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(6).unwrap(),
            page_type: PageType::Plain(PlainPageType::Artists),
            next_page: PageIndex::try_from(46).unwrap(),
            unknown1: 16,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(6)
                .with_num_rows_valid(6),
            page_flags: PageFlags(36),
            free_size: 2624,
            used_size: 1416,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 5,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/artist_page_long.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn albums_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x0310, 0x02d8, 0x02a8, 0x024c, 0x0228, 0x01f0, 0x01c0, 0x0194, 0x016c, 0x0140,
                0x010c, 0x00d8, 0x00ac, 0x007c, 0x0030, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0608, 0x05b4, 0x058c, 0x0560, 0x0530, 0x0508, 0x04d8, 0x04b4, 0x0474, 0x0440,
                0x0418, 0x03e8, 0x03bc, 0x0394, 0x036c, 0x0338,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x08cc, 0x0898, 0x0874, 0x0828, 0x0804, 0x07d8, 0x07b0, 0x078c, 0x0754, 0x0730,
                0x0708, 0x06e8, 0x06b8, 0x068c, 0x065c, 0x0630,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0bbc, 0x0b80, 0x0b5c, 0x0b34, 0x0b10, 0x0ae4, 0x0ab4, 0x0a8c, 0x0a38, 0x0a10,
                0x09d8, 0x09ac, 0x0970, 0x0944, 0x091c, 0x08f4,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0e80, 0x0e50, 0x0e24, 0x0df8, 0x0dc8, 0x0d90, 0x0d68, 0x0d3c, 0x0d14, 0x0cf0,
                0x0cc8, 0x0c9c, 0x0c78, 0x0c44, 0x0c1c, 0x0bf8,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0ed0, 0x0eac],
            row_presence_flags: 0x0003,
            unknown: 0x0002,
        },
    ];
    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0000,
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
            })),
        ),
        (
            0x0030,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0020,
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
            })),
        ),
        (
            0x007c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0040,
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
            })),
        ),
        (
            0x00ac,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0060,
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
            })),
        ),
        (
            0x00d8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0080,
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
            })),
        ),
        (
            0x010c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x00a0,
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
            })),
        ),
        (
            0x0140,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x00c0,
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
            })),
        ),
        (
            0x016c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x00e0,
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
            })),
        ),
        (
            0x0194,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0100,
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
            })),
        ),
        (
            0x01c0,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0120,
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
            })),
        ),
        (
            0x01f0,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0140,
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
            })),
        ),
        (
            0x0228,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0160,
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
            })),
        ),
        (
            0x024c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0180,
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
            })),
        ),
        (
            0x02a8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x01a0,
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
            })),
        ),
        (
            0x02d8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x01c0,
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
            })),
        ),
        (
            0x0310,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x01e0,
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
            })),
        ),
        (
            0x0338,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0200,
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
            })),
        ),
        (
            0x036c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0220,
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
            })),
        ),
        (
            0x0394,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0240,
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
            })),
        ),
        (
            0x03bc,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0260,
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
            })),
        ),
        (
            0x03e8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0280,
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
            })),
        ),
        (
            0x0418,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x02a0,
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
            })),
        ),
        (
            0x0440,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x02c0,
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
            })),
        ),
        (
            0x0474,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x02e0,
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
            })),
        ),
        (
            0x04b4,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0300,
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
            })),
        ),
        (
            0x04d8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0320,
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
            })),
        ),
        (
            0x0508,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0340,
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
            })),
        ),
        (
            0x0530,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0360,
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
            })),
        ),
        (
            0x0560,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0380,
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
            })),
        ),
        (
            0x058c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x03a0,
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
            })),
        ),
        (
            0x05b4,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x03c0,
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
            })),
        ),
        (
            0x0608,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x03e0,
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
            })),
        ),
        (
            0x0630,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0400,
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
            })),
        ),
        (
            0x065c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0420,
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
            })),
        ),
        (
            0x068c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0440,
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
            })),
        ),
        (
            0x06b8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0460,
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
            })),
        ),
        (
            0x06e8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0480,
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
            })),
        ),
        (
            0x0708,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x04a0,
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
            })),
        ),
        (
            0x0730,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x04c0,
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
            })),
        ),
        (
            0x0754,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x04e0,
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
            })),
        ),
        (
            0x078c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0500,
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
            })),
        ),
        (
            0x07b0,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0520,
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
            })),
        ),
        (
            0x07d8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0540,
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
            })),
        ),
        (
            0x0804,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0560,
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
            })),
        ),
        (
            0x0828,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0580,
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
            })),
        ),
        (
            0x0874,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x05a0,
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
            })),
        ),
        (
            0x0898,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x05c0,
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
            })),
        ),
        (
            0x08cc,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x05e0,
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
            })),
        ),
        (
            0x08f4,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0600,
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
            })),
        ),
        (
            0x091c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0620,
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
            })),
        ),
        (
            0x0944,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0640,
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
            })),
        ),
        (
            0x0970,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0660,
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
            })),
        ),
        (
            0x09ac,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0680,
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
            })),
        ),
        (
            0x09d8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x06a0,
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
            })),
        ),
        (
            0x0a10,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x06c0,
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
            })),
        ),
        (
            0x0a38,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x06e0,
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
            })),
        ),
        (
            0x0a8c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0700,
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
            })),
        ),
        (
            0x0ab4,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0720,
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
            })),
        ),
        (
            0x0ae4,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0740,
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
            })),
        ),
        (
            0x0b10,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0760,
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
            })),
        ),
        (
            0x0b34,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0780,
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
            })),
        ),
        (
            0x0b5c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x07a0,
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
            })),
        ),
        (
            0x0b80,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x07c0,
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
            })),
        ),
        (
            0x0bbc,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x07e0,
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
            })),
        ),
        (
            0x0bf8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0800,
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
            })),
        ),
        (
            0x0c1c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0820,
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
            })),
        ),
        (
            0x0c44,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0840,
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
            })),
        ),
        (
            0x0c78,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0860,
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
            })),
        ),
        (
            0x0c9c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0880,
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
            })),
        ),
        (
            0x0cc8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x08a0,
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
            })),
        ),
        (
            0x0cf0,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x08c0,
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
            })),
        ),
        (
            0x0d14,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x08e0,
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
            })),
        ),
        (
            0x0d3c,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0900,
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
            })),
        ),
        (
            0x0d68,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0920,
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
            })),
        ),
        (
            0x0d90,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0940,
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
            })),
        ),
        (
            0x0dc8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0960,
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
            })),
        ),
        (
            0x0df8,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0980,
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
            })),
        ),
        (
            0x0e24,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x09a0,
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
            })),
        ),
        (
            0x0e50,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x09c0,
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
            })),
        ),
        (
            0x0e80,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x09e0,
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
            })),
        ),
        (
            0x0eac,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0a00,
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
            })),
        ),
        (
            0x0ed0,
            Row::Plain(PlainRow::Album(Album {
                subtype: Subtype(0x80),
                index_shift: 0x0a20,
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
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(8).unwrap(),
            page_type: PageType::Plain(PlainPageType::Albums),
            next_page: PageIndex::try_from(49).unwrap(),
            unknown1: 772,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(82)
                .with_num_rows_valid(82),
            page_flags: PageFlags(36),
            free_size: 36,
            used_size: 3832,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 81,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size: u32 = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/albums_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn labels_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x0118, 0x0108, 0x00f0, 0x00dc, 0x00c8, 0x00c0, 0x00a8, 0x0090, 0x007c, 0x0074,
                0x0068, 0x0058, 0x0034, 0x0028, 0x0014, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0284, 0x026c, 0x025c, 0x024c, 0x0238, 0x021c, 0x0208, 0x01f4, 0x01d0, 0x01bc,
                0x019c, 0x0174, 0x015c, 0x0148, 0x0138, 0x012c,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x03b8, 0x03ac, 0x0394, 0x0378, 0x0364, 0x0344, 0x0334, 0x0324, 0x0318, 0x0300,
                0x02ec, 0x02e0, 0x02d8, 0x02c0, 0x02ac, 0x0298,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x04d0, 0x04c0, 0x04b8, 0x04ac, 0x0498, 0x047c, 0x0468, 0x0458, 0x0444, 0x0430,
                0x041c, 0x040c, 0x03fc, 0x03e8, 0x03dc, 0x03cc,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x05e8, 0x05dc, 0x05d0, 0x05c4, 0x05b0, 0x0598, 0x058c, 0x0580, 0x056c, 0x0554,
                0x0544, 0x0530, 0x0520, 0x0500, 0x04f0, 0x04e4,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0740, 0x072c, 0x0718, 0x070c, 0x06f0, 0x06e4, 0x06d4, 0x06c4, 0x06b0, 0x0674,
                0x0664, 0x064c, 0x0634, 0x061c, 0x060c, 0x05fc,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x087c, 0x0860, 0x0850, 0x0838, 0x0820, 0x0814, 0x0800, 0x07ec, 0x07d8, 0x07c8,
                0x07b4, 0x07a4, 0x078c, 0x0778, 0x0764, 0x074c,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x09c4, 0x09b0, 0x0994, 0x0978, 0x0960, 0x0950, 0x0944, 0x0938, 0x0924, 0x0914,
                0x08f4, 0x08dc, 0x08cc, 0x08b8, 0x08a8, 0x0890,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0b14, 0x0b00, 0x0af4, 0x0adc, 0x0ac8, 0x0ab4, 0x0aa0, 0x0a88, 0x0a6c, 0x0a60,
                0x0a38, 0x0a24, 0x0a18, 0x0a08, 0x09f0, 0x09d8,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0c7c, 0x0c68, 0x0c4c, 0x0c20, 0x0c08, 0x0bf4, 0x0bd4, 0x0bc0, 0x0ba8, 0x0b94,
                0x0b80, 0x0b74, 0x0b60, 0x0b50, 0x0b3c, 0x0b2c,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0e08, 0x0de0, 0x0dc0, 0x0da4, 0x0d8c, 0x0d7c, 0x0d68, 0x0d54, 0x0d2c, 0x0d14,
                0x0cf4, 0x0cdc, 0x0cd0, 0x0cc0, 0x0cac, 0x0c9c,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0e34],
            row_presence_flags: 0x0001,
            unknown: 0x0001,
        },
    ];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(1),
                name: "Solar One Music".parse().unwrap(),
            })),
        ),
        (
            0x0014,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(2),
                name: "Spectral Sound".parse().unwrap(),
            })),
        ),
        (
            0x0028,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(3),
                name: "TENG".parse().unwrap(),
            })),
        ),
        (
            0x0034,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(4),
                name: "Prescription Classic Recordings".parse().unwrap(),
            })),
        ),
        (
            0x0058,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(5),
                name: "Mathematics".parse().unwrap(),
            })),
        ),
        (
            0x0068,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(6),
                name: "Rawax".parse().unwrap(),
            })),
        ),
        (
            0x0074,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(7),
                name: "&nd".parse().unwrap(), //codespell:ignore nd
            })),
        ),
        (
            0x007c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(8),
                name: "Wild Oats Music".parse().unwrap(),
            })),
        ),
        (
            0x0090,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(9),
                name: "Creme Organization".parse().unwrap(),
            })),
        ),
        (
            0x00a8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(10),
                name: "Nobody's Bizzness".parse().unwrap(),
            })),
        ),
        (
            0x00c0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(11),
                name: "ADD".parse().unwrap(),
            })),
        ),
        (
            0x00c8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(12),
                name: "Footage Series".parse().unwrap(),
            })),
        ),
        (
            0x00dc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(13),
                name: "Lone Romantic".parse().unwrap(),
            })),
        ),
        (
            0x00f0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(14),
                name: "Clone Jack For Daze".parse().unwrap(),
            })),
        ),
        (
            0x0108,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(15),
                name: "Rat Life".parse().unwrap(),
            })),
        ),
        (
            0x0118,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(16),
                name: "Classicworks".parse().unwrap(),
            })),
        ),
        (
            0x012c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(17),
                name: "Machine".parse().unwrap(),
            })),
        ),
        (
            0x0138,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(18),
                name: "Modern Love".parse().unwrap(),
            })),
        ),
        (
            0x0148,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(19),
                name: "Transient Force".parse().unwrap(),
            })),
        ),
        (
            0x015c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(20),
                name: "Playlouderecordings".parse().unwrap(),
            })),
        ),
        (
            0x0174,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(21),
                name: "International DeeJay Gigolo Records".parse().unwrap(),
            })),
        ),
        (
            0x019c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(22),
                name: "Strength Music Recordings".parse().unwrap(),
            })),
        ),
        (
            0x01bc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(23),
                name: "Dial Records".parse().unwrap(),
            })),
        ),
        (
            0x01d0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(24),
                name: "Interdimensional Transmissions".parse().unwrap(),
            })),
        ),
        (
            0x01f4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(25),
                name: "Subself Records".parse().unwrap(),
            })),
        ),
        (
            0x0208,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(26),
                name: "Art of Dance".parse().unwrap(),
            })),
        ),
        (
            0x021c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(27),
                name: "Ostgut Ton (Germany)".parse().unwrap(),
            })),
        ),
        (
            0x0238,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(28),
                name: "Innervisions".parse().unwrap(),
            })),
        ),
        (
            0x024c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(29),
                name: "Beatstreet".parse().unwrap(),
            })),
        ),
        (
            0x025c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(30),
                name: "трип".parse().unwrap(),
            })),
        ),
        (
            0x026c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(31),
                name: "Caduceus Records".parse().unwrap(),
            })),
        ),
        (
            0x0284,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(32),
                name: "Stockholm LTD".parse().unwrap(),
            })),
        ),
        (
            0x0298,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(33),
                name: "Planet Rhythm".parse().unwrap(),
            })),
        ),
        (
            0x02ac,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(34),
                name: "Paranoid Dancer".parse().unwrap(),
            })),
        ),
        (
            0x02c0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(35),
                name: "Snork Enterprises".parse().unwrap(),
            })),
        ),
        (
            0x02d8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(36),
                name: "PKR".parse().unwrap(),
            })),
        ),
        (
            0x02e0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(37),
                name: "Mord".parse().unwrap(),
            })),
        ),
        (
            0x02ec,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(38),
                name: "Circus Company".parse().unwrap(),
            })),
        ),
        (
            0x0300,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(39),
                name: "Baalsaal Records".parse().unwrap(),
            })),
        ),
        (
            0x0318,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(40),
                name: "Mephyst".parse().unwrap(),
            })),
        ),
        (
            0x0324,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(41),
                name: "Peacefrog".parse().unwrap(),
            })),
        ),
        (
            0x0334,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(42),
                name: "Kanzleramt".parse().unwrap(),
            })),
        ),
        (
            0x0344,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(43),
                name: "Clone Jack For Daze Series".parse().unwrap(),
            })),
        ),
        (
            0x0364,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(44),
                name: "Chronocircle".parse().unwrap(),
            })),
        ),
        (
            0x0378,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(45),
                name: "Unknown To The Unknown".parse().unwrap(),
            })),
        ),
        (
            0x0394,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(46),
                name: "Super Rhythm Trax".parse().unwrap(),
            })),
        ),
        (
            0x03ac,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(47),
                name: "LABEL".parse().unwrap(),
            })),
        ),
        (
            0x03b8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(48),
                name: "Wilson Records".parse().unwrap(),
            })),
        ),
        (
            0x03cc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(49),
                name: "Houndstooth".parse().unwrap(),
            })),
        ),
        (
            0x03dc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(50),
                name: "WRKTRX".parse().unwrap(),
            })),
        ),
        (
            0x03e8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(51),
                name: "Apotek Records".parse().unwrap(),
            })),
        ),
        (
            0x03fc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(52),
                name: "Figure SPC".parse().unwrap(),
            })),
        ),
        (
            0x040c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(53),
                name: "Rush Hour".parse().unwrap(),
            })),
        ),
        (
            0x041c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(54),
                name: "Wagon Repair".parse().unwrap(),
            })),
        ),
        (
            0x0430,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(55),
                name: "Nada Records".parse().unwrap(),
            })),
        ),
        (
            0x0444,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(56),
                name: "Plus 8 Records".parse().unwrap(),
            })),
        ),
        (
            0x0458,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(57),
                name: "Ostgut Ton".parse().unwrap(),
            })),
        ),
        (
            0x0468,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(58),
                name: "Scion Versions".parse().unwrap(),
            })),
        ),
        (
            0x047c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(59),
                name: "Phil Kieran Recordings".parse().unwrap(),
            })),
        ),
        (
            0x0498,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(60),
                name: "Enemy Records".parse().unwrap(),
            })),
        ),
        (
            0x04ac,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(61),
                name: "UNCAGE".parse().unwrap(),
            })),
        ),
        (
            0x04b8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(62),
                name: "RMR".parse().unwrap(),
            })),
        ),
        (
            0x04c0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(63),
                name: "Warok Music".parse().unwrap(),
            })),
        ),
        (
            0x04d0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(64),
                name: "Axis Records".parse().unwrap(),
            })),
        ),
        (
            0x04e4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(65),
                name: "Rekids".parse().unwrap(),
            })),
        ),
        (
            0x04f0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(66),
                name: "GND Records".parse().unwrap(),
            })),
        ),
        (
            0x0500,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(67),
                name: "© Evod Music".parse().unwrap(),
            })),
        ),
        (
            0x0520,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(68),
                name: "Equalized".parse().unwrap(),
            })),
        ),
        (
            0x0530,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(69),
                name: "H-Productions".parse().unwrap(),
            })),
        ),
        (
            0x0544,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(70),
                name: "Drumcode".parse().unwrap(),
            })),
        ),
        (
            0x0554,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(71),
                name: "Eclectic Limited".parse().unwrap(),
            })),
        ),
        (
            0x056c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(72),
                name: "Subject Detroit".parse().unwrap(),
            })),
        ),
        (
            0x0580,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(73),
                name: "Ultra".parse().unwrap(),
            })),
        ),
        (
            0x058c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(74),
                name: "Chiwax".parse().unwrap(),
            })),
        ),
        (
            0x0598,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(75),
                name: "Supervoid Records".parse().unwrap(),
            })),
        ),
        (
            0x05b0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(76),
                name: "Soleil Records".parse().unwrap(),
            })),
        ),
        (
            0x05c4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(77),
                name: "Intacto".parse().unwrap(),
            })),
        ),
        (
            0x05d0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(78),
                name: "AYCB".parse().unwrap(),
            })),
        ),
        (
            0x05dc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(79),
                name: "Token".parse().unwrap(),
            })),
        ),
        (
            0x05e8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(80),
                name: "Purpose Maker".parse().unwrap(),
            })),
        ),
        (
            0x05fc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(81),
                name: "R&S Records".parse().unwrap(),
            })),
        ),
        (
            0x060c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(82),
                name: "Odd Even".parse().unwrap(),
            })),
        ),
        (
            0x061c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(83),
                name: "F Communications".parse().unwrap(),
            })),
        ),
        (
            0x0634,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(84),
                name: "430 West Records".parse().unwrap(),
            })),
        ),
        (
            0x064c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(85),
                name: "From Another Mind".parse().unwrap(),
            })),
        ),
        (
            0x0664,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(86),
                name: "100% Pure".parse().unwrap(),
            })),
        ),
        (
            0x0674,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(87),
                name: "© Dynamic Reflection 2015".parse().unwrap(),
            })),
        ),
        (
            0x06b0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(88),
                name: "Subsist Records".parse().unwrap(),
            })),
        ),
        (
            0x06c4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(89),
                name: "SK BLACK".parse().unwrap(),
            })),
        ),
        (
            0x06d4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(90),
                name: "Prologue".parse().unwrap(),
            })),
        ),
        (
            0x06e4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(91),
                name: "SUB tl".parse().unwrap(),
            })),
        ),
        (
            0x06f0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(92),
                name: "Granulart Recordings".parse().unwrap(),
            })),
        ),
        (
            0x070c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(93),
                name: "Voight".parse().unwrap(), // codespell:ignore
            })),
        ),
        (
            0x0718,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(94),
                name: "Ahrpe Records".parse().unwrap(),
            })),
        ),
        (
            0x072c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(95),
                name: "Ovum Recordings".parse().unwrap(),
            })),
        ),
        (
            0x0740,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(96),
                name: "Corpus".parse().unwrap(),
            })),
        ),
        (
            0x074c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(97),
                name: "Indistinct Approach".parse().unwrap(),
            })),
        ),
        (
            0x0764,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(98),
                name: "Counterchange".parse().unwrap(),
            })),
        ),
        (
            0x0778,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(99),
                name: "Fanzine Records".parse().unwrap(),
            })),
        ),
        (
            0x078c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(100),
                name: "Sandwell District".parse().unwrap(),
            })),
        ),
        (
            0x07a4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(101),
                name: "M_Rec Ltd".parse().unwrap(),
            })),
        ),
        (
            0x07b4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(102),
                name: "Recode Musik".parse().unwrap(),
            })),
        ),
        (
            0x07c8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(103),
                name: "Parabola".parse().unwrap(),
            })),
        ),
        (
            0x07d8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(104),
                name: "Perc Trax Ltd.".parse().unwrap(),
            })),
        ),
        (
            0x07ec,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(105),
                name: "U.K Executes".parse().unwrap(),
            })),
        ),
        (
            0x0800,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(106),
                name: "Cieli Di Orione".parse().unwrap(),
            })),
        ),
        (
            0x0814,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(107),
                name: "Figure".parse().unwrap(),
            })),
        ),
        (
            0x0820,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(108),
                name: "Illegal Alien LTD".parse().unwrap(),
            })),
        ),
        (
            0x0838,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(109),
                name: "Next Week Records".parse().unwrap(),
            })),
        ),
        (
            0x0850,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(110),
                name: "Labrynth".parse().unwrap(), // codespell:ignore
            })),
        ),
        (
            0x0860,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(111),
                name: "Children Of Tomorrow".parse().unwrap(),
            })),
        ),
        (
            0x087c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(112),
                name: "Gynoid Audio".parse().unwrap(),
            })),
        ),
        (
            0x0890,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(113),
                name: "Devotion Records".parse().unwrap(),
            })),
        ),
        (
            0x08a8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(114),
                name: "Gradient ".parse().unwrap(),
            })),
        ),
        (
            0x08b8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(115),
                name: "Bunker Record".parse().unwrap(),
            })),
        ),
        (
            0x08cc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(116),
                name: "Bio Rhythm".parse().unwrap(),
            })),
        ),
        (
            0x08dc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(117),
                name: "Logistic Records".parse().unwrap(),
            })),
        ),
        (
            0x08f4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(118),
                name: "BADs Label Larhon Records".parse().unwrap(),
            })),
        ),
        (
            0x0914,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(119),
                name: "Be As One".parse().unwrap(),
            })),
        ),
        (
            0x0924,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(120),
                name: "Cyclical Tracks".parse().unwrap(),
            })),
        ),
        (
            0x0938,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(121),
                name: "SUB TL".parse().unwrap(),
            })),
        ),
        (
            0x0944,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(122),
                name: "ANAOH".parse().unwrap(),
            })),
        ),
        (
            0x0950,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(123),
                name: "Datapunk".parse().unwrap(),
            })),
        ),
        (
            0x0960,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(124),
                name: "Lobster Theremin".parse().unwrap(),
            })),
        ),
        (
            0x0978,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(125),
                name: "Bass Agenda Recordings".parse().unwrap(),
            })),
        ),
        (
            0x0994,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(126),
                name: "Clone West Coast Series".parse().unwrap(),
            })),
        ),
        (
            0x09b0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(127),
                name: "Tresor Records".parse().unwrap(),
            })),
        ),
        (
            0x09c4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(128),
                name: "Self Reflektion".parse().unwrap(),
            })),
        ),
        (
            0x09d8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(129),
                name: "Hotflush Recordings".parse().unwrap(),
            })),
        ),
        (
            0x09f0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(130),
                name: "made of CONCRETE".parse().unwrap(),
            })),
        ),
        (
            0x0a08,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(131),
                name: "AKKOET LTD".parse().unwrap(),
            })),
        ),
        (
            0x0a18,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(132),
                name: "Zone".parse().unwrap(),
            })),
        ),
        (
            0x0a24,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(133),
                name: "Frigio Records".parse().unwrap(),
            })),
        ),
        (
            0x0a38,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(134),
                name: "International Deejay Gigolo Records".parse().unwrap(),
            })),
        ),
        (
            0x0a60,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(135),
                name: "Cod3 QR".parse().unwrap(),
            })),
        ),
        (
            0x0a6c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(136),
                name: "Central Processing Unit".parse().unwrap(),
            })),
        ),
        (
            0x0a88,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(137),
                name: "Transparent Sound".parse().unwrap(),
            })),
        ),
        (
            0x0aa0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(138),
                name: "Kneaded Pains".parse().unwrap(),
            })),
        ),
        (
            0x0ab4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(139),
                name: "The Third Room".parse().unwrap(),
            })),
        ),
        (
            0x0ac8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(140),
                name: "Allergy Season".parse().unwrap(),
            })),
        ),
        (
            0x0adc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(141),
                name: "Mechatronica Music".parse().unwrap(),
            })),
        ),
        (
            0x0af4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(142),
                name: "Minus".parse().unwrap(),
            })),
        ),
        (
            0x0b00,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(143),
                name: "Space Factory".parse().unwrap(),
            })),
        ),
        (
            0x0b14,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(144),
                name: "Music Man Records".parse().unwrap(),
            })),
        ),
        (
            0x0b2c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(145),
                name: "BCM Records".parse().unwrap(),
            })),
        ),
        (
            0x0b3c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(146),
                name: "Missing Records".parse().unwrap(),
            })),
        ),
        (
            0x0b50,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(147),
                name: "L.I.E.S.".parse().unwrap(),
            })),
        ),
        (
            0x0b60,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(148),
                name: "Sound Signature".parse().unwrap(),
            })),
        ),
        (
            0x0b74,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(149),
                name: "Mozaiku".parse().unwrap(),
            })),
        ),
        (
            0x0b80,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(150),
                name: "Boomstraat 1818".parse().unwrap(),
            })),
        ),
        (
            0x0b94,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(151),
                name: "TH Tar Hallow".parse().unwrap(),
            })),
        ),
        (
            0x0ba8,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(152),
                name: "Rowan Underground".parse().unwrap(),
            })),
        ),
        (
            0x0bc0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(153),
                name: "Rekktor Music".parse().unwrap(),
            })),
        ),
        (
            0x0bd4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(154),
                name: "Nachtstrom Schallplatten".parse().unwrap(),
            })),
        ),
        (
            0x0bf4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(155),
                name: "N&N Records.".parse().unwrap(),
            })),
        ),
        (
            0x0c08,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(156),
                name: "Greta Recordings".parse().unwrap(),
            })),
        ),
        (
            0x0c20,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(157),
                name: "© Jerical Records".parse().unwrap(),
            })),
        ),
        (
            0x0c4c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(158),
                name: "Illegal Alien Records".parse().unwrap(),
            })),
        ),
        (
            0x0c68,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(159),
                name: "KR/LF Records ".parse().unwrap(),
            })),
        ),
        (
            0x0c7c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(160),
                name: "Repetitive Rhythm Research".parse().unwrap(),
            })),
        ),
        (
            0x0c9c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(161),
                name: "Fides Tempo".parse().unwrap(),
            })),
        ),
        (
            0x0cac,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(162),
                name: "Starwork sas".parse().unwrap(),
            })),
        ),
        (
            0x0cc0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(163),
                name: "Blueprint".parse().unwrap(),
            })),
        ),
        (
            0x0cd0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(164),
                name: "Mirage".parse().unwrap(),
            })),
        ),
        (
            0x0cdc,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(165),
                name: "EPMmusic (V-Series)".parse().unwrap(),
            })),
        ),
        (
            0x0cf4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(166),
                name: "© West Rules".parse().unwrap(),
            })),
        ),
        (
            0x0d14,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(167),
                name: "Copyright Control".parse().unwrap(),
            })),
        ),
        (
            0x0d2c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(168),
                name: "© Aquae Sextiae".parse().unwrap(),
            })),
        ),
        (
            0x0d54,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(169),
                name: "Tsunami Records".parse().unwrap(),
            })),
        ),
        (
            0x0d68,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(170),
                name: "Amelie Records".parse().unwrap(),
            })),
        ),
        (
            0x0d7c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(171),
                name: "Hivemind".parse().unwrap(),
            })),
        ),
        (
            0x0d8c,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(172),
                name: "4 Track Recordings".parse().unwrap(),
            })),
        ),
        (
            0x0da4,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(173),
                name: "Exekutive Funktionen".parse().unwrap(),
            })),
        ),
        (
            0x0dc0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(174),
                name: "© Evod Music".parse().unwrap(),
            })),
        ),
        (
            0x0de0,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(175),
                name: "© Consumed Music".parse().unwrap(),
            })),
        ),
        (
            0x0e08,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(176),
                name: "© EP Digital Music".parse().unwrap(),
            })),
        ),
        (
            0x0e34,
            Row::Plain(PlainRow::Label(Label {
                id: LabelId(177),
                name: "Symbolism".parse().unwrap(),
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(10).unwrap(),
            page_type: PageType::Plain(PlainPageType::Labels),
            next_page: PageIndex::try_from(50).unwrap(),
            unknown1: 4627,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(177)
                .with_num_rows_valid(177),
            page_flags: PageFlags(36),
            free_size: 2,
            used_size: 3652,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 176,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/labels_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn keys_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x00dc, 0x00cc, 0x00c0, 0x00b4, 0x00a8, 0x0098, 0x0088, 0x0078, 0x0068, 0x0058,
                0x0048, 0x003c, 0x002c, 0x0020, 0x0010, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x01d0, 0x01c4, 0x01b4, 0x01a8, 0x0198, 0x0188, 0x0178, 0x016c, 0x015c, 0x014c,
                0x013c, 0x012c, 0x011c, 0x010c, 0x00fc, 0x00ec,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x02ac, 0x02a0, 0x0294, 0x0288, 0x0278, 0x026c, 0x025c, 0x0250, 0x0244, 0x0238,
                0x022c, 0x021c, 0x020c, 0x01fc, 0x01f0, 0x01e0,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02e0, 0x02d4, 0x02c8, 0x02b8,
            ],
            row_presence_flags: 0x000f,
            unknown: 0x0008,
        },
    ];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(1),
                id2: 1,
                name: "Emin".parse().unwrap(),
            })),
        ),
        (
            0x0010,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(2),
                id2: 2,
                name: "Fmaj".parse().unwrap(),
            })),
        ),
        (
            0x0020,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(3),
                id2: 3,
                name: "E".parse().unwrap(),
            })),
        ),
        (
            0x002c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(4),
                id2: 4,
                name: "Amin".parse().unwrap(),
            })),
        ),
        (
            0x003c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(5),
                id2: 5,
                name: "2d".parse().unwrap(),
            })),
        ),
        (
            0x0048,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(6),
                id2: 6,
                name: "Bmin".parse().unwrap(),
            })),
        ),
        (
            0x0058,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(7),
                id2: 7,
                name: "Cmin".parse().unwrap(),
            })),
        ),
        (
            0x0068,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(8),
                id2: 8,
                name: "Cmaj".parse().unwrap(),
            })),
        ),
        (
            0x0078,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(9),
                id2: 9,
                name: "Abmin".parse().unwrap(),
            })),
        ),
        (
            0x0088,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(10),
                id2: 10,
                name: "Dmin".parse().unwrap(),
            })),
        ),
        (
            0x0098,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(11),
                id2: 11,
                name: "Gmin".parse().unwrap(),
            })),
        ),
        (
            0x00a8,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(12),
                id2: 12,
                name: "Dm".parse().unwrap(),
            })),
        ),
        (
            0x00b4,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(13),
                id2: 13,
                name: "Am".parse().unwrap(),
            })),
        ),
        (
            0x00c0,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(14),
                id2: 14,
                name: "A#".parse().unwrap(),
            })),
        ),
        (
            0x00cc,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(15),
                id2: 15,
                name: "G#min".parse().unwrap(),
            })),
        ),
        (
            0x00dc,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(16),
                id2: 16,
                name: "A#min".parse().unwrap(),
            })),
        ),
        (
            0x00ec,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(17),
                id2: 17,
                name: "Amaj".parse().unwrap(),
            })),
        ),
        (
            0x00fc,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(18),
                id2: 18,
                name: "Gmaj".parse().unwrap(),
            })),
        ),
        (
            0x010c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(19),
                id2: 19,
                name: "D#min".parse().unwrap(),
            })),
        ),
        (
            0x011c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(20),
                id2: 20,
                name: "Dmaj".parse().unwrap(),
            })),
        ),
        (
            0x012c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(21),
                id2: 21,
                name: "Ebmin".parse().unwrap(),
            })),
        ),
        (
            0x013c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(22),
                id2: 22,
                name: "Emaj".parse().unwrap(),
            })),
        ),
        (
            0x014c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(23),
                id2: 23,
                name: "F#min".parse().unwrap(),
            })),
        ),
        (
            0x015c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(24),
                id2: 24,
                name: "A#maj".parse().unwrap(),
            })),
        ),
        (
            0x016c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(25),
                id2: 25,
                name: "D#".parse().unwrap(),
            })),
        ),
        (
            0x0178,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(26),
                id2: 26,
                name: "Gbmaj".parse().unwrap(),
            })),
        ),
        (
            0x0188,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(27),
                id2: 27,
                name: "D#maj".parse().unwrap(),
            })),
        ),
        (
            0x0198,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(28),
                id2: 28,
                name: "Bmaj".parse().unwrap(),
            })),
        ),
        (
            0x01a8,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(29),
                id2: 29,
                name: "7m".parse().unwrap(),
            })),
        ),
        (
            0x01b4,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(30),
                id2: 30,
                name: "C#min".parse().unwrap(),
            })),
        ),
        (
            0x01c4,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(31),
                id2: 31,
                name: "5m".parse().unwrap(),
            })),
        ),
        (
            0x01d0,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(32),
                id2: 32,
                name: "Dbmaj".parse().unwrap(),
            })),
        ),
        (
            0x01e0,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(33),
                id2: 33,
                name: "Bbmaj".parse().unwrap(),
            })),
        ),
        (
            0x01f0,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(34),
                id2: 34,
                name: "12m".parse().unwrap(),
            })),
        ),
        (
            0x01fc,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(35),
                id2: 35,
                name: "Bbmin".parse().unwrap(),
            })),
        ),
        (
            0x020c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(36),
                id2: 36,
                name: "Fmin".parse().unwrap(),
            })),
        ),
        (
            0x021c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(37),
                id2: 37,
                name: "F#maj".parse().unwrap(),
            })),
        ),
        (
            0x022c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(38),
                id2: 38,
                name: "10m".parse().unwrap(),
            })),
        ),
        (
            0x0238,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(39),
                id2: 39,
                name: "A".parse().unwrap(),
            })),
        ),
        (
            0x0244,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(40),
                id2: 40,
                name: "Bbm".parse().unwrap(),
            })),
        ),
        (
            0x0250,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(41),
                id2: 41,
                name: "C".parse().unwrap(),
            })),
        ),
        (
            0x025c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(42),
                id2: 42,
                name: "Dbmin".parse().unwrap(),
            })),
        ),
        (
            0x026c,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(43),
                id2: 43,
                name: "Gm".parse().unwrap(),
            })),
        ),
        (
            0x0278,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(44),
                id2: 44,
                name: "Gbmin".parse().unwrap(),
            })),
        ),
        (
            0x0288,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(45),
                id2: 45,
                name: "A m".parse().unwrap(),
            })),
        ),
        (
            0x0294,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(46),
                id2: 46,
                name: "3d".parse().unwrap(),
            })),
        ),
        (
            0x02a0,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(47),
                id2: 47,
                name: "7d".parse().unwrap(),
            })),
        ),
        (
            0x02ac,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(48),
                id2: 48,
                name: "F#m".parse().unwrap(),
            })),
        ),
        (
            0x02b8,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(49),
                id2: 49,
                name: "Unknown".parse().unwrap(),
            })),
        ),
        (
            0x02c8,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(50),
                id2: 50,
                name: "Em".parse().unwrap(),
            })),
        ),
        (
            0x02d4,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(51),
                id2: 51,
                name: "Bm".parse().unwrap(),
            })),
        ),
        (
            0x02e0,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(52),
                id2: 52,
                name: "Ab".parse().unwrap(),
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(12).unwrap(),
            page_type: PageType::Plain(PlainPageType::Keys),
            next_page: PageIndex::try_from(51).unwrap(),
            unknown1: 13484,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(52)
                .with_num_rows_valid(52),
            page_flags: PageFlags(36),
            free_size: 3188,
            used_size: 748,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 51,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/keys_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn colors_page() {
    let row_groups = vec![RowGroup {
        row_offsets: [
            0, 0, 0, 0, 0, 0, 0, 0, 0x006c, 0x005c, 0x004c, 0x003c, 0x002c, 0x001c, 0x0010, 0x0000,
        ],
        row_presence_flags: 0x00ff,
        unknown: 0x00ff,
    }];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Color(Color {
                unknown1: 0,
                unknown2: 1,
                color: ColorIndex::Pink,
                unknown3: 0,
                name: "Pink".parse().unwrap(),
            })),
        ),
        (
            0x0010,
            Row::Plain(PlainRow::Color(Color {
                unknown1: 0,
                unknown2: 2,
                color: ColorIndex::Red,
                unknown3: 0,
                name: "Red".parse().unwrap(),
            })),
        ),
        (
            0x001c,
            Row::Plain(PlainRow::Color(Color {
                unknown1: 0,
                unknown2: 3,
                color: ColorIndex::Orange,
                unknown3: 0,
                name: "Orange".parse().unwrap(),
            })),
        ),
        (
            0x002c,
            Row::Plain(PlainRow::Color(Color {
                unknown1: 0,
                unknown2: 4,
                color: ColorIndex::Yellow,
                unknown3: 0,
                name: "Yellow".parse().unwrap(),
            })),
        ),
        (
            0x003c,
            Row::Plain(PlainRow::Color(Color {
                unknown1: 0,
                unknown2: 5,
                color: ColorIndex::Green,
                unknown3: 0,
                name: "Green".parse().unwrap(),
            })),
        ),
        (
            0x004c,
            Row::Plain(PlainRow::Color(Color {
                unknown1: 0,
                unknown2: 6,
                color: ColorIndex::Aqua,
                unknown3: 0,
                name: "Aqua".parse().unwrap(),
            })),
        ),
        (
            0x005c,
            Row::Plain(PlainRow::Color(Color {
                unknown1: 0,
                unknown2: 7,
                color: ColorIndex::Blue,
                unknown3: 0,
                name: "Blue".parse().unwrap(),
            })),
        ),
        (
            0x006c,
            Row::Plain(PlainRow::Color(Color {
                unknown1: 0,
                unknown2: 8,
                color: ColorIndex::Purple,
                unknown3: 0,
                name: "Purple".parse().unwrap(),
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(14).unwrap(),
            page_type: PageType::Plain(PlainPageType::Colors),
            next_page: PageIndex::try_from(42).unwrap(),
            unknown1: 2,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(8)
                .with_num_rows_valid(8),
            page_flags: PageFlags(36),
            free_size: 3912,
            used_size: 124,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 8,
                unknown_not_num_rows_large: 0,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/colors_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn playlist_tree_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x01a4, 0x0188, 0x016c, 0x0150, 0x0134, 0x0118, 0x00fc, 0x00e0, 0x00c4, 0x00a8,
                0x008c, 0x0070, 0x0054, 0x0038, 0x001c, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0, 0, 0, 0, 0, 0x02d8, 0x02bc, 0x02a0, 0x0284, 0x0268, 0x024c, 0x0230, 0x0214,
                0x01f8, 0x01dc, 0x01c0,
            ],
            row_presence_flags: 0x07ff,
            unknown: 0x0400,
        },
    ];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(0),
                unknown: 0,
                sort_order: 0,
                id: PlaylistTreeNodeId(1),
                node_is_folder: 1,
                name: "folderb".parse().unwrap(),
            })),
        ),
        (
            0x001c,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 0,
                id: PlaylistTreeNodeId(2),
                node_is_folder: 0,
                name: "listaz".parse().unwrap(),
            })),
        ),
        (
            0x0038,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 2,
                id: PlaylistTreeNodeId(3),
                node_is_folder: 0,
                name: "listay".parse().unwrap(),
            })),
        ),
        (
            0x0054,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 4,
                id: PlaylistTreeNodeId(4),
                node_is_folder: 0,
                name: "listax".parse().unwrap(),
            })),
        ),
        (
            0x0070,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 6,
                id: PlaylistTreeNodeId(5),
                node_is_folder: 0,
                name: "listaw".parse().unwrap(),
            })),
        ),
        (
            0x008c,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 8,
                id: PlaylistTreeNodeId(6),
                node_is_folder: 0,
                name: "listav".parse().unwrap(),
            })),
        ),
        (
            0x00a8,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 10,
                id: PlaylistTreeNodeId(7),
                node_is_folder: 0,
                name: "listau".parse().unwrap(),
            })),
        ),
        (
            0x00c4,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 12,
                id: PlaylistTreeNodeId(8),
                node_is_folder: 0,
                name: "listat".parse().unwrap(),
            })),
        ),
        (
            0x00e0,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 14,
                id: PlaylistTreeNodeId(9),
                node_is_folder: 0,
                name: "listas".parse().unwrap(),
            })),
        ),
        (
            0x00fc,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 16,
                id: PlaylistTreeNodeId(10),
                node_is_folder: 0,
                name: "listar".parse().unwrap(),
            })),
        ),
        (
            0x0118,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 18,
                id: PlaylistTreeNodeId(11),
                node_is_folder: 0,
                name: "listaq".parse().unwrap(),
            })),
        ),
        (
            0x0134,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 20,
                id: PlaylistTreeNodeId(12),
                node_is_folder: 0,
                name: "listap".parse().unwrap(),
            })),
        ),
        (
            0x0150,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 22,
                id: PlaylistTreeNodeId(13),
                node_is_folder: 0,
                name: "listao".parse().unwrap(),
            })),
        ),
        (
            0x016c,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 24,
                id: PlaylistTreeNodeId(14),
                node_is_folder: 0,
                name: "listan".parse().unwrap(),
            })),
        ),
        (
            0x0188,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 26,
                id: PlaylistTreeNodeId(15),
                node_is_folder: 0,
                name: "listam".parse().unwrap(),
            })),
        ),
        (
            0x01a4,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 28,
                id: PlaylistTreeNodeId(16),
                node_is_folder: 0,
                name: "listak".parse().unwrap(),
            })),
        ),
        (
            0x01c0,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 30,
                id: PlaylistTreeNodeId(17),
                node_is_folder: 0,
                name: "listal".parse().unwrap(),
            })),
        ),
        (
            0x01dc,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 32,
                id: PlaylistTreeNodeId(18),
                node_is_folder: 0,
                name: "listaj".parse().unwrap(),
            })),
        ),
        (
            0x01f8,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 34,
                id: PlaylistTreeNodeId(19),
                node_is_folder: 0,
                name: "listag".parse().unwrap(),
            })),
        ),
        (
            0x0214,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 36,
                id: PlaylistTreeNodeId(20),
                node_is_folder: 0,
                name: "listai".parse().unwrap(),
            })),
        ),
        (
            0x0230,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 38,
                id: PlaylistTreeNodeId(21),
                node_is_folder: 0,
                name: "listae".parse().unwrap(),
            })),
        ),
        (
            0x024c,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 40,
                id: PlaylistTreeNodeId(22),
                node_is_folder: 0,
                name: "listaf".parse().unwrap(),
            })),
        ),
        (
            0x0268,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 42,
                id: PlaylistTreeNodeId(23),
                node_is_folder: 0,
                name: "listah".parse().unwrap(),
            })),
        ),
        (
            0x0284,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 44,
                id: PlaylistTreeNodeId(24),
                node_is_folder: 0,
                name: "listac".parse().unwrap(),
            })),
        ),
        (
            0x02a0,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 46,
                id: PlaylistTreeNodeId(25),
                node_is_folder: 0,
                name: "listad".parse().unwrap(),
            })),
        ),
        (
            0x02bc,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 48,
                id: PlaylistTreeNodeId(26),
                node_is_folder: 0,
                name: "listaa".parse().unwrap(),
            })),
        ),
        (
            0x02d8,
            Row::Plain(PlainRow::PlaylistTreeNode(PlaylistTreeNode {
                parent_id: PlaylistTreeNodeId(1),
                unknown: 0,
                sort_order: 50,
                id: PlaylistTreeNodeId(27),
                node_is_folder: 0,
                name: "listab".parse().unwrap(),
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(16).unwrap(),
            page_type: PageType::Plain(PlainPageType::PlaylistTree),
            next_page: PageIndex::try_from(46).unwrap(),
            unknown1: 36,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(27)
                .with_num_rows_valid(27),
            page_flags: PageFlags(36),
            free_size: 3238,
            used_size: 756,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 26,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/playlist_tree_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn playlist_entries_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x00b4, 0x00a8, 0x009c, 0x0090, 0x0084, 0x0078, 0x006c, 0x0060, 0x0054, 0x0048,
                0x003c, 0x0030, 0x0024, 0x0018, 0x000c, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0174, 0x0168, 0x015c, 0x0150, 0x0144, 0x0138, 0x012c, 0x0120, 0x0114, 0x0108,
                0x00fc, 0x00f0, 0x00e4, 0x00d8, 0x00cc, 0x00c0,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0234, 0x0228, 0x021c, 0x0210, 0x0204, 0x01f8, 0x01ec, 0x01e0, 0x01d4, 0x01c8,
                0x01bc, 0x01b0, 0x01a4, 0x0198, 0x018c, 0x0180,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x02f4, 0x02e8, 0x02dc, 0x02d0, 0x02c4, 0x02b8, 0x02ac, 0x02a0, 0x0294, 0x0288,
                0x027c, 0x0270, 0x0264, 0x0258, 0x024c, 0x0240,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x03b4, 0x03a8, 0x039c, 0x0390, 0x0384, 0x0378, 0x036c, 0x0360, 0x0354, 0x0348,
                0x033c, 0x0330, 0x0324, 0x0318, 0x030c, 0x0300,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0474, 0x0468, 0x045c, 0x0450, 0x0444, 0x0438, 0x042c, 0x0420, 0x0414, 0x0408,
                0x03fc, 0x03f0, 0x03e4, 0x03d8, 0x03cc, 0x03c0,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0534, 0x0528, 0x051c, 0x0510, 0x0504, 0x04f8, 0x04ec, 0x04e0, 0x04d4, 0x04c8,
                0x04bc, 0x04b0, 0x04a4, 0x0498, 0x048c, 0x0480,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x05f4, 0x05e8, 0x05dc, 0x05d0, 0x05c4, 0x05b8, 0x05ac, 0x05a0, 0x0594, 0x0588,
                0x057c, 0x0570, 0x0564, 0x0558, 0x054c, 0x0540,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x06b4, 0x06a8, 0x069c, 0x0690, 0x0684, 0x0678, 0x066c, 0x0660, 0x0654, 0x0648,
                0x063c, 0x0630, 0x0624, 0x0618, 0x060c, 0x0600,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0774, 0x0768, 0x075c, 0x0750, 0x0744, 0x0738, 0x072c, 0x0720, 0x0714, 0x0708,
                0x06fc, 0x06f0, 0x06e4, 0x06d8, 0x06cc, 0x06c0,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0834, 0x0828, 0x081c, 0x0810, 0x0804, 0x07f8, 0x07ec, 0x07e0, 0x07d4, 0x07c8,
                0x07bc, 0x07b0, 0x07a4, 0x0798, 0x078c, 0x0780,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x08f4, 0x08e8, 0x08dc, 0x08d0, 0x08c4, 0x08b8, 0x08ac, 0x08a0, 0x0894, 0x0888,
                0x087c, 0x0870, 0x0864, 0x0858, 0x084c, 0x0840,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x09b4, 0x09a8, 0x099c, 0x0990, 0x0984, 0x0978, 0x096c, 0x0960, 0x0954, 0x0948,
                0x093c, 0x0930, 0x0924, 0x0918, 0x090c, 0x0900,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0a74, 0x0a68, 0x0a5c, 0x0a50, 0x0a44, 0x0a38, 0x0a2c, 0x0a20, 0x0a14, 0x0a08,
                0x09fc, 0x09f0, 0x09e4, 0x09d8, 0x09cc, 0x09c0,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0b34, 0x0b28, 0x0b1c, 0x0b10, 0x0b04, 0x0af8, 0x0aec, 0x0ae0, 0x0ad4, 0x0ac8,
                0x0abc, 0x0ab0, 0x0aa4, 0x0a98, 0x0a8c, 0x0a80,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0bf4, 0x0be8, 0x0bdc, 0x0bd0, 0x0bc4, 0x0bb8, 0x0bac, 0x0ba0, 0x0b94, 0x0b88,
                0x0b7c, 0x0b70, 0x0b64, 0x0b58, 0x0b4c, 0x0b40,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0cb4, 0x0ca8, 0x0c9c, 0x0c90, 0x0c84, 0x0c78, 0x0c6c, 0x0c60, 0x0c54, 0x0c48,
                0x0c3c, 0x0c30, 0x0c24, 0x0c18, 0x0c0c, 0x0c00,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0, 0, 0, 0, 0x0d44, 0x0d38, 0x0d2c, 0x0d20, 0x0d14, 0x0d08, 0x0cfc, 0x0cf0, 0x0ce4,
                0x0cd8, 0x0ccc, 0x0cc0,
            ],
            row_presence_flags: 0x0fff,
            unknown: 0x0800,
        },
    ];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 1,
                track_id: TrackId(1),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x000c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 2,
                track_id: TrackId(2),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0018,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 3,
                track_id: TrackId(3),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0024,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 4,
                track_id: TrackId(4),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0030,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 5,
                track_id: TrackId(5),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x003c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 6,
                track_id: TrackId(6),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0048,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 7,
                track_id: TrackId(7),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0054,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 8,
                track_id: TrackId(8),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0060,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 9,
                track_id: TrackId(9),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x006c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 10,
                track_id: TrackId(10),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0078,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 11,
                track_id: TrackId(11),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0084,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 12,
                track_id: TrackId(12),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0090,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 13,
                track_id: TrackId(13),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x009c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 14,
                track_id: TrackId(14),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x00a8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 15,
                track_id: TrackId(15),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x00b4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 16,
                track_id: TrackId(16),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x00c0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 17,
                track_id: TrackId(17),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x00cc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 18,
                track_id: TrackId(18),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x00d8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 19,
                track_id: TrackId(19),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x00e4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 20,
                track_id: TrackId(20),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x00f0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 21,
                track_id: TrackId(21),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x00fc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 22,
                track_id: TrackId(22),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0108,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 23,
                track_id: TrackId(23),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0114,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 24,
                track_id: TrackId(24),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0120,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 25,
                track_id: TrackId(25),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x012c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 26,
                track_id: TrackId(26),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0138,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 27,
                track_id: TrackId(27),
                playlist_id: PlaylistTreeNodeId(6),
            })),
        ),
        (
            0x0144,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 1,
                track_id: TrackId(28),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0150,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 2,
                track_id: TrackId(29),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x015c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 3,
                track_id: TrackId(30),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0168,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 4,
                track_id: TrackId(31),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0174,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 5,
                track_id: TrackId(32),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0180,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 6,
                track_id: TrackId(33),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x018c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 7,
                track_id: TrackId(34),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0198,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 8,
                track_id: TrackId(35),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x01a4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 9,
                track_id: TrackId(36),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x01b0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 10,
                track_id: TrackId(37),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x01bc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 11,
                track_id: TrackId(15),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x01c8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 12,
                track_id: TrackId(38),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x01d4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 13,
                track_id: TrackId(39),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x01e0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 14,
                track_id: TrackId(40),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x01ec,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 15,
                track_id: TrackId(41),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x01f8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 16,
                track_id: TrackId(42),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0204,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 17,
                track_id: TrackId(22),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0210,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 18,
                track_id: TrackId(43),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x021c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 19,
                track_id: TrackId(44),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0228,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 20,
                track_id: TrackId(45),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0234,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 21,
                track_id: TrackId(46),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0240,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 22,
                track_id: TrackId(47),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x024c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 23,
                track_id: TrackId(48),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0258,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 24,
                track_id: TrackId(49),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0264,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 25,
                track_id: TrackId(50),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0270,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 26,
                track_id: TrackId(51),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x027c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 27,
                track_id: TrackId(52),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0288,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 28,
                track_id: TrackId(53),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0294,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 29,
                track_id: TrackId(54),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x02a0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 30,
                track_id: TrackId(55),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x02ac,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 31,
                track_id: TrackId(56),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x02b8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 32,
                track_id: TrackId(57),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x02c4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 33,
                track_id: TrackId(58),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x02d0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 34,
                track_id: TrackId(59),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x02dc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 35,
                track_id: TrackId(60),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x02e8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 36,
                track_id: TrackId(61),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x02f4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 37,
                track_id: TrackId(26),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0300,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 38,
                track_id: TrackId(4),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x030c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 39,
                track_id: TrackId(62),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0318,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 40,
                track_id: TrackId(63),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0324,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 41,
                track_id: TrackId(64),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0330,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 42,
                track_id: TrackId(65),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x033c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 43,
                track_id: TrackId(66),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0348,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 44,
                track_id: TrackId(67),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0354,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 45,
                track_id: TrackId(68),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0360,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 46,
                track_id: TrackId(69),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x036c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 47,
                track_id: TrackId(70),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0378,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 48,
                track_id: TrackId(71),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0384,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 49,
                track_id: TrackId(72),
                playlist_id: PlaylistTreeNodeId(7),
            })),
        ),
        (
            0x0390,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 1,
                track_id: TrackId(73),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x039c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 2,
                track_id: TrackId(74),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x03a8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 3,
                track_id: TrackId(75),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x03b4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 4,
                track_id: TrackId(76),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x03c0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 5,
                track_id: TrackId(77),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x03cc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 6,
                track_id: TrackId(78),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x03d8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 7,
                track_id: TrackId(79),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x03e4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 8,
                track_id: TrackId(80),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x03f0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 9,
                track_id: TrackId(81),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x03fc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 10,
                track_id: TrackId(82),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0408,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 11,
                track_id: TrackId(83),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0414,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 12,
                track_id: TrackId(84),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0420,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 13,
                track_id: TrackId(85),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x042c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 14,
                track_id: TrackId(86),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0438,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 15,
                track_id: TrackId(87),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0444,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 16,
                track_id: TrackId(88),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0450,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 17,
                track_id: TrackId(89),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x045c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 18,
                track_id: TrackId(90),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0468,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 19,
                track_id: TrackId(91),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0474,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 20,
                track_id: TrackId(92),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0480,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 21,
                track_id: TrackId(93),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x048c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 22,
                track_id: TrackId(94),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0498,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 23,
                track_id: TrackId(95),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x04a4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 24,
                track_id: TrackId(96),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x04b0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 25,
                track_id: TrackId(97),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x04bc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 26,
                track_id: TrackId(98),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x04c8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 27,
                track_id: TrackId(99),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x04d4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 28,
                track_id: TrackId(100),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x04e0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 29,
                track_id: TrackId(101),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x04ec,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 30,
                track_id: TrackId(102),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x04f8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 31,
                track_id: TrackId(103),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0504,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 32,
                track_id: TrackId(4),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0510,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 33,
                track_id: TrackId(33),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x051c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 34,
                track_id: TrackId(104),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0528,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 35,
                track_id: TrackId(105),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0534,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 36,
                track_id: TrackId(106),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0540,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 37,
                track_id: TrackId(107),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x054c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 38,
                track_id: TrackId(108),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0558,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 39,
                track_id: TrackId(109),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0564,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 40,
                track_id: TrackId(110),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0570,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 41,
                track_id: TrackId(111),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x057c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 42,
                track_id: TrackId(112),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0588,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 43,
                track_id: TrackId(113),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0594,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 44,
                track_id: TrackId(114),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x05a0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 45,
                track_id: TrackId(115),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x05ac,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 46,
                track_id: TrackId(116),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x05b8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 47,
                track_id: TrackId(117),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x05c4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 48,
                track_id: TrackId(118),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x05d0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 49,
                track_id: TrackId(119),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x05dc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 50,
                track_id: TrackId(120),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x05e8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 51,
                track_id: TrackId(121),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x05f4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 52,
                track_id: TrackId(122),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0600,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 53,
                track_id: TrackId(123),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x060c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 54,
                track_id: TrackId(124),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0618,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 55,
                track_id: TrackId(125),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0624,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 56,
                track_id: TrackId(126),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0630,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 57,
                track_id: TrackId(127),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x063c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 58,
                track_id: TrackId(128),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0648,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 59,
                track_id: TrackId(129),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0654,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 60,
                track_id: TrackId(130),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0660,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 61,
                track_id: TrackId(131),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x066c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 62,
                track_id: TrackId(132),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0678,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 63,
                track_id: TrackId(133),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0684,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 64,
                track_id: TrackId(134),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0690,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 65,
                track_id: TrackId(52),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x069c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 66,
                track_id: TrackId(135),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x06a8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 67,
                track_id: TrackId(136),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x06b4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 68,
                track_id: TrackId(137),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x06c0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 69,
                track_id: TrackId(138),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x06cc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 70,
                track_id: TrackId(139),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x06d8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 71,
                track_id: TrackId(140),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x06e4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 72,
                track_id: TrackId(141),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x06f0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 73,
                track_id: TrackId(142),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x06fc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 74,
                track_id: TrackId(143),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0708,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 75,
                track_id: TrackId(144),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0714,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 76,
                track_id: TrackId(145),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0720,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 77,
                track_id: TrackId(146),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x072c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 78,
                track_id: TrackId(66),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0738,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 79,
                track_id: TrackId(147),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0744,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 80,
                track_id: TrackId(148),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0750,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 81,
                track_id: TrackId(149),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x075c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 82,
                track_id: TrackId(150),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0768,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 83,
                track_id: TrackId(53),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0774,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 84,
                track_id: TrackId(151),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0780,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 85,
                track_id: TrackId(152),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x078c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 86,
                track_id: TrackId(153),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0798,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 87,
                track_id: TrackId(154),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x07a4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 88,
                track_id: TrackId(155),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x07b0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 89,
                track_id: TrackId(156),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x07bc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 90,
                track_id: TrackId(157),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x07c8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 91,
                track_id: TrackId(158),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x07d4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 92,
                track_id: TrackId(159),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x07e0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 93,
                track_id: TrackId(160),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x07ec,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 94,
                track_id: TrackId(161),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x07f8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 95,
                track_id: TrackId(162),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0804,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 96,
                track_id: TrackId(163),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0810,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 97,
                track_id: TrackId(164),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x081c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 98,
                track_id: TrackId(165),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0828,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 99,
                track_id: TrackId(166),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0834,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 100,
                track_id: TrackId(167),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0840,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 101,
                track_id: TrackId(54),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x084c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 102,
                track_id: TrackId(47),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0858,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 103,
                track_id: TrackId(168),
                playlist_id: PlaylistTreeNodeId(8),
            })),
        ),
        (
            0x0864,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 1,
                track_id: TrackId(169),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x0870,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 2,
                track_id: TrackId(170),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x087c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 3,
                track_id: TrackId(171),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x0888,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 4,
                track_id: TrackId(57),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x0894,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 5,
                track_id: TrackId(172),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x08a0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 6,
                track_id: TrackId(173),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x08ac,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 7,
                track_id: TrackId(174),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x08b8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 8,
                track_id: TrackId(175),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x08c4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 9,
                track_id: TrackId(125),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x08d0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 10,
                track_id: TrackId(176),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x08dc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 11,
                track_id: TrackId(177),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x08e8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 12,
                track_id: TrackId(178),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x08f4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 13,
                track_id: TrackId(179),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x0900,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 14,
                track_id: TrackId(180),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x090c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 15,
                track_id: TrackId(181),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x0918,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 16,
                track_id: TrackId(182),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x0924,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 17,
                track_id: TrackId(183),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x0930,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 18,
                track_id: TrackId(166),
                playlist_id: PlaylistTreeNodeId(9),
            })),
        ),
        (
            0x093c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 1,
                track_id: TrackId(184),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0948,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 2,
                track_id: TrackId(185),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0954,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 3,
                track_id: TrackId(77),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0960,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 4,
                track_id: TrackId(186),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x096c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 5,
                track_id: TrackId(187),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0978,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 6,
                track_id: TrackId(188),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0984,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 7,
                track_id: TrackId(189),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0990,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 8,
                track_id: TrackId(190),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x099c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 9,
                track_id: TrackId(191),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x09a8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 10,
                track_id: TrackId(90),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x09b4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 11,
                track_id: TrackId(192),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x09c0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 12,
                track_id: TrackId(193),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x09cc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 13,
                track_id: TrackId(194),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x09d8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 14,
                track_id: TrackId(195),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x09e4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 15,
                track_id: TrackId(101),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x09f0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 16,
                track_id: TrackId(196),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x09fc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 17,
                track_id: TrackId(197),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a08,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 18,
                track_id: TrackId(198),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a14,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 19,
                track_id: TrackId(199),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a20,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 20,
                track_id: TrackId(200),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a2c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 21,
                track_id: TrackId(201),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a38,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 22,
                track_id: TrackId(202),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a44,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 23,
                track_id: TrackId(203),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a50,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 24,
                track_id: TrackId(204),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a5c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 25,
                track_id: TrackId(205),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a68,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 26,
                track_id: TrackId(206),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a74,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 27,
                track_id: TrackId(207),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a80,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 28,
                track_id: TrackId(208),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a8c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 29,
                track_id: TrackId(209),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0a98,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 30,
                track_id: TrackId(210),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0aa4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 31,
                track_id: TrackId(211),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0ab0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 32,
                track_id: TrackId(212),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0abc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 33,
                track_id: TrackId(213),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0ac8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 34,
                track_id: TrackId(214),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0ad4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 35,
                track_id: TrackId(215),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0ae0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 36,
                track_id: TrackId(168),
                playlist_id: PlaylistTreeNodeId(10),
            })),
        ),
        (
            0x0aec,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 1,
                track_id: TrackId(74),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0af8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 2,
                track_id: TrackId(79),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b04,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 3,
                track_id: TrackId(80),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b10,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 4,
                track_id: TrackId(81),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b1c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 5,
                track_id: TrackId(82),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b28,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 6,
                track_id: TrackId(87),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b34,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 7,
                track_id: TrackId(189),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b40,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 8,
                track_id: TrackId(216),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b4c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 9,
                track_id: TrackId(217),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b58,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 10,
                track_id: TrackId(218),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b64,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 11,
                track_id: TrackId(219),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b70,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 12,
                track_id: TrackId(220),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b7c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 13,
                track_id: TrackId(221),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b88,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 14,
                track_id: TrackId(222),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0b94,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 15,
                track_id: TrackId(223),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0ba0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 16,
                track_id: TrackId(195),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0bac,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 17,
                track_id: TrackId(105),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0bb8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 18,
                track_id: TrackId(224),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0bc4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 19,
                track_id: TrackId(107),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0bd0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 20,
                track_id: TrackId(225),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0bdc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 21,
                track_id: TrackId(226),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0be8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 22,
                track_id: TrackId(227),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0bf4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 23,
                track_id: TrackId(228),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c00,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 24,
                track_id: TrackId(229),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c0c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 25,
                track_id: TrackId(10),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c18,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 26,
                track_id: TrackId(230),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c24,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 27,
                track_id: TrackId(231),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c30,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 28,
                track_id: TrackId(232),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c3c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 29,
                track_id: TrackId(233),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c48,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 30,
                track_id: TrackId(234),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c54,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 31,
                track_id: TrackId(17),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c60,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 32,
                track_id: TrackId(235),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c6c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 33,
                track_id: TrackId(138),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c78,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 34,
                track_id: TrackId(236),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c84,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 35,
                track_id: TrackId(147),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c90,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 36,
                track_id: TrackId(237),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0c9c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 37,
                track_id: TrackId(208),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0ca8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 38,
                track_id: TrackId(238),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0cb4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 39,
                track_id: TrackId(239),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0cc0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 40,
                track_id: TrackId(240),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0ccc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 41,
                track_id: TrackId(241),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0cd8,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 42,
                track_id: TrackId(242),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0ce4,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 43,
                track_id: TrackId(243),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0cf0,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 44,
                track_id: TrackId(244),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0cfc,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 45,
                track_id: TrackId(245),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0d08,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 46,
                track_id: TrackId(120),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0d14,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 47,
                track_id: TrackId(246),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0d20,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 48,
                track_id: TrackId(247),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0d2c,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 49,
                track_id: TrackId(248),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0d38,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 50,
                track_id: TrackId(249),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
        (
            0x0d44,
            Row::Plain(PlainRow::PlaylistEntry(PlaylistEntry {
                entry_index: 51,
                track_id: TrackId(250),
                playlist_id: PlaylistTreeNodeId(11),
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(18).unwrap(),
            page_type: PageType::Plain(PlainPageType::PlaylistEntries),
            next_page: PageIndex::try_from(54).unwrap(),
            unknown1: 1420,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(284)
                .with_num_rows_valid(284),
            page_flags: PageFlags(36),
            free_size: 8,
            used_size: 3408,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 283,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/playlist_entries_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn artworks_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x021c, 0x01f8, 0x01d4, 0x01b0, 0x018c, 0x0168, 0x0144, 0x0120, 0x00fc, 0x00d8,
                0x00b4, 0x0090, 0x006c, 0x0048, 0x0024, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x045c, 0x0438, 0x0414, 0x03f0, 0x03cc, 0x03a8, 0x0384, 0x0360, 0x033c, 0x0318,
                0x02f4, 0x02d0, 0x02ac, 0x0288, 0x0264, 0x0240,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x069c, 0x0678, 0x0654, 0x0630, 0x060c, 0x05e8, 0x05c4, 0x05a0, 0x057c, 0x0558,
                0x0534, 0x0510, 0x04ec, 0x04c8, 0x04a4, 0x0480,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x08dc, 0x08b8, 0x0894, 0x0870, 0x084c, 0x0828, 0x0804, 0x07e0, 0x07bc, 0x0798,
                0x0774, 0x0750, 0x072c, 0x0708, 0x06e4, 0x06c0,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0b1c, 0x0af8, 0x0ad4, 0x0ab0, 0x0a8c, 0x0a68, 0x0a44, 0x0a20, 0x09fc, 0x09d8,
                0x09b4, 0x0990, 0x096c, 0x0948, 0x0924, 0x0900,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x0d5c, 0x0d38, 0x0d14, 0x0cf0, 0x0ccc, 0x0ca8, 0x0c84, 0x0c60, 0x0c3c, 0x0c18,
                0x0bf4, 0x0bd0, 0x0bac, 0x0b88, 0x0b64, 0x0b40,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0, 0, 0, 0, 0, 0, 0x0ec4, 0x0ea0, 0x0e7c, 0x0e58, 0x0e34, 0x0e10, 0x0dec, 0x0dc8,
                0x0da4, 0x0d80,
            ],
            row_presence_flags: 0x03ff,
            unknown: 0x0200,
        },
    ];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(1),
                path: "/PIONEER/Artwork/00001/a1.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0024,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(2),
                path: "/PIONEER/Artwork/00001/a2.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0048,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(3),
                path: "/PIONEER/Artwork/00001/a3.jpg".parse().unwrap(),
            })),
        ),
        (
            0x006c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(4),
                path: "/PIONEER/Artwork/00001/a4.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0090,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(5),
                path: "/PIONEER/Artwork/00001/a5.jpg".parse().unwrap(),
            })),
        ),
        (
            0x00b4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(6),
                path: "/PIONEER/Artwork/00001/a6.jpg".parse().unwrap(),
            })),
        ),
        (
            0x00d8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(7),
                path: "/PIONEER/Artwork/00001/a7.jpg".parse().unwrap(),
            })),
        ),
        (
            0x00fc,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(8),
                path: "/PIONEER/Artwork/00001/a8.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0120,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(9),
                path: "/PIONEER/Artwork/00001/a9.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0144,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(10),
                path: "/PIONEER/Artwork/00001/a10.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0168,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(11),
                path: "/PIONEER/Artwork/00001/a11.jpg".parse().unwrap(),
            })),
        ),
        (
            0x018c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(12),
                path: "/PIONEER/Artwork/00001/a12.jpg".parse().unwrap(),
            })),
        ),
        (
            0x01b0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(13),
                path: "/PIONEER/Artwork/00001/a13.jpg".parse().unwrap(),
            })),
        ),
        (
            0x01d4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(14),
                path: "/PIONEER/Artwork/00001/a14.jpg".parse().unwrap(),
            })),
        ),
        (
            0x01f8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(15),
                path: "/PIONEER/Artwork/00001/a15.jpg".parse().unwrap(),
            })),
        ),
        (
            0x021c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(16),
                path: "/PIONEER/Artwork/00001/a16.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0240,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(17),
                path: "/PIONEER/Artwork/00001/a17.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0264,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(18),
                path: "/PIONEER/Artwork/00001/a18.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0288,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(19),
                path: "/PIONEER/Artwork/00001/a19.jpg".parse().unwrap(),
            })),
        ),
        (
            0x02ac,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(20),
                path: "/PIONEER/Artwork/00002/a20.jpg".parse().unwrap(),
            })),
        ),
        (
            0x02d0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(21),
                path: "/PIONEER/Artwork/00002/a21.jpg".parse().unwrap(),
            })),
        ),
        (
            0x02f4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(22),
                path: "/PIONEER/Artwork/00002/a22.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0318,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(23),
                path: "/PIONEER/Artwork/00002/a23.jpg".parse().unwrap(),
            })),
        ),
        (
            0x033c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(24),
                path: "/PIONEER/Artwork/00002/a24.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0360,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(25),
                path: "/PIONEER/Artwork/00002/a25.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0384,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(26),
                path: "/PIONEER/Artwork/00002/a26.jpg".parse().unwrap(),
            })),
        ),
        (
            0x03a8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(27),
                path: "/PIONEER/Artwork/00002/a27.jpg".parse().unwrap(),
            })),
        ),
        (
            0x03cc,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(28),
                path: "/PIONEER/Artwork/00002/a28.jpg".parse().unwrap(),
            })),
        ),
        (
            0x03f0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(29),
                path: "/PIONEER/Artwork/00002/a29.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0414,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(30),
                path: "/PIONEER/Artwork/00002/a30.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0438,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(31),
                path: "/PIONEER/Artwork/00002/a31.jpg".parse().unwrap(),
            })),
        ),
        (
            0x045c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(32),
                path: "/PIONEER/Artwork/00002/a32.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0480,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(33),
                path: "/PIONEER/Artwork/00002/a33.jpg".parse().unwrap(),
            })),
        ),
        (
            0x04a4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(34),
                path: "/PIONEER/Artwork/00002/a34.jpg".parse().unwrap(),
            })),
        ),
        (
            0x04c8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(35),
                path: "/PIONEER/Artwork/00002/a35.jpg".parse().unwrap(),
            })),
        ),
        (
            0x04ec,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(36),
                path: "/PIONEER/Artwork/00002/a36.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0510,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(37),
                path: "/PIONEER/Artwork/00002/a37.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0534,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(38),
                path: "/PIONEER/Artwork/00002/a38.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0558,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(39),
                path: "/PIONEER/Artwork/00002/a39.jpg".parse().unwrap(),
            })),
        ),
        (
            0x057c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(40),
                path: "/PIONEER/Artwork/00003/a40.jpg".parse().unwrap(),
            })),
        ),
        (
            0x05a0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(41),
                path: "/PIONEER/Artwork/00003/a41.jpg".parse().unwrap(),
            })),
        ),
        (
            0x05c4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(42),
                path: "/PIONEER/Artwork/00003/a42.jpg".parse().unwrap(),
            })),
        ),
        (
            0x05e8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(43),
                path: "/PIONEER/Artwork/00003/a43.jpg".parse().unwrap(),
            })),
        ),
        (
            0x060c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(44),
                path: "/PIONEER/Artwork/00003/a44.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0630,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(45),
                path: "/PIONEER/Artwork/00003/a45.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0654,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(46),
                path: "/PIONEER/Artwork/00003/a46.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0678,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(47),
                path: "/PIONEER/Artwork/00003/a47.jpg".parse().unwrap(),
            })),
        ),
        (
            0x069c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(48),
                path: "/PIONEER/Artwork/00003/a48.jpg".parse().unwrap(),
            })),
        ),
        (
            0x06c0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(49),
                path: "/PIONEER/Artwork/00003/a49.jpg".parse().unwrap(),
            })),
        ),
        (
            0x06e4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(50),
                path: "/PIONEER/Artwork/00003/a50.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0708,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(51),
                path: "/PIONEER/Artwork/00003/a51.jpg".parse().unwrap(),
            })),
        ),
        (
            0x072c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(52),
                path: "/PIONEER/Artwork/00003/a52.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0750,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(53),
                path: "/PIONEER/Artwork/00003/a53.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0774,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(54),
                path: "/PIONEER/Artwork/00003/a54.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0798,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(55),
                path: "/PIONEER/Artwork/00003/a55.jpg".parse().unwrap(),
            })),
        ),
        (
            0x07bc,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(56),
                path: "/PIONEER/Artwork/00003/a56.jpg".parse().unwrap(),
            })),
        ),
        (
            0x07e0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(57),
                path: "/PIONEER/Artwork/00003/a57.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0804,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(58),
                path: "/PIONEER/Artwork/00003/a58.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0828,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(59),
                path: "/PIONEER/Artwork/00003/a59.jpg".parse().unwrap(),
            })),
        ),
        (
            0x084c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(60),
                path: "/PIONEER/Artwork/00004/a60.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0870,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(61),
                path: "/PIONEER/Artwork/00004/a61.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0894,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(62),
                path: "/PIONEER/Artwork/00004/a62.jpg".parse().unwrap(),
            })),
        ),
        (
            0x08b8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(63),
                path: "/PIONEER/Artwork/00004/a63.jpg".parse().unwrap(),
            })),
        ),
        (
            0x08dc,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(64),
                path: "/PIONEER/Artwork/00004/a64.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0900,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(65),
                path: "/PIONEER/Artwork/00004/a65.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0924,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(66),
                path: "/PIONEER/Artwork/00004/a66.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0948,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(67),
                path: "/PIONEER/Artwork/00004/a67.jpg".parse().unwrap(),
            })),
        ),
        (
            0x096c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(68),
                path: "/PIONEER/Artwork/00004/a68.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0990,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(69),
                path: "/PIONEER/Artwork/00004/a69.jpg".parse().unwrap(),
            })),
        ),
        (
            0x09b4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(70),
                path: "/PIONEER/Artwork/00004/a70.jpg".parse().unwrap(),
            })),
        ),
        (
            0x09d8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(71),
                path: "/PIONEER/Artwork/00004/a71.jpg".parse().unwrap(),
            })),
        ),
        (
            0x09fc,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(72),
                path: "/PIONEER/Artwork/00004/a72.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0a20,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(73),
                path: "/PIONEER/Artwork/00004/a73.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0a44,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(74),
                path: "/PIONEER/Artwork/00004/a74.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0a68,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(75),
                path: "/PIONEER/Artwork/00004/a75.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0a8c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(76),
                path: "/PIONEER/Artwork/00004/a76.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0ab0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(77),
                path: "/PIONEER/Artwork/00004/a77.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0ad4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(78),
                path: "/PIONEER/Artwork/00004/a78.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0af8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(79),
                path: "/PIONEER/Artwork/00004/a79.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0b1c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(80),
                path: "/PIONEER/Artwork/00005/a80.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0b40,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(81),
                path: "/PIONEER/Artwork/00005/a81.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0b64,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(82),
                path: "/PIONEER/Artwork/00005/a82.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0b88,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(83),
                path: "/PIONEER/Artwork/00005/a83.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0bac,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(84),
                path: "/PIONEER/Artwork/00005/a84.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0bd0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(85),
                path: "/PIONEER/Artwork/00005/a85.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0bf4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(86),
                path: "/PIONEER/Artwork/00005/a86.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0c18,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(87),
                path: "/PIONEER/Artwork/00005/a87.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0c3c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(88),
                path: "/PIONEER/Artwork/00005/a88.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0c60,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(89),
                path: "/PIONEER/Artwork/00005/a89.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0c84,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(90),
                path: "/PIONEER/Artwork/00005/a90.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0ca8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(91),
                path: "/PIONEER/Artwork/00005/a91.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0ccc,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(92),
                path: "/PIONEER/Artwork/00005/a92.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0cf0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(93),
                path: "/PIONEER/Artwork/00005/a93.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0d14,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(94),
                path: "/PIONEER/Artwork/00005/a94.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0d38,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(95),
                path: "/PIONEER/Artwork/00005/a95.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0d5c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(96),
                path: "/PIONEER/Artwork/00005/a96.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0d80,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(97),
                path: "/PIONEER/Artwork/00005/a97.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0da4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(98),
                path: "/PIONEER/Artwork/00005/a98.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0dc8,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(99),
                path: "/PIONEER/Artwork/00005/a99.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0dec,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(100),
                path: "/PIONEER/Artwork/00006/a100.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0e10,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(101),
                path: "/PIONEER/Artwork/00006/a101.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0e34,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(102),
                path: "/PIONEER/Artwork/00006/a102.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0e58,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(103),
                path: "/PIONEER/Artwork/00006/a103.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0e7c,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(104),
                path: "/PIONEER/Artwork/00006/a104.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0ea0,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(105),
                path: "/PIONEER/Artwork/00006/a105.jpg".parse().unwrap(),
            })),
        ),
        (
            0x0ec4,
            Row::Plain(PlainRow::Artwork(Artwork {
                id: ArtworkId(106),
                path: "/PIONEER/Artwork/00006/a106.jpg".parse().unwrap(),
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(28).unwrap(),
            page_type: PageType::Plain(PlainPageType::Artwork),
            next_page: PageIndex::try_from(53).unwrap(),
            unknown1: 1019,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(106)
                .with_num_rows_valid(106),
            page_flags: PageFlags(36),
            free_size: 0,
            used_size: 3816,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 105,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/artworks_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn tag_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x0320, 0x02ec, 0x02b8, 0x0280, 0x024c, 0x0218, 0x01e4, 0x01b0, 0x017c, 0x0148,
                0x010c, 0x00d8, 0x00a0, 0x006c, 0x0038, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0xffff, // interestingly the same as row_presence_flags
        },
        RowGroup {
            row_offsets: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0490, 0x0458, 0x0424, 0x03f0, 0x03bc, 0x0388, 0x0354,
            ],
            row_presence_flags: 0x007f,
            unknown: 0x007f, // interestingly the same as row_presence_flags
        },
    ];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0038,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x006c,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x00a0,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x00d8,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x010c,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0148,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x017c,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x01b0,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x01e4,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0218,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x024c,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0280,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x02b8,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x02ec,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0320,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0354,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0388,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x03bc,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x03f0,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0424,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0458,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
        (
            0x0490,
            Row::Ext(ExtRow::Tag(TagOrCategory {
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
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(8).unwrap(),
            page_type: PageType::Ext(ExtPageType::Tag),
            next_page: PageIndex::try_from(20).unwrap(),
            unknown1: 2,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(23)
                .with_num_rows_valid(23),
            page_flags: PageFlags(36),
            free_size: 2770,
            used_size: 1232,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 23,
                unknown_not_num_rows_large: 0,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/tag_page.bin"),
        page,
        (page_size, DatabaseType::Ext),
        (page_size,),
    );
}

#[test]
fn track_tag_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x00f0, 0x00e0, 0x00d0, 0x00c0, 0x00b0, 0x00a0, 0x0090, 0x0080, 0x0070, 0x0060,
                0x0050, 0x0040, 0x0030, 0x0020, 0x0010, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x01f0, 0x01e0, 0x01d0, 0x01c0, 0x01b0, 0x01a0, 0x0190, 0x0180, 0x0170, 0x0160,
                0x0150, 0x0140, 0x0130, 0x0120, 0x0110, 0x0100,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0x02f0, 0x02e0, 0x02d0, 0x02c0, 0x02b0, 0x02a0, 0x0290, 0x0280, 0x0270, 0x0260,
                0x0250, 0x0240, 0x0230, 0x0220, 0x0210, 0x0200,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        },
        RowGroup {
            row_offsets: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0330, 0x0320, 0x0310, 0x0300,
            ],
            row_presence_flags: 0x000f,
            unknown: 0x0008,
        },
    ];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(1),
                tag_id: TagId(2498240426),
                unknown_const: 3,
            })),
        ),
        (
            0x0010,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(2),
                tag_id: TagId(4052665282),
                unknown_const: 3,
            })),
        ),
        (
            0x0020,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(2),
                tag_id: TagId(2498240426),
                unknown_const: 3,
            })),
        ),
        (
            0x0030,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(3),
                tag_id: TagId(3456350885),
                unknown_const: 3,
            })),
        ),
        (
            0x0040,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(3),
                tag_id: TagId(3518593467),
                unknown_const: 3,
            })),
        ),
        (
            0x0050,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(3),
                tag_id: TagId(3074636465),
                unknown_const: 3,
            })),
        ),
        (
            0x0060,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(4),
                tag_id: TagId(3456350885),
                unknown_const: 3,
            })),
        ),
        (
            0x0070,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(4),
                tag_id: TagId(3518593467),
                unknown_const: 3,
            })),
        ),
        (
            0x0080,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(4),
                tag_id: TagId(4026144338),
                unknown_const: 3,
            })),
        ),
        (
            0x0090,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(4),
                tag_id: TagId(3074636465),
                unknown_const: 3,
            })),
        ),
        (
            0x00a0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(5),
                tag_id: TagId(4052665282),
                unknown_const: 3,
            })),
        ),
        (
            0x00b0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(5),
                tag_id: TagId(218937570),
                unknown_const: 3,
            })),
        ),
        (
            0x00c0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(5),
                tag_id: TagId(3074636465),
                unknown_const: 3,
            })),
        ),
        (
            0x00d0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(6),
                tag_id: TagId(3211624224),
                unknown_const: 3,
            })),
        ),
        (
            0x00e0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(6),
                tag_id: TagId(3043071597),
                unknown_const: 3,
            })),
        ),
        (
            0x00f0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(7),
                tag_id: TagId(2923592519),
                unknown_const: 3,
            })),
        ),
        (
            0x0100,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(7),
                tag_id: TagId(712200756),
                unknown_const: 3,
            })),
        ),
        (
            0x0110,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(8),
                tag_id: TagId(3456350885),
                unknown_const: 3,
            })),
        ),
        (
            0x0120,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(8),
                tag_id: TagId(4263562201),
                unknown_const: 3,
            })),
        ),
        (
            0x0130,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(8),
                tag_id: TagId(3074636465),
                unknown_const: 3,
            })),
        ),
        (
            0x0140,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(9),
                tag_id: TagId(4052665282),
                unknown_const: 3,
            })),
        ),
        (
            0x0150,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(9),
                tag_id: TagId(3074636465),
                unknown_const: 3,
            })),
        ),
        (
            0x0160,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(10),
                tag_id: TagId(3216792858),
                unknown_const: 3,
            })),
        ),
        (
            0x0170,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(10),
                tag_id: TagId(4026144338),
                unknown_const: 3,
            })),
        ),
        (
            0x0180,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(11),
                tag_id: TagId(3456350885),
                unknown_const: 3,
            })),
        ),
        (
            0x0190,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(11),
                tag_id: TagId(598441108),
                unknown_const: 3,
            })),
        ),
        (
            0x01a0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(11),
                tag_id: TagId(707481115),
                unknown_const: 3,
            })),
        ),
        (
            0x01b0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(12),
                tag_id: TagId(2923592519),
                unknown_const: 3,
            })),
        ),
        (
            0x01c0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(12),
                tag_id: TagId(3518593467),
                unknown_const: 3,
            })),
        ),
        (
            0x01d0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(12),
                tag_id: TagId(926017397),
                unknown_const: 3,
            })),
        ),
        (
            0x1e0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(13),
                tag_id: TagId(712200756),
                unknown_const: 3,
            })),
        ),
        (
            0x01f0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(13),
                tag_id: TagId(4263562201),
                unknown_const: 3,
            })),
        ),
        (
            0x0200,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(14),
                tag_id: TagId(3211624224),
                unknown_const: 3,
            })),
        ),
        (
            0x0210,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(14),
                tag_id: TagId(4026144338),
                unknown_const: 3,
            })),
        ),
        (
            0x0220,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(15),
                tag_id: TagId(3456350885),
                unknown_const: 3,
            })),
        ),
        (
            0x0230,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(15),
                tag_id: TagId(4166869272),
                unknown_const: 3,
            })),
        ),
        (
            0x0240,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(16),
                tag_id: TagId(4052665282),
                unknown_const: 3,
            })),
        ),
        (
            0x0250,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(16),
                tag_id: TagId(3043071597),
                unknown_const: 3,
            })),
        ),
        (
            0x0260,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(17),
                tag_id: TagId(4166869272),
                unknown_const: 3,
            })),
        ),
        (
            0x0270,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(17),
                tag_id: TagId(926017397),
                unknown_const: 3,
            })),
        ),
        (
            0x0280,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(18),
                tag_id: TagId(3518593467),
                unknown_const: 3,
            })),
        ),
        (
            0x0290,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(18),
                tag_id: TagId(870902105),
                unknown_const: 3,
            })),
        ),
        (
            0x02a0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(19),
                tag_id: TagId(3456350885),
                unknown_const: 3,
            })),
        ),
        (
            0x02b0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(19),
                tag_id: TagId(3211624224),
                unknown_const: 3,
            })),
        ),
        (
            0x02c0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(20),
                tag_id: TagId(3456350885),
                unknown_const: 3,
            })),
        ),
        (
            0x02d0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(21),
                tag_id: TagId(3456350885),
                unknown_const: 3,
            })),
        ),
        (
            0x02e0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(22),
                tag_id: TagId(4166869272),
                unknown_const: 3,
            })),
        ),
        (
            0x02f0,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(24),
                tag_id: TagId(4166869272),
                unknown_const: 3,
            })),
        ),
        (
            0x0300,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(25),
                tag_id: TagId(3211624224),
                unknown_const: 3,
            })),
        ),
        (
            0x0310,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(25),
                tag_id: TagId(3216792858),
                unknown_const: 3,
            })),
        ),
        (
            0x0320,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(29),
                tag_id: TagId(2498240426),
                unknown_const: 3,
            })),
        ),
        (
            0x0330,
            Row::Ext(ExtRow::TrackTag(TrackTag {
                track_id: TrackId(29),
                tag_id: TagId(598441108),
                unknown_const: 3,
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(10).unwrap(),
            page_type: PageType::Ext(ExtPageType::TrackTag),
            next_page: PageIndex::try_from(21).unwrap(),
            unknown1: 54,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(52)
                .with_num_rows_valid(52),
            page_flags: PageFlags(36),
            free_size: 3104,
            used_size: 832,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 51,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/track_tag_page.bin"),
        page,
        (page_size, DatabaseType::Ext),
        (page_size,),
    );
}

/* TODO the CDJ-350 seems to create a HistoryPlaylists page for each row.
Find a player that properly fills a page and improve this test. */
#[test]
fn history_playlists_page() {
    let row_groups = vec![RowGroup {
        row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0000],
        row_presence_flags: 0x0001,
        unknown: 0x0001,
    }];

    let rows: BTreeMap<u16, Row> = vec![(
        0x0000,
        Row::Plain(PlainRow::HistoryPlaylist(HistoryPlaylist {
            id: HistoryPlaylistId(1),
            name: "HISTORY 001".parse().unwrap(),
        })),
    )]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(24).unwrap(),
            page_type: PageType::Plain(PlainPageType::HistoryPlaylists),
            next_page: PageIndex::try_from(59).unwrap(),
            unknown1: 240,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(1)
                .with_num_rows_valid(1),
            page_flags: PageFlags(36),
            free_size: 4034,
            used_size: 16,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 0,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/history_playlists_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

// TODO improve the test with a fuller page
#[test]
fn history_entries_page() {
    let row_groups = vec![RowGroup {
        row_offsets: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0048, 0x003c, 0x0030, 0x0024, 0x0018, 0x000c, 0x0000,
        ],
        row_presence_flags: 0x007f,
        unknown: 0x0040,
    }];

    let rows: BTreeMap<u16, Row> = vec![
        (
            0x0000,
            Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
                track_id: TrackId(35),
                playlist_id: HistoryPlaylistId(2),
                entry_index: 1,
            })),
        ),
        (
            0x000c,
            Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
                track_id: TrackId(18),
                playlist_id: HistoryPlaylistId(2),
                entry_index: 2,
            })),
        ),
        (
            0x0018,
            Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
                track_id: TrackId(25),
                playlist_id: HistoryPlaylistId(2),
                entry_index: 3,
            })),
        ),
        (
            0x0024,
            Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
                track_id: TrackId(5),
                playlist_id: HistoryPlaylistId(2),
                entry_index: 4,
            })),
        ),
        (
            0x0030,
            Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
                track_id: TrackId(12),
                playlist_id: HistoryPlaylistId(2),
                entry_index: 5,
            })),
        ),
        (
            0x003c,
            Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
                track_id: TrackId(19),
                playlist_id: HistoryPlaylistId(2),
                entry_index: 6,
            })),
        ),
        (
            0x0048,
            Row::Plain(PlainRow::HistoryEntry(HistoryEntry {
                track_id: TrackId(6),
                playlist_id: HistoryPlaylistId(2),
                entry_index: 7,
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(60).unwrap(),
            page_type: PageType::Plain(PlainPageType::HistoryEntries),
            next_page: PageIndex::try_from(62).unwrap(),
            unknown1: 254,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(7)
                .with_num_rows_valid(7),
            page_flags: PageFlags(36),
            free_size: 3954,
            used_size: 84,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 1,
                unknown_not_num_rows_large: 6,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/history_entries_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn menu_page() {
    let row_groups = vec![
        RowGroup {
            row_offsets: [
                0x0078, 0x0070, 0x0068, 0x0060, 0x0058, 0x0050, 0x0048, 0x0040, 0x0038, 0x0030,
                0x0028, 0x0020, 0x0018, 0x0010, 0x0008, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0xffff,
        },
        RowGroup {
            row_offsets: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x00a8, 0x00a0, 0x0098, 0x0090, 0x0088, 0x0080,
            ],
            row_presence_flags: 0x003f,
            unknown: 0x003f,
        },
    ];

    let rows = vec![
        (
            0x0000,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 1,
                content_pointer: 1,
                unknown: 99,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0008,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 5,
                content_pointer: 6,
                unknown: 5,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0010,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 6,
                content_pointer: 7,
                unknown: 99,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0018,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 7,
                content_pointer: 8,
                unknown: 99,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0020,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 8,
                content_pointer: 9,
                unknown: 99,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0028,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 9,
                content_pointer: 10,
                unknown: 99,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0030,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 10,
                content_pointer: 11,
                unknown: 99,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0038,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 13,
                content_pointer: 15,
                unknown: 99,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0040,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 14,
                content_pointer: 19,
                unknown: 4,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0048,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 15,
                content_pointer: 20,
                unknown: 6,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0050,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 16,
                content_pointer: 21,
                unknown: 99,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0058,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 18,
                content_pointer: 23,
                unknown: 99,
                visibility: MenuVisibility::Hidden,
                sort_order: 0,
            })),
        ),
        (
            0x0060,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 2,
                content_pointer: 2,
                unknown: 2,
                visibility: MenuVisibility::Visible,
                sort_order: 1,
            })),
        ),
        (
            0x0068,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 3,
                content_pointer: 3,
                unknown: 3,
                visibility: MenuVisibility::Visible,
                sort_order: 2,
            })),
        ),
        (
            0x0070,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 4,
                content_pointer: 4,
                unknown: 1,
                visibility: MenuVisibility::Visible,
                sort_order: 3,
            })),
        ),
        (
            0x0078,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 11,
                content_pointer: 12,
                unknown: 99,
                visibility: MenuVisibility::Visible,
                sort_order: 4,
            })),
        ),
        (
            0x0080,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 17,
                content_pointer: 5,
                unknown: 99,
                visibility: MenuVisibility::Visible,
                sort_order: 5,
            })),
        ),
        (
            0x0088,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 19,
                content_pointer: 22,
                unknown: 99,
                visibility: MenuVisibility::Visible,
                sort_order: 6,
            })),
        ),
        (
            0x0090,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 20,
                content_pointer: 18,
                unknown: 99,
                visibility: MenuVisibility::Visible,
                sort_order: 7,
            })),
        ),
        (
            0x0098,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 27,
                content_pointer: 26,
                unknown: 99,
                visibility: MenuVisibility::Unknown(2),
                sort_order: 8,
            })),
        ),
        (
            0x00a0,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 24,
                content_pointer: 17,
                unknown: 99,
                visibility: MenuVisibility::Visible,
                sort_order: 9,
            })),
        ),
        (
            0x00a8,
            Row::Plain(PlainRow::Menu(Menu {
                category_id: 22,
                content_pointer: 27,
                unknown: 99,
                visibility: MenuVisibility::Visible,
                sort_order: 10,
            })),
        ),
    ]
    .into_iter()
    .collect();

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(36).unwrap(),
            page_type: PageType::Plain(PlainPageType::Menu),
            next_page: PageIndex::try_from(44).unwrap(),
            unknown1: 4,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(22)
                .with_num_rows_valid(22),
            page_flags: PageFlags(36),
            free_size: 3828,
            used_size: 176,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 22,
                unknown_not_num_rows_large: 0,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups,
            rows,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/menu_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}

#[test]
fn index_page() {
    let entries = vec![
        IndexEntry::try_from((PageIndex::try_from(604).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(371).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(441).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(471).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(210).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(412).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(327).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(434).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(251).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(238).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(574).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(141).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(234).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(67).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(162).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(486).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(61).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(116).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(585).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(501).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(691).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(164).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(235).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(692).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(243).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(481).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(174).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(246).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(566).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(653).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(480).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(168).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(575).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(552).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(170).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(182).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(155).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(59).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(548).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(249).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(250).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(707).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(248).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(121).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(484).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(253).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(670).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(153).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(255).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(108).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(258).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(259).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(485).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(555).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(583).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(637).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(658).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(125).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(714).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(618).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(84).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(678).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(662).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(528).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(482).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(535).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(586).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(470).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(213).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(157).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(627).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(161).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(705).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(611).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(292).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(710).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(711).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(641).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(706).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(712).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(98).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(551).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(700).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(57).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(474).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(487).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(540).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(642).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(646).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(564).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(159).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(546).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(130).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(708).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(716).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(683).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(247).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(625).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(257).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(506).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(60).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(527).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(516).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(416).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(601).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(623).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(114).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(69).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(460).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(577).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(584).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(600).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(431).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(571).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(149).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(541).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(699).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(680).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(518).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(2).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(438).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(358).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(179).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(147).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(587).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(190).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(617).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(56).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(226).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(201).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(135).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(652).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(543).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(422).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(94).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(86).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(205).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(687).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(671).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(644).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(688).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(513).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(240).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(560).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(215).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(214).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(624).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(224).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(614).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(689).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(703).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(427).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(171).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(158).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(589).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(193).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(647).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(163).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(549).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(633).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(694).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(68).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(204).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(519).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(545).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(561).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(695).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(85).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(616).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(88).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(718).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(440).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(241).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(638).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(442).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(380).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(634).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(488).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(697).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(603).unwrap(), 5)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(572).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(675).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(677).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(361).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(520).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(423).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(439).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(160).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(580).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(146).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(631).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(630).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(398).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(417).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(682).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(558).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(245).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(458).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(701).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(599).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(628).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(684).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(459).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(648).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(650).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(645).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(690).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(456).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(229).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(109).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(286).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(531).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(632).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(185).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(303).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(651).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(696).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(239).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(620).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(639).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(192).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(411).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(154).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(212).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(483).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(282).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(490).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(476).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(609).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(508).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(279).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(194).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(570).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(172).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(233).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(588).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(504).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(491).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(505).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(507).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(151).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(523).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(360).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(525).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(370).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(595).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(668).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(579).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(655).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(660).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(679).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(394).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(661).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(278).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(659).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(713).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(698).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(715).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(309).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(252).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(615).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(702).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(717).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(685).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(665).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(673).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(521).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(550).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(622).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(704).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(709).unwrap(), 0)).unwrap(),
        IndexEntry::try_from((PageIndex::try_from(603).unwrap(), 3)).unwrap(),
    ];

    let page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(1).unwrap(),
            page_type: PageType::Plain(PlainPageType::Tracks),
            next_page: PageIndex::try_from(2).unwrap(),
            unknown1: 29871,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(0)
                .with_num_rows_valid(0),
            page_flags: PageFlags(100),
            free_size: 0,
            used_size: 0,
        },
        content: PageContent::Index(IndexPageContent {
            header: IndexPageHeader {
                unknown_a: 2,
                unknown_b: 179,
                next_offset: 272,
                page_index: PageIndex::try_from(1).unwrap(),
                next_page: PageIndex::try_from(2).unwrap(),
                num_entries: 272,
                first_empty: 8191,
            },
            entries,
        }),
    };

    let page_size = 4096;
    test_roundtrip_with_args(
        include_bytes!("../../data/pdb/unit_tests/index_page.bin"),
        page,
        (page_size, DatabaseType::Plain),
        (page_size,),
    );
}
