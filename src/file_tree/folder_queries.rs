//! Read-only questions the folder dialogs ask: would a folder rename/move be
//! rejected (and why), and what does deleting a folder do. The messages are
//! shown as-is by the web app.

use super::folder_ops::{create_folder, move_folder};
use super::folder_share::{folder_delete_share_warning, ShareOverview};
use super::path::TreePath;
use super::store_state::{FileOperationContext, FileStoreState};

/// What the "Delete folder" confirmation tells the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderDeleteSummary {
    /// Files that would move to the bin (those inside, at any depth).
    pub file_count: usize,
    /// The sentence to show, e.g. `2 files will move to the bin ...`.
    pub message: String,
    /// What deleting does to live links involved (this folder shared, shared
    /// folders or files inside, or an enclosing shared folder); `None` when
    /// nothing is shared.
    pub share_warning: Option<String>,
}

/// Why creating a folder `name` (trimmed) inside `parent` would be rejected,
/// as the message to show; `None` when it would succeed. Same rules as
/// [`create_folder`], which this simply asks.
pub fn create_folder_error(
    state: &FileStoreState,
    parent: Option<&str>,
    name: &str,
    context: &FileOperationContext,
) -> Option<String> {
    let parent = match parent.map(TreePath::parse).transpose() {
        Ok(parent) => parent,
        Err(error) => return Some(error.to_string()),
    };
    create_folder(state, parent.as_ref(), name, context)
        .err()
        .map(|error| error.to_string())
}

/// Why renaming/moving the folder `from` to `to_raw` (trimmed) would be
/// rejected, as the message to show; `None` when it would succeed (or change
/// nothing). Same rules as [`move_folder`], which this simply asks.
pub fn folder_move_error(
    state: &FileStoreState,
    from: &str,
    to_raw: &str,
    context: &FileOperationContext,
) -> Option<String> {
    let outcome = TreePath::parse(from)
        .and_then(|from| Ok((from, TreePath::parse(to_raw.trim())?)))
        .map_err(|error| error.to_string())
        .and_then(|(from, to)| {
            move_folder(state, &from, &to, context).map_err(|error| error.to_string())
        });
    outcome.err()
}

/// The confirmation text for deleting `folder`; `None` if it does not exist
/// (no file in it and no explicit folder at or inside it).
pub fn folder_delete_summary(
    state: &FileStoreState,
    folder: &TreePath,
    shares: &ShareOverview,
) -> Option<FolderDeleteSummary> {
    let file_count = state.files_within(folder).len();
    if file_count == 0 && state.explicit_folders_within(folder).is_empty() {
        return None;
    }
    let message = match file_count {
        0 => "This empty folder will be removed.".to_string(),
        1 => "1 file will move to the bin. You can restore it from there, back to this folder."
            .to_string(),
        count => format!(
            "{count} files will move to the bin. You can restore them from there, back to this folder."
        ),
    };
    Some(FolderDeleteSummary {
        file_count,
        message,
        share_warning: folder_delete_share_warning(shares, folder),
    })
}

/// The folder `path` and every folder above it, outermost first: what the file
/// list opens to show a folder that was just created or moved. Empty if
/// `path` is not a valid path.
pub fn folders_to_reveal(path: &str) -> Vec<String> {
    TreePath::parse(path)
        .map(|path| {
            path.ancestors()
                .into_iter()
                .chain(std::iter::once(path))
                .map(|folder| folder.as_str().to_string())
                .collect()
        })
        .unwrap_or_default()
}
