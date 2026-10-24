// Copyright (c) 2026
// SPDX-License-Identifier: MPL-2.0

//! Typed parsing and generation of Rekordbox playlist-sharing XML documents.

use quick_xml::{de::from_str, se::to_string};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type returned by XML operations.
pub type Result<T> = std::result::Result<T, XmlError>;

/// Errors returned while decoding or encoding Rekordbox XML.
#[derive(Debug, Error)]
pub enum XmlError {
    /// The XML document cannot be decoded into the Rekordbox schema.
    #[error("invalid Rekordbox XML: {0}")]
    Deserialize(#[from] quick_xml::DeError),

    /// The Rekordbox document cannot be encoded as XML.
    #[error("cannot serialize Rekordbox XML: {0}")]
    Serialize(#[from] quick_xml::SeError),
}

/// Parse a Rekordbox playlist-sharing XML document.
pub fn parse(input: &str) -> Result<Document> {
    Ok(from_str(input)?)
}

/// Serialize a Rekordbox playlist-sharing XML document with an XML declaration.
pub fn serialize(document: &Document) -> Result<String> {
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}",
        to_string(document)?
    ))
}

/// Root `DJ_PLAYLISTS` XML document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "DJ_PLAYLISTS")]
pub struct Document {
    /// XML format version, normally `1.0.0`.
    #[serde(rename = "@Version")]
    pub version: String,

    /// Software that generated the document.
    #[serde(rename = "PRODUCT")]
    pub product: Product,

    /// Tracks available to playlists.
    #[serde(rename = "COLLECTION")]
    pub collection: Collection,

    /// Playlist folder tree.
    #[serde(rename = "PLAYLISTS")]
    pub playlists: Playlists,
}

/// Producer metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Product {
    /// Producer company.
    #[serde(rename = "@Company")]
    pub company: String,

    /// Producer name.
    #[serde(rename = "@Name")]
    pub name: String,

    /// Producer version.
    #[serde(rename = "@Version")]
    pub version: String,
}

/// Track collection.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Collection {
    /// Number of tracks declared by the producer.
    #[serde(rename = "@Entries")]
    pub entries: usize,

    /// Collection tracks.
    #[serde(rename = "TRACK", default)]
    pub tracks: Vec<Track>,
}

/// A Rekordbox track and its interoperability metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Track {
    /// Stable track identifier.
    #[serde(rename = "@TrackID")]
    pub track_id: u32,

    /// File URI.
    #[serde(rename = "@Location")]
    pub location: String,

    /// Track title.
    #[serde(rename = "@Name", default)]
    pub name: Option<String>,

    /// Artist name.
    #[serde(rename = "@Artist", default)]
    pub artist: Option<String>,

    /// Average BPM used by basic hardware/software interoperability.
    #[serde(rename = "@AverageBpm", default)]
    pub average_bpm: Option<f64>,

    /// Track length in seconds.
    #[serde(rename = "@TotalTime", default)]
    pub total_time: Option<f64>,

    /// Musical key.
    #[serde(rename = "@Tonality", default)]
    pub tonality: Option<String>,

    /// Beat-grid tempo changes.
    #[serde(rename = "TEMPO", default)]
    pub tempos: Vec<Tempo>,

    /// Cue points, loops, and memory cues.
    #[serde(rename = "POSITION_MARK", default)]
    pub position_marks: Vec<PositionMark>,
}

impl Track {
    /// Return valid hot cues ordered by cue slot.
    ///
    /// Memory cues (`Num="-1"`), loops, invalid cue numbers, and non-finite
    /// positions are excluded. `limit` is normally `4` for old CDJ hardware.
    #[must_use]
    pub fn hot_cues(&self, limit: usize) -> Vec<&PositionMark> {
        let mut cues: Vec<&PositionMark> = self
            .position_marks
            .iter()
            .filter(|mark| {
                mark.mark_type == PositionMarkType::Cue
                    && mark.num >= 0
                    && mark.start.is_finite()
                    && mark.start >= 0.0
            })
            .collect();

        cues.sort_by_key(|mark| mark.num);
        cues.truncate(limit);
        cues
    }

