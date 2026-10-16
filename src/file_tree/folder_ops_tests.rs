use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn context() -> FileOperationContext {
    FileOperationContext {
        demo_files: vec![DemoFile {
            path: path("Demo.jianpu"),
            content: String::new(),
        }],
        new_file_id: "new-id".to_string(),
    }
}

fn store(files: &[&str], folders: &[&str], bin: &[&str]) -> FileStoreState {
    FileStoreState {
        active: files.first().copied().unwrap_or_default().to_string(),
        user_files: files
            .iter()
            .map(|name| (name.to_string(), format!("content of {name}")))
            .collect(),
        bin: bin
            .iter()
            .map(|name| (name.to_string(), format!("binned {name}")))
            .collect(),
        file_ids: files
            .iter()
            .chain(bin)
            .map(|name| (name.to_string(), format!("id:{name}")))
            .collect(),
        folders: folders.iter().map(|raw| path(raw)).collect(),
    }
}

fn file_names(state: &FileStoreState) -> Vec<&str> {
    state.user_files.keys().map(String::as_str).collect()
}

fn mv(state: &FileStoreState, from: &str, to: &str) -> Result<FolderMoveOutcome, FileStoreError> {
    move_folder(state, &path(from), &path(to), &context())
}

#[test]
fn move_carries_files_ids_and_explicit_empty_subfolders() {
    let state = store(
        &["A/x.jianpu", "A/B/y.jianpu", "other.jianpu"],
        &["A/Empty", "A/B/Deeper/Empty"],
        &[],
    );
    let outcome = mv(&state, "A", "Z/Moved").unwrap();
    assert_eq!(
        file_names(&outcome.state),
        vec!["Z/Moved/B/y.jianpu", "Z/Moved/x.jianpu", "other.jianpu"]
    );
    assert_eq!(
        outcome.state.user_files["Z/Moved/x.jianpu"],
        "content of A/x.jianpu"
    );
    assert_eq!(
        outcome.state.file_ids["Z/Moved/B/y.jianpu"],
        "id:A/B/y.jianpu"
    );
    assert_eq!(outcome.state.file_ids["other.jianpu"], "id:other.jianpu");
    assert_eq!(
        outcome.state.folders,
        vec![path("Z/Moved/Empty"), path("Z/Moved/B/Deeper/Empty")]
    );
    assert_eq!(outcome.state.active, "Z/Moved/x.jianpu");
    assert_eq!(outcome.changes.len(), 4);
    assert!(outcome.changes.contains(&PathChange {
        kind: PathKind::File,
        old_path: path("A/x.jianpu"),
        new_path: path("Z/Moved/x.jianpu"),
    }));
    assert!(outcome.changes.contains(&PathChange {
        kind: PathKind::ExplicitFolder,
        old_path: path("A/Empty"),
        new_path: path("Z/Moved/Empty"),
    }));
}

#[test]
fn rename_of_an_explicit_empty_folder_itself() {
    let state = store(&[], &["Old"], &[]);
    let outcome = mv(&state, "Old", "New").unwrap();
    assert_eq!(outcome.state.folders, vec![path("New")]);
}

#[test]
fn moving_a_folder_onto_itself_changes_nothing() {
    let state = store(&["A/x.jianpu"], &[], &[]);
    let outcome = mv(&state, "A", "A").unwrap();
    assert_eq!(outcome.state, state);
    assert!(outcome.changes.is_empty());
}

#[test]
fn missing_folder_is_not_found() {
    let state = store(&["A/x.jianpu"], &[], &[]);
    assert!(matches!(
        mv(&state, "A/x.jianpu", "Q"),
        Err(FileStoreError::NotFound { .. })
    ));
    assert!(matches!(
        mv(&state, "Ax", "Q"),
        Err(FileStoreError::NotFound { .. })
    ));
}

#[test]
fn moving_into_itself_or_a_descendant_is_rejected() {
    let state = store(&["A/B/x.jianpu"], &[], &[]);
    assert!(matches!(
        mv(&state, "A", "A/B/C"),
        Err(FileStoreError::MoveIntoItself { .. })
    ));
}

#[test]
fn name_prefix_is_not_descendant() {
    let state = store(&["A/x.jianpu", "AB/y.jianpu"], &[], &[]);
    let outcome = mv(&state, "A", "AB2").unwrap();
    assert_eq!(
        file_names(&outcome.state),
        vec!["AB/y.jianpu", "AB2/x.jianpu"]
    );
}

#[test]
fn any_collision_rejects_the_whole_batch() {
    let state = store(
        &["A/x.jianpu", "A/y.jianpu", "B/y.jianpu", "F"],
        &["Taken"],
        &["C/x.jianpu"],
    );
    // Onto an existing folder, file, explicit folder, and demo file.
    for target in ["B", "F", "Taken", "Demo.jianpu"] {
        let result = mv(&state, "A", target);
        assert!(
            matches!(result, Err(FileStoreError::Collision { .. })),
            "{target}: {result:?}"
        );
    }
    // A bin entry at a would-be descendant path blocks the move too.
    let result = mv(&state, "A", "C");
    assert_eq!(
        result,
        Err(FileStoreError::Collision {
            path: path("C/x.jianpu")
        })
    );
    let blocked = store(&["A/x.jianpu"], &[], &["C/x.jianpu"]);
    assert_eq!(
        mv(&blocked, "A", "C").unwrap_err(),
        FileStoreError::Collision {
            path: path("C/x.jianpu")
        }
    );
}

