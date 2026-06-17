use super::*;
use crate::ast::parsed::{Accidental, ParsedTrack};

use super::test_helpers::{chord_track, decl, notes_track, parse};

#[test]
fn overfull_measure_is_recoverable() {
    // Beat overflow is recoverable: parse succeeds, overflow note is trimmed,
    // and the error is recorded in per_measure_beat_errors.
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4 5\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).expect("overfull measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    assert_eq!(track.per_measure_beat_errors.len(), 1);
    let error = track.per_measure_beat_errors[0]
        .as_ref()
        .expect("overflow error must be recorded");
    assert!(
        error.message.contains("beat overflow"),
        "error message should mention beat overflow, got: {}",
        error.message
    );
}

#[test]
fn overfull_measure_error_span_covers_notes_line() {
    // The overflow error span should cover the whole notes line, not just the offending note.
    // "1 2 3 4 5" starts at byte 26 and ends at byte 35.
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4 5\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let ParsedTrack::Timed(track) = &tracks[0];
    let error = track.per_measure_beat_errors[0].as_ref().unwrap();
    assert_eq!(
        error.span.start, 26,
        "span must start at the beginning of the notes line"
    );
    assert_eq!(
        error.span.end, 35,
        "span must end at the end of the notes line"
    );
}

#[test]
fn overfull_measure_with_slurred_overflow_note_error_span_covers_notes_line() {
    // When the overflowing note carries a slur start '(', the error span must still
    // cover the whole notes line — not just the note token or its bare pitch.
    // "(time=4/4 key=C4 bpm=120)\n" = 26 bytes; "1 2 3 0_ (4" ends at byte 37.
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 0_ (4\n\n5) 6 7 0\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let ParsedTrack::Timed(track) = &tracks[0];
    let error = track.per_measure_beat_errors[0].as_ref().unwrap();
    assert_eq!(
        error.span.start, 26,
        "span must start at the beginning of the notes line"
    );
    assert_eq!(
        error.span.end, 37,
        "span must end at the end of the notes line"
    );
}

#[test]
fn implicit_trailing_extensions_match_explicit() {
    let declarations = vec![decl("", PartKind::Notes)];
    let explicit = "(time=4/4 key=C4 bpm=120)\n1---\n";
    let implicit = "(time=4/4 key=C4 bpm=120)\n1\n";
    let explicit_parsed = parse(explicit, 0, &declarations).unwrap();
    let implicit_parsed = parse(implicit, 0, &declarations).unwrap();
    let explicit_track = notes_track(&explicit_parsed, "");
    let implicit_track = notes_track(&implicit_parsed, "");
    assert_eq!(
        explicit_track.score.events.len(),
        implicit_track.score.events.len()
    );
    for (a, b) in explicit_track
        .score
        .events
        .iter()
        .zip(implicit_track.score.events.iter())
    {
        assert_eq!(
            std::mem::discriminant(&a.value),
            std::mem::discriminant(&b.value)
        );
    }
}

#[test]
fn implicit_trailing_extensions_after_partial_fill() {
    let declarations = vec![decl("", PartKind::Notes)];
    let explicit = "(time=4/4 key=C4 bpm=120)\n1 2 3-\n";
    let implicit = "(time=4/4 key=C4 bpm=120)\n1 2 3\n";
    let explicit_parsed = parse(explicit, 0, &declarations).unwrap();
    let implicit_parsed = parse(implicit, 0, &declarations).unwrap();
    let explicit_track = notes_track(&explicit_parsed, "");
    let implicit_track = notes_track(&implicit_parsed, "");
    assert_eq!(
        explicit_track.score.events.len(),
        implicit_track.score.events.len()
    );
}

fn timed_cluster_duration(
    events: &[crate::error::Spanned<crate::ast::parsed::ScoreEvent>],
    start: usize,
) -> u32 {
    use crate::ast::parsed::ScoreEvent;
    let Some(event) = events.get(start) else {
        return 0;
    };
    let mut duration = match &event.value {
        ScoreEvent::Chord(c) => c.duration,
        ScoreEvent::Rest(r) => r.duration,
        _ => return 0,
    };
    let mut index = start + 1;
    while let Some(event) = events.get(index) {
        if matches!(event.value, ScoreEvent::Extension) {
            duration += 4;
            index += 1;
        } else {
            break;
        }
    }
    duration
}

