// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0
use super::*;

#[test]
fn allocate_row_empty_page() {
    let page = {
        let row_groups = vec![];
        let rows = BTreeMap::<u16, Row>::new();

        let mut page = Page {
            header: PageHeader {
                page_index: PageIndex::try_from(12).unwrap(),
                page_type: PageType::Plain(PlainPageType::Keys),
                next_page: PageIndex::try_from(51).unwrap(),
                unknown1: 13484,
                unknown2: 0,
                packed_row_counts: PackedRowCounts::new()
                    .with_num_rows(0)
                    .with_num_rows_valid(0),
                page_flags: PageFlags::new_data_page(),
                free_size: 3000,
                used_size: 0,
            },
            content: PageContent::Data(DataPageContent {
                header: DataPageHeader {
                    unknown5: 0,
                    unknown_not_num_rows_large: 0,
                    unknown6: 0,
                    unknown7: 0,
                },
                row_groups,
                rows,
            }),
        };

        let row = Row::Plain(PlainRow::Key(Key {
            id: KeyId(1),
            id2: 1,
            name: "Emin".parse().unwrap(),
        }));
        let bytes_required = row.heap_bytes_required(());
        let insert = page.allocate_row(bytes_required).unwrap();
        insert(row);

        page
    };

    let expected_page = {
        let row_groups = vec![RowGroup {
            row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0000],
            row_presence_flags: 0x0001,
            unknown: 0x0000,
        }];

        let rows: BTreeMap<u16, Row> = vec![(
            0x0000,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(1),
                id2: 1,
                name: "Emin".parse().unwrap(),
            })),
        )]
        .into_iter()
        .collect();

        Page {
            header: PageHeader {
                page_index: PageIndex::try_from(12).unwrap(),
                page_type: PageType::Plain(PlainPageType::Keys),
                next_page: PageIndex::try_from(51).unwrap(),
                unknown1: 13484,
                unknown2: 0,
                packed_row_counts: PackedRowCounts::new()
                    .with_num_rows(1)
                    .with_num_rows_valid(1),
                page_flags: PageFlags::new_data_page(),
                // The row requires 13 bytes but we align to 4 bytes => 16 bytes.
                // The row group and offset require 4 + 2 = 6 bytes, deducted from free_size.
                free_size: 2978,
                used_size: 16,
            },
            content: PageContent::Data(DataPageContent {
                header: DataPageHeader {
                    unknown5: 0,
                    unknown_not_num_rows_large: 0,
                    unknown6: 0,
                    unknown7: 0,
                },
                row_groups,
                rows,
            }),
        }
    };

    assert_eq!(expected_page, page);
}

#[test]
fn allocate_row_existing_row_group() {
    let page = {
        let row_groups = vec![RowGroup {
            row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0000],
            row_presence_flags: 0x0001,
            unknown: 0x0000,
        }];

        let rows: BTreeMap<u16, Row> = vec![(
            0x0000,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(1),
                id2: 1,
                name: "Emin".parse().unwrap(),
            })),
        )]
        .into_iter()
        .collect();

        let mut page = Page {
            header: PageHeader {
                page_index: PageIndex::try_from(12).unwrap(),
                page_type: PageType::Plain(PlainPageType::Keys),
                next_page: PageIndex::try_from(51).unwrap(),
                unknown1: 13484,
                unknown2: 0,
                packed_row_counts: PackedRowCounts::new()
                    .with_num_rows(1)
                    .with_num_rows_valid(1),
                page_flags: PageFlags::new_data_page(),
                free_size: 2978,
                used_size: 16,
            },
            content: PageContent::Data(DataPageContent {
                header: DataPageHeader {
                    unknown5: 0,
                    unknown_not_num_rows_large: 0,
                    unknown6: 0,
                    unknown7: 0,
                },
                row_groups,
                rows,
            }),
        };

        let row = Row::Plain(PlainRow::Key(Key {
            id: KeyId(2),
            id2: 2,
            name: "Fmaj".parse().unwrap(),
        }));
        let bytes_required = row.heap_bytes_required(());
        let insert = page.allocate_row(bytes_required).unwrap();
        insert(row);

        page
    };

    let expected_page = {
        let row_groups = vec![RowGroup {
            row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0010, 0x0000],
            row_presence_flags: 0x0003,
            unknown: 0x0000,
        }];

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
        ]
        .into_iter()
        .collect();

        Page {
            header: PageHeader {
                page_index: PageIndex::try_from(12).unwrap(),
                page_type: PageType::Plain(PlainPageType::Keys),
                next_page: PageIndex::try_from(51).unwrap(),
                unknown1: 13484,
                unknown2: 0,
                packed_row_counts: PackedRowCounts::new()
                    .with_num_rows(2)
                    .with_num_rows_valid(2),
                page_flags: PageFlags::new_data_page(),
                // The row requires 13 bytes but we align to 4 bytes => 16 bytes.
                // The additional row group space is just 2 bytes for the offset.
                free_size: 2960,
                used_size: 32,
            },
            content: PageContent::Data(DataPageContent {
                header: DataPageHeader {
                    unknown5: 0,
                    unknown_not_num_rows_large: 0,
                    unknown6: 0,
                    unknown7: 0,
                },
                row_groups,
                rows,
            }),
        }
    };

    assert_eq!(expected_page, page);
}

