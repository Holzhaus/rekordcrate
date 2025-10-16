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

#![cfg(feature = "xml")]

use chrono::naive::NaiveDate;
use rgb::RGB8;
use serde::{de::Error, ser::Serializer, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::borrow::Cow;
use std::fmt;

/// The XML root element of a rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "DJ_PLAYLISTS")]
pub struct Document {
    /// Version of the XML format for share the playlists.
    ///
    /// The latest version is 1,0,0.
    #[serde(rename = "@Version")]
    pub version: String,
    /// Product
    #[serde(rename = "PRODUCT")]
    pub product: Product,
    /// Collection of all tracks
    #[serde(rename = "COLLECTION")]
    pub collection: Collection,
    /// Playlist Tree
    #[serde(rename = "PLAYLISTS")]
    pub playlists: Playlists,
}

/// Product
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Product {
    /// Name of product
    ///
    /// This name will be displayed in each application software.
    #[serde(rename = "@Name")]
    pub name: String,
    /// Version of application
    #[serde(rename = "@Version")]
    pub version: String,
    /// Name of company
    #[serde(rename = "@Company")]
    pub company: String,
}

/// The information of the tracks who are not included in any playlist are unnecessary.
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Collection {
    // The "Entries" attribute that contains the "Number of TRACK in COLLECTION" is omitted here,
    // because we can just take the number of elements in the `tracks` vector instead.
    /// Tracks in the collection.
    #[serde(rename = "TRACK")]
    pub tracks: Vec<Track>,
}

impl Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Value<'a> {
            /// Number of TRACK in COLLECTION
            #[serde(rename = "@Entries")]
            entries: usize,
            /// Tracks
            #[serde(rename = "TRACK")]
            tracks: &'a Vec<Track>,
        }

        let value = Value {
            entries: self.tracks.len(),
            tracks: &self.tracks,
        };

        value.serialize(serializer)
    }
}

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
    pub rating: Option<StarRating>,
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
    pub colour: Option<Color>,
    /// Tempo Markers (Beatgrid)
    #[serde(rename = "TEMPO")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tempos: Vec<Tempo>,
    /// Position Marks (Cues)
    #[serde(rename = "POSITION_MARK")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub position_marks: Vec<PositionMark>,
}

/// Color of a Cue Point.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    /// Rose Color
    Rose,
    /// Red Color
    Red,
    /// Orange Color
    Orange,
    /// Lemon Color
    Lemon,
    /// Green Color
    Green,
    /// Turquoise Color
    Turquoise,
    /// Blue Color
    Blue,
    /// Violet Color
    Violet,
    /// Custom RGB Color
    Custom(RGB8),
}

impl Color {
    const RGB_ROSE: RGB8 = RGB8::new(0xFF, 0x00, 0x7F);
    const RGB_RED: RGB8 = RGB8::new(0xFF, 0x00, 0x00);
    const RGB_ORANGE: RGB8 = RGB8::new(0xFF, 0xA5, 0x00);
    const RGB_LEMON: RGB8 = RGB8::new(0xFF, 0xFF, 0x00);
    const RGB_GREEN: RGB8 = RGB8::new(0x00, 0xFF, 0x00);
    const RGB_TURQUOISE: RGB8 = RGB8::new(0x25, 0xFD, 0xE9);
    const RGB_BLUE: RGB8 = RGB8::new(0x00, 0x00, 0xFF);
    const RGB_VIOLET: RGB8 = RGB8::new(0x66, 0x00, 0x99);

    /// Get RGB value for this color.
    #[must_use]
    pub fn rgb(&self) -> &RGB8 {
        match self {
            Self::Rose => &Self::RGB_ROSE,
            Self::Red => &Self::RGB_RED,
            Self::Orange => &Self::RGB_ORANGE,
            Self::Lemon => &Self::RGB_LEMON,
            Self::Green => &Self::RGB_GREEN,
            Self::Turquoise => &Self::RGB_TURQUOISE,
            Self::Blue => &Self::RGB_BLUE,
            Self::Violet => &Self::RGB_VIOLET,
            Self::Custom(rgb_color) => rgb_color,
        }
    }

