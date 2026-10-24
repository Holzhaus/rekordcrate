#![cfg(feature = "xml")]

use rekordcrate::xml::{parse, serialize, PositionMarkType};

const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<DJ_PLAYLISTS Version="1.0.0">
  <PRODUCT Company="Mixxx" Name="Mixxx" Version="2.5.0"/>
  <COLLECTION Entries="1">
    <TRACK TrackID="42" Name="Night Drive" Artist="Example Artist"
           AverageBpm="128.000" TotalTime="301.250"
           Location="file://localhost/music/Night%20Drive.wav">
      <TEMPO Inizio="0.000" Bpm="128.000" Metro="4/4" Battito="1"/>
      <POSITION_MARK Name="A" Type="0" Start="0.000" Num="0"/>
      <POSITION_MARK Name="B" Type="0" Start="32.000" Num="1"/>
      <POSITION_MARK Name="C" Type="0" Start="64.000" Num="2"/>
      <POSITION_MARK Name="D" Type="0" Start="96.000" Num="3"/>
      <POSITION_MARK Name="Memory" Type="0" Start="120.000" Num="-1"/>
      <POSITION_MARK Name="Loop" Type="4" Start="160.000" End="176.000" Num="4"/>
    </TRACK>
  </COLLECTION>
  <PLAYLISTS>
    <NODE Name="ROOT" Type="0" Count="1">
      <NODE Name="Main Set" Type="1" Entries="1" KeyType="0">
        <TRACK Key="42"/>
      </NODE>
    </NODE>
  </PLAYLISTS>
</DJ_PLAYLISTS>"#;

#[test]
fn parses_artist_title_bpm_and_first_four_hot_cues() {
    let document = parse(FIXTURE).expect("fixture must decode");
    let track = &document.collection.tracks[0];

    assert_eq!(track.name.as_deref(), Some("Night Drive"));
    assert_eq!(track.artist.as_deref(), Some("Example Artist"));
    assert_eq!(track.bpm(), Some(128.0));

    let hot_cues = track.hot_cues(4);
    assert_eq!(hot_cues.len(), 4);
    assert_eq!(
        hot_cues.iter().map(|cue| cue.num).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        hot_cues.iter().map(|cue| cue.start).collect::<Vec<_>>(),
        vec![0.0, 32.0, 64.0, 96.0]
    );
}

#[test]
fn excludes_memory_cues_and_loops_from_hot_cue_bank() {
    let document = parse(FIXTURE).expect("fixture must decode");
    let track = &document.collection.tracks[0];

    let hot_cues = track.hot_cues(16);

    assert_eq!(hot_cues.len(), 4);
    assert!(hot_cues.iter().all(|cue| cue.num >= 0));
    assert!(hot_cues
        .iter()
        .all(|cue| cue.mark_type == PositionMarkType::Cue));
}

#[test]
fn falls_back_to_first_tempo_when_average_bpm_is_absent() {
    let xml = FIXTURE.replace(r#" AverageBpm="128.000""#, "");
    let document = parse(&xml).expect("fixture without AverageBpm must decode");

    assert_eq!(document.collection.tracks[0].bpm(), Some(128.0));
}

#[test]
fn round_trip_preserves_playlist_references_and_cue_positions() {
    let input = parse(FIXTURE).expect("fixture must decode");
    let encoded = serialize(&input).expect("fixture must encode");
    let output = parse(&encoded).expect("serialized fixture must decode");

    assert_eq!(output, input);
    assert_eq!(output.playlists.nodes[0].nodes[0].tracks[0].key, "42");
    assert_eq!(output.collection.tracks[0].hot_cues(4)[3].start, 96.0);
}

#[test]
fn accepts_tracks_without_optional_metadata_or_marks() {
    let xml = r#"<DJ_PLAYLISTS Version="1.0.0">
        <PRODUCT Company="Test" Name="Test" Version="1"/>
        <COLLECTION Entries="1">
            <TRACK TrackID="1" Location="file://localhost/test.mp3"/>
        </COLLECTION>
        <PLAYLISTS/>
    </DJ_PLAYLISTS>"#;

    let document = parse(xml).expect("minimal valid XML must decode");
    let track = &document.collection.tracks[0];

    assert_eq!(track.name, None);
    assert_eq!(track.artist, None);
    assert_eq!(track.bpm(), None);
    assert!(track.hot_cues(4).is_empty());
}
