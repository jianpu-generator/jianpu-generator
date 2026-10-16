use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn context() -> FileOperationContext {
    FileOperationContext {
        demo_files: vec![DemoFile {
            path: path("Demo.jianpu"),
            content: "demo content".to_string(),
        }],
        new_file_id: "new-id".to_string(),
    }
}

fn store(files: &[&str], bin: &[&str]) -> FileStoreState {
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
        folders: Vec::new(),
    }
}

fn file_names(state: &FileStoreState) -> Vec<&str> {
    state.user_files.keys().map(String::as_str).collect()
}

#[test]
fn create_file_at_top_level_is_named_and_active() {
    let state = store(&["a.jianpu"], &[]);
    let next = create_file(&state, None, "  Song ", "template", &context()).unwrap();
    assert_eq!(next.active, "Song.jianpu");
    assert_eq!(next.user_files["Song.jianpu"], "template");
    assert_eq!(next.file_ids["Song.jianpu"], "new-id");
}

#[test]
fn create_file_keeps_an_existing_extension() {
    let state = store(&[], &[]);
    let next = create_file(&state, None, "Song.jianpu", "t", &context()).unwrap();
    assert_eq!(next.active, "Song.jianpu");
}

#[test]
fn create_file_in_folder_is_scoped_to_that_folder() {
    let state = store(&["Song.jianpu", "Hymns/Other.jianpu"], &[]);
    let next = create_file(&state, Some(&path("Hymns")), "Song", "t", &context()).unwrap();
    assert_eq!(next.active, "Hymns/Song.jianpu");
}

#[test]
fn create_file_rejects_a_taken_name() {
    let state = store(&["Song.jianpu"], &[]);
    for taken in ["Song", "Song.jianpu"] {
        assert!(
            matches!(
                create_file(&state, None, taken, "t", &context()),
                Err(FileStoreError::Collision { .. })
            ),
            "{taken}"
        );
    }
}

#[test]
fn create_file_rejects_an_empty_or_nested_name() {
    let state = store(&[], &[]);
    assert!(create_file(&state, None, "  ", "t", &context()).is_err());
    assert!(create_file(&state, None, "A/B", "t", &context()).is_err());
}

#[test]
fn create_file_error_explains_a_clash_but_not_success() {
    let state = store(&["Song.jianpu"], &[]);
    assert!(create_file_error(&state, None, "Song", &context()).is_some());
    assert_eq!(create_file_error(&state, None, "New", &context()), None);
}

#[test]
fn create_file_inside_a_file_is_rejected() {
    let state = store(&["a.jianpu"], &[]);
    assert_eq!(
        create_file(&state, Some(&path("a.jianpu")), "t", "t", &context()),
        Err(FileStoreError::AncestorIsFile {
            path: path("a.jianpu")
        })
    );
}

#[test]
fn duplicate_stays_in_the_same_folder() {
    let state = store(&["Hymns/Advent/song.jianpu"], &[]);
    let next = duplicate_file(&state, &path("Hymns/Advent/song.jianpu"), &context()).unwrap();
    assert_eq!(next.active, "Hymns/Advent/song 2.jianpu");
    assert_eq!(
        next.user_files["Hymns/Advent/song 2.jianpu"],
        "content of Hymns/Advent/song.jianpu"
    );
    assert_ne!(
        next.file_ids["Hymns/Advent/song 2.jianpu"],
        "id:Hymns/Advent/song.jianpu"
    );
}

#[test]
fn duplicate_skips_binned_names_and_copies_demo_files_to_top_level() {
    let state = store(&["a.jianpu"], &["a 2.jianpu"]);
    let next = duplicate_file(&state, &path("a.jianpu"), &context()).unwrap();
    assert_eq!(next.active, "a 3.jianpu");
    let next = duplicate_file(&state, &path("Demo.jianpu"), &context()).unwrap();
    assert_eq!(next.active, "Demo 2.jianpu");
    assert_eq!(next.user_files["Demo 2.jianpu"], "demo content");
}

#[test]
fn rename_keeps_content_and_file_id() {
    let state = store(&["a.jianpu"], &[]);
    let next = rename_file(&state, "a.jianpu", "b", &context()).unwrap();
    assert_eq!(file_names(&next), vec!["b.jianpu"]);
    assert_eq!(next.user_files["b.jianpu"], "content of a.jianpu");
    assert_eq!(next.file_ids["b.jianpu"], "id:a.jianpu");
    assert!(!next.file_ids.contains_key("a.jianpu"));
    assert_eq!(next.active, "b.jianpu");
}

#[test]
fn move_file_into_new_folder_keeps_file_id() {
    let state = store(&["a.jianpu"], &[]);
    let next = rename_file(&state, "a.jianpu", "Hymns/Advent/a.jianpu", &context()).unwrap();
    assert_eq!(file_names(&next), vec!["Hymns/Advent/a.jianpu"]);
    assert_eq!(next.file_ids["Hymns/Advent/a.jianpu"], "id:a.jianpu");
}

#[test]
fn rename_to_same_path_is_a_no_op() {
    let state = store(&["a.jianpu"], &[]);
    assert_eq!(
        rename_file(&state, "a.jianpu", " a ", &context()),
        Ok(state)
    );
}

