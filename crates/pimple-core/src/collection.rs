use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::CollectionId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/ipc/types/")]
pub struct Collection {
    pub id: CollectionId,
    #[ts(type = "string")]
    pub path: PathBuf,
    pub display_name: String,
    /// CSS hex string, `#RRGGBB`. Read from the `color` file or assigned from the palette.
    pub color: String,
    /// App-local visibility; never written to the vdir.
    pub visible: bool,
}
