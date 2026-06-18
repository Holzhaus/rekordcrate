// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Parser for the Rekordbox XML file format for playlists sharing.
//!
//! The XML format includes all playlists information.
//!
//! # References
//!
//! - <https://rekordbox.com/en/support/developer/>
//! - <https://cdn.rekordbox.com/files/20200410160904/xml_format_list.pdf>
//! - <https://pyrekordbox.readthedocs.io/en/stable/formats/xml.html>
type NaiveDate = String; //Replace with "use chrono::naive::NaiveDate;"
use crate::anlz::{Content, CueListType, CueType, ANLZ};
use crate::device::{get_playlists, DeviceExportLoader, PlaylistNode};
use crate::pdb::{self, ArtistId, PlaylistEntry, PlaylistTreeNodeId, TrackId};
use crate::util::FileType;
use binrw::BinRead;
use fallible_iterator::FallibleIterator;
use serde::{de::Error, ser::Serializer, Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::{Path as FsPath, PathBuf};

/// The XML root element of a rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "DJ_PLAYLISTS")]
pub struct Document {
    /// Version of the XML format for share the playlists.
    ///
    /// The latest version is 1,0,0.
    #[serde(rename = "@Version")]
    pub version: String,

    /// Product information about the used rekordbox edition.
    #[serde(rename = "PRODUCT")]
    pub product: Product,

    /// The entire imported track collection.
    #[serde(rename = "COLLECTION")]
    pub collection: Collection,

    /// All playlists in the collection.
    #[serde(rename = "PLAYLISTS")]
    pub playlists: Playlists,
}

/// Information about the current rekordbox release.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Product {
    /// Name of product
    ///
    /// This name will be displayed in each application software.
    #[serde(rename = "@Name")]
    pub name: String,

    /// Version of application.
    #[serde(rename = "@Version")]
    pub version: String,

    /// Name of company.
    #[serde(rename = "@Company")]
    pub company: String,
}

/// The collection of all imported tracks in rekordbox.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Number of TRACK in COLLECTION.
    #[serde(rename = "@Entries")]
    pub entries: i32,

    /// Track entries of collection.
    #[serde(rename = "TRACK")]
    pub track: Vec<Track>,
}

