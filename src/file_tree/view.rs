use super::path::TreePath;
use super::tree::{FileTree, FolderContents};

/// Expansion key of the built-in `Demo/` row. It ends in `/`, so it can never
/// equal a valid user folder path (a trailing separator is an empty segment).
pub const DEMO_EXPANSION_KEY: &str = "Demo/";
const DEMO_FOLDER_NAME: &str = "Demo";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeViewRowKind {
    Folder,
    File,
    /// The read-only `Demo/` folder row.
    DemoFolder,
    /// A read-only demo file; its `key` is the bare demo file name.
    DemoFile,
    /// Placeholder shown under an expanded folder that holds nothing.
    EmptyFolderHint,
}

/// What the view depends on besides the tree itself. All raw UI state: the
/// view decides which rows are visible and which folders count as open.
#[derive(Debug, Clone, Copy)]
pub struct TreeViewInput<'a> {
    /// Path (or bare demo file name) of the file being edited.
    pub active: &'a str,
    /// Expansion keys the user opened (folder paths, plus [`DEMO_EXPANSION_KEY`]).
    pub expanded: &'a [String],
    pub demo_file_names: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeViewRow {
    /// Folder path, file path, bare demo file name, or [`DEMO_EXPANSION_KEY`].
    /// For a folder this is also its expansion key.
    pub key: String,
    pub name: String,
    /// 0 at the top level.
    pub depth: usize,
    pub kind: TreeViewRowKind,
    /// Open folder (always false for files). The ancestors of the active file
    /// are open regardless of `expanded`.
    pub expanded: bool,
    /// Files at any depth below a folder row; 0 for the other kinds.
    pub file_count: usize,
    pub is_active: bool,
}

/// The active file's location split for the toolbar: its folder names
/// (outermost first) and its own name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveLabel {
    pub folder_names: Vec<String>,
    pub file_name: String,
}

impl FileTree {
    /// The visible rows of the file list: the user's tree (collapsed folders
    /// hide their contents), then the `Demo/` folder last. Folders and
    /// files are in [`FileTree::rows`] order.
    pub fn view_rows(&self, input: &TreeViewInput) -> Vec<TreeViewRow> {
        let active_path = TreePath::parse(input.active).ok();
        folder_rows(&self.root, 0, input, active_path.as_ref())
            .into_iter()
            .chain(demo_rows(input))
            .collect()
    }
}

fn folder_rows(
    contents: &FolderContents,
    depth: usize,
    input: &TreeViewInput,
    active_path: Option<&TreePath>,
) -> Vec<TreeViewRow> {
    contents
        .folders
        .iter()
        .flat_map(|folder| {
            let expanded = input.expanded.iter().any(|key| key == folder.path.as_str())
                || active_path.is_some_and(|active| active.is_within(&folder.path));
            let header = TreeViewRow {
                key: folder.path.as_str().to_string(),
                name: folder.name.clone(),
                depth,
                kind: TreeViewRowKind::Folder,
                expanded,
                file_count: count_files(&folder.contents),
                is_active: false,
            };
            let children = if !expanded {
                vec![]
            } else if folder.contents.folders.is_empty() && folder.contents.files.is_empty() {
                vec![hint_row(depth + 1)]
            } else {
                folder_rows(&folder.contents, depth + 1, input, active_path)
            };
            std::iter::once(header).chain(children)
        })
        .chain(contents.files.iter().map(|file| TreeViewRow {
            key: file.path.as_str().to_string(),
            name: file.name.clone(),
            depth,
            kind: TreeViewRowKind::File,
            expanded: false,
            file_count: 0,
            is_active: active_path.is_some_and(|active| *active == file.path),
        }))
        .collect()
}

fn demo_rows(input: &TreeViewInput) -> Vec<TreeViewRow> {
    if input.demo_file_names.is_empty() {
        return vec![];
    }
    let active_is_demo = input
        .demo_file_names
        .iter()
        .any(|name| name == input.active);
    let expanded = active_is_demo || input.expanded.iter().any(|key| key == DEMO_EXPANSION_KEY);
    let header = TreeViewRow {
        key: DEMO_EXPANSION_KEY.to_string(),
        name: DEMO_FOLDER_NAME.to_string(),
        depth: 0,
        kind: TreeViewRowKind::DemoFolder,
        expanded,
        file_count: input.demo_file_names.len(),
        is_active: false,
    };
    let files = input
        .demo_file_names
        .iter()
        .filter(|_| expanded)
        .map(|name| TreeViewRow {
            key: name.clone(),
            name: name.clone(),
            depth: 1,
            kind: TreeViewRowKind::DemoFile,
            expanded: false,
            file_count: 0,
            is_active: name == input.active,
        });
    std::iter::once(header).chain(files).collect()
}

fn hint_row(depth: usize) -> TreeViewRow {
    TreeViewRow {
        key: String::new(),
        name: "Empty folder".to_string(),
        depth,
        kind: TreeViewRowKind::EmptyFolderHint,
        expanded: false,
        file_count: 0,
        is_active: false,
    }
}

fn count_files(contents: &FolderContents) -> usize {
    contents.files.len()
        + contents
            .folders
            .iter()
            .map(|folder| count_files(&folder.contents))
            .sum::<usize>()
}

/// Splits the active file for the toolbar button. A demo file shows under
/// `Demo`; a path that fails to parse is shown whole.
pub fn active_label(active: &str, demo_file_names: &[String]) -> ActiveLabel {
    if demo_file_names.iter().any(|name| name == active) {
        return ActiveLabel {
            folder_names: vec![DEMO_FOLDER_NAME.to_string()],
            file_name: active.to_string(),
        };
    }
    match TreePath::parse(active) {
        Ok(path) => ActiveLabel {
            folder_names: path
                .ancestors()
                .iter()
                .map(|ancestor| ancestor.name().to_string())
                .collect(),
            file_name: path.name().to_string(),
        },
        Err(_) => ActiveLabel {
            folder_names: vec![],
            file_name: active.to_string(),
        },
    }
}