fn chord_event_duration(tracks: &[ParsedTrack], abbrev: &str) -> u32 {
    use crate::ast::parsed::ScoreEvent;
    let events = &chord_track(tracks, abbrev).score.events;
    let start = events
        .iter()
        .position(|e| matches!(e.value, ScoreEvent::Chord(_) | ScoreEvent::Rest(_)))
        .expect("expected a chord or rest event");
    timed_cluster_duration(events, start)
}

fn last_chord_event_duration(tracks: &[ParsedTrack], abbrev: &str) -> u32 {
    use crate::ast::parsed::ScoreEvent;
    let events = &chord_track(tracks, abbrev).score.events;
    let start = events
        .iter()
        .rposition(|e| matches!(e.value, ScoreEvent::Chord(_) | ScoreEvent::Rest(_)))
        .expect("expected a chord or rest event");
    timed_cluster_duration(events, start)
}

#[test]
fn implicit_trailing_chord_extensions_match_explicit() {
    let declarations = vec![
        decl("chord", PartKind::Chord),
        decl("Melody", PartKind::Notes),
    ];
    let explicit = "(time=4/4 key=C4 bpm=120)\n1 - - -\n1\n";
    let implicit = "(time=4/4 key=C4 bpm=120)\n1\n1\n";
    let explicit_parsed = parse(explicit, 0, &declarations).unwrap();
    let implicit_parsed = parse(implicit, 0, &declarations).unwrap();
    assert_eq!(chord_event_duration(&explicit_parsed, "chord"), 16);
    assert_eq!(chord_event_duration(&implicit_parsed, "chord"), 16);
}

#[test]
fn implicit_trailing_chord_extensions_after_partial_fill() {
    let declarations = vec![
        decl("chord", PartKind::Chord),
        decl("Melody", PartKind::Notes),
    ];
    let explicit = "(time=4/4 key=C4 bpm=120)\n1m 2m - -\n1 2 3 4\n";
    let implicit = "(time=4/4 key=C4 bpm=120)\n1m 2m\n1 2 3 4\n";
    let explicit_parsed = parse(explicit, 0, &declarations).unwrap();
    let implicit_parsed = parse(implicit, 0, &declarations).unwrap();
    assert_eq!(last_chord_event_duration(&explicit_parsed, "chord"), 12);
    assert_eq!(last_chord_event_duration(&implicit_parsed, "chord"), 12);
}

