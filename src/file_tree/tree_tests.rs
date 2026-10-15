use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn paths(raw: &[&str]) -> Vec<TreePath> {
    raw.iter().map(|raw| path(raw)).collect()
}

fn names(contents: &FolderContents) -> Vec<String> {
    contents
        .folders
        .iter()
        .map(|folder| format!("{}/", folder.name))
        .chain(contents.files.iter().map(|file| file.name.clone()))
        .collect()
}

#[test]
fn empty_input_gives_empty_tree() {
    assert_eq!(FileTree::build(&[], &[]).unwrap(), FileTree::default());
}

#[test]
fn derives_folders_from_file_prefixes() {
    let tree = FileTree::build(
        &paths(&[
            "Hymns/Advent/O Come.jianpu",
            "Hymns/Grace.jianpu",
            "top.jianpu",
        ]),
        &[],
    )
    .unwrap();
    assert_eq!(names(&tree.root), vec!["Hymns/", "top.jianpu"]);
    let hymns = tree.root.folders.first().unwrap();
    assert_eq!(hymns.path, path("Hymns"));
    assert_eq!(names(&hymns.contents), vec!["Advent/", "Grace.jianpu"]);
    let advent = hymns.contents.folders.first().unwrap();
    assert_eq!(advent.path, path("Hymns/Advent"));
    assert_eq!(names(&advent.contents), vec!["O Come.jianpu"]);
    let file = advent.contents.files.first().unwrap();
    assert_eq!(file.path, path("Hymns/Advent/O Come.jianpu"));
}

#[test]
fn explicit_empty_folders_appear() {
    let tree = FileTree::build(&paths(&["a.jianpu"]), &paths(&["Empty", "x/y/z"])).unwrap();
    assert_eq!(names(&tree.root), vec!["Empty/", "x/", "a.jianpu"]);
    let x = tree.root.folders.get(1).unwrap();
    assert_eq!(names(&x.contents), vec!["y/"]);
    let z = x
        .contents
        .folders
        .first()
        .unwrap()
        .contents
        .folders
        .first()
        .unwrap();
    assert_eq!(z.path, path("x/y/z"));
    assert_eq!(z.contents, FolderContents::default());
}

#[test]
fn explicit_folder_that_already_has_files_is_merged() {
    let tree = FileTree::build(&paths(&["a/b.jianpu"]), &paths(&["a", "a"])).unwrap();
    assert_eq!(tree.root.folders.len(), 1);
}

#[test]
fn duplicate_file_paths_are_merged() {
    let tree = FileTree::build(&paths(&["a.jianpu", "a.jianpu"]), &[]).unwrap();
    assert_eq!(tree.root.files.len(), 1);
}

#[test]
fn sorts_folders_first_then_files_alphabetically() {
    let tree = FileTree::build(
        &paths(&[
            "b.jianpu",
            "a.jianpu",
            "z/f.jianpu",
            "m/f.jianpu",
            "A/f.jianpu",
        ]),
        &[],
    )
    .unwrap();
    assert_eq!(
        names(&tree.root),
        vec!["A/", "m/", "z/", "a.jianpu", "b.jianpu"]
    );
}

#[test]
fn sort_is_case_insensitive_with_deterministic_tiebreak() {
    let tree = FileTree::build(
        &paths(&[
            "banana.jianpu",
            "Apple.jianpu",
            "cherry.jianpu",
            "apple.jianpu",
        ]),
        &[],
    )
    .unwrap();
    assert_eq!(
        names(&tree.root),
        vec![
            "Apple.jianpu",
            "apple.jianpu",
            "banana.jianpu",
            "cherry.jianpu"
        ]
    );
}

#[test]
fn input_order_does_not_matter() {
    let forward = FileTree::build(&paths(&["a/x.jianpu", "b.jianpu"]), &paths(&["c"])).unwrap();
    let backward = FileTree::build(&paths(&["b.jianpu", "a/x.jianpu"]), &paths(&["c"])).unwrap();
    assert_eq!(forward, backward);
}

#[test]
fn file_that_is_a_prefix_of_another_file_conflicts() {
    assert_eq!(
        FileTree::build(&paths(&["a", "a/b.jianpu"]), &[]),
        Err(FileTreeError::FileFolderConflict { path: path("a") })
    );
}

#[test]
fn file_that_is_an_explicit_folder_conflicts() {
    assert_eq!(
        FileTree::build(&paths(&["a"]), &paths(&["a"])),
        Err(FileTreeError::FileFolderConflict { path: path("a") })
    );
}

#[test]
fn file_inside_ancestor_of_explicit_folder_conflicts() {
    assert_eq!(
        FileTree::build(&paths(&["a/b"]), &paths(&["a/b/c"])),
        Err(FileTreeError::FileFolderConflict { path: path("a/b") })
    );
}

#[test]
fn same_name_in_different_folders_is_fine() {
    assert!(FileTree::build(&paths(&["a/x.jianpu", "b/x.jianpu", "x.jianpu"]), &[]).is_ok());
}

#[test]
fn all_paths_lists_files_and_folders() {
    let tree = FileTree::build(&paths(&["a/b.jianpu", "c.jianpu"]), &paths(&["e"])).unwrap();
    let mut all = tree.all_paths();
    all.sort();
    assert_eq!(all, paths(&["a", "a/b.jianpu", "c.jianpu", "e"]));
}

#[test]
fn tree_unique_path_considers_folders_and_extra_taken() {
    let tree = FileTree::build(&paths(&["a/x.jianpu", "a/x 2.jianpu"]), &[]).unwrap();
    assert_eq!(
        tree.unique_path(&path("a/x.jianpu"), &paths(&["a/x 3.jianpu"]))
            .unwrap(),
        path("a/x 4.jianpu")
    );
    assert_eq!(
        tree.unique_path(&path("b/x.jianpu"), &[]).unwrap(),
        path("b/x.jianpu")
    );
}
