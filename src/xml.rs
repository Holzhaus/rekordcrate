// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
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
use serde::{de::Error, ser::Serializer, Deserialize, Serialize};

/// The XML root element of a rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "DJ_PLAYLISTS")]
pub struct Document {
    /// Version of the XML format for share the playlists.
    ///
    /// The latest version is 1,0,0.
    #[serde(rename = "@Version")]
    version: String,
    #[serde(rename = "PRODUCT")]
    product: Product,
    #[serde(rename = "COLLECTION")]
    collection: Collection,
    #[serde(rename = "PLAYLISTS")]
    playlists: Playlists,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct Product {
    /// Name of product
    ///
    /// This name will be displayed in each application software.
    #[serde(rename = "@Name")]
    name: String,
    /// Version of application
    #[serde(rename = "@Version")]
    version: String,
    /// Name of company
    #[serde(rename = "@Company")]
    company: String,
}

/// The information of the tracks who are not included in any playlist are unnecessary.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct Collection {
    /// Number of TRACK in COLLECTION
    #[serde(rename = "@Entries")]
    entries: i32,
    #[serde(rename = "TRACK")]
    track: Vec<Track>,
}

/// "Location" is essential for each track ;
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct Track {
    /// Identification of track
    #[serde(rename = "@TrackID")]
    trackid: i32,
    /// Name of track
    #[serde(rename = "@Name")]
    name: Option<String>,
    /// Name of artist
    #[serde(rename = "@Artist")]
    artist: Option<String>,
    /// Name of composer (or producer)
    #[serde(rename = "@Composer")]
    composer: Option<String>,
    /// Name of Album
    #[serde(rename = "@Album")]
    album: Option<String>,
    /// Name of goupe
    #[serde(rename = "@Grouping")]
    grouping: Option<String>,
    /// Name of genre
    #[serde(rename = "@Genre")]
    genre: Option<String>,
    /// Type of audio file
    #[serde(rename = "@Kind")]
    kind: Option<String>,
    /// Size of audio file
    /// Unit : Octet
    #[serde(rename = "@Size")]
    size: Option<i64>,
    /// Duration of track
    /// Unit : Second (without decimal numbers)
    #[serde(rename = "@TotalTime")]
    totaltime: Option<f64>,
    /// Order number of the disc of the album
    #[serde(rename = "@DiscNumber")]
    discnumber: Option<i32>,
    /// Order number of the track in the album
    #[serde(rename = "@TrackNumber")]
    tracknumber: Option<i32>,
    /// Year of release
    #[serde(rename = "@Year")]
    year: Option<i32>,
    /// Value of average BPM
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@AverageBpm")]
    averagebpm: Option<f64>,
    /// Date of last modification
    /// Format : yyyy- mm- dd ; ex. : 2010- 08- 21
    #[serde(rename = "@DateModified")]
    #[serde(skip_serializing_if = "Option::is_none")]
    datemodified: Option<NaiveDate>,
    /// Date of addition
    /// Format : yyyy- mm- dd ; ex. : 2010- 08- 21
    #[serde(rename = "@DateAdded")]
    #[serde(skip_serializing_if = "Option::is_none")]
    dateadded: Option<NaiveDate>,
    /// Encoding bit rate
    /// Unit : Kbps
    #[serde(rename = "@BitRate")]
    bitrate: Option<i32>,
    /// Frequency of sampling
    /// Unit : Hertz
    #[serde(rename = "@SampleRate")]
    samplerate: Option<f64>,
    /// Comments
    #[serde(rename = "@Comments")]
    comments: Option<String>,
    /// Play count of the track
    #[serde(rename = "@PlayCount")]
    playcount: Option<i32>,
    /// Date of last playing
    /// Format : yyyy- mm- dd ; ex. : 2010- 08- 21
    #[serde(rename = "@LastPlayed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    lastplayed: Option<NaiveDate>,
    /// Rating of the track
    /// 0 star = "@0", 1 star = "51", 2 stars = "102", 3 stars = "153", 4 stars = "204", 5 stars = "255"
    #[serde(rename = "@Rating")]
    rating: Option<i32>,
    /// Location of the file
    /// includes the file name (URI formatted)
    #[serde(rename = "@Location")]
    location: String,
    /// Name of remixer
    #[serde(rename = "@Remixer")]
    remixer: Option<String>,
    /// Tonality (Kind of musical key)
    #[serde(rename = "@Tonality")]
    tonality: Option<String>,
    /// Name of record label
    #[serde(rename = "@Label")]
    label: Option<String>,
    /// Name of mix
    #[serde(rename = "@Mix")]
    mix: Option<String>,
    /// Colour for track grouping
    /// RGB format (3 bytes) ; rekordbox : Rose(0xFF007F), Red(0xFF0000), Orange(0xFFA500), Lemon(0xFFFF00), Green(0x00FF00), Turquoise(0x25FDE9),  Blue(0x0000FF), Violet(0x660099)
    #[serde(rename = "@Colour")]
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<String>,
    #[serde(rename = "TEMPO")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    tempos: Vec<Tempo>,
    #[serde(rename = "POSITION_MARK")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    position_marks: Vec<PositionMark>,
}