    /// Return the effective BPM for basic synchronization.
    ///
    /// `AverageBpm` is preferred. If absent, the first finite positive tempo
    /// marker is used.
    #[must_use]
    pub fn bpm(&self) -> Option<f64> {
        self.average_bpm
            .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
            .or_else(|| {
                self.tempos
                    .iter()
                    .map(|tempo| tempo.bpm)
                    .find(|bpm| bpm.is_finite() && *bpm > 0.0)
            })
    }
}

/// A beat-grid tempo marker.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Tempo {
    /// Position in seconds.
    #[serde(rename = "@Inizio")]
    pub start: f64,

    /// BPM at this position.
    #[serde(rename = "@Bpm")]
    pub bpm: f64,

    /// Time signature, for example `4/4`.
    #[serde(rename = "@Metro", default)]
    pub meter: Option<String>,

    /// Beat within the measure.
    #[serde(rename = "@Battito", default)]
    pub beat: Option<u32>,
}

/// Rekordbox position-marker category.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionMarkType {
    /// Cue / hot-cue marker.
    #[serde(rename = "0")]
    Cue,

    /// Fade-in marker.
    #[serde(rename = "1")]
    FadeIn,

    /// Fade-out marker.
    #[serde(rename = "2")]
    FadeOut,

    /// Load marker.
    #[serde(rename = "3")]
    Load,

    /// Loop marker.
    #[serde(rename = "4")]
    Loop,
}

/// A cue, hot cue, memory cue, or loop marker.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PositionMark {
    /// User-visible cue name.
    #[serde(rename = "@Name", default)]
    pub name: Option<String>,

    /// Marker category.
    #[serde(rename = "@Type")]
    pub mark_type: PositionMarkType,

    /// Start position in seconds.
    #[serde(rename = "@Start")]
    pub start: f64,

    /// Loop end position in seconds.
    #[serde(rename = "@End", default)]
    pub end: Option<f64>,

    /// Cue slot. `-1` denotes a memory cue.
    #[serde(rename = "@Num")]
    pub num: i32,
}

/// Playlist root.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Playlists {
    /// Top-level playlist nodes.
    #[serde(rename = "NODE", default)]
    pub nodes: Vec<PlaylistNode>,
}

/// Folder or playlist node.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PlaylistNode {
    /// Folder or playlist name.
    #[serde(rename = "@Name")]
    pub name: String,

    /// `0` for a folder, `1` for a playlist.
    #[serde(rename = "@Type")]
    pub node_type: u8,

    /// Number of child nodes for folders.
    #[serde(rename = "@Count", default)]
    pub count: Option<usize>,

    /// Number of track references for playlists.
    #[serde(rename = "@Entries", default)]
    pub entries: Option<usize>,

    /// Playlist reference key type: `0` for TrackID, `1` for Location.
    #[serde(rename = "@KeyType", default)]
    pub key_type: Option<u8>,

    /// Child folders/playlists.
    #[serde(rename = "NODE", default)]
    pub nodes: Vec<PlaylistNode>,

    /// Referenced collection tracks.
    #[serde(rename = "TRACK", default)]
    pub tracks: Vec<PlaylistTrack>,
}

impl PlaylistNode {
    /// Construct a folder node with correct count metadata.
    #[must_use]
    pub fn folder(name: impl Into<String>, nodes: Vec<Self>) -> Self {
        Self {
            name: name.into(),
            node_type: 0,
            count: Some(nodes.len()),
            entries: None,
            key_type: None,
            nodes,
            tracks: Vec::new(),
        }
    }

    /// Construct a TrackID-keyed playlist with correct entry metadata.
    #[must_use]
    pub fn playlist(name: impl Into<String>, tracks: Vec<PlaylistTrack>) -> Self {
        Self {
            name: name.into(),
            node_type: 1,
            count: None,
            entries: Some(tracks.len()),
            key_type: Some(0),
            nodes: Vec::new(),
            tracks,
        }
    }
}

/// A reference from a playlist to a collection track.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PlaylistTrack {
    /// TrackID or Location according to the parent `KeyType`.
    #[serde(rename = "@Key")]
    pub key: String,
}
