use rekordcrate::xml::Document;
use rekordcrate::DeviceExportLoader;
use std::path::PathBuf;

#[test]
fn export_demo_device_to_rekordbox_xml() {
    let export_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("complete_export")
        .join("demo_tracks");
    let loader = DeviceExportLoader::new(export_path);

    let document = loader.export_xml_document().expect("export XML document");

    assert_eq!(document.version, "1.0.0");
    assert_eq!(document.collection.entries, 2);
    assert_eq!(document.collection.track.len(), 2);

    let first_track = &document.collection.track[0];
    assert_eq!(first_track.name.as_deref(), Some("Demo Track 1"));
    assert_eq!(first_track.artist.as_deref(), Some("Loopmasters"));
    assert_eq!(first_track.averagebpm, Some(128.0));
    assert!(first_track.location.starts_with("file://localhost/"));
    assert!(first_track.location.contains("Demo%20Track%201.mp3"));
    assert!(!first_track.tempos.is_empty());
    assert_eq!(first_track.tempos[0].bpm, 128.0);
    assert_eq!(first_track.tempos[0].battito, 1);

    assert!(document.playlists.node.nodes.is_empty());

    let serialized = quick_xml::se::to_string(&document).expect("serialize XML");
    let reparsed: Document = quick_xml::de::from_str(&serialized).expect("parse generated XML");
    assert_eq!(reparsed.collection.entries, document.collection.entries);
    assert_eq!(
        reparsed.collection.track.len(),
        document.collection.track.len()
    );
    assert_eq!(
        reparsed.collection.track[0].name.as_deref(),
        Some("Demo Track 1")
    );
    assert_eq!(reparsed.collection.track[0].tempos[0].bpm, 128.0);
    assert!(reparsed.playlists.node.nodes.is_empty());
}