/// A track imported to rekordbox.
/// "Location" is essential for each track ;
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Identification of track
    #[serde(rename = "@TrackID")]
    pub trackid: i32,

    /// Name of track
    #[serde(rename = "@Name")]
    pub name: Option<String>,

    /// Name of artist
    #[serde(rename = "@Artist")]
    pub artist: Option<String>,

    /// Name of composer (or producer)
    #[serde(rename = "@Composer")]
    pub composer: Option<String>,

    /// Name of Album
    #[serde(rename = "@Album")]
    pub album: Option<String>,

    /// Name of goupe
    #[serde(rename = "@Grouping")]
    pub grouping: Option<String>,

    /// Name of genre
    #[serde(rename = "@Genre")]
    pub genre: Option<String>,

    /// Type of audio file
    #[serde(rename = "@Kind")]
    pub kind: Option<String>,

    /// Size of audio file
    /// Unit : Octet
    #[serde(rename = "@Size")]
    pub size: Option<i64>,

    /// Duration of track
    /// Unit : Second (without decimal numbers)
    #[serde(rename = "@TotalTime")]
    pub totaltime: Option<f64>,

    /// Order number of the disc of the album
    #[serde(rename = "@DiscNumber")]
    pub discnumber: Option<i32>,

    /// Order number of the track in the album
    #[serde(rename = "@TrackNumber")]
    pub tracknumber: Option<i32>,

    /// Year of release
    #[serde(rename = "@Year")]
    pub year: Option<i32>,

    /// Value of average BPM
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@AverageBpm")]
    pub averagebpm: Option<f64>,

    /// Date of last modification
    /// Format : yyyy- mm- dd ; ex. : 2010- 08- 21
    #[serde(rename = "@DateModified")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datemodified: Option<NaiveDate>,

    /// Date of addition
    /// Format : yyyy- mm- dd ; ex. : 2010- 08- 21
    #[serde(rename = "@DateAdded")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dateadded: Option<NaiveDate>,

    /// Encoding bit rate
    /// Unit : Kbps
    #[serde(rename = "@BitRate")]
    pub bitrate: Option<i32>,

    /// Frequency of sampling
    /// Unit : Hertz
    #[serde(rename = "@SampleRate")]
    pub samplerate: Option<f64>,

    /// Comments
    #[serde(rename = "@Comments")]
    pub comments: Option<String>,

    /// Play count of the track
    #[serde(rename = "@PlayCount")]
    pub playcount: Option<i32>,

    /// Date of last playing
    /// Format : yyyy- mm- dd ; ex. : 2010- 08- 21
    #[serde(rename = "@LastPlayed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastplayed: Option<NaiveDate>,

    /// Rating of the track
    /// 0 star = "@0", 1 star = "51", 2 stars = "102", 3 stars = "153", 4 stars = "204", 5 stars = "255"
    #[serde(rename = "@Rating")]
    pub rating: Option<i32>, // TODO: Use StarRating type here

    /// Location of the file
    /// includes the file name (URI formatted)
    #[serde(rename = "@Location")]
    pub location: String,

    /// Name of remixer
    #[serde(rename = "@Remixer")]
    pub remixer: Option<String>,

    /// Tonality (Kind of musical key)
    #[serde(rename = "@Tonality")]
    pub tonality: Option<String>,

    /// Name of record label
    #[serde(rename = "@Label")]
    pub label: Option<String>,

    /// Name of mix
    #[serde(rename = "@Mix")]
    pub mix: Option<String>,

    /// Colour for track grouping
    /// RGB format (3 bytes) ; rekordbox : Rose(0xFF007F), Red(0xFF0000), Orange(0xFFA500), Lemon(0xFFFF00), Green(0x00FF00), Turquoise(0x25FDE9),  Blue(0x0000FF), Violet(0x660099)
    #[serde(rename = "@Colour")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,

    /// Tempo analysis results.
    #[serde(rename = "TEMPO")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tempos: Vec<Tempo>,

    /// Position marks in the track.
    #[serde(rename = "POSITION_MARK")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub position_marks: Vec<PositionMark>,
}

/// User rating of a track.
/// 0 star = "@0", 1 star = "51", 2 stars = "102", 3 stars = "153", 4 stars = "204", 5 stars = "255"
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum StarRating {
    /// Zero stars.
    Zero,

    /// One star.
    One,

    /// Two stars.
    Two,

    /// Three stars.
    Three,

    /// Four stars.
    Four,

    /// Five stars.
    Five,

    /// An unknown rating with its value.
    Unknown(i32),
}

/// Tempo analysis result of a track.
/// For BeatGrid; More than two "TEMPO" can exist for each track
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Tempo {
    /// Start position of BeatGrid
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@Inizio")]
    pub inizio: f64,

    /// Value of BPM
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@Bpm")]
    pub bpm: f64,

    /// Kind of musical meter (formatted)
    /// ex. 3/ 4, 4/ 4, 7/ 8…
    #[serde(rename = "@Metro")]
    pub metro: String,

    /// Beat number in the bar
    /// If the value of "Metro" is 4/ 4, the value should be 1, 2, 3 or 4.
    #[serde(rename = "@Battito")]
    pub battito: i32,
}

/// Position marker in a track.
/// More than two "POSITION MARK" can exist for each track
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PositionMark {
    /// Name of position mark
    #[serde(rename = "@Name")]
    pub name: String,

    /// Type of position mark
    /// Cue = "@0", Fade- In = "1", Fade- Out = "2", Load = "3",  Loop = " 4"
    #[serde(rename = "@Type")]
    pub mark_type: i32,

    /// Start position of position mark
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@Start")]
    pub start: f64,

    /// End position of position mark
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@End")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,

    /// Number for identification of the position mark
    /// rekordbox : Hot Cue A,  B,  C : "0", "1", "2"; Memory Cue : "- 1"
    #[serde(rename = "@Num")]
    pub num: i32,
}

