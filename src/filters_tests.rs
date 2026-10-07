use super::*;
use crate::part_info::PartInfo;

fn part(abbreviation: &str, display_name: &str, has_lyrics: bool) -> PartInfo {
    PartInfo {
        abbreviation: abbreviation.to_string(),
        display_name: display_name.to_string(),
        has_lyrics,
    }
}

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

fn melody_and_chords() -> Vec<PartInfo> {
    vec![part("M", "Melody", true), part("C", "Chords", false)]
}

#[test]
fn default_state_shows_everything() {
    let resolved = resolve_part_visibility(&melody_and_chords(), &PartToggleState::default());

    assert_eq!(resolved, ResolvedPartVisibility::default());
}

#[test]
fn hiding_the_notes_row_keeps_the_lyrics_row() {
    let state = PartToggleState {
        hidden_notes: names(&["M"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, None);
    assert_eq!(resolved.sounding_tracks, Some(names(&["C"])));
    assert_eq!(resolved.sounding_display_names, Some(names(&["Chords"])));
    assert_eq!(resolved.lyrics_only_tracks, names(&["M"]));
    assert!(resolved.disabled_lyrics.is_empty());
}

#[test]
fn hiding_the_lyrics_row_keeps_the_notes_row() {
    let state = PartToggleState {
        hidden_lyrics: names(&["M"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, None);
    assert_eq!(resolved.sounding_tracks, None);
    assert!(resolved.lyrics_only_tracks.is_empty());
    assert_eq!(resolved.disabled_lyrics, names(&["M"]));
}

#[test]
fn hiding_both_rows_removes_the_part() {
    let state = PartToggleState {
        hidden_notes: names(&["M"]),
        hidden_lyrics: names(&["M"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, Some(names(&["C"])));
    assert_eq!(resolved.sounding_tracks, Some(names(&["C"])));
    assert!(resolved.lyrics_only_tracks.is_empty());
    assert!(resolved.disabled_lyrics.is_empty());
}

#[test]
fn soloing_a_notes_row_hides_every_other_row_including_its_own_lyrics() {
    let state = PartToggleState {
        soloed_notes: names(&["M"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, Some(names(&["M"])));
    assert_eq!(resolved.sounding_tracks, Some(names(&["M"])));
    assert!(resolved.lyrics_only_tracks.is_empty());
    assert_eq!(resolved.disabled_lyrics, names(&["M"]));
}

#[test]
fn soloing_only_a_lyrics_row_hides_every_notes_row() {
    let state = PartToggleState {
        soloed_lyrics: names(&["M"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, Some(names(&["M"])));
    assert_eq!(resolved.sounding_tracks, Some(Vec::new()));
    assert_eq!(resolved.lyrics_only_tracks, names(&["M"]));
    assert!(resolved.disabled_lyrics.is_empty());
}

#[test]
fn pianist_view_solos_the_chords_and_the_vocal_lyrics() {
    let state = PartToggleState {
        soloed_notes: names(&["C"]),
        soloed_lyrics: names(&["M"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_and_chords(), &state);

    assert_eq!(resolved.rendered_tracks, None);
    assert_eq!(resolved.sounding_tracks, Some(names(&["C"])));
    assert_eq!(resolved.lyrics_only_tracks, names(&["M"]));
    assert!(resolved.disabled_lyrics.is_empty());
}

#[test]
fn solo_overrides_hide_flags() {
    let state = PartToggleState {
        hidden_notes: names(&["C"]),
        soloed_notes: names(&["C"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_and_chords(), &state);

    assert_eq!(resolved.sounding_tracks, Some(names(&["C"])));
}

#[test]
fn solo_entries_for_rows_that_do_not_exist_are_ignored() {
    let state = PartToggleState {
        soloed_notes: names(&["Gone"]),
        soloed_lyrics: names(&["C"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_and_chords(), &state);

    assert_eq!(resolved, ResolvedPartVisibility::default());
}

#[test]
fn a_part_without_lyrics_never_reports_lyrics_visibility() {
    let state = PartToggleState {
        hidden_lyrics: names(&["C"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_and_chords(), &state);

    assert_eq!(resolved, ResolvedPartVisibility::default());
}
