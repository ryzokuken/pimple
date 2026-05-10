use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
pub struct AppConfig {
    /// `None` until the first-run picker resolves a path. See v0.2 spec §4.4.
    #[ts(type = "string | null")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vdir_root: Option<PathBuf>,
    #[serde(default)]
    pub collection_visibility: HashMap<String, bool>,
    #[serde(default)]
    pub week_start: WeekStart,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_geometry: Option<WindowGeometry>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
#[serde(rename_all = "snake_case")]
pub enum WeekStart {
    #[default]
    Monday,
    Sunday,
}

/// Window position/size persisted across launches. v0.2 §4.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
pub struct WindowGeometry {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}