/// Playlist collection.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Playlists {
    /// The root node of all playlists.
    #[serde(rename = "NODE")]
    pub node: PlaylistFolderNode,
}

/// The type of a playlist node.
#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "@Type")]
pub enum PlaylistGenericNode {
    /// A folder containing more playlists.
    #[serde(rename = "0")]
    Folder(PlaylistFolderNode),

    /// A playlist.
    #[serde(rename = "1")]
    Playlist(PlaylistPlaylistNode),
}

impl<'de> Deserialize<'de> for PlaylistGenericNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PlaylistGenericNodeVisitor;

        impl<'de> serde::de::Visitor<'de> for PlaylistGenericNodeVisitor {
            type Value = PlaylistGenericNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct PlaylistGenericNode")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut node_type = None;
                let mut name = None;
                let mut count = None;
                let mut key_type = None;
                let mut entries = None;

                while let Some(key) = map.next_key::<Cow<'_, str>>()? {
                    match key.as_ref() {
                        "@Name" => name = map.next_value::<Cow<'_, str>>()?.into(),
                        "@Type" => node_type = map.next_value::<Cow<'_, str>>()?.into(),
                        "@Count" => count = map.next_value::<usize>()?.into(),
                        "@KeyType" => key_type = map.next_value::<Cow<'_, str>>()?.into(),
                        "@Entries" => entries = map.next_value::<usize>()?.into(),
                        unknown => {
                            return Err(A::Error::unknown_field(
                                unknown,
                                &["@Name", "@Type", "@Count", "@KeyType", "@Entries"],
                            ));
                        }
                    }

                    match node_type.as_deref() {
                        Some("0") => {
                            if let (Some(n), Some(_c)) = (&name, count) {
                                let nodes = {
                                    // Create anonymous type
                                    #[derive(serde::Deserialize)]
                                    struct Nodes {
                                        #[serde(rename = "NODE")]
                                        content: Vec<PlaylistGenericNode>,
                                    }
                                    let de = serde::de::value::MapAccessDeserializer::new(map);
                                    Nodes::deserialize(de)?.content
                                };
                                // FIXME: Should we check if nodes.len() == count here?
                                return Ok(PlaylistGenericNode::Folder(PlaylistFolderNode {
                                    name: n.to_string(),
                                    nodes,
                                }));
                            }
                        }
                        Some("1") => {
                            if let (Some(n), Some(_c), Some(t)) = (&name, entries, &key_type) {
                                let tracks = {
                                    // Create anonymous type
                                    #[derive(serde::Deserialize)]
                                    struct Tracks {
                                        #[serde(rename = "TRACK")]
                                        content: Vec<PlaylistTrack>,
                                    }
                                    let de = serde::de::value::MapAccessDeserializer::new(map);
                                    Tracks::deserialize(de)?.content
                                };
                                // FIXME: Should we check if nodes.len() == count here?
                                return Ok(PlaylistGenericNode::Playlist(PlaylistPlaylistNode {
                                    name: n.to_string(),
                                    keytype: t.to_string(),
                                    tracks,
                                }));
                            }
                        }
                        Some(unknown) => {
                            return Err(A::Error::unknown_variant(unknown, &["0", "1"]))
                        }
                        None => (),
                    }
                }

                match node_type.as_deref() {
                    Some("0") => {
                        if name.is_none() {
                            Err(A::Error::missing_field("@Name"))
                        } else {
                            Err(A::Error::missing_field("@Count"))
                        }
                    }
                    Some("1") => {
                        if name.is_none() {
                            Err(A::Error::missing_field("@Name"))
                        } else if entries.is_none() {
                            Err(A::Error::missing_field("@Entries"))
                        } else {
                            Err(A::Error::missing_field("@KeyType"))
                        }
                    }
                    _ => Err(A::Error::missing_field("@Type")),
                }
            }
        }

        deserializer.deserialize_map(PlaylistGenericNodeVisitor)
    }
}

