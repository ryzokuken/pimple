#![allow(clippy::expect_used)]

use std::path::PathBuf;

use pimple_core::vdir::layout::enumerate_collections;

#[test]
fn enumerate_finds_all_collection_subdirs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vdir");
    let mut ids: Vec<String> = enumerate_collections(&root)
        .expect("enumerate")
        .into_iter()
        .map(|c| c.id.as_str().to_string())
        .collect();
    ids.sort();
    assert_eq!(ids, vec!["no_metadata", "personal", "work"]);
}

#[test]
fn collection_with_metadata_uses_displayname_and_color() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vdir");
    let collections = enumerate_collections(&root).expect("enumerate");
    let personal = collections
        .iter()
        .find(|c| c.id.as_str() == "personal")
        .expect("personal collection");
    assert_eq!(personal.display_name, "Personal");
    assert_eq!(personal.color, "#3b82f6");
}

#[test]
fn collection_without_metadata_falls_back() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vdir");
    let collections = enumerate_collections(&root).expect("enumerate");
    let bare = collections
        .iter()
        .find(|c| c.id.as_str() == "no_metadata")
        .expect("no_metadata collection");
    assert_eq!(bare.display_name, "no_metadata");
    assert!(bare.color.starts_with('#'));
    assert_eq!(bare.color.len(), 7);
}
