#![cfg(feature = "device_library_plus")]

use rekordcrate::device_library_plus::*;

const COMPLETE_EXPORT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/complete_export/device_library_plus/PIONEER/rekordbox/exportLibrary.db"
);

#[test]
fn test_open_sample_device_library_plus() -> rekordcrate::Result<()> {
    let mut db = Database::open(COMPLETE_EXPORT_PATH)?;

    let property = Property::first(db.connection_mut())?.expect("missing property row");
    assert_eq!(property.device_name, "Robin Piotr");
    assert_eq!(property.number_of_contents, 417);

    let content = Content::by_id(db.connection_mut(), ContentId(1))?.expect("missing content row");
    assert_eq!(content.title.as_deref(), Some("You Make Me Feel So Good"));
    assert_eq!(content.artist_id_artist, Some(ArtistId(1)));
    assert_eq!(content.artist_id_original_artist, Some(ArtistId(2)));
    assert_eq!(content.key_id, Some(KeyId(1)));

    Ok(())
}
