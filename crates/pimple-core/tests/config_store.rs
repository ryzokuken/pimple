//! Tests for `pimple_core::config_store`.
//!
//! The store reads/writes TOML at an OS-appropriate config path. To keep tests
//! hermetic, callers pass an explicit directory (a `tempdir` here). The
//! `pimple-tauri` layer is responsible for resolving the real path via the
//! `dirs` crate at runtime.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashMap;
use std::path::PathBuf;

use pimple_core::config::{AppConfig, WeekStart, WindowGeometry};
use pimple_core::config_store;
use tempfile::TempDir;

fn populated_config(root: PathBuf) -> AppConfig {
    let mut visibility = HashMap::new();
    visibility.insert("personal".to_owned(), true);
    visibility.insert("work".to_owned(), false);
    AppConfig {
        vdir_root: Some(root),
        collection_visibility: visibility,
        week_start: WeekStart::Sunday,
        window_geometry: Some(WindowGeometry {
            width: 1280,
            height: 800,
            x: 100,
            y: 50,
        }),
    }
}

#[test]
fn load_returns_defaults_when_file_missing() {
    let tmp = TempDir::new().expect("tempdir");
    let config = config_store::load(tmp.path()).expect("load with no file should return defaults");
    assert!(config.vdir_root.is_none());
    assert!(config.collection_visibility.is_empty());
    assert_eq!(config.week_start, WeekStart::Monday); // Default::default()
    assert!(config.window_geometry.is_none());
}

#[test]
fn save_then_load_round_trips_all_fields() {
    let tmp = TempDir::new().expect("tempdir");
    let original = populated_config(PathBuf::from("/home/me/.calendars"));
    config_store::save(&original, tmp.path()).expect("save");
    let loaded = config_store::load(tmp.path()).expect("load");
    assert_eq!(loaded, original);
}

#[test]
fn save_creates_intermediate_directories() {
    let tmp = TempDir::new().expect("tempdir");
    let nested = tmp.path().join("not/yet/created");
    let original = populated_config(PathBuf::from("/home/me/.calendars"));
    config_store::save(&original, &nested).expect("save creates dirs");
    assert!(nested.join("config.toml").exists());
}

#[test]
fn save_is_atomic_via_temp_file_then_rename() {
    // Writing twice in succession should leave exactly one config.toml plus no
    // stale temp files. The atomic-rename pattern is the same one
    // pimple-core::write::create_event uses, but the test here just asserts
    // the directory ends up clean.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = populated_config(PathBuf::from("/x"));
    config_store::save(&cfg, tmp.path()).expect("first save");
    config_store::save(&cfg, tmp.path()).expect("second save");
    let entries: Vec<_> = std::fs::read_dir(tmp.path())
        .expect("readdir")
        .map(|e| e.expect("entry").file_name())
        .collect();
    let names: Vec<_> = entries.iter().filter_map(|n| n.to_str()).collect();
    assert_eq!(
        names,
        vec!["config.toml"],
        "directory should hold only config.toml"
    );
}

#[test]
fn unknown_toml_fields_are_ignored_on_load() {
    // Forward-compatibility: a config.toml written by a future version may
    // contain fields we don't model yet. Loading must succeed and use defaults
    // for the rest.
    let tmp = TempDir::new().expect("tempdir");
    let toml_with_extras = r#"
week_start = "monday"

[collection_visibility]
personal = true

[future_unknown_section]
some_key = "some_value"
"#;
    std::fs::write(tmp.path().join("config.toml"), toml_with_extras).expect("write");
    let loaded = config_store::load(tmp.path()).expect("load tolerates unknown fields");
    assert_eq!(loaded.week_start, WeekStart::Monday);
    assert_eq!(loaded.collection_visibility.get("personal"), Some(&true));
}
