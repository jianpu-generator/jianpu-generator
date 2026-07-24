use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use syn::Item;

use crate::attrs::{has_cfg_test, path_attr_value};

pub fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if matches!(name, "target" | "pkg" | "node_modules" | ".git") {
                continue;
            }
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn resolve_mod_file(
    current_file: &Path,
    mod_name: &str,
    path_attr: Option<String>,
) -> Option<PathBuf> {
    let dir = current_file.parent()?;
    if let Some(p) = path_attr {
        return Some(dir.join(p));
    }
    let flat = dir.join(format!("{mod_name}.rs"));
    if flat.exists() {
        return Some(flat);
    }
    let nested = dir.join(mod_name).join("mod.rs");
    if nested.exists() {
        return Some(nested);
    }
    None
}

/// Walks the `mod` tree starting from a crate root (`lib.rs`), propagating
/// test-context down: a file reached only through a `#[cfg(test)] mod foo;`
/// declaration is test-only even though it carries no `#[cfg(test)]`
/// attribute of its own, and so are all files *its* `mod` declarations point
/// to, transitively.
pub fn walk_mod_tree(
    file: &Path,
    in_test: bool,
    parsed_cache: &HashMap<PathBuf, syn::File>,
    test_only: &mut HashSet<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) {
    if !visited.insert(file.to_path_buf()) {
        return;
    }
    if in_test {
        test_only.insert(file.to_path_buf());
    }
    let Some(parsed) = parsed_cache.get(file) else {
        return;
    };
    walk_items_for_mods(
        &parsed.items,
        file,
        in_test,
        parsed_cache,
        test_only,
        visited,
    );
}

fn walk_items_for_mods(
    items: &[Item],
    current_file: &Path,
    in_test: bool,
    parsed_cache: &HashMap<PathBuf, syn::File>,
    test_only: &mut HashSet<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) {
    for syntax_item in items {
        let Item::Mod(m) = syntax_item else { continue };
        let nested_test = in_test || has_cfg_test(&m.attrs);
        if let Some((_, inner)) = &m.content {
            walk_items_for_mods(
                inner,
                current_file,
                nested_test,
                parsed_cache,
                test_only,
                visited,
            );
        } else if let Some(target) = resolve_mod_file(
            current_file,
            &m.ident.to_string(),
            path_attr_value(&m.attrs),
        ) {
            walk_mod_tree(&target, nested_test, parsed_cache, test_only, visited);
        }
    }
}
