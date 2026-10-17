//! The bin as a list: each binned file's own name and where it used to live.

use super::path::TreePath;
use super::store_state::FileStoreState;

/// One binned file. `key` is the bin key (the full path it was deleted from)
/// to hand back to `restore_file`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinEntry {
    pub key: String,
    /// The file's own name (last path segment).
    pub name: String,
    /// The folder it was deleted from; `None` for the top level.
    pub location: Option<String>,
}

/// Every bin entry, sorted by name (case-insensitive), then location. A key
/// that is not a valid path is listed whole as a top-level name.
pub fn bin_entries(state: &FileStoreState) -> Vec<BinEntry> {
    let mut entries: Vec<BinEntry> = state
        .bin
        .keys()
        .map(|key| match TreePath::parse(key) {
            Ok(path) => BinEntry {
                key: key.clone(),
                name: path.name().to_string(),
                location: path.parent().map(|parent| parent.as_str().to_string()),
            },
            Err(_) => BinEntry {
                key: key.clone(),
                name: key.clone(),
                location: None,
            },
        })
        .collect();
    entries.sort_by_cached_key(|entry| {
        (
            entry.name.to_lowercase(),
            entry.location.clone().unwrap_or_default().to_lowercase(),
            entry.key.clone(),
        )
    });
    entries
}
