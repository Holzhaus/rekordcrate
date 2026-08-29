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
use diesel::helper_types::{AsSelect, Find};
use diesel::prelude::*;
use diesel::query_builder::{InsertStatement, QueryId};
use diesel::query_dsl::methods::{ExecuteDsl, SelectDsl};
use diesel::query_dsl::LoadQuery;
use diesel::sqlite::Sqlite;
use diesel_derive_newtype::DieselNewType;

use super::schema;
use crate::util::FileType;

/// Base trait for table-backed model types.
///
/// Provides [`insert`] and [`all`] as default implementations built on diesel.
/// Table linkage comes from [`HasTable`]: single-PK structs derive it via
/// `#[derive(Identifiable)]`; composite-PK structs implement it manually.
///
/// The `where` clause repeats diesel's internal constraints so the default
/// method bodies compile in a generic context without downstream callers needing
/// to spell them out.
pub trait TableRecord: Sized + HasTable + Selectable<Sqlite>
where
    for<'a> &'a Self: Insertable<<Self as HasTable>::Table>,
    for<'a> InsertStatement<
        <Self as HasTable>::Table,
        <&'a Self as Insertable<<Self as HasTable>::Table>>::Values,
    >: ExecuteDsl<SqliteConnection>,
{
    fn insert(&self, conn: &mut SqliteConnection) -> QueryResult<usize> {
        diesel::insert_into(Self::table())
            .values(self)
            .execute(conn)
    }

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

/// Extension of [`TableRecord`] for tables with a single integer primary key.
///
/// Adds [`by_id`](Self::by_id) using the associated [`Id`](Self::Id) type.
/// The boilerplate `where` clause is identical to [`TableRecord`]'s because
/// Rust does not yet propagate supertrait bounds to concrete method bodies.
pub trait IdentifiableRecord: TableRecord
where
    for<'a> &'a Self: Insertable<<Self as HasTable>::Table>,
    for<'a> InsertStatement<
        <Self as HasTable>::Table,
        <&'a Self as Insertable<<Self as HasTable>::Table>>::Values,
    >: ExecuteDsl<SqliteConnection>,
{
    /// The primary key type for this table.
    type Id;

    fn by_id(conn: &mut SqliteConnection, id: Self::Id) -> QueryResult<Option<Self>>
    where
        <Self as Selectable<Sqlite>>::SelectExpression: QueryId,
        <Self as HasTable>::Table: diesel::query_dsl::methods::FindDsl<Self::Id>,
        Find<<Self as HasTable>::Table, Self::Id>: SelectDsl<AsSelect<Self, Sqlite>>,
        <Find<<Self as HasTable>::Table, Self::Id> as SelectDsl<AsSelect<Self, Sqlite>>>::Output:
            for<'query> LoadQuery<'query, SqliteConnection, Self>,
    {
        Self::table()
            .find(id)
            .select(Self::as_select())
            .get_result(conn)
            .optional()
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

// All Id newtypes wrap an i32. These conversions let callers use `Id::from(n)` and
// `i32::from(id)` without needing to access the inner field directly.
macro_rules! impl_id_conversions {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<i32> for $ty {
                fn from(value: i32) -> Self { Self(value) }
            }
            impl From<$ty> for i32 {
                fn from(value: $ty) -> Self { value.0 }
            }
        )*
    }
}
impl_id_conversions!(
    AlbumId,
    ArtistId,
    CategoryId,
    ColorId,
    ContentId,
    CueId,
    GenreId,
    HistoryId,
    HotCueBankListId,
    ImageId,
    KeyId,
    LabelId,
    MenuItemId,
    MyTagId,
    PlaylistId,
    SortId,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFlag {
    Off,
    On,
    Other(i32),
}

impl BinaryFlag {
    #[must_use]
    pub fn as_i32(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::On => 1,
            Self::Other(value) => value,
        }
    }
}

impl From<bool> for BinaryFlag {
    fn from(value: bool) -> Self {
        if value {
            Self::On
        } else {
            Self::Off
        }
    }
}

impl From<i32> for BinaryFlag {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::On,
            other => Self::Other(other),
        }
    }
}

