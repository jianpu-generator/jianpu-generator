//! Tests for `[GroupAbbrev]` broadcast desugaring — filling group members'
//! slots, member overrides, and `group` provenance tagging (including a
//! group resting as a whole rather than being explicitly broadcast to).

use super::desugar_groups;
use super::tests::{decl, group};
use crate::ast::parsed::PartKind;
use crate::parser::group_parser::ResolvedGroup;

fn resolved_group(abbrev: &str, members: &[&str]) -> ResolvedGroup {
    ResolvedGroup {
        abbreviation: abbrev.to_string(),
        members: members.iter().map(|m| m.to_string()).collect(),
    }
}

#[test]
fn group_key_broadcasts_content_to_all_members() {
    let groups = vec![group(&["[s] 1 2 3 4"])];
    let declarations = vec![decl("s1", PartKind::Notes), decl("s2", PartKind::Notes)];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, _slots, errors, _refs) =
        desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(result[0][0].content, "1 2 3 4", "s1: from group broadcast");
    assert_eq!(result[0][1].content, "1 2 3 4", "s2: from group broadcast");
}

#[test]
fn member_specific_line_overrides_group_broadcast() {
    let groups = vec![group(&["[s] 1 2 3 4", "[s2] 5 6 7 0"])];
    let declarations = vec![decl("s1", PartKind::Notes), decl("s2", PartKind::Notes)];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, _slots, errors, _refs) =
        desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(result[0][0].content, "1 2 3 4", "s1: from group broadcast");
    assert_eq!(
        result[0][1].content, "5 6 7 0",
        "s2: explicit line overrides group"
    );
}

#[test]
fn group_broadcast_fills_multiple_slots_in_occurrence_order() {
    let groups = vec![group(&["[s] 1 2 3 4", "[s] la la la la"])];
    let declarations = vec![
        decl("s1", PartKind::NotesWithLyrics),
        decl("s2", PartKind::NotesWithLyrics),
    ];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, _slots, errors, _refs) =
        desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(result[0][0].content, "1 2 3 4", "s1 notes");
    assert_eq!(result[0][1].content, "la la la la", "s1 lyrics");
    assert_eq!(result[0][2].content, "1 2 3 4", "s2 notes");
    assert_eq!(result[0][3].content, "la la la la", "s2 lyrics");
}

#[test]
fn group_key_unknown_to_desugar_is_reported_as_unknown_key() {
    // Not in `resolved` (e.g. it failed group validation) → treated like any unknown key.
    let groups = vec![group(&["[s] 1 2 3 4"])];
    let declarations = vec![decl("s1", PartKind::Notes)];
    let (_, _slots, errors, _refs) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert!(errors[0].is_some(), "unresolved group key should error");
}

#[test]
fn group_broadcast_lines_are_tagged_with_group_provenance() {
    let groups = vec![group(&["[s] 1 2 3 4", "[s2] 5 6 7 0"])];
    let declarations = vec![decl("s1", PartKind::Notes), decl("s2", PartKind::Notes)];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, _slots, errors, _refs) =
        desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(
        result[0][0].group,
        Some("s".to_string()),
        "s1: unmodified broadcast line carries the group's provenance"
    );
    assert_eq!(
        result[0][1].group, None,
        "s2: explicit override line carries no group provenance"
    );
}

#[test]
fn own_direct_line_carries_no_group_provenance() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(
        result[0][0].group, None,
        "a part's own direct line was never broadcast by a group"
    );
}

#[test]
fn a_group_implicitly_resting_as_a_whole_tags_its_members_with_group_provenance() {
    // Neither s1 nor s2 is mentioned by any key line (own or a `[s]` broadcast)
    // in this measure, so both implicit-fill to a rest — but since *every*
    // member of "s" rests here, that's equivalent to an implicit `[s]`
    // broadcast of silence, and should be tagged the same way an explicit one
    // would be, so a later merge of these two rows into one (e.g. because
    // they're both hidden as resting parts) can still label the row "s"
    // instead of "s1 s2" (see `grid_layout::layout_systems::resolve_label`).
    let groups = vec![group(&["[t] 1 2 3 4"])];
    let declarations = vec![
        decl("s1", PartKind::Notes),
        decl("s2", PartKind::Notes),
        decl("t", PartKind::Notes),
    ];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, _slots, errors, _refs) =
        desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(
        result[0][0].group,
        Some("s".to_string()),
        "s1: implicitly rests along with every other member of \"s\""
    );
    assert_eq!(
        result[0][1].group,
        Some("s".to_string()),
        "s2: implicitly rests along with every other member of \"s\""
    );
    assert_eq!(
        result[0][2].group, None,
        "t: its own direct line, not a group member"
    );
}

#[test]
fn a_group_only_partially_resting_does_not_tag_the_resting_member() {
    // s1 has its own direct line; s2 is left unmentioned and implicit-fills to
    // a rest. Since s1 (a member of "s") is *not* resting, "s" as a whole
    // isn't implicitly resting here, so s2's rest must not be tagged with
    // "s"'s provenance — that would incorrectly suggest the whole group
    // shared one broadcast when only one member is actually silent.
    let groups = vec![group(&["[s1] 1 2 3 4"])];
    let declarations = vec![decl("s1", PartKind::Notes), decl("s2", PartKind::Notes)];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, _slots, errors, _refs) =
        desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(result[0][1].content, "0 0 0 0", "s2: implicit rest");
    assert_eq!(
        result[0][1].group, None,
        "s2: rests alone, not as part of a wholly-resting group"
    );
}