    #[must_use]
    fn from_hex<S: AsRef<str>>(value: S) -> Option<RGB8> {
        let hexstr = value
            .as_ref()
            .strip_prefix("#")
            .or_else(|| value.as_ref().strip_prefix("0x"))
            .unwrap_or(value.as_ref());
        match hexstr.len() {
            3 => {
                let mut r = u8::from_str_radix(hexstr.get(0..1)?, 16).ok()?;
                r |= r << 4;
                let mut g = u8::from_str_radix(hexstr.get(1..2)?, 16).ok()?;
                g |= g << 4;
                let mut b = u8::from_str_radix(hexstr.get(2..3)?, 16).ok()?;
                b |= b << 4;
                Some(RGB8 { r, g, b })
            }
            6 => {
                let r = u8::from_str_radix(hexstr.get(0..=1)?, 16).ok()?;
                let g = u8::from_str_radix(hexstr.get(2..=3)?, 16).ok()?;
                let b = u8::from_str_radix(hexstr.get(4..=5)?, 16).ok()?;
                Some(RGB8 { r, g, b })
            }
            _ => None,
        }
    }
}

impl From<RGB8> for Color {
    fn from(rgb_color: RGB8) -> Self {
        match rgb_color {
            Self::RGB_ROSE => Self::Rose,
            Self::RGB_RED => Self::Red,
            Self::RGB_ORANGE => Self::Orange,
            Self::RGB_LEMON => Self::Lemon,
            Self::RGB_GREEN => Self::Green,
            Self::RGB_TURQUOISE => Self::Turquoise,
            Self::RGB_BLUE => Self::Blue,
            Self::RGB_VIOLET => Self::Violet,
            rgb_color => Self::Custom(rgb_color),
        }
    }
}

impl<'a> TryFrom<&'a str> for Color {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "Rose" => Ok(Self::Rose),
            "Red" => Ok(Self::Red),
            "Orange" => Ok(Self::Orange),
            "Lemon" => Ok(Self::Lemon),
            "Green" => Ok(Self::Green),
            "Turquoise" => Ok(Self::Turquoise),
            "Blue" => Ok(Self::Blue),
            "Violet" => Ok(Self::Violet),
            color_str => Color::from_hex(color_str)
                .map(Color::Custom)
                .ok_or(value.as_ref()),
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rose => write!(f, "Rose"),
            Self::Red => write!(f, "Red"),
            Self::Orange => write!(f, "Orange"),
            Self::Lemon => write!(f, "Lemon"),
            Self::Green => write!(f, "Green"),
            Self::Turquoise => write!(f, "Turquoise"),
            Self::Blue => write!(f, "Blue"),
            Self::Violet => write!(f, "Violet"),
            Self::Custom(rgb) => write!(f, "#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b),
        }
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ColorVisitor;
        impl serde::de::Visitor<'_> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a color name or hex code")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Color::try_from(value)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(value), &self))
            }
        }
        deserializer.deserialize_str(ColorVisitor)
    }
}

/// Star Rating
///
/// 0 star = "@0", 1 star = "51", 2 stars = "102", 3 stars = "153", 4 stars = "204", 5 stars = "255"
#[derive(Debug, PartialEq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum StarRating {
    /// Zero Stars
    Zero = 0x00,
    /// One Star
    One = 0x33,
    /// Two Stars
    Two = 0x66,
    /// Three Stars
    Three = 0x99,
    /// Four Stars
    Four = 0xCC,
    /// Five Stars
    Five = 0xFF,
}

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

/// The Playlist Tree
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Playlists {
    /// Root node of the tree.
    #[serde(rename = "NODE")]
    pub node: PlaylistFolderNode,
}

/// Node in the playlist tree.
///
/// Can be either a folder or a playlist.
#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "@Type")]
pub enum PlaylistGenericNode {
    /// A folder in the playlist tree.
    #[serde(rename = "0")]
    Folder(PlaylistFolderNode),
    /// A playlist in the playlist tree.
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

/// A folder in the playlist tree.
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

/// A playlist in the playlist tree.
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
    /// Tracks in the playlist.
    #[serde(rename = "TRACK")]
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

/// A track in the playlist.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    /// Identification of track
    /// "Track ID" or "Location" in "COLLECTION"
    #[serde(rename = "@Key")]
    pub key: i32,
}
