// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Export Pioneer device exports as Rekordbox XML documents.

use crate::anlz::{Content, Cue, CueList, CueType, ExtendedCue, ExtendedCueList, ANLZ};
use crate::device::{get_playlists, DeviceExportLoader, PlaylistNode};
use crate::pdb::io::Database;
use crate::pdb::{
    Album, AlbumId, Artist, ArtistId, Genre, GenreId, Key, KeyId, Label, LabelId, PlaylistEntry,
    PlaylistTreeNodeId, Track, TrackId,
};
use crate::util::FileType;
use crate::xml::{
    Collection, Document, PlaylistFolderNode, PlaylistGenericNode, PlaylistPlaylistNode,
    PlaylistTrack, Playlists, PositionMark, Product, Tempo,
};
use binrw::BinRead;
use fallible_iterator::FallibleIterator;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

impl DeviceExportLoader {
    /// Export this device export to a Rekordbox XML document.
    pub fn export_xml_document(&self) -> crate::Result<Document> {
        export_device_to_xml(self)
    }
}

/// Export a device export to a Rekordbox XML document.
pub fn export_device_to_xml(loader: &DeviceExportLoader) -> crate::Result<Document> {
    let mut db = loader.open_pdb_non_persistent()?;
    let export_path = loader.get_path();

    let artists = collect_artist_names(&mut db)?;
    let albums = collect_album_names(&mut db)?;
    let genres = collect_genre_names(&mut db)?;
    let keys = collect_key_names(&mut db)?;
    let labels = collect_label_names(&mut db)?;
    let playlist_entries = collect_playlist_entries(&mut db)?;
    let playlists = get_playlists(&mut db)?
        .into_iter()
        .map(|node| playlist_node_to_xml(node, &playlist_entries))
        .collect::<Vec<_>>();

    let mut tracks = Vec::new();
    db.iter_rows::<Track>()?.for_each(|track| {
        tracks.push(track_to_xml(
            export_path,
            track,
            &artists,
            &albums,
            &genres,
            &keys,
            &labels,
        )?);
        Ok(())
    })?;

    Ok(Document {
        version: "1.0.0".to_string(),
        product: Product {
            name: "rekordcrate".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            company: "rekordcrate".to_string(),
        },
        collection: Collection {
            entries: tracks.len() as i32,
            track: tracks,
        },
        playlists: Playlists {
            node: PlaylistFolderNode {
                name: "ROOT".to_string(),
                nodes: playlists,
            },
        },
    })
}

fn collect_artist_names<R: Read + Seek>(
    db: &mut Database<R>,
) -> crate::Result<HashMap<ArtistId, String>> {
    db.iter_rows::<Artist>()?
        .map(|artist| Ok((artist.id, string_value(&artist.offsets.name)?)))
        .collect()
}

fn collect_album_names<R: Read + Seek>(
    db: &mut Database<R>,
) -> crate::Result<HashMap<AlbumId, String>> {
    db.iter_rows::<Album>()?
        .map(|album| Ok((album.id(), string_value(album.name())?)))
        .collect()
}

fn collect_genre_names<R: Read + Seek>(
    db: &mut Database<R>,
) -> crate::Result<HashMap<GenreId, String>> {
    db.iter_rows::<Genre>()?
        .map(|genre| Ok((genre.id(), string_value(genre.name())?)))
        .collect()
}

fn collect_key_names<R: Read + Seek>(
    db: &mut Database<R>,
) -> crate::Result<HashMap<KeyId, String>> {
    db.iter_rows::<Key>()?
        .map(|key| Ok((key.id(), string_value(key.name())?)))
        .collect()
}

fn collect_label_names<R: Read + Seek>(
    db: &mut Database<R>,
) -> crate::Result<HashMap<LabelId, String>> {
    db.iter_rows::<Label>()?
        .map(|label| Ok((label.id(), string_value(label.name())?)))
        .collect()
}

fn collect_playlist_entries<R: Read + Seek>(
    db: &mut Database<R>,
) -> crate::Result<HashMap<PlaylistTreeNodeId, BTreeMap<u32, TrackId>>> {
    db.iter_rows::<PlaylistEntry>()?
        .map(|entry| Ok((entry.playlist_id, entry.entry_index, entry.track_id)))
        .fold(
            HashMap::<PlaylistTreeNodeId, BTreeMap<u32, TrackId>>::new(),
            |mut acc, (playlist_id, entry_index, track_id)| {
                acc.entry(playlist_id)
                    .or_default()
                    .insert(entry_index, track_id);
                Ok(acc)
            },
        )
}