#[test]
fn incomplete_measure_is_recoverable() {
    // 4_ is an eighth note (2 beats) in 4/4 (16 beats); cannot be padded by extending the last
    // note. Must not abort parsing — measure should carry a recoverable beat error.
    let content = "(time=4/4 key=C4 bpm=120)\n4_\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks =
        parse(content, 0, &declarations).expect("incomplete measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    assert_eq!(track.per_measure_beat_errors.len(), 1);
    assert!(
        track.per_measure_beat_errors[0].is_some(),
        "incomplete measure must record a recoverable error"
    );
}

#[test]
fn underfull_measure_that_cannot_be_padded_records_recoverable_error() {
    let content = "(time=4/4 key=C4 bpm=120)\n4_\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks =
        parse(content, 0, &declarations).expect("underfull measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    let error = track.per_measure_beat_errors[0]
        .as_ref()
        .expect("recoverable error must be recorded");
    assert!(
        error.message.contains("incomplete measure"),
        "error should mention incomplete measure, got: {}",
        error.message
    );
}

#[test]
fn underfull_measure_with_short_trailing_notes_records_recoverable_error() {
    let content = "(time=4/4 key=C4 bpm=120)\n3_ 1_ 1 0_ 1= 1=\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks =
        parse(content, 0, &declarations).expect("underfull measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    assert!(
        track.per_measure_beat_errors[0].is_some(),
        "recoverable error must be recorded for the incomplete measure"
    );
}

#[test]
fn underfull_measure_recoverable_error_span_covers_notes_line() {
    // The recoverable error span should cover the whole notes line, not just the last note.
    // "4 4 4 4_" starts at byte 26 and ends at byte 34.
    let content = "(time=4/4 key=C4 bpm=120)\n4 4 4 4_\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks =
        parse(content, 0, &declarations).expect("underfull measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    let error = track.per_measure_beat_errors[0].as_ref().unwrap();
    assert_eq!(
        error.span.start, 26,
        "span must start at the beginning of the notes line"
    );
    assert_eq!(
        error.span.end, 34,
        "span must end at the end of the notes line"
    );
}

#[test]
fn underfull_measure_in_second_bar_recoverable_error_span_covers_notes_line() {
    // "4 4 4 4_" in the second bar starts at byte 35 and ends at byte 43.
    let content = "(time=4/4 key=C4 bpm=120)\n5 5 5 5\n\n4 4 4 4_\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks =
        parse(content, 0, &declarations).expect("underfull measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    let error = track.per_measure_beat_errors[1].as_ref().unwrap();
    assert_eq!(
        error.span.start, 35,
        "span must start at the beginning of the second bar's notes line"
    );
    assert_eq!(error.span.end, 43);
}

#[test]
fn directive_row_is_optional() {
    let content = concat!("(time=4/4 key=C4 bpm=120)\n1 2 3 4\n", "\n", "5 6 7 1\n",);
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(notes_track(&tracks, "").score.events.len(), 11);
}

#[test]
fn time_sig_change_updates_beat_tracking() {
    let content = concat!(
        "(time=4/4 key=C4 bpm=120)\n1 2 3 4\n",
        "\n",
        "(time=3/4)\n1 2 3\n",
    );
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert!(!notes_track(&tracks, "").score.events.is_empty());
}

#[test]
fn rejects_unknown_directive() {
    let content = "(foo=bar)\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    assert!(parse(content, 0, &declarations).is_err());
}

#[test]
fn key_directive_parses_flat() {
    let content = "(time=4/4 key=Bb4 bpm=120)\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let key_event = notes_track(&tracks, "")
        .score
        .events
        .iter()
        .find(|e| matches!(&e.value, ScoreEvent::KeyChange(_)));
    assert!(key_event.is_some());
    if let ScoreEvent::KeyChange(kc) = &key_event.unwrap().value {
        assert_eq!(kc.note.accidental, Accidental::Flat);
    }
}

#[test]
fn label_directive_parsed() {
    let content = "(time=4/4 key=C4 bpm=120 label=\"Verse 1\")\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let label_event = notes_track(&tracks, "")
        .score
        .events
        .iter()
        .find(|e| matches!(&e.value, ScoreEvent::LabelChange(_)));
    assert!(label_event.is_some(), "expected a LabelChange event");
    if let ScoreEvent::LabelChange(text) = &label_event.unwrap().value {
        assert_eq!(text, "Verse 1");
    }
}

#[test]
fn label_directive_rejects_unclosed_quote() {
    let content = "(label=\"Verse 1)\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    assert!(parse(content, 0, &declarations).is_err());
}

#[test]
fn label_directive_rejects_empty_label() {
    let content = "(label=\"\")\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    assert!(parse(content, 0, &declarations).is_err());
}

#[test]
fn notes_ditto_resolves_in_full_parse() {
    let content = concat!("(time=4/4 key=C4 bpm=120)\n", "1 2 3 4\n", "\"\n",);
    let declarations = vec![decl("S", PartKind::Notes), decl("A", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(notes_track(&tracks, "S").score.events.len(), 7);
    assert_eq!(
        notes_track(&tracks, "A").score.events.len(),
        4,
        "Alto should have 4 note events after ditto resolution"
    );
}

#[test]
fn lyrics_ditto_resolves_in_full_parse() {
    let content = concat!(
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "do re mi fa\n",
        "\"\n",
        "\"\n",
    );
    let declarations = vec![
        decl("S", PartKind::NotesWithLyrics),
        decl("A", PartKind::NotesWithLyrics),
    ];
    let tracks = parse(content, 0, &declarations).unwrap();
    let s_lyrics = notes_track(&tracks, "S").lyrics.as_ref().unwrap();
    let a_lyrics = notes_track(&tracks, "A").lyrics.as_ref().unwrap();
    assert_eq!(s_lyrics.measure_syllables[0].len(), 4);
    assert_eq!(a_lyrics.measure_syllables[0].len(), 4);
    assert_eq!(
        s_lyrics.measure_syllables[0][0].text,
        a_lyrics.measure_syllables[0][0].text
    );
}

#[test]
fn key_directive_parses_sharp() {
    let content = "(time=4/4 key=F#3 bpm=120)\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let key_event = notes_track(&tracks, "")
        .score
        .events
        .iter()
        .find(|e| matches!(&e.value, ScoreEvent::KeyChange(_)));
    if let ScoreEvent::KeyChange(kc) = &key_event.unwrap().value {
        assert_eq!(kc.note.accidental, Accidental::Sharp);
    }
}