/// A folder containing more playlists.
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct PlaylistFolderNode {
    /// Name of NODE
    #[serde(rename = "@Name")]
    pub name: String,

    // The "Count" attribute that contains the "Number of NODE in NODE" is omitted here, because we
    // can just take the number of elements in the `tracks` vector instead.
    /// Nodes
    #[serde(rename = "NODE")]
    pub nodes: Vec<PlaylistGenericNode>,
}

impl Serialize for PlaylistFolderNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Value<'a> {
            /// Name of NODE
            #[serde(rename = "@Name")]
            name: &'a String,
            /// Count
            #[serde(rename = "@Count")]
            count: usize,
            /// Nodes
            #[serde(rename = "NODE")]
            nodes: &'a Vec<PlaylistGenericNode>,
        }

        let value = Value {
            name: &self.name,
            count: self.nodes.len(),
            nodes: &self.nodes,
        };

        value.serialize(serializer)
    }
}

/// A playlist.
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct PlaylistPlaylistNode {
    /// Name of NODE
    #[serde(rename = "@Name")]
    pub name: String,

    // The "Entries" attribute that contains the "Number of TRACK in PLAYLIST" is omitted here,
    // because we can just take the number of elements in the `tracks` vector instead.
    /// Kind of identification
    /// "0" (Track ID) or "1"(Location)
    #[serde(rename = "@KeyType")]
    pub keytype: String,

    /// The tracks contained in the playlist.
    #[serde(rename = "TRACK")]
    #[serde(default = "Vec::new")]
    pub tracks: Vec<PlaylistTrack>,
}

impl Serialize for PlaylistPlaylistNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Value<'a> {
            /// Name of NODE
            #[serde(rename = "@Name")]
            name: &'a String,
            /// Number of TRACK in PLAYLIST
            #[serde(rename = "@Entries")]
            entries: usize,
            /// Kind of identification
            /// "0" (Track ID) or "1"(Location)
            #[serde(rename = "@KeyType")]
            keytype: &'a String,
            #[serde(rename = "TRACK")]
            tracks: &'a Vec<PlaylistTrack>,
        }

        let value = Value {
            name: &self.name,
            entries: self.tracks.len(),
            keytype: &self.keytype,
            tracks: &self.tracks,
        };

        value.serialize(serializer)
    }
}

/// A playlist entry.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    /// Identification of track
    /// "Track ID" or "Location" in "COLLECTION"
    #[serde(rename = "@Key")]
    pub key: i32,
}

#[derive(Default)]
struct TrackAnalysis {
    tempos: Vec<Tempo>,
    position_marks: BTreeMap<(i32, u32, i32), PositionMark>,
}

/// Build a Rekordbox XML document from a device export directory.
///
/// The export directory must contain the usual `PIONEER/rekordbox/export.pdb`
/// file. If matching `ANLZ` analysis files are present, beatgrid tempo entries
/// and cue points are included as `TEMPO` and `POSITION_MARK` elements.
pub fn document_from_device_export(path: &FsPath) -> crate::Result<Document> {
    let loader = DeviceExportLoader::new(path.to_path_buf());
    let export_path = loader.get_path().to_path_buf();
    let mut db = loader.open_pdb_non_persistent()?;

    let artists = db
        .iter_rows::<pdb::Artist>()?
        .map(|artist| Ok((artist.id, artist.offsets.name.clone().into_string()?)))
        .collect::<HashMap<ArtistId, String>>()?;

    let playlist_entries = db
        .iter_rows::<PlaylistEntry>()?
        .map(|entry| Ok((entry.playlist_id, (entry.entry_index, entry.track_id))))
        .fold(
            HashMap::<PlaylistTreeNodeId, BTreeMap<u32, TrackId>>::new(),
            |mut acc, (playlist_id, entry)| {
                acc.entry(playlist_id).or_default().insert(entry.0, entry.1);
                Ok(acc)
            },
        )?;

    let playlists = get_playlists(&mut db)?;
    let tracks = db
        .iter_rows::<pdb::Track>()?
        .map(|track| Ok(track.clone()))
        .collect::<Vec<pdb::Track>>()?;

    let track = tracks
        .iter()
        .map(|track| track_from_device_export(&export_path, track, &artists))
        .collect::<crate::Result<Vec<Track>>>()?;

    Ok(Document {
        version: "1.0.0".to_string(),
        product: Product {
            name: "rekordbox".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            company: "rekordcrate".to_string(),
        },
        collection: Collection {
            entries: track.len() as i32,
            track,
        },
        playlists: Playlists {
            node: PlaylistFolderNode {
                name: "ROOT".to_string(),
                nodes: playlist_nodes_from_device_export(playlists, &playlist_entries),
            },
        },
    })
}