fn track_to_xml(
    export_path: &Path,
    track: &Track,
    artists: &HashMap<ArtistId, String>,
    albums: &HashMap<AlbumId, String>,
    genres: &HashMap<GenreId, String>,
    keys: &HashMap<KeyId, String>,
    labels: &HashMap<LabelId, String>,
) -> crate::Result<crate::xml::Track> {
    let analysis = load_track_analysis(export_path, &string_value(track.offsets.analyze_path())?)?;
    let file_path = string_value(&track.offsets.file_path)?;

    Ok(crate::xml::Track {
        trackid: track.id.0 as i32,
        name: optional_string(string_value(&track.offsets.title)?),
        artist: optional_lookup(track.artist_id, artists),
        composer: empty_attribute(),
        album: optional_lookup(track.album_id(), albums),
        grouping: empty_attribute(),
        genre: optional_lookup(track.genre_id(), genres),
        kind: Some(file_type_name(track.file_type()).to_string()),
        size: Some(i64::from(track.file_size())),
        totaltime: Some(f64::from(track.duration())),
        discnumber: Some(i32::from(track.disc_number())),
        tracknumber: Some(track.track_number() as i32),
        year: Some(i32::from(track.year())),
        averagebpm: Some(f64::from(track.tempo()) / 100.0),
        datemodified: None,
        dateadded: optional_string(string_value(track.offsets.date_added())?),
        bitrate: Some(track.bitrate() as i32),
        samplerate: Some(f64::from(track.sample_rate())),
        comments: optional_string(string_value(track.offsets.comment())?),
        playcount: Some(i32::from(track.play_count())),
        lastplayed: None,
        rating: Some(i32::from(track.rating) * 51),
        location: file_location(export_path, &file_path)?,
        remixer: empty_attribute(),
        tonality: optional_lookup(track.key_id(), keys),
        label: optional_lookup(track.label_id(), labels),
        mix: optional_string(string_value(track.offsets.mix_name())?),
        colour: None,
        tempos: analysis.tempos,
        position_marks: analysis.position_marks,
    })
}

fn playlist_node_to_xml(
    node: PlaylistNode,
    playlist_entries: &HashMap<PlaylistTreeNodeId, BTreeMap<u32, TrackId>>,
) -> PlaylistGenericNode {
    match node {
        PlaylistNode::Folder(folder) => PlaylistGenericNode::Folder(PlaylistFolderNode {
            name: folder.name,
            nodes: folder
                .children
                .into_iter()
                .map(|child| playlist_node_to_xml(child, playlist_entries))
                .collect(),
        }),
        PlaylistNode::Playlist(playlist) => {
            let tracks = playlist_entries
                .get(&playlist.id)
                .into_iter()
                .flat_map(|entries| entries.values())
                .map(|track_id| PlaylistTrack {
                    key: track_id.0 as i32,
                })
                .collect();
            PlaylistGenericNode::Playlist(PlaylistPlaylistNode {
                name: playlist.name,
                keytype: "0".to_string(),
                tracks,
            })
        }
    }
}

#[derive(Default)]
struct TrackAnalysis {
    tempos: Vec<Tempo>,
    position_marks: Vec<PositionMark>,
}

fn load_track_analysis(export_path: &Path, analyze_path: &str) -> crate::Result<TrackAnalysis> {
    let mut analysis = TrackAnalysis::default();
    if analyze_path.is_empty() {
        return Ok(analysis);
    }

    let dat_path = export_file_path(export_path, analyze_path);
    if let Some(anlz) = read_anlz_if_present(&dat_path)? {
        analysis.tempos.extend(tempos_from_anlz(&anlz));
        analysis
            .position_marks
            .extend(position_marks_from_anlz(&anlz));
    }

    let ext_path = dat_path.with_extension("EXT");
    if let Some(anlz) = read_anlz_if_present(&ext_path)? {
        analysis
            .position_marks
            .extend(position_marks_from_anlz(&anlz));
    }

    let two_ex_path = dat_path.with_extension("2EX");
    if let Some(anlz) = read_anlz_if_present(&two_ex_path)? {
        analysis
            .position_marks
            .extend(position_marks_from_anlz(&anlz));
    }

    Ok(analysis)
}

