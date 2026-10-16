use super::path::{PathError, TreePath};
use super::tree::{FileTree, FileTreeError};
use std::collections::{BTreeMap, HashSet};
use std::fmt;

/// Rust mirror of the web app's `FileStoreState` (`web/src/fileStore.ts`),
/// plus the optional explicit `folders` list. Keys of `user_files`, `bin` and
/// `file_ids` are full paths (a legacy flat name is just a top-level path).
/// They stay plain strings so a legacy store with an odd name still loads;
/// operations parse them when they need path structure.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileStoreState {
    pub active: String,
    pub user_files: BTreeMap<String, String>,
    /// Binned files, keyed by the full path they were deleted from, so a
    /// restore knows where to put them back.
    pub bin: BTreeMap<String, String>,
    /// Stable id per path (active and binned); follows the file on move.
    pub file_ids: BTreeMap<String, String>,
    /// Explicitly created folders (possibly empty). Folders that merely
    /// hold files are implied by the file paths and need no entry.
    pub folders: Vec<TreePath>,
}

/// A read-only built-in file. Demo paths are reserved names and the only
/// files that are not in `user_files`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoFile {
    pub path: TreePath,
    pub content: String,
}

/// Everything an operation needs from outside the store, so operations stay
/// pure: the demo files, and a fresh file id (the caller generates the UUID;
/// it is used only when the operation creates a file or finds a missing id).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileOperationContext {
    pub demo_files: Vec<DemoFile>,
    pub new_file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStoreError {
    InvalidPath {
        error: PathError,
    },
    /// No such file, folder or bin entry.
    NotFound {
        path: String,
    },
    /// The destination is already a file, folder, demo file or bin entry.
    Collision {
        path: TreePath,
    },
    /// A parent of the destination is a file, so it cannot hold anything.
    AncestorIsFile {
        path: TreePath,
    },
    MoveIntoItself {
        folder: TreePath,
        destination: TreePath,
    },
    DemoFileReadOnly {
        path: String,
    },
    /// The destination's last segment is the reserved placeholder leaf.
    ReservedName {
        path: TreePath,
    },
}

impl From<PathError> for FileStoreError {
    fn from(error: PathError) -> Self {
        Self::InvalidPath { error }
    }
}

impl fmt::Display for FileStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { error } => write!(f, "invalid path: {error}"),
            Self::NotFound { path } => write!(f, "`{path}` does not exist"),
            Self::Collision { path } => write!(f, "`{}` already exists", path.as_str()),
            Self::AncestorIsFile { path } => {
                write!(f, "`{}` is a file, not a folder", path.as_str())
            }
            Self::MoveIntoItself {
                folder,
                destination,
            } => write!(
                f,
                "cannot move `{}` into itself (`{}`)",
                folder.as_str(),
                destination.as_str()
            ),
            Self::DemoFileReadOnly { path } => write!(f, "`{path}` is a read-only demo file"),
            Self::ReservedName { path } => {
                write!(f, "`{}` is a reserved name and cannot be used", path.name())
            }
        }
    }
}

impl std::error::Error for FileStoreError {}

impl FileStoreState {
    /// The folder tree of the user's files plus explicit folders (demo files
    /// are shown separately and not part of it).
    pub fn tree(&self) -> Result<FileTree, FileTreeError> {
        FileTree::build(&self.user_file_paths(), &self.folders)
    }

    pub(super) fn user_file_paths(&self) -> Vec<TreePath> {
        parse_keys(&self.user_files)
    }

    /// Paths of user files at or inside `folder`.
    pub(super) fn files_within(&self, folder: &TreePath) -> Vec<TreePath> {
        self.user_file_paths()
            .into_iter()
            .filter(|path| path.is_within(folder))
            .collect()
    }

    /// Explicit folders at or inside `folder`.
    pub(super) fn explicit_folders_within(&self, folder: &TreePath) -> Vec<TreePath> {
        self.folders
            .iter()
            .filter(|path| *path == folder || path.is_within(folder))
            .cloned()
            .collect()
    }

    /// Every path that currently exists: user files, demo files, and every
    /// folder (derived from file paths, or explicit, with all ancestors).
    pub(super) fn occupied_paths(&self, context: &FileOperationContext) -> HashSet<TreePath> {
        let files = self.user_file_paths();
        let folders = files
            .iter()
            .flat_map(TreePath::ancestors)
            .chain(
                self.folders
                    .iter()
                    .flat_map(|folder| folder.ancestors().into_iter().chain([folder.clone()])),
            )
            .collect::<Vec<_>>();
        files
            .into_iter()
            .chain(folders)
            .chain(context.demo_files.iter().map(|demo| demo.path.clone()))
            .collect()
    }

    /// Occupied paths plus bin keys: names a new or moved path must not
    /// take, so that binned files stay restorable to their own path.
    pub(super) fn reserved_paths(&self, context: &FileOperationContext) -> HashSet<TreePath> {
        self.occupied_paths(context)
            .into_iter()
            .chain(parse_keys(&self.bin))
            .collect()
    }

    /// The first proper ancestor of `path` that is a file (user or demo).
    pub(super) fn ancestor_that_is_file(
        &self,
        path: &TreePath,
        context: &FileOperationContext,
    ) -> Option<TreePath> {
        let files = self.user_file_paths();
        path.ancestors().into_iter().find(|ancestor| {
            files.contains(ancestor) || context.demo_files.iter().any(|d| d.path == *ancestor)
        })
    }

    /// The active file to fall back to after the active one disappears.
    pub(super) fn fallback_active(&self, context: &FileOperationContext) -> String {
        self.user_files
            .keys()
            .next()
            .cloned()
            .or_else(|| {
                context
                    .demo_files
                    .first()
                    .map(|demo| demo.path.as_str().to_string())
            })
            .unwrap_or_default()
    }
}

fn parse_keys(map: &BTreeMap<String, String>) -> Vec<TreePath> {
    map.keys()
        .filter_map(|key| TreePath::parse(key).ok())
        .collect()
}