#[test]
fn moving_under_a_file_is_rejected() {
    let state = store(&["A/x.jianpu", "F"], &[], &[]);
    assert!(matches!(
        mv(&state, "A", "F/inside"),
        Err(FileStoreError::AncestorIsFile { .. })
    ));
}

#[test]
fn a_resulting_path_over_1024_bytes_rejects_the_whole_batch() {
    let long_name = "n".repeat(1000);
    let state = store(
        &["A/short.jianpu", &format!("A/{long_name}.jianpu")],
        &[],
        &[],
    );
    let result = mv(&state, "A", &"d".repeat(30));
    assert!(
        matches!(result, Err(FileStoreError::InvalidPath { .. })),
        "{result:?}"
    );
    // The caller still holds the unchanged state.
    assert_eq!(state.user_files.len(), 2);
}

#[test]
fn folder_can_move_to_where_it_already_was_nested_under_a_sibling() {
    let state = store(&["A/x.jianpu", "B/y.jianpu"], &[], &[]);
    let outcome = mv(&state, "A", "B/A").unwrap();
    assert_eq!(
        file_names(&outcome.state),
        vec!["B/A/x.jianpu", "B/y.jianpu"]
    );
}

#[test]
fn delete_folder_bins_each_file_under_its_path_and_drops_empty_subfolders() {
    let state = store(
        &["A/x.jianpu", "A/B/y.jianpu", "other.jianpu"],
        &["A/Empty", "Keep"],
        &[],
    );
    let next = delete_folder(&state, &path("A"), &context()).unwrap();
    assert_eq!(file_names(&next), vec!["other.jianpu"]);
    assert_eq!(next.bin["A/x.jianpu"], "content of A/x.jianpu");
    assert_eq!(next.bin["A/B/y.jianpu"], "content of A/B/y.jianpu");
    assert_eq!(next.folders, vec![path("Keep")]);
    assert_eq!(next.active, "other.jianpu");
    assert_eq!(next.file_ids["A/x.jianpu"], "id:A/x.jianpu");
}

#[test]
fn delete_then_restore_recreates_the_folder() {
    let state = store(&["A/B/y.jianpu"], &[], &[]);
    let deleted = delete_folder(&state, &path("A"), &context()).unwrap();
    assert!(deleted.user_files.is_empty());
    let restored = restore_file(&deleted, "A/B/y.jianpu", &context()).unwrap();
    assert_eq!(
        restored.user_files["A/B/y.jianpu"],
        "content of A/B/y.jianpu"
    );
    assert_eq!(restored.file_ids["A/B/y.jianpu"], "id:A/B/y.jianpu");
}

#[test]
fn deleting_a_missing_folder_is_not_found() {
    let state = store(&["A/x.jianpu"], &[], &[]);
    assert!(matches!(
        delete_folder(&state, &path("Nope"), &context()),
        Err(FileStoreError::NotFound { .. })
    ));
}

#[test]
fn create_folder_adds_an_explicit_folder_and_leaves_the_active_file() {
    let state = store(&["Hymns/a.jianpu"], &[], &[]);
    let next = create_folder(&state, Some(&path("Hymns")), "Psalms", &context()).unwrap();
    assert_eq!(next.folders, vec![path("Hymns/Psalms")]);
    assert_eq!(next.active, state.active);
}

#[test]
fn create_folder_trims_the_name() {
    let state = store(&["a.jianpu"], &[], &[]);
    let next = create_folder(&state, None, "  Hymns ", &context()).unwrap();
    assert_eq!(next.folders, vec![path("Hymns")]);
}

#[test]
fn create_folder_rejects_a_name_taken_by_a_file_folder_or_implied_folder() {
    let state = store(&["Hymns/a.jianpu", "b.jianpu"], &["Empty"], &[]);
    for taken in ["Hymns", "Empty", "b.jianpu"] {
        assert!(
            matches!(
                create_folder(&state, None, taken, &context()),
                Err(FileStoreError::Collision { .. })
            ),
            "{taken}"
        );
    }
}

#[test]
fn create_folder_rejects_an_empty_or_nested_name() {
    let state = store(&["a.jianpu"], &[], &[]);
    assert!(create_folder(&state, None, "  ", &context()).is_err());
    assert!(create_folder(&state, None, "A/B", &context()).is_err());
}

#[test]
fn create_folder_inside_a_file_fails() {
    let state = store(&["a.jianpu"], &[], &[]);
    assert!(matches!(
        create_folder(&state, Some(&path("a.jianpu")), "X", &context()),
        Err(FileStoreError::AncestorIsFile { .. })
    ));
}

#[test]
fn moving_a_folder_to_the_reserved_leaf_is_rejected() {
    let state = store(&["A/a.jianpu"], &[], &[]);
    let error = move_folder(&state, &path("A"), &path("B/.keep"), &context()).unwrap_err();
    assert_eq!(
        error,
        FileStoreError::ReservedName {
            path: path("B/.keep")
        }
    );
}