fn read_anlz_if_present(path: &Path) -> crate::Result<Option<ANLZ>> {
    match File::open(path) {
        Ok(mut file) => Ok(Some(ANLZ::read(&mut file)?)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn tempos_from_anlz(anlz: &ANLZ) -> Vec<Tempo> {
    anlz.sections
        .iter()
        .filter_map(|section| match &section.content {
            Content::BeatGrid(beat_grid) => Some(&beat_grid.beats),
            _ => None,
        })
        .flat_map(|beats| beats.iter())
        .map(|beat| Tempo {
            inizio: f64::from(beat.time) / 1000.0,
            bpm: f64::from(beat.tempo) / 100.0,
            metro: "4/4".to_string(),
            battito: i32::from(beat.beat_number),
        })
        .collect()
}

fn position_marks_from_anlz(anlz: &ANLZ) -> Vec<PositionMark> {
    let mut basic_marks = Vec::new();
    let mut extended_marks = Vec::new();

    for section in &anlz.sections {
        match &section.content {
            Content::CueList(cue_list) => {
                basic_marks.extend(position_marks_from_cue_list(cue_list))
            }
            Content::ExtendedCueList(cue_list) => {
                extended_marks.extend(position_marks_from_extended_cue_list(cue_list));
            }
            _ => {}
        }
    }

    if extended_marks.is_empty() {
        basic_marks
    } else {
        extended_marks
    }
}

fn position_marks_from_cue_list(cue_list: &CueList) -> Vec<PositionMark> {
    cue_list
        .cues
        .iter()
        .map(|cue| position_mark_from_cue(cue, ""))
        .collect()
}

fn position_marks_from_extended_cue_list(cue_list: &ExtendedCueList) -> Vec<PositionMark> {
    cue_list
        .cues
        .iter()
        .map(position_mark_from_extended_cue)
        .collect()
}

fn position_mark_from_cue(cue: &Cue, name: &str) -> PositionMark {
    position_mark(cue.hot_cue, &cue.cue_type, cue.time, cue.loop_time, name)
}

fn position_mark_from_extended_cue(cue: &ExtendedCue) -> PositionMark {
    position_mark(
        cue.hot_cue,
        &cue.cue_type,
        cue.time,
        cue.loop_time,
        &cue.comment,
    )
}

fn position_mark(
    hot_cue: u32,
    cue_type: &CueType,
    time: u32,
    loop_time: u32,
    name: &str,
) -> PositionMark {
    let is_loop = matches!(cue_type, CueType::Loop);
    PositionMark {
        name: name.to_string(),
        mark_type: if is_loop { 4 } else { 0 },
        start: f64::from(time) / 1000.0,
        end: (is_loop && loop_time != u32::MAX).then_some(f64::from(loop_time) / 1000.0),
        num: if hot_cue == 0 {
            -1
        } else {
            hot_cue.saturating_sub(1) as i32
        },
    }
}

fn export_file_path(export_path: &Path, device_path: &str) -> PathBuf {
    export_path.join(device_path.trim_start_matches(['/', '\\']))
}

fn file_location(export_path: &Path, device_path: &str) -> crate::Result<String> {
    let path = export_path
        .canonicalize()?
        .join(device_path.trim_start_matches(['/', '\\']));
    let mut path = path.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = path.strip_prefix("//?/") {
        path = stripped.to_string();
    }
    Ok(format!("file://localhost/{}", percent_encode_path(&path)))
}

fn percent_encode_path(path: &str) -> String {
    path.bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn file_type_name(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Mp3 => "MP3 File",
        FileType::M4a => "M4A File",
        FileType::Flac => "FLAC File",
        FileType::Wav => "WAV File",
        FileType::Aiff => "AIFF File",
        FileType::Unknown | FileType::Other(_) => "Unknown File",
    }
}

fn optional_lookup<Id>(id: Id, names: &HashMap<Id, String>) -> Option<String>
where
    Id: Eq + std::hash::Hash,
{
    names.get(&id).cloned().and_then(optional_string)
}

fn optional_string(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn empty_attribute() -> Option<String> {
    Some(String::new())
}

fn string_value(value: &crate::pdb::string::DeviceSQLString) -> crate::Result<String> {
    Ok(value.clone().into_string()?)
}

#[cfg(test)]
mod tests {
    use super::{percent_encode_path, playlist_node_to_xml, position_mark};
    use crate::anlz::CueType;
    use crate::device::{Playlist, PlaylistNode};
    use crate::pdb::{PlaylistTreeNodeId, TrackId};
    use crate::xml::PlaylistGenericNode;
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn hot_cue_number_is_zero_based_for_xml() {
        let mark = position_mark(2, &CueType::Point, 12_345, u32::MAX, "B");

        assert_eq!(mark.name, "B");
        assert_eq!(mark.mark_type, 0);
        assert_eq!(mark.start, 12.345);
        assert_eq!(mark.end, None);
        assert_eq!(mark.num, 1);
    }

    #[test]
    fn memory_loop_keeps_end_position() {
        let mark = position_mark(0, &CueType::Loop, 1_000, 5_000, "");

        assert_eq!(mark.mark_type, 4);
        assert_eq!(mark.start, 1.0);
        assert_eq!(mark.end, Some(5.0));
        assert_eq!(mark.num, -1);
    }

    #[test]
    fn file_locations_escape_spaces() {
        assert_eq!(
            percent_encode_path("C:/Music/Demo Track 1.mp3"),
            "C:/Music/Demo%20Track%201.mp3"
        );
    }

    #[test]
    fn playlist_export_keeps_track_order() {
        let playlist_id = PlaylistTreeNodeId(7);
        let node = PlaylistNode::Playlist(Playlist {
            id: playlist_id,
            name: "Main".to_string(),
        });
        let entries = HashMap::from([(
            playlist_id,
            BTreeMap::from([(2, TrackId(20)), (1, TrackId(10))]),
        )]);

        let xml_node = playlist_node_to_xml(node, &entries);

        match xml_node {
            PlaylistGenericNode::Playlist(playlist) => {
                let keys = playlist
                    .tracks
                    .into_iter()
                    .map(|track| track.key)
                    .collect::<Vec<_>>();
                assert_eq!(playlist.name, "Main");
                assert_eq!(keys, vec![10, 20]);
            }
            PlaylistGenericNode::Folder(_) => panic!("expected playlist"),
        }
    }
}
