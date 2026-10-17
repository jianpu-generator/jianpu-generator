//! What the share UI knows about the owner's live links (the worker's
//! `POST /shares/overview`), and the questions it asks of it: is this folder
//! shared or inside a shared one, which files inside already have their own
//! link, what does the Share dialog say about the folder.

use itertools::Itertools;

use super::path::TreePath;
use super::store_state::FileStoreState;

/// How many names a warning lists before it says "and N more".
const LISTED_NAMES: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileShareLink {
    pub share_id: String,
    pub file_id: String,
    /// Full cloud path of the file.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderShareLink {
    pub share_id: String,
    /// Full path of the shared folder, no trailing slash.
    pub folder: String,
}

/// The owner's live (not stopped) links, as the worker last reported them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShareOverview {
    pub files: Vec<FileShareLink>,
    pub folders: Vec<FolderShareLink>,
}

/// The shared folder a path lies inside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedVia {
    pub folder: String,
    /// The shared folder's own name, for the "via <name>" tag.
    pub name: String,
    pub share_id: String,
}

/// What the Share dialog of one folder shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderShareInfo {
    /// The folder's own name (last segment), for the dialog title.
    pub name: String,
    /// Files inside, at any depth.
    pub file_count: usize,
    /// Folders inside, at any depth (the folder itself not counted).
    pub folder_count: usize,
    /// The folder's own live link, if it has one.
    pub share_id: Option<String>,
    /// The shared folder this one lies inside, if any.
    pub shared_via: Option<SharedVia>,
    /// Said before starting a link: files inside that already have their own
    /// link, which keep working independently. `None` when there are none.
    pub own_link_warning: Option<String>,
}

impl ShareOverview {
    /// The live link of exactly this folder.
    pub fn folder_share(&self, folder: &TreePath) -> Option<&FolderShareLink> {
        self.folders
            .iter()
            .find(|share| share.folder == folder.as_str())
    }

    /// The innermost shared folder strictly above `path`.
    pub fn shared_via(&self, path: &TreePath) -> Option<SharedVia> {
        path.ancestors()
            .iter()
            .rev()
            .find_map(|ancestor| self.folder_share(ancestor))
            .map(|share| SharedVia {
                name: TreePath::parse(&share.folder)
                    .map(|folder| folder.name().to_string())
                    .unwrap_or_else(|_| share.folder.clone()),
                folder: share.folder.clone(),
                share_id: share.share_id.clone(),
            })
    }

    /// Live links of single files inside `folder` (any depth), by path.
    pub fn file_links_within(&self, folder: &TreePath) -> Vec<&FileShareLink> {
        self.files
            .iter()
            .filter(|link| TreePath::parse(&link.name).is_ok_and(|path| path.is_within(folder)))
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .collect()
    }

    /// Live links of folders at or inside `folder`, by path.
    pub fn folder_links_at_or_within(&self, folder: &TreePath) -> Vec<&FolderShareLink> {
        self.folders
            .iter()
            .filter(|link| {
                TreePath::parse(&link.folder)
                    .is_ok_and(|path| path == *folder || path.is_within(folder))
            })
            .sorted_by(|a, b| a.folder.cmp(&b.folder))
            .collect()
    }
}

/// The Share dialog's facts about `folder`; `None` if it does not exist
/// (no file in it and no explicit folder at or inside it).
pub fn folder_share_info(
    state: &FileStoreState,
    overview: &ShareOverview,
    folder: &TreePath,
) -> Option<FolderShareInfo> {
    let files = state.files_within(folder);
    let explicit = state.explicit_folders_within(folder);
    if files.is_empty() && explicit.is_empty() {
        return None;
    }
    let folder_count = files
        .iter()
        .flat_map(TreePath::ancestors)
        .chain(
            explicit
                .iter()
                .flat_map(|path| path.ancestors().into_iter().chain([path.clone()])),
        )
        .filter(|path| path.is_within(folder))
        .unique()
        .count();
    Some(FolderShareInfo {
        name: folder.name().to_string(),
        file_count: files.len(),
        folder_count,
        share_id: overview
            .folder_share(folder)
            .map(|share| share.share_id.clone()),
        shared_via: overview.shared_via(folder),
        own_link_warning: own_link_warning(overview, folder),
    })
}

fn own_link_warning(overview: &ShareOverview, folder: &TreePath) -> Option<String> {
    let links = overview.file_links_within(folder);
    if links.is_empty() {
        return None;
    }
    let prefix_len = folder.as_str().len() + 1;
    let names = links
        .iter()
        .take(LISTED_NAMES)
        .map(|link| link.name.get(prefix_len..).unwrap_or(&link.name))
        .join(", ");
    let more = links.len().saturating_sub(LISTED_NAMES);
    let listed = if more == 0 {
        names
    } else {
        format!("{names} and {more} more")
    };
    Some(match links.len() {
        1 => format!(
            "1 file inside already has its own synced link ({listed}). That link keeps working independently of this one."
        ),
        count => format!(
            "{count} files inside already have their own synced links ({listed}). Those links keep working independently of this one."
        ),
    })
}

/// The extra sentences of the "Delete folder" confirmation when links are
/// involved; `None` when nothing is shared. Deleting moves the files to the
/// bin: a folder link stays alive and shows an empty folder until they are
/// restored (the link follows the folder path), a file link reads as ended
/// until its file is restored.
pub fn folder_delete_share_warning(overview: &ShareOverview, folder: &TreePath) -> Option<String> {
    let own = overview.folder_share(folder).is_some();
    let nested = overview
        .folder_links_at_or_within(folder)
        .into_iter()
        .filter(|link| link.folder != folder.as_str())
        .map(|link| link.folder.clone())
        .collect::<Vec<_>>();
    let via = overview.shared_via(folder);
    let file_links = overview.file_links_within(folder).len();
    let sentences = [
        own.then(|| {
            "This folder is shared with a link. Viewers will see an empty folder until you restore its files from the bin; the link itself keeps working.".to_string()
        }),
        (!nested.is_empty()).then(|| {
            format!(
                "Shared folder{} inside ({}) will show as empty to viewers until you restore the files from the bin.",
                if nested.len() == 1 { "" } else { "s" },
                nested.join(", ")
            )
        }),
        via.map(|via| {
            format!(
                "This folder is inside the shared folder \"{}\", so viewers will no longer see these files until you restore them from the bin.",
                via.name
            )
        }),
        (file_links > 0).then(|| {
            format!(
                "{file_links} file{} inside {} their own synced link{}, which will show as ended until restored.",
                if file_links == 1 { "" } else { "s" },
                if file_links == 1 { "has" } else { "have" },
                if file_links == 1 { "" } else { "s" },
            )
        }),
    ];
    let joined = sentences.into_iter().flatten().join(" ");
    (!joined.is_empty()).then_some(joined)
}
