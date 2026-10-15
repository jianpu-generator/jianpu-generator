//! Pure model of the web app's file tree: files live at slash-separated
//! paths, a folder is a shared path prefix (or an explicitly created empty
//! folder). No I/O and no wasm glue here; see ARCHITECTURE.md "File tree".

mod file_ops;
mod folder_ops;
mod path;
mod store_state;
mod tree;
mod unique_name;

pub use file_ops::{create_file, delete_file, duplicate_file, rename_file, restore_file};
pub use folder_ops::{delete_folder, move_folder, FolderMoveOutcome, PathChange, PathKind};
pub use path::{PathError, TreePath, MAX_PATH_BYTES};
pub use store_state::{DemoFile, FileOperationContext, FileStoreError, FileStoreState};
pub use tree::{FileNode, FileTree, FileTreeError, FolderContents, FolderNode, TreeRow};
pub use unique_name::unique_path;

#[cfg(test)]
mod file_ops_tests;
#[cfg(test)]
mod folder_ops_tests;
#[cfg(test)]
mod path_tests;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod unique_name_tests;
