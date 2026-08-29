#![cfg(feature = "device_library_plus")]

use rekordcrate::device_library_plus::{Content, Database, Property, TableRecord};

const COMPLETE_EXPORT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/complete_export/device_library_plus/PIONEER/rekordbox/exportLibrary.db"
);

/// Verifies that the read-only DLP path can decrypt the fixture and load tables
/// through the same `TableRecord::all` API used by `dump-dlp`.
#[test]
fn opens_and_loads_sample_device_library_plus() -> rekordcrate::Result<()> {
    let mut db = Database::open(COMPLETE_EXPORT_PATH)?;
    let conn = db.connection_mut();

    let properties = Property::all(conn)?;
    assert_eq!(properties.len(), 1);
    assert_eq!(properties[0].device_name, "Robin Piotr");
    assert_eq!(properties[0].number_of_contents, 417);

    let contents = Content::all(conn)?;
    assert_eq!(contents.len(), 417);
    assert_eq!(
        contents[0].title.as_deref(),
        Some("You Make Me Feel So Good")
    );

    Ok(())
}
