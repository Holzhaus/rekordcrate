use rekordcrate::xml::document_from_device_export;

#[test]
fn export_device_export_to_xml_document() {
    let document = document_from_device_export("data/complete_export/demo_tracks".as_ref())
        .expect("failed to export demo device export as XML");

    assert_eq!(document.version, "1.0.0");
    assert_eq!(document.collection.entries, 2);

    let demo_track = document
        .collection
        .track
        .iter()
        .find(|track| track.name.as_deref() == Some("Demo Track 1"))
        .expect("demo track should be exported");
    assert_eq!(demo_track.artist.as_deref(), Some("Loopmasters"));
    assert_eq!(demo_track.averagebpm, Some(128.0));
    assert!(!demo_track.tempos.is_empty());
    assert!(demo_track.location.starts_with("file://localhost/"));
    assert!(demo_track.location.contains("Demo%20Track%201.mp3"));

    let second_track = document
        .collection
        .track
        .iter()
        .find(|track| track.name.as_deref() == Some("Demo Track 2"))
        .expect("second demo track should be exported");
    assert_eq!(second_track.averagebpm, Some(120.0));
    assert!(!second_track.tempos.is_empty());

    let xml = quick_xml::se::to_string(&document).expect("failed to serialize XML document");
    assert!(xml.contains("<DJ_PLAYLISTS"));
    assert!(xml.contains("Name=\"Demo Track 1\""));
    assert!(xml.contains("<TEMPO"));
    assert!(xml.contains("Location=\"file://localhost/"));
}
