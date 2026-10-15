//! Bodies of the file-store `Guest` methods: convert WIT values to
//! `jianpu_generator::file_tree` types, call it, convert back. No logic
//! beyond conversion.
#![allow(clippy::needless_pass_by_value)]
use super::*;
use jianpu_generator::file_tree as tree;
use std::collections::BTreeMap;

pub(super) fn file_store_create_file(
    state: FileStoreState,
    folder: Option<String>,
    content: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    let outcome = parse_optional(folder.as_deref()).and_then(|folder| {
        tree::create_file(&state.into(), folder.as_ref(), &content, &context.into())
    });
    store_response(outcome)
}

pub(super) fn file_store_create_folder(
    state: FileStoreState,
    parent: Option<String>,
    context: FileOperationContext,
) -> FileStoreResponse {
    let outcome = parse_optional(parent.as_deref())
        .and_then(|parent| tree::create_folder(&state.into(), parent.as_ref(), &context.into()));
    store_response(outcome)
}

pub(super) fn file_store_move_targets(
    state: FileStoreState,
    excluded_folder: Option<String>,
) -> Vec<MoveTarget> {
    let excluded = excluded_folder
        .as_deref()
        .and_then(|raw| tree::TreePath::parse(raw).ok());
    tree::move_targets(&state.into(), excluded.as_ref())
        .into_iter()
        .map(|target| MoveTarget {
            path: target.path,
            name: target.name,
            depth: u32::try_from(target.depth).unwrap_or(u32::MAX),
        })
        .collect()
}

pub(super) fn file_store_move_destination(
    source_path: String,
    folder: Option<String>,
) -> Option<String> {
    let folder = parse_optional(folder.as_deref()).ok()?;
    tree::move_destination(&source_path, folder.as_ref())
        .ok()
        .map(|path| path.as_str().to_string())
}

pub(super) fn file_store_rename_error(
    state: FileStoreState,
    from_path: String,
    to_path: String,
    context: FileOperationContext,
) -> Option<String> {
    tree::rename_error(&state.into(), &from_path, &to_path, &context.into())
}

pub(super) fn file_store_folder_move_error(
    state: FileStoreState,
    from_path: String,
    to_path: String,
    context: FileOperationContext,
) -> Option<String> {
    tree::folder_move_error(&state.into(), &from_path, &to_path, &context.into())
}

pub(super) fn file_store_folder_delete_summary(
    state: FileStoreState,
    folder: String,
) -> Option<FolderDeleteSummary> {
    let folder = tree::TreePath::parse(&folder).ok()?;
    tree::folder_delete_summary(&state.into(), &folder).map(|summary| FolderDeleteSummary {
        file_count: u32::try_from(summary.file_count).unwrap_or(u32::MAX),
        message: summary.message,
    })
}

pub(super) fn file_store_folders_to_reveal(path: String) -> Vec<String> {
    tree::folders_to_reveal(&path)
}

pub(super) fn file_store_bin_entries(state: FileStoreState) -> Vec<BinEntry> {
    tree::bin_entries(&state.into())
        .into_iter()
        .map(|entry| BinEntry {
            key: entry.key,
            name: entry.name,
            location: entry.location,
        })
        .collect()
}

pub(super) fn file_store_duplicate_file(
    state: FileStoreState,
    source: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    let outcome = parse(&source)
        .and_then(|source| tree::duplicate_file(&state.into(), &source, &context.into()));
    store_response(outcome)
}

pub(super) fn file_store_rename_file(
    state: FileStoreState,
    from_path: String,
    to_path: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    store_response(tree::rename_file(
        &state.into(),
        &from_path,
        &to_path,
        &context.into(),
    ))
}

pub(super) fn file_store_delete_file(
    state: FileStoreState,
    path: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    store_response(tree::delete_file(&state.into(), &path, &context.into()))
}

pub(super) fn file_store_restore_file(
    state: FileStoreState,
    bin_key: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    store_response(tree::restore_file(&state.into(), &bin_key, &context.into()))
}

pub(super) fn file_store_delete_folder(
    state: FileStoreState,
    folder: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    let outcome = parse(&folder)
        .and_then(|folder| tree::delete_folder(&state.into(), &folder, &context.into()));
    store_response(outcome)
}

pub(super) fn file_store_move_folder(
    state: FileStoreState,
    from_path: String,
    to_path: String,
    context: FileOperationContext,
) -> FolderMoveResponse {
    let outcome = parse(&from_path).and_then(|from| {
        let to = parse(&to_path)?;
        tree::move_folder(&state.into(), &from, &to, &context.into())
    });
    match outcome {
        Ok(moved) => FolderMoveResponse::Ok(FolderMoveSuccess {
            state: moved.state.into(),
            changes: moved.changes.into_iter().map(Into::into).collect(),
        }),
        Err(error) => FolderMoveResponse::Err(error.into()),
    }
}

pub(super) fn file_store_tree(state: FileStoreState) -> FileTreeResponse {
    let state: tree::FileStoreState = state.into();
    match state.tree() {
        Ok(file_tree) => FileTreeResponse::Ok(
            file_tree
                .rows()
                .into_iter()
                .map(|row| FileTreeRow {
                    path: row.path.as_str().to_string(),
                    name: row.name,
                    depth: u32::try_from(row.depth).unwrap_or(u32::MAX),
                    is_folder: row.is_folder,
                })
                .collect(),
        ),
        Err(error) => FileTreeResponse::Err(error.to_string()),
    }
}