impl From<BinaryFlag> for i32 {
    fn from(value: BinaryFlag) -> Self {
        value.as_i32()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFileType {
    Unknown,
    Mp3,
    M4a,
    Flac,
    Wav,
    Aiff,
    Other(i32),
}

impl ContentFileType {
    #[must_use]
    pub fn as_i32(self) -> i32 {
        match self {
            Self::Unknown => 0,
            Self::Mp3 => 1,
            Self::M4a => 4,
            Self::Flac => 5,
            Self::Wav => 11,
            Self::Aiff => 12,
            Self::Other(value) => value,
        }
    }
}

impl From<i32> for ContentFileType {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Mp3,
            4 => Self::M4a,
            5 => Self::Flac,
            11 => Self::Wav,
            12 => Self::Aiff,
            other => Self::Other(other),
        }
    }
}

impl From<FileType> for ContentFileType {
    fn from(value: FileType) -> Self {
        match value {
            FileType::Unknown => Self::Unknown,
            FileType::Mp3 => Self::Mp3,
            FileType::M4a => Self::M4a,
            FileType::Flac => Self::Flac,
            FileType::Wav => Self::Wav,
            FileType::Aiff => Self::Aiff,
            FileType::Other(value) => Self::Other(i32::from(value)),
        }
    }
}