#[test]
fn rename_collisions_are_rejected() {
    let state = store(
        &["a.jianpu", "b.jianpu", "F.jianpu/x.jianpu"],
        &["gone.jianpu"],
    );
    for target in ["b.jianpu", "gone.jianpu", "F.jianpu", "Demo.jianpu"] {
        let result = rename_file(&state, "a.jianpu", target, &context());
        assert!(
            matches!(result, Err(FileStoreError::Collision { .. })),
            "{target}: {result:?}"
        );
    }
}

#[test]
fn rename_under_a_file_or_with_bad_path_is_rejected() {
    let state = store(&["a.jianpu", "b.jianpu"], &[]);
    assert!(matches!(
        rename_file(&state, "a.jianpu", "b.jianpu/c", &context()),
        Err(FileStoreError::AncestorIsFile { .. })
    ));
    assert!(matches!(
        rename_file(&state, "a.jianpu", "x//y", &context()),
        Err(FileStoreError::InvalidPath { .. })
    ));
    assert!(matches!(
        rename_file(&state, "a.jianpu", "folder/", &context()),
        Err(FileStoreError::InvalidPath { .. })
    ));
    assert!(matches!(
        rename_file(&state, "a.jianpu", &"x".repeat(1020), &context()),
        Err(FileStoreError::InvalidPath { .. })
    ));
}

#[test]
fn demo_and_missing_files_cannot_be_renamed_or_deleted() {
    let state = store(&["a.jianpu"], &[]);
    assert!(matches!(
        rename_file(&state, "Demo.jianpu", "x", &context()),
        Err(FileStoreError::DemoFileReadOnly { .. })
    ));
    assert!(matches!(
        delete_file(&state, "Demo.jianpu", &context()),
        Err(FileStoreError::DemoFileReadOnly { .. })
    ));
    assert!(matches!(
        delete_file(&state, "nope.jianpu", &context()),
        Err(FileStoreError::NotFound { .. })
    ));
}

#[test]
fn delete_bins_under_full_path_and_picks_next_active() {
    let state = store(&["A/a.jianpu", "b.jianpu"], &[]);
    let next = delete_file(&state, "A/a.jianpu", &context()).unwrap();
    assert_eq!(next.bin["A/a.jianpu"], "content of A/a.jianpu");
    assert_eq!(next.active, "b.jianpu");
    assert_eq!(next.file_ids["A/a.jianpu"], "id:A/a.jianpu");
    let next = delete_file(&next, "b.jianpu", &context()).unwrap();
    assert_eq!(next.active, "Demo.jianpu");
}

#[test]
fn restore_goes_back_to_original_path_recreating_the_folder() {
    let state = store(&["b.jianpu"], &["Hymns/Advent/a.jianpu"]);
    let next = restore_file(&state, "Hymns/Advent/a.jianpu", &context()).unwrap();
    assert_eq!(next.active, "Hymns/Advent/a.jianpu");
    assert_eq!(
        next.user_files["Hymns/Advent/a.jianpu"],
        "binned Hymns/Advent/a.jianpu"
    );
    assert!(next.bin.is_empty());
    assert_eq!(
        next.file_ids["Hymns/Advent/a.jianpu"],
        "id:Hymns/Advent/a.jianpu"
    );
}

#[test]
fn restore_picks_unique_name_when_path_is_taken() {
    let state = store(&["F/a.jianpu"], &["F/a.jianpu_"]);
    let mut state = state;
    state.bin.insert("F/a.jianpu".into(), "old".into());
    state.bin.remove("F/a.jianpu_");
    // A live file now shares the bin key (e.g. from an older flat store).
    let next = restore_file(&state, "F/a.jianpu", &context()).unwrap();
    assert_eq!(next.active, "F/a 2.jianpu");
    assert_eq!(next.user_files["F/a 2.jianpu"], "old");
    assert_eq!(next.user_files["F/a.jianpu"], "content of F/a.jianpu");
    assert_eq!(next.file_ids["F/a 2.jianpu"], "id:F/a.jianpu");
}

#[test]
fn restore_under_a_file_fails_and_keeps_the_bin() {
    let state = store(&["F"], &["F/a.jianpu"]);
    assert!(matches!(
        restore_file(&state, "F/a.jianpu", &context()),
        Err(FileStoreError::AncestorIsFile { .. })
    ));
    assert!(matches!(
        restore_file(&state, "nope", &context()),
        Err(FileStoreError::NotFound { .. })
    ));
}

#[test]
fn rename_to_the_reserved_placeholder_leaf_is_rejected() {
    let state = store(&["a.jianpu"], &[]);
    let error = rename_file(&state, "a.jianpu", "Songs/.keep", &context()).unwrap_err();
    assert_eq!(
        error,
        FileStoreError::ReservedName {
            path: path("Songs/.keep")
        }
    );
    assert_eq!(
        error.to_string(),
        "`.keep` is a reserved name and cannot be used"
    );
}

#[test]
fn rename_error_reports_the_reserved_leaf() {
    let state = store(&["a.jianpu"], &[]);
    assert_eq!(
        rename_error(&state, "a.jianpu", "Songs/.keep", &context()).as_deref(),
        Some("`.keep` is a reserved name and cannot be used")
    );
}

#[test]
fn a_name_that_merely_starts_with_keep_is_fine() {
    let state = store(&["a.jianpu"], &[]);
    let next = rename_file(&state, "a.jianpu", "Songs/.keep.jianpu", &context()).unwrap();
    assert!(next.user_files.contains_key("Songs/.keep.jianpu"));
}
