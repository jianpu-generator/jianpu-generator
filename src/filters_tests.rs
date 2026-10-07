use super::*;
use crate::part_info::PartInfo;

fn part(abbreviation: &str, display_name: &str) -> PartInfo {
    PartInfo {
        abbreviation: abbreviation.to_string(),
        display_name: display_name.to_string(),
        sounds: true,
    }
}

fn lyric_part(abbreviation: &str, display_name: &str) -> PartInfo {
    PartInfo {
        sounds: false,
        ..part(abbreviation, display_name)
    }
}

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

fn melody_verse_and_chords() -> Vec<PartInfo> {
    vec![
        part("M", "Melody"),
        lyric_part("v1", "Verse 1"),
        part("C", "Chords"),
    ]
}

#[test]
fn default_state_shows_everything() {
    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &PartToggleState::default());

    assert_eq!(resolved, ResolvedPartVisibility::default());
}

#[test]
fn hiding_a_part_removes_only_that_part() {
    let state = PartToggleState {
        hidden_parts: names(&["M"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, Some(names(&["v1", "C"])));
    assert_eq!(resolved.sounding_tracks, Some(names(&["C"])));
    assert_eq!(resolved.sounding_display_names, Some(names(&["Chords"])));
}

#[test]
fn hiding_a_lyric_part_is_the_same_as_hiding_any_other_part() {
    let state = PartToggleState {
        hidden_parts: names(&["v1"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, Some(names(&["M", "C"])));
}

#[test]
fn a_lyric_part_never_counts_as_sounding() {
    let hidden = PartToggleState {
        hidden_parts: names(&["v1"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &hidden);

    assert_eq!(resolved.sounding_tracks, None);
    assert_eq!(resolved.sounding_display_names, None);
}

#[test]
fn soloing_a_part_shows_only_soloed_parts() {
    let state = PartToggleState {
        soloed_parts: names(&["v1", "C"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, Some(names(&["v1", "C"])));
}

#[test]
fn solo_wins_over_hide() {
    let state = PartToggleState {
        hidden_parts: names(&["C"]),
        soloed_parts: names(&["C"]),
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, Some(names(&["C"])));
}

#[test]
fn soloing_an_unknown_part_is_ignored() {
    let state = PartToggleState {
        soloed_parts: names(&["nope"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state);

    assert_eq!(resolved, ResolvedPartVisibility::default());
}
