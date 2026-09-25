// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

// model.rs contains generated/mechanical Diesel struct definitions. Public doc comments
// on every field and method would add ~1000 lines of boilerplate with negligible value,
// so we allow missing_docs here; the module-level doc on device_library_plus covers the
// overall design.
#![allow(missing_docs)]
#![allow(clippy::must_use_candidate)]

use diesel::associations::HasTable;
use diesel::helper_types::AsSelect;
use diesel::prelude::*;
use diesel::query_builder::QueryId;
use diesel::query_dsl::methods::SelectDsl;
use diesel::query_dsl::LoadQuery;
use diesel::sqlite::Sqlite;
use diesel_derive_newtype::DieselNewType;

use super::schema;

/// Base trait for table-backed model types.
///
/// Provides [`all`](Self::all) for the CLI dumper. Table linkage comes from
/// [`HasTable`]: single-PK structs derive it via `#[derive(Identifiable)]`, while
/// composite-PK structs implement it manually.
pub trait TableRecord: Sized + HasTable + Selectable<Sqlite> {
    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>>
    where
        <Self as Selectable<Sqlite>>::SelectExpression: QueryId,
        <Self as HasTable>::Table: SelectDsl<AsSelect<Self, Sqlite>>,
        <<Self as HasTable>::Table as SelectDsl<AsSelect<Self, Sqlite>>>::Output:
            for<'query> LoadQuery<'query, SqliteConnection, Self>,
    {
        diesel::QueryDsl::select(Self::table(), Self::as_select()).load(conn)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct AlbumId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct ArtistId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct CategoryId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct ColorId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct ContentId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct CueId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct GenreId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct HistoryId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct HotCueBankListId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct ImageId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct KeyId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct LabelId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct MenuItemId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct MyTagId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct PlaylistId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct SortId(pub i32);

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::album, primary_key(album_id), check_for_backend(Sqlite))]
pub struct Album {
    pub album_id: AlbumId,
    pub name: String,
    pub artist_id: Option<ArtistId>,
    pub image_id: Option<ImageId>,
    pub is_complation: Option<i32>,
    pub name_for_search: Option<String>,
}

impl TableRecord for Album {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::artist, primary_key(artist_id), check_for_backend(Sqlite))]
pub struct Artist {
    pub artist_id: ArtistId,
    pub name: String,
    pub name_for_search: Option<String>,
}

impl TableRecord for Artist {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::category, primary_key(category_id), check_for_backend(Sqlite))]
pub struct Category {
    pub category_id: CategoryId,
    pub menu_item_id: MenuItemId,
    pub sequence_no: i32,
    pub is_visible: i32,
}

impl TableRecord for Category {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::color, primary_key(color_id), check_for_backend(Sqlite))]
pub struct Color {
    pub color_id: ColorId,
    pub name: String,
}

impl TableRecord for Color {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::content, primary_key(content_id), check_for_backend(Sqlite))]
pub struct Content {
    pub content_id: ContentId,
    pub title: Option<String>,
    pub title_for_search: Option<String>,
    pub subtitle: Option<String>,
    pub bpmx100: Option<i32>,
    pub length: Option<i32>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub artist_id_artist: Option<ArtistId>,
    pub artist_id_remixer: Option<ArtistId>,
    pub artist_id_original_artist: Option<ArtistId>,
    pub artist_id_composer: Option<ArtistId>,
    pub artist_id_lyricist: Option<ArtistId>,
    pub album_id: Option<AlbumId>,
    pub genre_id: Option<GenreId>,
    pub label_id: Option<LabelId>,
    pub key_id: Option<KeyId>,
    pub color_id: Option<ColorId>,
    pub image_id: Option<ImageId>,
    pub dj_comment: Option<String>,
    pub rating: Option<i32>,
    pub release_year: Option<i32>,
    pub release_date: Option<String>,
    pub date_created: Option<String>,
    pub date_added: Option<String>,
    pub path: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i32>,
    pub file_type: Option<i32>,
    pub bitrate: Option<i32>,
    pub bit_depth: Option<i32>,
    pub sampling_rate: Option<i32>,
    pub isrc: Option<String>,
    pub dj_play_count: Option<i32>,
    pub is_hot_cue_auto_load_on: Option<i32>,
    pub is_kuvo_deliver_status_on: Option<i32>,
    pub kuvo_delivery_comment: Option<String>,
    pub master_db_id: Option<i32>,
    pub master_content_id: Option<i32>,
    pub analysis_data_file_path: Option<String>,
    pub analysed_bits: Option<i32>,
    pub content_link: Option<i32>,
    pub has_modified: Option<i32>,
    pub cue_update_count: Option<i32>,
    pub analysis_data_update_count: Option<i32>,
    pub information_update_count: Option<i32>,
}

