use super::path::{PathError, TreePath};
use super::unique_name::unique_path;
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTreeError {
    /// The same path is used as a file and as a folder (explicitly, or as
    /// the prefix of another path).
    FileFolderConflict { path: TreePath },
}

impl fmt::Display for FileTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileFolderConflict { path } => {
                write!(f, "`{}` is both a file and a folder", path.as_str())
            }
        }
    }
}

impl std::error::Error for FileTreeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNode {
    pub name: String,
    pub path: TreePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderNode {
    pub name: String,
    pub path: TreePath,
    pub contents: FolderContents,
}

/// What one folder (or the top level) holds, already sorted.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FolderContents {
    pub folders: Vec<FolderNode>,
    pub files: Vec<FileNode>,
}

/// Folder tree derived from a flat set of file paths plus explicitly
/// created (possibly empty) folders.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileTree {
    pub root: FolderContents,
}

impl FileTree {
    /// Folders are every ancestor of every file, every explicit folder, and
    /// every ancestor of those. Duplicates are merged. Fails if any file
    /// path is also a folder.
    pub fn build(
        file_paths: &[TreePath],
        explicit_folders: &[TreePath],
    ) -> Result<Self, FileTreeError> {
        let files: BTreeSet<&TreePath> = file_paths.iter().collect();
        let folders: BTreeSet<TreePath> = file_paths
            .iter()
            .flat_map(TreePath::ancestors)
            .chain(
                explicit_folders
                    .iter()
                    .flat_map(|folder| folder.ancestors().into_iter().chain([folder.clone()])),
            )
            .collect();
        if let Some(conflict) = files.iter().find(|file| folders.contains(**file)) {
            return Err(FileTreeError::FileFolderConflict {
                path: (*conflict).clone(),
            });
        }
        let files: Vec<&TreePath> = files.into_iter().collect();
        let folders: Vec<&TreePath> = folders.iter().collect();
        Ok(Self {
            root: contents_of(None, &folders, &files),
        })
    }
}

impl FileTree {
    /// Every file and folder path in the tree.
    pub fn all_paths(&self) -> Vec<TreePath> {
        collect_paths(&self.root)
    }

    /// `desired` made unique among the siblings of this tree (files and
    /// folders) and `also_taken` (e.g. binned files that must stay
    /// restorable). See [`unique_path`].
    pub fn unique_path(
        &self,
        desired: &TreePath,
        also_taken: &[TreePath],
    ) -> Result<TreePath, PathError> {
        let taken: HashSet<TreePath> = self
            .all_paths()
            .into_iter()
            .chain(also_taken.iter().cloned())
            .collect();
        unique_path(desired, &taken)
    }
}

fn collect_paths(contents: &FolderContents) -> Vec<TreePath> {
    contents
        .files
        .iter()
        .map(|file| file.path.clone())
        .chain(contents.folders.iter().flat_map(|folder| {
            std::iter::once(folder.path.clone()).chain(collect_paths(&folder.contents))
        }))
        .collect()
}

fn contents_of(
    parent: Option<&TreePath>,
    folders: &[&TreePath],
    files: &[&TreePath],
) -> FolderContents {
    let is_child = |path: &&&TreePath| path.parent().as_ref() == parent;
    FolderContents {
        folders: folders
            .iter()
            .filter(is_child)
            .map(|path| FolderNode {
                name: path.name().to_string(),
                path: (**path).clone(),
                contents: contents_of(Some(path), folders, files),
            })
            .sorted_by(|a, b| compare_names(&a.name, &b.name))
            .collect(),
        files: files
            .iter()
            .filter(is_child)
            .map(|path| FileNode {
                name: path.name().to_string(),
                path: (**path).clone(),
            })
            .sorted_by(|a, b| compare_names(&a.name, &b.name))
            .collect(),
    }
}

/// Alphabetical, case-insensitive (closest to the TS `localeCompare` this
/// replaces for ASCII and CJK names); equal-ignoring-case names fall back to
/// a plain comparison so the order is total and deterministic.
fn compare_names(a: &str, b: &str) -> Ordering {
    a.to_lowercase()
        .cmp(&b.to_lowercase())
        .then_with(|| a.cmp(b))
}
