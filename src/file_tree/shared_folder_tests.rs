use super::*;

fn snapshot(files: &[&str], empty_folders: &[&str]) -> FolderSnapshot {
    FolderSnapshot {
        folder: "Hymns".to_string(),
        files: files
            .iter()
            .map(|path| SnapshotFile {
                path: (*path).to_string(),
                content: format!("content of {path}"),
            })
            .collect(),
        empty_folders: empty_folders.iter().map(|raw| (*raw).to_string()).collect(),
    }
}

fn store(files: &[&str], folders: &[&str]) -> FileStoreState {
    FileStoreState {
        user_files: files
            .iter()
            .map(|path| ((*path).to_string(), "mine".to_string()))
            .collect(),
        folders: folders
            .iter()
            .map(|raw| TreePath::parse(raw).unwrap())
            .collect(),
        ..FileStoreState::default()
    }
}

fn ids(count: usize) -> Vec<String> {
    (0..count).map(|n| format!("id-{n}")).collect()
}

fn names(rows: &[TreeViewRow]) -> Vec<String> {
    rows.iter()
        .map(|row| format!("{}{}", "  ".repeat(row.depth), row.name))
        .collect()
}

#[test]
fn the_first_file_is_the_first_in_tree_order_folders_before_files() {
    let shared = snapshot(
        &["Zebra.jianpu", "Advent/Veni.jianpu", "Advent/O Come.jianpu"],
        &[],
    );
    assert_eq!(
        shared_folder_first_file(&shared).as_deref(),
        Some("Advent/O Come.jianpu")
    );
}

#[test]
fn a_folder_without_files_has_no_first_file() {
    assert_eq!(shared_folder_first_file(&snapshot(&[], &["Empty"])), None);
}

#[test]
fn the_view_is_rooted_at_the_shared_folder_without_demo_or_share_markers() {
    let shared = snapshot(&["Advent/O Come.jianpu", "Amazing.jianpu"], &["Rounds"]);
    let rows = shared_folder_view(&shared, "Advent/O Come.jianpu", &[]).unwrap();
    assert_eq!(
        names(&rows),
        vec!["Advent", "  O Come.jianpu", "Rounds", "Amazing.jianpu"]
    );
    assert!(rows.iter().all(|row| row.share_id.is_none()
        && row.shared_via.is_none()
        && !matches!(
            row.kind,
            TreeViewRowKind::DemoFolder | TreeViewRowKind::DemoFile
        )));
    assert!(rows.iter().any(|row| row.is_active));
}

#[test]
fn view_rows_for_an_invalid_snapshot_are_an_error() {
    let invalid = snapshot(&["../x.jianpu"], &[]);
    assert!(shared_folder_view(&invalid, "", &[]).is_err());
}

#[test]
fn import_adds_the_folder_with_relative_paths_ids_and_empty_folders() {
    let shared = snapshot(&["Advent/O Come.jianpu", "Amazing.jianpu"], &["Rounds"]);
    let imported = import_shared_folder(
        &store(&["Mine.jianpu"], &[]),
        &shared,
        &ids(2),
        &FileOperationContext::default(),
    )
    .unwrap();
    assert_eq!(
        imported.user_files.keys().cloned().collect::<Vec<_>>(),
        vec![
            "Hymns/Advent/O Come.jianpu",
            "Hymns/Amazing.jianpu",
            "Mine.jianpu"
        ]
    );
    assert_eq!(
        imported.user_files["Hymns/Amazing.jianpu"],
        "content of Amazing.jianpu"
    );
    assert_eq!(imported.file_ids.len(), 2);
    assert_eq!(
        imported.folders,
        vec![TreePath::parse("Hymns/Rounds").unwrap()]
    );
    assert_eq!(imported.active, "Hymns/Advent/O Come.jianpu");
}

#[test]
fn a_name_clash_with_a_folder_gets_a_numbered_name_and_leaves_the_original_alone() {
    let existing = store(&["Hymns/Old.jianpu", "Hymns 2/Old.jianpu"], &[]);
    let imported = import_shared_folder(
        &existing,
        &snapshot(&["New.jianpu"], &[]),
        &ids(1),
        &FileOperationContext::default(),
    )
    .unwrap();
    assert_eq!(imported.user_files["Hymns/Old.jianpu"], "mine");
    assert_eq!(imported.user_files["Hymns 2/Old.jianpu"], "mine");
    assert_eq!(
        imported.user_files["Hymns 3/New.jianpu"],
        "content of New.jianpu"
    );
}

#[test]
fn a_name_clash_with_a_file_gets_a_numbered_name() {
    let imported = import_shared_folder(
        &store(&["Hymns"], &[]),
        &snapshot(&["New.jianpu"], &[]),
        &ids(1),
        &FileOperationContext::default(),
    )
    .unwrap();
    assert!(imported.user_files.contains_key("Hymns 2/New.jianpu"));
}

#[test]
fn a_shared_folder_with_nothing_inside_is_imported_as_an_empty_folder() {
    let imported = import_shared_folder(
        &store(&[], &[]),
        &snapshot(&[], &[]),
        &[],
        &FileOperationContext::default(),
    )
    .unwrap();
    assert_eq!(imported.folders, vec![TreePath::parse("Hymns").unwrap()]);
}

#[test]
fn missing_ids_are_derived_from_the_context_id() {
    let context = FileOperationContext {
        new_file_id: "fresh".to_string(),
        ..FileOperationContext::default()
    };
    let imported = import_shared_folder(
        &store(&[], &[]),
        &snapshot(&["a.jianpu", "b.jianpu"], &[]),
        &["only".to_string()],
        &context,
    )
    .unwrap();
    assert_eq!(imported.file_ids["Hymns/a.jianpu"], "only");
    assert_eq!(imported.file_ids["Hymns/b.jianpu"], "fresh-1");
}

#[test]
fn an_invalid_snapshot_is_refused_and_nothing_is_imported() {
    let result = import_shared_folder(
        &store(&[], &[]),
        &snapshot(&["a.jianpu", "a.jianpu/b.jianpu"], &[]),
        &ids(2),
        &FileOperationContext::default(),
    );
    assert!(matches!(
        result,
        Err(FileStoreError::InvalidSharedFolder { .. })
    ));
}
