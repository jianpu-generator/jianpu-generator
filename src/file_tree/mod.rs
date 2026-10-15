//! Pure model of the web app's file tree: files live at slash-separated
//! paths, a folder is a shared path prefix (or an explicitly created empty
//! folder). No I/O and no wasm glue here; see ARCHITECTURE.md "File tree".

mod path;
mod tree;
mod unique_name;

pub use path::{PathError, TreePath, MAX_PATH_BYTES};
pub use tree::{FileNode, FileTree, FileTreeError, FolderContents, FolderNode};
pub use unique_name::unique_path;

#[cfg(test)]
mod path_tests;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod unique_name_tests;