impl TableRecord for Content {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::cue, primary_key(cue_id), check_for_backend(Sqlite))]
pub struct Cue {
    pub cue_id: CueId,
    pub content_id: Option<ContentId>,
    pub kind: Option<i32>,
    pub color_table_index: Option<i32>,
    pub cue_comment: Option<String>,
    pub is_active_loop: Option<i32>,
    pub beat_loop_numerator: Option<i32>,
    pub beat_loop_denominator: Option<i32>,
    pub in_usec: Option<i32>,
    pub out_usec: Option<i32>,
    pub in_150_frame_per_sec: Option<i32>,
    pub out_150_frame_per_sec: Option<i32>,
    pub in_mpeg_frame_number: Option<i32>,
    pub out_mpeg_frame_number: Option<i32>,
    pub in_mpeg_abs: Option<i32>,
    pub out_mpeg_abs: Option<i32>,
    pub in_decoding_start_frame_position: Option<i32>,
    pub out_decoding_start_frame_position: Option<i32>,
    pub in_file_offset_in_block: Option<i32>,
    pub out_file_offset_in_block: Option<i32>,
    pub in_number_of_sample_in_block: Option<i32>,
    pub out_number_of_sample_in_block: Option<i32>,
}

impl TableRecord for Cue {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::genre, primary_key(genre_id), check_for_backend(Sqlite))]
pub struct Genre {
    pub genre_id: GenreId,
    pub name: String,
}

impl TableRecord for Genre {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::history, primary_key(history_id), check_for_backend(Sqlite))]
pub struct History {
    pub history_id: HistoryId,
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub history_id_parent: HistoryId,
}

impl TableRecord for History {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable)]
#[diesel(
    table_name = schema::history_content,
    primary_key(history_id, content_id),
    check_for_backend(Sqlite)
)]
pub struct HistoryContent {
    pub history_id: HistoryId,
    pub content_id: ContentId,
    pub sequence_no: i32,
}

// Composite-PK tables cannot use #[derive(Identifiable)] for HasTable because
// diesel's Identifiable derive only supports single-column primary keys.
// We implement HasTable manually so TableRecord::all() still works.
impl HasTable for HistoryContent {
    type Table = schema::history_content::table;

    fn table() -> Self::Table {
        schema::history_content::table
    }
}

impl TableRecord for HistoryContent {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(
    table_name = schema::hot_cue_bank_list,
    primary_key(hot_cue_bank_list_id),
    check_for_backend(Sqlite)
)]
pub struct HotCueBankList {
    pub hot_cue_bank_list_id: HotCueBankListId,
    pub sequence_no: i32,
    pub name: Option<String>,
    pub image_id: Option<ImageId>,
    pub attribute: i32,
    pub hot_cue_bank_list_id_parent: Option<HotCueBankListId>,
}

impl TableRecord for HotCueBankList {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable)]
#[diesel(
    table_name = schema::hot_cue_bank_list_cue,
    primary_key(hot_cue_bank_list_id, cue_id),
    check_for_backend(Sqlite)
)]
pub struct HotCueBankListCue {
    pub hot_cue_bank_list_id: HotCueBankListId,
    pub cue_id: CueId,
    pub sequence_no: i32,
}

// Composite-PK tables cannot use #[derive(Identifiable)] for HasTable because
// diesel's Identifiable derive only supports single-column primary keys.
// We implement HasTable manually so TableRecord::all() still works.
impl HasTable for HotCueBankListCue {
    type Table = schema::hot_cue_bank_list_cue::table;

    fn table() -> Self::Table {
        schema::hot_cue_bank_list_cue::table
    }
}

impl TableRecord for HotCueBankListCue {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::image, primary_key(image_id), check_for_backend(Sqlite))]
pub struct Image {
    pub image_id: ImageId,
    pub path: String,
}

