//! Queries behind the "Move to…" dialog and the rename input: which folders a
//! file or folder may be moved into, where a file lands in a chosen folder,
//! and whether a rename/move would be rejected (with the reason).

use super::file_ops::rename_file;
use super::path::{PathError, TreePath};
use super::store_state::{FileOperationContext, FileStoreState};

/// One choice in the folder picker, in display order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveTarget {
    /// Full folder path; empty for the top level.
    pub path: String,
    /// Display name (the folder's own name, or `Top level`).
    pub name: String,
    /// 0 for the top level, 1 for its folders, and so on.
    pub depth: usize,
}

const TOP_LEVEL_LABEL: &str = "Top level";

/// The top level followed by every folder (explicit or implied), depth-first
/// and sorted like the file list. When moving a folder, pass it as
/// `excluded_folder`: it and everything inside it are left out, so it cannot
/// be moved into itself. A file move passes `None`. Only the top level is
/// offered if the paths conflict.
pub fn move_targets(state: &FileStoreState, excluded_folder: Option<&TreePath>) -> Vec<MoveTarget> {
    let top_level = MoveTarget {
        path: String::new(),
        name: TOP_LEVEL_LABEL.to_string(),
        depth: 0,
    };
    let folders = state
        .tree()
        .map(|tree| tree.rows())
        .unwrap_or_default()
        .into_iter()
        .filter(|row| row.is_folder)
        .filter(|row| {
            excluded_folder
                .is_none_or(|excluded| row.path != *excluded && !row.path.is_within(excluded))
        })
        .map(|row| MoveTarget {
            path: row.path.as_str().to_string(),
            name: row.name,
            depth: row.depth + 1,
        });
    std::iter::once(top_level).chain(folders).collect()
}

/// The path `source` (a file or folder path) gets when moved into `folder`
/// (`None` = top level), keeping its own name.
pub fn move_destination(source: &str, folder: Option<&TreePath>) -> Result<TreePath, PathError> {
    let source = TreePath::parse(source)?;
    TreePath::child(folder, source.name())
}

/// Why renaming/moving the file `from` to `to_raw` would be rejected, as the
/// message to show the user; `None` when it would succeed (or change nothing).
pub fn rename_error(
    state: &FileStoreState,
    from: &str,
    to_raw: &str,
    context: &FileOperationContext,
) -> Option<String> {
    rename_file(state, from, to_raw, context)
        .err()
        .map(|error| error.to_string())
}
