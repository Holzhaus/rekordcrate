// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

#![allow(missing_docs)]

diesel::table! {
    album (album_id) {
        album_id -> Integer,
        name -> Text,
        artist_id -> Nullable<Integer>,
        image_id -> Nullable<Integer>,
        #[sql_name = "isComplation"]
        is_complation -> Nullable<Integer>,
        #[sql_name = "nameForSearch"]
        name_for_search -> Nullable<Text>,
    }
}

diesel::table! {
    artist (artist_id) {
        artist_id -> Integer,
        name -> Text,
        #[sql_name = "nameForSearch"]
        name_for_search -> Nullable<Text>,
    }
}

diesel::table! {
    category (category_id) {
        category_id -> Integer,
        #[sql_name = "menuItem_id"]
        menu_item_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        #[sql_name = "isVisible"]
        is_visible -> Integer,
    }
}

diesel::table! {
    color (color_id) {
        color_id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    content (content_id) {
        content_id -> Integer,
        title -> Nullable<Text>,
        #[sql_name = "titleForSearch"]
        title_for_search -> Nullable<Text>,
        subtitle -> Nullable<Text>,
        bpmx100 -> Nullable<Integer>,
        length -> Nullable<Integer>,
        #[sql_name = "trackNo"]
        track_no -> Nullable<Integer>,
        #[sql_name = "discNo"]
        disc_no -> Nullable<Integer>,
        artist_id_artist -> Nullable<Integer>,
        artist_id_remixer -> Nullable<Integer>,
        #[sql_name = "artist_id_originalArtist"]
        artist_id_original_artist -> Nullable<Integer>,
        artist_id_composer -> Nullable<Integer>,
        artist_id_lyricist -> Nullable<Integer>,
        album_id -> Nullable<Integer>,
        genre_id -> Nullable<Integer>,
        label_id -> Nullable<Integer>,
        key_id -> Nullable<Integer>,
        color_id -> Nullable<Integer>,
        image_id -> Nullable<Integer>,
        #[sql_name = "djComment"]
        dj_comment -> Nullable<Text>,
        rating -> Nullable<Integer>,
        #[sql_name = "releaseYear"]
        release_year -> Nullable<Integer>,
        #[sql_name = "releaseDate"]
        release_date -> Nullable<Text>,
        #[sql_name = "dateCreated"]
        date_created -> Nullable<Text>,
        #[sql_name = "dateAdded"]
        date_added -> Nullable<Text>,
        path -> Nullable<Text>,
        #[sql_name = "fileName"]
        file_name -> Nullable<Text>,
        #[sql_name = "fileSize"]
        file_size -> Nullable<Integer>,
        #[sql_name = "fileType"]
        file_type -> Nullable<Integer>,
        bitrate -> Nullable<Integer>,
        #[sql_name = "bitDepth"]
        bit_depth -> Nullable<Integer>,
        #[sql_name = "samplingRate"]
        sampling_rate -> Nullable<Integer>,
        isrc -> Nullable<Text>,
        #[sql_name = "djPlayCount"]
        dj_play_count -> Nullable<Integer>,
        #[sql_name = "isHotCueAutoLoadOn"]
        is_hot_cue_auto_load_on -> Nullable<Integer>,
        #[sql_name = "isKuvoDeliverStatusOn"]
        is_kuvo_deliver_status_on -> Nullable<Integer>,
        #[sql_name = "kuvoDeliveryComment"]
        kuvo_delivery_comment -> Nullable<Text>,
        #[sql_name = "masterDbId"]
        master_db_id -> Nullable<Integer>,
        #[sql_name = "masterContentId"]
        master_content_id -> Nullable<Integer>,
        #[sql_name = "analysisDataFilePath"]
        analysis_data_file_path -> Nullable<Text>,
        #[sql_name = "analysedBits"]
        analysed_bits -> Nullable<Integer>,
        #[sql_name = "contentLink"]
        content_link -> Nullable<Integer>,
        #[sql_name = "hasModified"]
        has_modified -> Nullable<Integer>,
        #[sql_name = "cueUpdateCount"]
        cue_update_count -> Nullable<Integer>,
        #[sql_name = "analysisDataUpdateCount"]
        analysis_data_update_count -> Nullable<Integer>,
        #[sql_name = "informationUpdateCount"]
        information_update_count -> Nullable<Integer>,
    }
}

diesel::table! {
    cue (cue_id) {
        cue_id -> Integer,
        content_id -> Nullable<Integer>,
        kind -> Nullable<Integer>,
        #[sql_name = "colorTableIndex"]
        color_table_index -> Nullable<Integer>,
        #[sql_name = "cueComment"]
        cue_comment -> Nullable<Text>,
        #[sql_name = "isActiveLoop"]
        is_active_loop -> Nullable<Integer>,
        #[sql_name = "beatLoopNumerator"]
        beat_loop_numerator -> Nullable<Integer>,
        #[sql_name = "beatLoopDenominator"]
        beat_loop_denominator -> Nullable<Integer>,
        #[sql_name = "inUsec"]
        in_usec -> Nullable<Integer>,
        #[sql_name = "outUsec"]
        out_usec -> Nullable<Integer>,
        #[sql_name = "in150FramePerSec"]
        in_150_frame_per_sec -> Nullable<Integer>,
        #[sql_name = "out150FramePerSec"]
        out_150_frame_per_sec -> Nullable<Integer>,
        #[sql_name = "inMpegFrameNumber"]
        in_mpeg_frame_number -> Nullable<Integer>,
        #[sql_name = "outMpegFrameNumber"]
        out_mpeg_frame_number -> Nullable<Integer>,
        #[sql_name = "inMpegAbs"]
        in_mpeg_abs -> Nullable<Integer>,
        #[sql_name = "outMpegAbs"]
        out_mpeg_abs -> Nullable<Integer>,
        #[sql_name = "inDecodingStartFramePosition"]
        in_decoding_start_frame_position -> Nullable<Integer>,
        #[sql_name = "outDecodingStartFramePosition"]
        out_decoding_start_frame_position -> Nullable<Integer>,
        #[sql_name = "inFileOffsetInBlock"]
        in_file_offset_in_block -> Nullable<Integer>,
        #[sql_name = "OutFileOffsetInBlock"]
        out_file_offset_in_block -> Nullable<Integer>,
        #[sql_name = "inNumberOfSampleInBlock"]
        in_number_of_sample_in_block -> Nullable<Integer>,
        #[sql_name = "outNumberOfSampleInBlock"]
        out_number_of_sample_in_block -> Nullable<Integer>,
    }
}

diesel::table! {
    genre (genre_id) {
        genre_id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    history (history_id) {
        history_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        name -> Text,
        attribute -> Integer,
        #[sql_name = "history_id_parent"]
        history_id_parent -> Integer,
    }
}

diesel::table! {
    history_content (history_id, content_id) {
        history_id -> Integer,
        content_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
    }
}

diesel::table! {
    #[sql_name = "hotCueBankList"]
    hot_cue_bank_list (hot_cue_bank_list_id) {
        #[sql_name = "hotCueBankList_id"]
        hot_cue_bank_list_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        name -> Nullable<Text>,
        image_id -> Nullable<Integer>,
        attribute -> Integer,
        #[sql_name = "hotCueBankList_id_parent"]
        hot_cue_bank_list_id_parent -> Nullable<Integer>,
    }
}

diesel::table! {
    #[sql_name = "hotCueBankList_cue"]
    hot_cue_bank_list_cue (hot_cue_bank_list_id, cue_id) {
        #[sql_name = "hotCueBankList_id"]
        hot_cue_bank_list_id -> Integer,
        cue_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
    }
}

diesel::table! {
    image (image_id) {
        image_id -> Integer,
        path -> Text,
    }
}

diesel::table! {
    key (key_id) {
        key_id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    label (label_id) {
        label_id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    #[sql_name = "menuItem"]
    menu_item (menu_item_id) {
        #[sql_name = "menuItem_id"]
        menu_item_id -> Integer,
        kind -> Integer,
        name -> Text,
    }
}

diesel::table! {
    #[sql_name = "myTag"]
    my_tag (my_tag_id) {
        #[sql_name = "myTag_id"]
        my_tag_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        name -> Text,
        attribute -> Integer,
        #[sql_name = "myTag_id_parent"]
        my_tag_id_parent -> Integer,
    }
}

diesel::table! {
    #[sql_name = "myTag_content"]
    my_tag_content (my_tag_id, content_id) {
        #[sql_name = "myTag_id"]
        my_tag_id -> Integer,
        content_id -> Integer,
    }
}

diesel::table! {
    playlist (playlist_id) {
        playlist_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        name -> Text,
        image_id -> Nullable<Integer>,
        attribute -> Integer,
        #[sql_name = "playlist_id_parent"]
        playlist_id_parent -> Integer,
    }
}

diesel::table! {
    playlist_content (playlist_id, content_id) {
        playlist_id -> Integer,
        content_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
    }
}

diesel::table! {
    property (device_name) {
        #[sql_name = "deviceName"]
        device_name -> Text,
        #[sql_name = "dbVersion"]
        db_version -> Text,
        #[sql_name = "numberOfContents"]
        number_of_contents -> Integer,
        #[sql_name = "createdDate"]
        created_date -> Text,
        #[sql_name = "backGroundColorType"]
        back_ground_color_type -> Integer,
        #[sql_name = "myTagMasterDBID"]
        my_tag_master_dbid -> BigInt,
    }
}

diesel::table! {
    #[sql_name = "recommendedLike"]
    recommended_like (content_id_1, content_id_2) {
        content_id_1 -> Integer,
        content_id_2 -> Integer,
        rating -> Integer,
        #[sql_name = "createdDate"]
        created_date -> Integer,
    }
}

diesel::table! {
    sort (sort_id) {
        sort_id -> Integer,
        #[sql_name = "menuItem_id"]
        menu_item_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        #[sql_name = "isVisible"]
        is_visible -> Integer,
        #[sql_name = "isSelectedAsSubColumn"]
        is_selected_as_sub_column -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    album,
    artist,
    category,
    color,
    content,
    cue,
    genre,
    history,
    history_content,
    hot_cue_bank_list,
    hot_cue_bank_list_cue,
    image,
    key,
    label,
    menu_item,
    my_tag,
    my_tag_content,
    playlist,
    playlist_content,
    property,
    recommended_like,
    sort,
);