impl From<ContentFileType> for i32 {
    fn from(value: ContentFileType) -> Self {
        value.as_i32()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaylistAttribute {
    Default,
    Other(i32),
}

impl PlaylistAttribute {
    #[must_use]
    pub fn as_i32(self) -> i32 {
        match self {
            Self::Default => 0,
            Self::Other(value) => value,
        }
    }
}

impl From<PlaylistAttribute> for i32 {
    fn from(value: PlaylistAttribute) -> Self {
        value.as_i32()
    }
}

/// Represents the background color theme of the Rekordbox library view.
/// Currently always [`Default`](Self::Default) in exported databases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundColorType {
    Default,
    Other(i32),
}

impl BackgroundColorType {
    #[must_use]
    pub fn as_i32(self) -> i32 {
        match self {
            Self::Default => 0,
            Self::Other(value) => value,
        }
    }
}

impl From<BackgroundColorType> for i32 {
    fn from(value: BackgroundColorType) -> Self {
        value.as_i32()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
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

impl IdentifiableRecord for Album {
    type Id = AlbumId;
}

impl Album {
    pub fn new(album_id: AlbumId, name: impl Into<String>) -> Self {
        Self {
            album_id,
            name: name.into(),
            artist_id: None,
            image_id: None,
            is_complation: None,
            name_for_search: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::artist, primary_key(artist_id), check_for_backend(Sqlite))]
pub struct Artist {
    pub artist_id: ArtistId,
    pub name: String,
    pub name_for_search: Option<String>,
}

impl TableRecord for Artist {}

impl IdentifiableRecord for Artist {
    type Id = ArtistId;
}

impl Artist {
    pub fn new(artist_id: ArtistId, name: impl Into<String>) -> Self {
        Self {
            artist_id,
            name: name.into(),
            name_for_search: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::category, primary_key(category_id), check_for_backend(Sqlite))]
pub struct Category {
    pub category_id: CategoryId,
    pub menu_item_id: MenuItemId,
    pub sequence_no: i32,
    pub is_visible: i32,
}

impl Category {
    pub fn new(
        category_id: CategoryId,
        menu_item_id: MenuItemId,
        sequence_no: i32,
        is_visible: BinaryFlag,
    ) -> Self {
        Self {
            category_id,
            menu_item_id,
            sequence_no,
            is_visible: i32::from(is_visible),
        }
    }
}

impl TableRecord for Category {}

impl IdentifiableRecord for Category {
    type Id = CategoryId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::color, primary_key(color_id), check_for_backend(Sqlite))]
pub struct Color {
    pub color_id: ColorId,
    pub name: String,
}

impl Color {
    pub fn new(color_id: ColorId, name: impl Into<String>) -> Self {
        Self {
            color_id,
            name: name.into(),
        }
    }
}

impl TableRecord for Color {}

impl IdentifiableRecord for Color {
    type Id = ColorId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
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

impl IdentifiableRecord for Content {
    type Id = ContentId;
}

impl Content {
    pub fn builder(content_id: ContentId) -> ContentBuilder {
        ContentBuilder::new(content_id)
    }
}

/// Builder for [`Content`] rows.
#[derive(Debug)]
pub struct ContentBuilder(Content);

impl ContentBuilder {
    fn new(content_id: ContentId) -> Self {
        Self(Content {
            content_id,
            title: None,
            title_for_search: None,
            subtitle: None,
            bpmx100: None,
            length: None,
            track_no: None,
            disc_no: None,
            artist_id_artist: None,
            artist_id_remixer: None,
            artist_id_original_artist: None,
            artist_id_composer: None,
            artist_id_lyricist: None,
            album_id: None,
            genre_id: None,
            label_id: None,
            key_id: None,
            color_id: None,
            image_id: None,
            dj_comment: None,
            rating: None,
            release_year: None,
            release_date: None,
            date_created: None,
            date_added: None,
            path: None,
            file_name: None,
            file_size: None,
            file_type: None,
            bitrate: None,
            bit_depth: None,
            sampling_rate: None,
            isrc: None,
            dj_play_count: None,
            is_hot_cue_auto_load_on: None,
            is_kuvo_deliver_status_on: None,
            kuvo_delivery_comment: None,
            master_db_id: None,
            master_content_id: None,
            analysis_data_file_path: None,
            analysed_bits: None,
            content_link: None,
            has_modified: None,
            cue_update_count: None,
            analysis_data_update_count: None,
            information_update_count: None,
        })
    }

    pub fn build(self) -> Content {
        self.0
    }

    pub fn title(mut self, v: impl Into<String>) -> Self {
        self.0.title = Some(v.into());
        self
    }
    pub fn bpmx100(mut self, v: i32) -> Self {
        self.0.bpmx100 = Some(v);
        self
    }
    pub fn length(mut self, v: i32) -> Self {
        self.0.length = Some(v);
        self
    }
    pub fn track_no(mut self, v: i32) -> Self {
        self.0.track_no = Some(v);
        self
    }
    pub fn artist_id_artist(mut self, v: ArtistId) -> Self {
        self.0.artist_id_artist = Some(v);
        self
    }
    pub fn album_id(mut self, v: AlbumId) -> Self {
        self.0.album_id = Some(v);
        self
    }
    pub fn genre_id(mut self, v: GenreId) -> Self {
        self.0.genre_id = Some(v);
        self
    }
    pub fn key_id(mut self, v: KeyId) -> Self {
        self.0.key_id = Some(v);
        self
    }
    pub fn color_id(mut self, v: ColorId) -> Self {
        self.0.color_id = Some(v);
        self
    }
    pub fn dj_comment(mut self, v: impl Into<String>) -> Self {
        self.0.dj_comment = Some(v.into());
        self
    }
    pub fn rating(mut self, v: i32) -> Self {
        self.0.rating = Some(v);
        self
    }
    pub fn release_year(mut self, v: i32) -> Self {
        self.0.release_year = Some(v);
        self
    }
    pub fn date_created(mut self, v: impl Into<String>) -> Self {
        self.0.date_created = Some(v.into());
        self
    }
    pub fn date_added(mut self, v: impl Into<String>) -> Self {
        self.0.date_added = Some(v.into());
        self
    }
    pub fn path(mut self, v: impl Into<String>) -> Self {
        self.0.path = Some(v.into());
        self
    }
    pub fn file_name(mut self, v: impl Into<String>) -> Self {
        self.0.file_name = Some(v.into());
        self
    }
    pub fn file_size(mut self, v: i32) -> Self {
        self.0.file_size = Some(v);
        self
    }
    pub fn file_type(mut self, v: ContentFileType) -> Self {
        self.0.file_type = Some(v.into());
        self
    }
    pub fn bitrate(mut self, v: i32) -> Self {
        self.0.bitrate = Some(v);
        self
    }
    pub fn sampling_rate(mut self, v: i32) -> Self {
        self.0.sampling_rate = Some(v);
        self
    }
    pub fn dj_play_count(mut self, v: i32) -> Self {
        self.0.dj_play_count = Some(v);
        self
    }
    pub fn hot_cue_auto_load(mut self, v: BinaryFlag) -> Self {
        self.0.is_hot_cue_auto_load_on = Some(v.into());
        self
    }
    pub fn kuvo_deliver_status(mut self, v: BinaryFlag) -> Self {
        self.0.is_kuvo_deliver_status_on = Some(v.into());
        self
    }
    pub fn analysed_bits(mut self, v: i32) -> Self {
        self.0.analysed_bits = Some(v);
        self
    }
    pub fn analysis_data_file_path(mut self, v: impl Into<String>) -> Self {
        self.0.analysis_data_file_path = Some(v.into());
        self
    }
    pub fn has_modified(mut self, v: BinaryFlag) -> Self {
        self.0.has_modified = Some(v.into());
        self
    }
    pub fn cue_update_count(mut self, v: i32) -> Self {
        self.0.cue_update_count = Some(v);
        self
    }
    pub fn analysis_data_update_count(mut self, v: i32) -> Self {
        self.0.analysis_data_update_count = Some(v);
        self
    }
    pub fn information_update_count(mut self, v: i32) -> Self {
        self.0.information_update_count = Some(v);
        self
    }
    pub fn master_db_id(mut self, v: i32) -> Self {
        self.0.master_db_id = Some(v);
        self
    }
    pub fn master_content_id(mut self, v: i32) -> Self {
        self.0.master_content_id = Some(v);
        self
    }
    pub fn content_link(mut self, v: i32) -> Self {
        self.0.content_link = Some(v);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
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

impl IdentifiableRecord for Cue {
    type Id = CueId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::genre, primary_key(genre_id), check_for_backend(Sqlite))]
pub struct Genre {
    pub genre_id: GenreId,
    pub name: String,
}

impl TableRecord for Genre {}

impl IdentifiableRecord for Genre {
    type Id = GenreId;
}

impl Genre {
    pub fn new(genre_id: GenreId, name: impl Into<String>) -> Self {
        Self {
            genre_id,
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::history, primary_key(history_id), check_for_backend(Sqlite))]
pub struct History {
    pub history_id: HistoryId,
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub history_id_parent: HistoryId,
}

impl TableRecord for History {}

impl IdentifiableRecord for History {
    type Id = HistoryId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable)]
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
// We implement HasTable manually so TableRecord::insert() still works.
impl HasTable for HistoryContent {
    type Table = schema::history_content::table;

    fn table() -> Self::Table {
        schema::history_content::table
    }
}

impl TableRecord for HistoryContent {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
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

impl IdentifiableRecord for HotCueBankList {
    type Id = HotCueBankListId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable)]
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

impl HasTable for HotCueBankListCue {
    type Table = schema::hot_cue_bank_list_cue::table;

    fn table() -> Self::Table {
        schema::hot_cue_bank_list_cue::table
    }
}

impl TableRecord for HotCueBankListCue {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::image, primary_key(image_id), check_for_backend(Sqlite))]
pub struct Image {
    pub image_id: ImageId,
    pub path: String,
}

impl TableRecord for Image {}

impl IdentifiableRecord for Image {
    type Id = ImageId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::key, primary_key(key_id), check_for_backend(Sqlite))]
pub struct Key {
    pub key_id: KeyId,
    pub name: String,
}

impl Key {
    pub fn new(key_id: KeyId, name: impl Into<String>) -> Self {
        Self {
            key_id,
            name: name.into(),
        }
    }
}

impl TableRecord for Key {}

impl IdentifiableRecord for Key {
    type Id = KeyId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::label, primary_key(label_id), check_for_backend(Sqlite))]
pub struct Label {
    pub label_id: LabelId,
    pub name: String,
}

impl TableRecord for Label {}

impl IdentifiableRecord for Label {
    type Id = LabelId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::menu_item, primary_key(menu_item_id), check_for_backend(Sqlite))]
pub struct MenuItem {
    pub menu_item_id: MenuItemId,
    pub kind: i32,
    pub name: String,
}

impl MenuItem {
    pub fn new(menu_item_id: MenuItemId, kind: i32, name: impl Into<String>) -> Self {
        Self {
            menu_item_id,
            kind,
            name: name.into(),
        }
    }
}

impl TableRecord for MenuItem {}

impl IdentifiableRecord for MenuItem {
    type Id = MenuItemId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::my_tag, primary_key(my_tag_id), check_for_backend(Sqlite))]
pub struct MyTag {
    pub my_tag_id: MyTagId,
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub my_tag_id_parent: MyTagId,
}

impl TableRecord for MyTag {}

impl IdentifiableRecord for MyTag {
    type Id = MyTagId;
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable)]
#[diesel(
    table_name = schema::my_tag_content,
    primary_key(my_tag_id, content_id),
    check_for_backend(Sqlite)
)]
pub struct MyTagContent {
    pub my_tag_id: MyTagId,
    pub content_id: ContentId,
}