#[test]
fn allocate_row_interrupted() {
    let page = {
        let row_groups = vec![RowGroup {
            row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0000],
            row_presence_flags: 0x0001,
            unknown: 0x0000,
        }];

        let rows: BTreeMap<u16, Row> = vec![(
            0x0000,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(1),
                id2: 1,
                name: "Emin".parse().unwrap(),
            })),
        )]
        .into_iter()
        .collect();

        let mut page = Page {
            header: PageHeader {
                page_index: PageIndex::try_from(12).unwrap(),
                page_type: PageType::Plain(PlainPageType::Keys),
                next_page: PageIndex::try_from(51).unwrap(),
                unknown1: 13484,
                unknown2: 0,
                packed_row_counts: PackedRowCounts::new()
                    .with_num_rows(1)
                    .with_num_rows_valid(1),
                page_flags: PageFlags::new_data_page(),
                free_size: 2978,
                used_size: 16,
            },
            content: PageContent::Data(DataPageContent {
                header: DataPageHeader {
                    unknown5: 0,
                    unknown_not_num_rows_large: 0,
                    unknown6: 0,
                    unknown7: 0,
                },
                row_groups,
                rows,
            }),
        };

        let bytes_required = 16;
        let insert = page.allocate_row(bytes_required).unwrap();
        drop(insert);

        page
    };

    let expected_page = {
        // The insert was interrupted so we should have an offset
        // but no matching presence flag.
        let row_groups = vec![RowGroup {
            row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0010, 0x0000],
            row_presence_flags: 0x0001,
            unknown: 0x0000,
        }];

        let rows: BTreeMap<u16, Row> = vec![(
            0x0000,
            Row::Plain(PlainRow::Key(Key {
                id: KeyId(1),
                id2: 1,
                name: "Emin".parse().unwrap(),
            })),
        )]
        .into_iter()
        .collect();

        Page {
            header: PageHeader {
                page_index: PageIndex::try_from(12).unwrap(),
                page_type: PageType::Plain(PlainPageType::Keys),
                next_page: PageIndex::try_from(51).unwrap(),
                unknown1: 13484,
                unknown2: 0,
                // The insert was interrupted so we should have an
                // extra allocated row but no extra valid rows.
                packed_row_counts: PackedRowCounts::new()
                    .with_num_rows(2)
                    .with_num_rows_valid(1),
                page_flags: PageFlags::new_data_page(),
                // We allocated 16 bytes for the row we didn't insert.
                // The additional row group space is just 2 bytes for the offset.
                free_size: 2960,
                used_size: 32,
            },
            content: PageContent::Data(DataPageContent {
                header: DataPageHeader {
                    unknown5: 0,
                    unknown_not_num_rows_large: 0,
                    unknown6: 0,
                    unknown7: 0,
                },
                row_groups,
                rows,
            }),
        }
    };

    assert_eq!(expected_page, page);
}

