use super::*;

#[test]
fn absent_section_returns_none() {
    let (section, errors) = parse_group("", 0);
    assert!(section.is_none());
    assert!(errors.is_empty());
}

#[test]
fn parses_named_group_with_abbreviation() {
    let (section, errors) = parse_group("Vocal [v] = s1 s2\n", 0);
    assert!(errors.is_empty());
    let section = section.unwrap();
    assert_eq!(section.groups.len(), 1);
    assert_eq!(section.groups[0].display_name, "Vocal");
    assert_eq!(section.groups[0].abbreviation, "v");
    assert_eq!(section.groups[0].members, vec!["s1", "s2"]);
}

#[test]
fn parses_group_without_bracket_abbreviation() {
    let (section, errors) = parse_group("Vocal = s1 s2\n", 0);
    assert!(errors.is_empty());
    let section = section.unwrap();
    assert_eq!(section.groups[0].display_name, "Vocal");
    assert_eq!(section.groups[0].abbreviation, "Vocal");
}

#[test]
fn malformed_line_without_equals_is_recoverable() {
    let (section, errors) = parse_group("not a declaration\n", 0);
    assert_eq!(section.unwrap().groups.len(), 0);
    assert_eq!(errors.len(), 1);
}

#[test]
fn empty_member_list_is_recoverable() {
    let (section, errors) = parse_group("Vocal [v] = \n", 0);
    assert_eq!(section.unwrap().groups.len(), 0);
    assert_eq!(errors.len(), 1);
}

// --- resolve_and_validate_groups ---

use crate::ast::parsed::{PartDecl, PartKind};

fn decl(name: &str, kind: PartKind) -> PartDecl {
    PartDecl {
        abbreviation: name.to_string(),
        display_name: name.to_string(),
        kind,
        follow_target: None,
        soundfont: Default::default(),
        volume: 100,
        octave_offset: 0,
    }
}

#[test]
fn valid_group_resolves_direct_members() {
    let (section, _) = parse_group("Soprano [s] = s1 s2\n", 0);
    let declarations = vec![decl("s1", PartKind::Notes), decl("s2", PartKind::Notes)];
    let (resolved, errors) = resolve_and_validate_groups(&section.unwrap(), &declarations);
    assert!(errors.is_empty());
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].abbreviation, "s");
    assert_eq!(resolved[0].members, vec!["s1", "s2"]);
}

#[test]
fn nested_group_resolves_transitively() {
    let (section, _) = parse_group("Soprano [s] = s1 s2\nAlto [a] = a1 a2\nAll [x] = s a\n", 0);
    let declarations = vec![
        decl("s1", PartKind::Notes),
        decl("s2", PartKind::Notes),
        decl("a1", PartKind::Notes),
        decl("a2", PartKind::Notes),
    ];
    let (resolved, errors) = resolve_and_validate_groups(&section.unwrap(), &declarations);
    assert!(errors.is_empty());
    let all = resolved.iter().find(|g| g.abbreviation == "x").unwrap();
    assert_eq!(all.members, vec!["s1", "s2", "a1", "a2"]);
}

#[test]
fn abbreviation_colliding_with_part_is_recoverable() {
    let (section, _) = parse_group("Soprano [s1] = s1 s2\n", 0);
    let declarations = vec![decl("s1", PartKind::Notes), decl("s2", PartKind::Notes)];
    let (resolved, errors) = resolve_and_validate_groups(&section.unwrap(), &declarations);
    assert!(resolved.is_empty());
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message().contains("collides"));
}

#[test]
fn unknown_member_is_recoverable() {
    let (section, _) = parse_group("Soprano [s] = s1 unknown\n", 0);
    let declarations = vec![decl("s1", PartKind::Notes)];
    let (resolved, errors) = resolve_and_validate_groups(&section.unwrap(), &declarations);
    assert!(resolved.is_empty());
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message().contains("unknown"));
}

#[test]
fn mismatched_member_kinds_is_recoverable() {
    let (section, _) = parse_group("Both [b] = s1 s2\n", 0);
    let declarations = vec![
        decl("s1", PartKind::Notes),
        decl("s2", PartKind::NotesWithLyrics),
    ];
    let (resolved, errors) = resolve_and_validate_groups(&section.unwrap(), &declarations);
    assert!(resolved.is_empty());
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message().contains("same part kind"));
}
