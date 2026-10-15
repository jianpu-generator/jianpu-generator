//! Single-file operations over [`FileStoreState`]; same semantics as the
//! `createFile`/`duplicateFile`/`renameFile`/`deleteFile`/`restoreFile` in
//! `web/src/fileStore.ts`, generalised to paths. Each returns a new state.

use super::path::TreePath;
use super::store_state::{FileOperationContext, FileStoreError, FileStoreState};
use super::unique_name::unique_path;

const NEW_FILE_NAME: &str = "untitled.jianpu";
const EXTENSION: &str = ".jianpu";

/// A new file (named `untitled.jianpu`, made unique) in `folder` (`None` =
/// top level) holding `content`; it becomes active.
pub fn create_file(
    state: &FileStoreState,
    folder: Option<&TreePath>,
    content: &str,
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    let desired = TreePath::child(folder, NEW_FILE_NAME)?;
    add_new_file(state, &desired, content.to_string(), context)
}

/// Copies `source` (a user or demo file) next to itself, in the same folder,
/// as `stem 2.ext` etc.; the copy becomes active.
pub fn duplicate_file(
    state: &FileStoreState,
    source: &TreePath,
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    let content = state
        .user_files
        .get(source.as_str())
        .or_else(|| {
            context
                .demo_files
                .iter()
                .find(|demo| demo.path == *source)
                .map(|demo| &demo.content)
        })
        .ok_or_else(|| FileStoreError::NotFound {
            path: source.as_str().to_string(),
        })?;
    add_new_file(state, source, content.clone(), context)
}

fn add_new_file(
    state: &FileStoreState,
    desired: &TreePath,
    content: String,
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    if let Some(path) = state.ancestor_that_is_file(desired, context) {
        return Err(FileStoreError::AncestorIsFile { path });
    }
    let path = unique_path(desired, &state.reserved_paths(context))?;
    let name = path.as_str().to_string();
    let mut next = state.clone();
    next.active = name.clone();
    next.user_files.insert(name.clone(), content);
    next.file_ids.insert(name, context.new_file_id.clone());
    Ok(next)
}

/// Renames and/or moves a file to the path `to_raw` (trimmed, `.jianpu`
/// appended when missing). The file keeps its id and content. An existing
/// file, folder, demo file or bin entry at the destination is a
/// [`FileStoreError::Collision`] (never overwritten); moving onto itself is a
/// no-op. Parent folders are created implicitly.
pub fn rename_file(
    state: &FileStoreState,
    from: &str,
    to_raw: &str,
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    if context
        .demo_files
        .iter()
        .any(|demo| demo.path.as_str() == from)
    {
        return Err(FileStoreError::DemoFileReadOnly {
            path: from.to_string(),
        });
    }
    let Some(content) = state.user_files.get(from) else {
        return Err(FileStoreError::NotFound {
            path: from.to_string(),
        });
    };
    let to = with_extension(to_raw)?;
    if to.as_str() == from {
        return Ok(state.clone());
    }
    if let Some(path) = state.ancestor_that_is_file(&to, context) {
        return Err(FileStoreError::AncestorIsFile { path });
    }
    if state.reserved_paths(context).contains(&to) {
        return Err(FileStoreError::Collision { path: to });
    }
    let mut next = state.clone();
    next.user_files.remove(from);
    next.user_files
        .insert(to.as_str().to_string(), content.clone());
    let id = next
        .file_ids
        .remove(from)
        .unwrap_or_else(|| context.new_file_id.clone());
    next.file_ids.insert(to.as_str().to_string(), id);
    if state.active == from {
        next.active = to.as_str().to_string();
    }
    Ok(next)
}

/// Moves a file to the bin under its full path (its id stays registered).
/// If it was active, the first remaining file (else the first demo) is.
pub fn delete_file(
    state: &FileStoreState,
    path: &str,
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    if context
        .demo_files
        .iter()
        .any(|demo| demo.path.as_str() == path)
    {
        return Err(FileStoreError::DemoFileReadOnly {
            path: path.to_string(),
        });
    }
    let mut next = state.clone();
    let content = next
        .user_files
        .remove(path)
        .ok_or_else(|| FileStoreError::NotFound {
            path: path.to_string(),
        })?;
    next.bin.insert(path.to_string(), content);
    if state.active == path {
        next.active = next.fallback_active(context);
    }
    Ok(next)
}

/// Puts a binned file back at the path it was deleted from, recreating
/// missing folders (they are implied by the path). If that path is taken now
/// (by a file, folder or demo file) a unique name in the same folder is used
/// (`stem 2.ext`); the file keeps its id and becomes active. Other bin
/// entries do not block the name (same as the flat store). If a parent is now
/// a file the restore fails with `AncestorIsFile` and the bin is untouched.
pub fn restore_file(
    state: &FileStoreState,
    bin_key: &str,
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    let content = state
        .bin
        .get(bin_key)
        .ok_or_else(|| FileStoreError::NotFound {
            path: bin_key.to_string(),
        })?;
    let desired = TreePath::parse(bin_key)?;
    if let Some(path) = state.ancestor_that_is_file(&desired, context) {
        return Err(FileStoreError::AncestorIsFile { path });
    }
    let path = unique_path(&desired, &state.occupied_paths(context))?;
    let mut next = state.clone();
    next.bin.remove(bin_key);
    next.user_files
        .insert(path.as_str().to_string(), content.clone());
    let id = next
        .file_ids
        .remove(bin_key)
        .unwrap_or_else(|| context.new_file_id.clone());
    next.file_ids.insert(path.as_str().to_string(), id);
    next.active = path.as_str().to_string();
    Ok(next)
}

fn with_extension(raw: &str) -> Result<TreePath, FileStoreError> {
    let trimmed = raw.trim();
    TreePath::parse(trimmed)?;
    let named = if trimmed.ends_with(EXTENSION) {
        trimmed.to_string()
    } else {
        format!("{trimmed}{EXTENSION}")
    };
    Ok(TreePath::parse(&named)?)
}