pub(super) fn file_store_tree_view(
    state: FileStoreState,
    expanded: Vec<String>,
    demo_file_names: Vec<String>,
) -> FileTreeViewResponse {
    let active = state.active.clone();
    let state: tree::FileStoreState = state.into();
    match state.tree() {
        Ok(file_tree) => FileTreeViewResponse::Ok(
            file_tree
                .view_rows(&tree::TreeViewInput {
                    active: &active,
                    expanded: &expanded,
                    demo_file_names: &demo_file_names,
                })
                .into_iter()
                .map(|row| FileTreeViewRow {
                    key: row.key,
                    name: row.name,
                    depth: u32::try_from(row.depth).unwrap_or(u32::MAX),
                    kind: match row.kind {
                        tree::TreeViewRowKind::Folder => FileTreeViewRowKind::Folder,
                        tree::TreeViewRowKind::File => FileTreeViewRowKind::File,
                        tree::TreeViewRowKind::DemoFolder => FileTreeViewRowKind::DemoFolder,
                        tree::TreeViewRowKind::DemoFile => FileTreeViewRowKind::DemoFile,
                        tree::TreeViewRowKind::EmptyFolderHint => {
                            FileTreeViewRowKind::EmptyFolderHint
                        }
                    },
                    expanded: row.expanded,
                    file_count: u32::try_from(row.file_count).unwrap_or(u32::MAX),
                    is_active: row.is_active,
                })
                .collect(),
        ),
        Err(error) => FileTreeViewResponse::Err(error.to_string()),
    }
}

pub(super) fn file_store_active_label(
    active: String,
    demo_file_names: Vec<String>,
) -> FileActiveLabel {
    let label = tree::active_label(&active, &demo_file_names);
    FileActiveLabel {
        folder_names: label.folder_names,
        file_name: label.file_name,
        folder_path: label.folder_path,
    }
}

fn parse(raw: &str) -> Result<tree::TreePath, tree::FileStoreError> {
    Ok(tree::TreePath::parse(raw)?)
}

fn parse_optional(raw: Option<&str>) -> Result<Option<tree::TreePath>, tree::FileStoreError> {
    raw.map(parse).transpose()
}

fn store_response(
    outcome: Result<tree::FileStoreState, tree::FileStoreError>,
) -> FileStoreResponse {
    match outcome {
        Ok(state) => FileStoreResponse::Ok(state.into()),
        Err(error) => FileStoreResponse::Err(error.into()),
    }
}

fn entries(map: BTreeMap<String, String>) -> Vec<StringEntry> {
    map.into_iter()
        .map(|(key, value)| StringEntry { key, value })
        .collect()
}

fn map_of(entries: Vec<StringEntry>) -> BTreeMap<String, String> {
    entries
        .into_iter()
        .map(|entry| (entry.key, entry.value))
        .collect()
}

impl From<FileStoreState> for tree::FileStoreState {
    fn from(state: FileStoreState) -> Self {
        Self {
            active: state.active,
            user_files: map_of(state.user_files),
            bin: map_of(state.bin),
            file_ids: map_of(state.file_ids),
            // A malformed stored folder path is dropped rather than failing
            // every operation; the files themselves are unaffected.
            folders: state
                .folders
                .iter()
                .filter_map(|raw| tree::TreePath::parse(raw).ok())
                .collect(),
        }
    }
}

impl From<tree::FileStoreState> for FileStoreState {
    fn from(state: tree::FileStoreState) -> Self {
        Self {
            active: state.active,
            user_files: entries(state.user_files),
            bin: entries(state.bin),
            file_ids: entries(state.file_ids),
            folders: state
                .folders
                .iter()
                .map(|folder| folder.as_str().to_string())
                .collect(),
        }
    }
}

impl From<FileOperationContext> for tree::FileOperationContext {
    fn from(context: FileOperationContext) -> Self {
        Self {
            demo_files: context
                .demo_files
                .into_iter()
                .filter_map(|demo| {
                    Some(tree::DemoFile {
                        path: tree::TreePath::parse(&demo.path).ok()?,
                        content: demo.content,
                    })
                })
                .collect(),
            new_file_id: context.new_file_id,
        }
    }
}

impl From<tree::PathChange> for PathChange {
    fn from(change: tree::PathChange) -> Self {
        Self {
            kind: match change.kind {
                tree::PathKind::File => PathKind::File,
                tree::PathKind::ExplicitFolder => PathKind::ExplicitFolder,
            },
            old_path: change.old_path.as_str().to_string(),
            new_path: change.new_path.as_str().to_string(),
        }
    }
}

impl From<tree::FileStoreError> for FileStoreFailure {
    fn from(error: tree::FileStoreError) -> Self {
        let kind = match &error {
            tree::FileStoreError::InvalidPath { .. } => FileStoreErrorKind::InvalidPath,
            tree::FileStoreError::NotFound { .. } => FileStoreErrorKind::NotFound,
            tree::FileStoreError::Collision { .. } => FileStoreErrorKind::Collision,
            tree::FileStoreError::AncestorIsFile { .. } => FileStoreErrorKind::AncestorIsFile,
            tree::FileStoreError::MoveIntoItself { .. } => FileStoreErrorKind::MoveIntoItself,
            tree::FileStoreError::DemoFileReadOnly { .. } => FileStoreErrorKind::DemoFileReadOnly,
        };
        Self {
            kind,
            message: error.to_string(),
        }
    }
}