fn track_from_device_export(
    export_path: &FsPath,
    track: &pdb::Track,
    artists: &HashMap<ArtistId, String>,
) -> crate::Result<Track> {
    let analysis = load_track_analysis(export_path, track)?;
    let file_path = track.offsets.file_path.clone().into_string()?;
    let average_bpm = if track.average_bpm() > 0.0 {
        Some(track.average_bpm())
    } else {
        analysis.tempos.first().map(|tempo| tempo.bpm)
    };

    Ok(Track {
        trackid: track.id.0 as i32,
        name: non_empty(track.offsets.title.clone().into_string()?),
        artist: artists.get(&track.artist_id).cloned().and_then(non_empty),
        composer: None,
        album: None,
        grouping: None,
        genre: None,
        kind: file_type_name(track.file_type()).map(str::to_string),
        size: Some(i64::from(track.file_size())),
        totaltime: Some(f64::from(track.duration())),
        discnumber: Some(i32::from(track.disc_number())),
        tracknumber: Some(track.track_number() as i32),
        year: Some(i32::from(track.year())),
        averagebpm: average_bpm,
        datemodified: None,
        dateadded: non_empty(track.date_added().clone().into_string()?),
        bitrate: Some(track.bitrate() as i32),
        samplerate: Some(f64::from(track.sample_rate())),
        comments: non_empty(track.comment().clone().into_string()?),
        playcount: Some(i32::from(track.play_count())),
        lastplayed: None,
        rating: Some(i32::from(track.rating) * 51),
        location: export_location_uri(export_path, &file_path)?,
        remixer: None,
        tonality: None,
        label: None,
        mix: non_empty(track.mix_name().clone().into_string()?),
        colour: None,
        tempos: analysis.tempos,
        position_marks: analysis.position_marks.into_values().collect(),
    })
}

fn load_track_analysis(export_path: &FsPath, track: &pdb::Track) -> crate::Result<TrackAnalysis> {
    let analyze_path = track.analyze_path().clone().into_string()?;
    let mut analysis = TrackAnalysis::default();
    if analyze_path.is_empty() {
        return Ok(analysis);
    }

    for path in analysis_file_candidates(export_path, &analyze_path) {
        if !path.exists() {
            continue;
        }

        let mut file = File::open(path)?;
        let anlz = ANLZ::read(&mut file)?;
        merge_analysis_file(&mut analysis, anlz);
    }

    Ok(analysis)
}