#[test]
fn allocate_row_full_row_group() {
    let page = {
        let row_groups = vec![RowGroup {
            row_offsets: [
                0x00dc, 0x00cc, 0x00c0, 0x00b4, 0x00a8, 0x0098, 0x0088, 0x0078, 0x0068, 0x0058,
                0x0048, 0x003c, 0x002c, 0x0020, 0x0010, 0x0000,
            ],
            row_presence_flags: 0xffff,
            unknown: 0x0000,
        }];

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
        ]
        .into_iter()
        .collect();

        let mut page = Page {
            header: PageHeader {
                page_index: PageIndex::try_from(12).unwrap(),
                page_type: PageType::Plain(PlainPageType::Keys),
                next_page: PageIndex::try_from(51).unwrap(),
                unknown1: 13484,
                unknown2: 0,
                packed_row_counts: PackedRowCounts::new()
                    .with_num_rows(16)
                    .with_num_rows_valid(16),
                page_flags: PageFlags::new_data_page(),
                free_size: 2000,
                used_size: 0x00ec,
            },
            content: PageContent::Data(DataPageContent {
                header: DataPageHeader {
                    unknown5: 0,
                    unknown_not_num_rows_large: 0,
                    unknown6: 0,
                    unknown7: 0,
                },
                row_groups,
                rows,
            }),
        };

        let new_row = Row::Plain(PlainRow::Key(Key {
            id: KeyId(17),
            id2: 17,
            name: "Amaj".parse().unwrap(),
        }));
        let bytes_required = new_row.heap_bytes_required(());
        let insert = page.allocate_row(bytes_required).unwrap();
        insert(new_row);

        page
    };

    let expected_page = {
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
                row_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x00ec],
                row_presence_flags: 0x0001,
                unknown: 0x0000,
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
        ]
        .into_iter()
        .collect();

        Page {
            header: PageHeader {
                page_index: PageIndex::try_from(12).unwrap(),
                page_type: PageType::Plain(PlainPageType::Keys),
                next_page: PageIndex::try_from(51).unwrap(),
                unknown1: 13484,
                unknown2: 0,
                packed_row_counts: PackedRowCounts::new()
                    .with_num_rows(17)
                    .with_num_rows_valid(17),
                page_flags: PageFlags::new_data_page(),
                free_size: 1978,
                used_size: 0x00fc,
            },
            content: PageContent::Data(DataPageContent {
                header: DataPageHeader {
                    unknown5: 0,
                    unknown_not_num_rows_large: 0,
                    unknown6: 0,
                    unknown7: 0,
                },
                row_groups,
                rows,
            }),
        }
    };

    assert_eq!(expected_page, page);
}

#[test]
fn allocate_row_full_page() {
    let mut page = Page {
        header: PageHeader {
            page_index: PageIndex::try_from(12).unwrap(),
            page_type: PageType::Plain(PlainPageType::Keys),
            next_page: PageIndex::try_from(51).unwrap(),
            unknown1: 13484,
            unknown2: 0,
            packed_row_counts: PackedRowCounts::new()
                .with_num_rows(0)
                .with_num_rows_valid(0),
            page_flags: PageFlags::new_data_page(),
            // Intentionally tiny to fake a full page without constructing any rows.
            free_size: 1,
            used_size: 0,
        },
        content: PageContent::Data(DataPageContent {
            header: DataPageHeader {
                unknown5: 0,
                unknown_not_num_rows_large: 0,
                unknown6: 0,
                unknown7: 0,
            },
            row_groups: vec![],
            rows: BTreeMap::new(),
        }),
    };

    let row = Row::Plain(PlainRow::Key(Key {
        id: KeyId(1),
        id2: 1,
        name: "Emin".parse().unwrap(),
    }));
    let bytes_required = row.heap_bytes_required(());

    let insert = page.allocate_row(bytes_required);
    assert!(insert.is_none());
    drop(insert);
    assert_eq!(0, page.header.used_size);
}
