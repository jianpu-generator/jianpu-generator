use super::path::{PathError, TreePath};
use std::collections::HashSet;

const DEFAULT_EXTENSION: &str = ".jianpu";

/// `desired` if free, else the first of `stem 2.ext`, `stem 3.ext`, ... that
/// is free in the same folder. `taken` may hold paths from anywhere (files,
/// folders, binned files); only siblings of `desired` matter. A name with no
/// extension (or only a leading dot) gets `.jianpu`, like the TS it replaces.
/// Fails only when the suffixed path would exceed the length limit.
pub fn unique_path(desired: &TreePath, taken: &HashSet<TreePath>) -> Result<TreePath, PathError> {
    let parent = desired.parent();
    let sibling_names: HashSet<&str> = taken
        .iter()
        .filter(|path| path.parent() == parent)
        .map(TreePath::name)
        .collect();
    if !sibling_names.contains(desired.name()) {
        return Ok(desired.clone());
    }
    let name = desired.name();
    let (stem, extension) = match name.rfind('.') {
        Some(dot) if dot > 0 => name.split_at(dot),
        _ => (name, DEFAULT_EXTENSION),
    };
    let candidate = (2..)
        .map(|n| format!("{stem} {n}{extension}"))
        .find(|candidate| !sibling_names.contains(candidate.as_str()))
        .unwrap_or_default();
    TreePath::child(parent.as_ref(), &candidate)
}