impl HasTable for MyTagContent {
    type Table = schema::my_tag_content::table;

    fn table() -> Self::Table {
        schema::my_tag_content::table
    }
}

impl TableRecord for MyTagContent {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
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

impl IdentifiableRecord for Playlist {
    type Id = PlaylistId;
}

impl Playlist {
    pub fn new(
        playlist_id: PlaylistId,
        sequence_no: i32,
        name: impl Into<String>,
        attribute: PlaylistAttribute,
    ) -> Self {
        Self {
            playlist_id,
            sequence_no,
            name: name.into(),
            image_id: None,
            attribute: i32::from(attribute),
            playlist_id_parent: PlaylistId(0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable)]
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

impl HasTable for PlaylistContent {
    type Table = schema::playlist_content::table;

    fn table() -> Self::Table {
        schema::playlist_content::table
    }
}

impl TableRecord for PlaylistContent {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::property, check_for_backend(Sqlite))]
pub struct Property {
    pub device_name: String,
    pub db_version: String,
    pub number_of_contents: i32,
    pub created_date: String,
    pub back_ground_color_type: i32,
    pub my_tag_master_dbid: i64,
}

impl HasTable for Property {
    type Table = schema::property::table;

    fn table() -> Self::Table {
        schema::property::table
    }
}

impl TableRecord for Property {}

impl Property {
    pub fn new(
        device_name: impl Into<String>,
        db_version: impl Into<String>,
        number_of_contents: i32,
        created_date: impl Into<String>,
        my_tag_master_dbid: i64,
    ) -> Self {
        Self {
            device_name: device_name.into(),
            db_version: db_version.into(),
            number_of_contents,
            created_date: created_date.into(),
            back_ground_color_type: 0,
            my_tag_master_dbid,
        }
    }

