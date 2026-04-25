//! vdir filesystem layout: enumerating collections and reading metadata.

use std::fs;
use std::path::Path;

use crate::collection::Collection;
use crate::error::{CoreError, Result};
use crate::id::CollectionId;

/// Palette used when a collection lacks an explicit `color` file.
/// Tailwind 500 hues; deliberately unfussy.
const PALETTE: &[&str] = &[
    "#ef4444", "#f97316", "#eab308", "#22c55e", "#06b6d4", "#3b82f6", "#8b5cf6", "#ec4899",
];

/// Enumerate every subdirectory of `root` as a `Collection`.
///
/// Reads the optional `displayname` and `color` files in each subdirectory; falls
/// back to the directory name and a palette-assigned color when absent.
///
/// # Errors
///
/// Returns `CoreError::Io` if `root` cannot be read or any of its entries
/// cannot be statted, and `CoreError::VdirLayout` for non-UTF-8 directory names.
pub fn enumerate_collections(root: &Path) -> Result<Vec<Collection>> {
    let mut entries: Vec<_> = fs::read_dir(root)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);

    let mut out = Vec::with_capacity(entries.len());
    for (palette_index, entry) in entries.iter().enumerate() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id_str = entry
            .file_name()
            .to_str()
            .ok_or_else(|| {
                CoreError::VdirLayout(format!("non-utf8 collection name at {}", path.display()))
            })?
            .to_owned();
        let id = CollectionId::new(&id_str);

        let display_name = read_optional_file(&path.join("displayname"))?
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| id_str.clone());
        let color = read_optional_file(&path.join("color"))?
            .map(|s| s.trim().to_owned())
            .filter(|s| s.starts_with('#') && s.len() == 7)
            .unwrap_or_else(|| PALETTE[palette_index % PALETTE.len()].to_owned());

        out.push(Collection {
            id,
            path,
            display_name,
            color,
            visible: true,
        });
    }
    Ok(out)
}

fn read_optional_file(p: &Path) -> Result<Option<String>> {
    match fs::read_to_string(p) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