/// 0 star = "@0", 1 star = "51", 2 stars = "102", 3 stars = "153", 4 stars = "204", 5 stars = "255"
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum StarRating {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Unknown(i32),
}

/// For BeatGrid; More than two "TEMPO" can exist for each track
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct Tempo {
    /// Start position of BeatGrid
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@Inizio")]
    inizio: f64,
    /// Value of BPM
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@Bpm")]
    bpm: f64,
    /// Kind of musical meter (formatted)
    /// ex. 3/ 4, 4/ 4, 7/ 8…
    #[serde(rename = "@Metro")]
    metro: String,
    /// Beat number in the bar
    /// If the value of "Metro" is 4/ 4, the value should be 1, 2, 3 or 4.
    #[serde(rename = "@Battito")]
    battito: i32,
}

/// More than two "POSITION MARK" can exist for each track
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct PositionMark {
    /// Name of position mark
    #[serde(rename = "@Name")]
    name: String,
    /// Type of position mark
    /// Cue = "@0", Fade- In = "1", Fade- Out = "2", Load = "3",  Loop = " 4"
    #[serde(rename = "@Type")]
    mark_type: i32,
    /// Start position of position mark
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@Start")]
    start: f64,
    /// End position of position mark
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@End")]
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<f64>,
    /// Number for identification of the position mark
    /// rekordbox : Hot Cue A,  B,  C : "0", "1", "2"; Memory Cue : "- 1"
    #[serde(rename = "@Num")]
    num: i32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct Playlists {
    #[serde(rename = "NODE")]
    node: PlaylistFolderNode,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "@Type")]
enum PlaylistGenericNode {
    #[serde(rename = "0")]
    Folder(PlaylistFolderNode),
    #[serde(rename = "1")]
    Playlist(PlaylistPlaylistNode),
}

impl<'de> Deserialize<'de> for PlaylistGenericNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // stores a node with fields for a playlist or folder
        #[derive(Deserialize)]
        struct Node {
            #[serde(rename = "@Name")]
            name: String,
            // indicates playlist or folder
            #[serde(rename = "@Type")]
            node_type: String,
            // appears on playlists only
            #[serde(rename = "@KeyType", default)]
            key_type: Option<String>,
            // child nodes in a folder
            #[serde(rename = "NODE", default)]
            nodes: Vec<PlaylistGenericNode>,
            // tracks in a playlist
            #[serde(rename = "TRACK", default)]
            tracks: Vec<PlaylistTrack>,
        }

        let node = Node::deserialize(deserializer)?;

        match node.node_type.as_str() {
            // Folder node
            "0" => Ok(PlaylistGenericNode::Folder(PlaylistFolderNode {
                name: node.name,
                nodes: node.nodes,
            })),
            // Playlist node
            "1" => {
                if let Some(key_type) = node.key_type {
                    Ok(PlaylistGenericNode::Playlist(PlaylistPlaylistNode {
                        name: node.name,
                        keytype: key_type,
                        tracks: node.tracks,
                    }))
                } else {
                    Err(D::Error::missing_field("@KeyType"))
                }
            }
            t => Err(D::Error::unknown_variant(t, &["0", "1"])),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
struct PlaylistFolderNode {
    /// Name of NODE
    #[serde(rename = "@Name")]
    name: String,
    // The "Count" attribute that contains the "Number of NODE in NODE" is omitted here, because we
    // can just take the number of elements in the `tracks` vector instead.
    /// Nodes
    #[serde(rename = "NODE")]
    nodes: Vec<PlaylistGenericNode>,
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

#[derive(Debug, PartialEq, Clone, Deserialize)]
struct PlaylistPlaylistNode {
    /// Name of NODE
    #[serde(rename = "@Name")]
    name: String,
    // The "Entries" attribute that contains the "Number of TRACK in PLAYLIST" is omitted here,
    // because we can just take the number of elements in the `tracks` vector instead.
    /// Kind of identification
    /// "0" (Track ID) or "1"(Location)
    #[serde(rename = "@KeyType")]
    keytype: String,
    #[serde(rename = "TRACK")]
    tracks: Vec<PlaylistTrack>,
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct PlaylistTrack {
    /// Identification of track
    /// "Track ID" or "Location" in "COLLECTION"
    #[serde(rename = "@Key")]
    key: i32,
}