impl TableRecord for Image {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::key, primary_key(key_id), check_for_backend(Sqlite))]
pub struct Key {
    pub key_id: KeyId,
    pub name: String,
}

impl TableRecord for Key {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::label, primary_key(label_id), check_for_backend(Sqlite))]
pub struct Label {
    pub label_id: LabelId,
    pub name: String,
}

impl TableRecord for Label {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::menu_item, primary_key(menu_item_id), check_for_backend(Sqlite))]
pub struct MenuItem {
    pub menu_item_id: MenuItemId,
    pub kind: i32,
    pub name: String,
}

impl TableRecord for MenuItem {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::my_tag, primary_key(my_tag_id), check_for_backend(Sqlite))]
pub struct MyTag {
    pub my_tag_id: MyTagId,
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub my_tag_id_parent: MyTagId,
}

impl TableRecord for MyTag {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable)]
#[diesel(
    table_name = schema::my_tag_content,
    primary_key(my_tag_id, content_id),
    check_for_backend(Sqlite)
)]
pub struct MyTagContent {
    pub my_tag_id: MyTagId,
    pub content_id: ContentId,
}

// Composite-PK tables cannot use #[derive(Identifiable)] for HasTable because
// diesel's Identifiable derive only supports single-column primary keys.
// We implement HasTable manually so TableRecord::all() still works.
impl HasTable for MyTagContent {
    type Table = schema::my_tag_content::table;

    fn table() -> Self::Table {
        schema::my_tag_content::table
    }
}

impl TableRecord for MyTagContent {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::playlist, primary_key(playlist_id), check_for_backend(Sqlite))]
pub struct Playlist {
    pub playlist_id: PlaylistId,
    pub sequence_no: i32,
    pub name: String,
    pub image_id: Option<ImageId>,
    pub attribute: i32,
    pub playlist_id_parent: PlaylistId,
}

impl TableRecord for Playlist {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable)]
#[diesel(
    table_name = schema::playlist_content,
    primary_key(playlist_id, content_id),
    check_for_backend(Sqlite)
)]
pub struct PlaylistContent {
    pub playlist_id: PlaylistId,
    pub content_id: ContentId,
    pub sequence_no: i32,
}

// Composite-PK tables cannot use #[derive(Identifiable)] for HasTable because
// diesel's Identifiable derive only supports single-column primary keys.
// We implement HasTable manually so TableRecord::all() still works.
impl HasTable for PlaylistContent {
    type Table = schema::playlist_content::table;

    fn table() -> Self::Table {
        schema::playlist_content::table
    }
}

impl TableRecord for PlaylistContent {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable)]
#[diesel(table_name = schema::property, check_for_backend(Sqlite))]
pub struct Property {
    pub device_name: String,
    pub db_version: String,
    pub number_of_contents: i32,
    pub created_date: String,
    pub back_ground_color_type: i32,
    pub my_tag_master_dbid: i64,
}

// `property` has a single-column primary key (`device_name`) that is not named `id`, so
// #[derive(Identifiable)] cannot be used and we implement HasTable manually.
impl HasTable for Property {
    type Table = schema::property::table;

    fn table() -> Self::Table {
        schema::property::table
    }
}

impl TableRecord for Property {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable)]
#[diesel(
    table_name = schema::recommended_like,
    primary_key(content_id_1, content_id_2),
    check_for_backend(Sqlite)
)]
pub struct RecommendedLike {
    pub content_id_1: ContentId,
    pub content_id_2: ContentId,
    pub rating: i32,
    pub created_date: i32,
}

// Composite-PK tables cannot use #[derive(Identifiable)] for HasTable because
// diesel's Identifiable derive only supports single-column primary keys.
// We implement HasTable manually so TableRecord::all() still works.
impl HasTable for RecommendedLike {
    type Table = schema::recommended_like::table;

    fn table() -> Self::Table {
        schema::recommended_like::table
    }
}

impl TableRecord for RecommendedLike {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::sort, primary_key(sort_id), check_for_backend(Sqlite))]
pub struct Sort {
    pub sort_id: SortId,
    pub menu_item_id: MenuItemId,
    pub sequence_no: i32,
    pub is_visible: i32,
    pub is_selected_as_sub_column: i32,
}

impl TableRecord for Sort {}