fn merge_analysis_file(analysis: &mut TrackAnalysis, anlz: ANLZ) {
    for section in anlz.sections {
        match section.content {
            Content::BeatGrid(beat_grid) => {
                if analysis.tempos.is_empty() {
                    analysis.tempos = beat_grid
                        .beats
                        .into_iter()
                        .map(|beat| Tempo {
                            inizio: f64::from(beat.time) / 1000.0,
                            bpm: f64::from(beat.tempo) / 100.0,
                            metro: "4/4".to_string(),
                            battito: i32::from(beat.beat_number),
                        })
                        .collect();
                }
            }
            Content::CueList(cue_list) => {
                let hot_cues = cue_list.list_type == CueListType::HotCues;
                for cue in cue_list.cues {
                    insert_position_mark(
                        &mut analysis.position_marks,
                        hot_cues,
                        cue.hot_cue,
                        cue.cue_type,
                        cue.time,
                        cue.loop_time,
                        hot_cue_name(cue.hot_cue).unwrap_or_default().to_string(),
                    );
                }
            }
            Content::ExtendedCueList(cue_list) => {
                let hot_cues = cue_list.list_type == CueListType::HotCues;
                for cue in cue_list.cues {
                    let fallback_name = hot_cue_name(cue.hot_cue).unwrap_or_default();
                    let name = if cue.comment.is_empty() {
                        fallback_name.to_string()
                    } else {
                        cue.comment.0
                    };
                    insert_position_mark(
                        &mut analysis.position_marks,
                        hot_cues,
                        cue.hot_cue,
                        cue.cue_type,
                        cue.time,
                        cue.loop_time,
                        name,
                    );
                }
            }
            _ => {}
        }
    }
}

fn insert_position_mark(
    marks: &mut BTreeMap<(i32, u32, i32), PositionMark>,
    hot_cues: bool,
    hot_cue: u32,
    cue_type: CueType,
    time: u32,
    loop_time: u32,
    name: String,
) {
    let num = if hot_cues {
        if hot_cue == 0 || hot_cue > 4 {
            return;
        }
        hot_cue as i32 - 1
    } else {
        -1
    };
    let mark_type = match cue_type {
        CueType::Point => 0,
        CueType::Loop => 4,
    };
    let end = (loop_time > time).then_some(f64::from(loop_time) / 1000.0);

    marks.insert(
        (num, time, mark_type),
        PositionMark {
            name,
            mark_type,
            start: f64::from(time) / 1000.0,
            end,
            num,
        },
    );
}

fn analysis_file_candidates(export_path: &FsPath, analyze_path: &str) -> Vec<PathBuf> {
    let relative_path = analyze_path.strip_prefix('/').unwrap_or(analyze_path);
    let path = export_path.join(relative_path);
    let mut candidates = vec![path.clone()];
    for extension in ["EXT", "DAT", "2EX"] {
        let candidate = path.with_extension(extension);
        if !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

fn playlist_nodes_from_device_export(
    nodes: Vec<PlaylistNode>,
    playlist_entries: &HashMap<PlaylistTreeNodeId, BTreeMap<u32, TrackId>>,
) -> Vec<PlaylistGenericNode> {
    nodes
        .into_iter()
        .map(|node| match node {
            PlaylistNode::Folder(folder) => PlaylistGenericNode::Folder(PlaylistFolderNode {
                name: folder.name,
                nodes: playlist_nodes_from_device_export(folder.children, playlist_entries),
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
        })
        .collect()
}

fn export_location_uri(export_path: &FsPath, device_path: &str) -> crate::Result<String> {
    let relative_path = device_path.strip_prefix('/').unwrap_or(device_path);
    let absolute_path = export_path.canonicalize()?.join(relative_path);
    Ok(format!(
        "file://localhost{}",
        percent_encode_path(&absolute_path.to_string_lossy())
    ))
}

fn percent_encode_path(path: &str) -> String {
    path.bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                vec![byte as char]
            }
            byte => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn file_type_name(file_type: &FileType) -> Option<&'static str> {
    match file_type {
        FileType::Mp3 => Some("MP3 File"),
        FileType::M4a => Some("M4A File"),
        FileType::Flac => Some("FLAC File"),
        FileType::Wav => Some("WAV File"),
        FileType::Aiff => Some("AIFF File"),
        FileType::Unknown | FileType::Other(_) => None,
    }
}

fn hot_cue_name(hot_cue: u32) -> Option<&'static str> {
    match hot_cue {
        1 => Some("A"),
        2 => Some("B"),
        3 => Some("C"),
        4 => Some("D"),
        _ => None,
    }
}

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
