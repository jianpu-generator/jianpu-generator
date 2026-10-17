use super::*;
use std::collections::HashSet;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn taken(raw: &[&str]) -> HashSet<TreePath> {
    raw.iter().map(|raw| path(raw)).collect()
}

fn unique(desired: &str, taken_paths: &[&str]) -> String {
    unique_path(&path(desired), &taken(taken_paths))
        .unwrap()
        .as_str()
        .to_string()
}

#[test]
fn free_name_is_kept() {
    assert_eq!(unique("untitled.jianpu", &[]), "untitled.jianpu");
    assert_eq!(unique("a/b.jianpu", &["a/c.jianpu"]), "a/b.jianpu");
}

#[test]
fn collision_gets_numeric_suffix_before_extension() {
    assert_eq!(
        unique("untitled.jianpu", &["untitled.jianpu"]),
        "untitled 2.jianpu"
    );
}

#[test]
fn skips_taken_suffixes() {
    assert_eq!(
        unique(
            "untitled.jianpu",
            &["untitled.jianpu", "untitled 2.jianpu", "untitled 3.jianpu"]
        ),
        "untitled 4.jianpu"
    );
}

#[test]
fn collisions_only_matter_within_the_same_folder() {
    assert_eq!(
        unique("a/x.jianpu", &["x.jianpu", "b/x.jianpu", "a/b/x.jianpu"]),
        "a/x.jianpu"
    );
    assert_eq!(
        unique("a/x.jianpu", &["a/x.jianpu", "x 2.jianpu", "b/x 2.jianpu"]),
        "a/x 2.jianpu"
    );
}

#[test]
fn sibling_folder_with_same_name_counts_as_taken() {
    assert_eq!(unique("a/x", &["a/x"]), "a/x 2.jianpu");
}

#[test]
fn name_without_extension_gets_default_extension() {
    assert_eq!(unique("song", &["song"]), "song 2.jianpu");
}

#[test]
fn leading_dot_only_is_not_an_extension() {
    assert_eq!(unique(".hidden", &[".hidden"]), ".hidden 2.jianpu");
}

#[test]
fn other_extension_is_preserved() {
    assert_eq!(unique("a.txt", &["a.txt"]), "a 2.txt");
}

#[test]
fn suffix_pushing_past_the_limit_is_an_error() {
    let long = format!("{}a.jianpu", "x".repeat(1024 - 8));
    assert_eq!(long.len(), 1024);
    let result = unique_path(&path(&long), &taken(&[&long]));
    assert!(matches!(result, Err(PathError::TooLong { .. })));
}

fn after_collision(rejected: &str) -> String {
    path_after_collision(&path(rejected))
        .unwrap()
        .as_str()
        .to_string()
}

#[test]
fn after_collision_appends_two_then_increments() {
    assert_eq!(after_collision("a.jianpu"), "a 2.jianpu");
    assert_eq!(after_collision("a 2.jianpu"), "a 3.jianpu");
    assert_eq!(after_collision("a 9.jianpu"), "a 10.jianpu");
}

#[test]
fn after_collision_keeps_the_folder_prefix() {
    assert_eq!(
        after_collision("Hymns/Advent/O Come.jianpu"),
        "Hymns/Advent/O Come 2.jianpu"
    );
    assert_eq!(after_collision("Hymns/X 2.jianpu"), "Hymns/X 3.jianpu");
}

#[test]
fn after_collision_ignores_a_one_or_oversized_suffix() {
    assert_eq!(after_collision("a 1.jianpu"), "a 1 2.jianpu");
    assert_eq!(after_collision("a 99999.jianpu"), "a 99999 2.jianpu");
}

#[test]
fn after_collision_without_extension_gets_jianpu() {
    assert_eq!(after_collision("notes"), "notes 2.jianpu");
}

#[test]
fn after_collision_fails_when_the_suffix_would_exceed_the_limit() {
    let long = format!("{}.jianpu", "a".repeat(MAX_PATH_BYTES - ".jianpu".len()));
    assert!(path_after_collision(&path(&long)).is_err());
}