    pub fn first(conn: &mut SqliteConnection) -> QueryResult<Option<Self>> {
        diesel::QueryDsl::select(schema::property::table, Self::as_select())
            .first(conn)
            .optional()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable)]
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

impl HasTable for RecommendedLike {
    type Table = schema::recommended_like::table;

    fn table() -> Self::Table {
        schema::recommended_like::table
    }
}

impl TableRecord for RecommendedLike {}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = schema::sort, primary_key(sort_id), check_for_backend(Sqlite))]
pub struct Sort {
    pub sort_id: SortId,
    pub menu_item_id: MenuItemId,
    pub sequence_no: i32,
    pub is_visible: i32,
    pub is_selected_as_sub_column: i32,
}

impl Sort {
    pub fn new(
        sort_id: SortId,
        menu_item_id: MenuItemId,
        sequence_no: i32,
        is_visible: BinaryFlag,
        is_selected_as_sub_column: BinaryFlag,
    ) -> Self {
        Self {
            sort_id,
            menu_item_id,
            sequence_no,
            is_visible: i32::from(is_visible),
            is_selected_as_sub_column: i32::from(is_selected_as_sub_column),
        }
    }
}

impl TableRecord for Sort {}

impl IdentifiableRecord for Sort {
    type Id = SortId;
}
