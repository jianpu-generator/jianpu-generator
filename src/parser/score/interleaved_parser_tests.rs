use super::*;
use crate::ast::parsed::{Accidental, JianPuPitch, ParsedChordNote, ScoreEvent, TriadQuality};

use super::test_helpers::{chord_track, decl, notes_track, parse, total_lyrics_syllables};

#[test]
fn chord_line_parses_spaced_slur_group() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Chord = chord\n",
        "Melody = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "(1 - 6m -)\n",
        "1 1 5 5\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let chord_events: Vec<_> = chord_track(&doc.tracks, "Chord")
        .score
        .events
        .iter()
        .filter(|e| matches!(e.value, ScoreEvent::Chord(_)))
        .collect();
    assert_eq!(chord_events.len(), 2, "expected chord 1 and 6m in group");
}

#[test]
fn chord_column_events_are_parsed() {
    let declarations = vec![decl("main", PartKind::Chord), decl("main", PartKind::Notes)];
    let content = "(time=4/4 key=C4 bpm=120)\n1 - - -\n1---\n";
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(tracks.len(), 2);
    let chord = chord_track(&tracks, "main");
    let events: Vec<_> = chord.score.events.iter().map(|e| &e.value).collect();
    assert_eq!(
        events[0],
        &ScoreEvent::Chord(ParsedChordNote {
            degree: JianPuPitch::One,
            accidental: Accidental::Natural,
            triad: TriadQuality::Major,
            extension: None,
            bass: None,
            duration: 4,
            tie: false,
            group_membership: 0,
            group_continuation: 0,
            dotted: false,
            slur_group_close_at_duration: None,
        })
    );
    assert!(matches!(events[1], ScoreEvent::Extension));
    assert_eq!(notes_track(&tracks, "main").score.events.len(), 4);
}

#[test]
fn single_unnamed_part_no_lyrics() {
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(tracks.len(), 1);
    let notes = notes_track(&tracks, "");
    assert!(notes.lyrics.is_none());
    assert_eq!(notes.score.events.len(), 7);
}

#[test]
fn single_part_with_lyrics() {
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4\ndo re mi fa\n";
    let declarations = vec![decl("", PartKind::NotesWithLyrics)];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(tracks.len(), 1);
    let notes = notes_track(&tracks, "");
    assert!(notes.lyrics.is_some());
    assert_eq!(notes.lyrics.as_ref().unwrap().measure_syllables[0].len(), 4);
}

#[test]
fn two_parts_two_bars() {
    let content = concat!(
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
        "\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
    );
    let declarations = vec![
        decl("Soprano", PartKind::Notes),
        decl("Alto", PartKind::Notes),
    ];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(notes_track(&tracks, "Soprano").score.events.len(), 11);
    assert_eq!(notes_track(&tracks, "Alto").score.events.len(), 8);
}

#[test]
fn too_many_lines_in_group_is_recoverable() {
    // Extra data line beyond what the declared parts expect must not abort parsing.
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c d\nextra line\n";
    let declarations = vec![decl("", PartKind::NotesWithLyrics)];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "extra data lines must not abort parsing"
    );
}

#[test]
fn underscore_on_lyrics_line_means_no_lyrics_for_that_bar() {
    let content = concat!(
        "(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c d\n",
        "\n",
        "5 6 7 1\n",
        "_\n",
    );
    let declarations = vec![decl("", PartKind::NotesWithLyrics)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let lyrics = notes_track(&tracks, "").lyrics.as_ref().unwrap();
    assert_eq!(lyrics.measure_syllables.len(), 2);
    assert_eq!(lyrics.measure_syllables[0].len(), 4);
    assert!(lyrics.measure_syllables[1].is_empty());
}

#[test]
fn allows_too_few_lyrics_syllables_for_notes() {
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c\n";
    let declarations = vec![decl("", PartKind::NotesWithLyrics)];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(
        notes_track(&tracks, "")
            .lyrics
            .as_ref()
            .unwrap()
            .measure_syllables[0]
            .len(),
        3
    );
}

#[test]
fn accepts_too_many_lyrics_syllables_for_notes() {
    // Overflow is recoverable — parsing succeeds and the grouper attaches an error to the measure.
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c d e\n";
    let declarations = vec![decl("", PartKind::NotesWithLyrics)];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "too many syllables must not abort parsing"
    );
}

#[test]
fn cross_measure_paren_group_parses() {
    let content = concat!("(time=4/4 key=C4 bpm=120)\n", "111(1\n", "\n", "2)345\n",);
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let notes = notes_track(&tracks, "");
    let note_events: Vec<_> = notes
        .score
        .events
        .iter()
        .filter_map(|e| match &e.value {
            ScoreEvent::Note(n) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(note_events.len(), 8);
    assert!(note_events[3].tie);
    assert!(!note_events[4].tie);
}

#[test]
fn rejects_unclosed_paren_group_at_eof() {
    let content = "(time=4/4 key=C4 bpm=120)\n111(1\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let err = parse(content, 0, &declarations).unwrap_err();
    assert!(err.message().contains("unclosed '(' group"));
}

#[test]
fn tied_notes_share_one_lyric_slot_in_bar() {
    let content = "(time=4/4 key=C4 bpm=120)\n(33) 1 2\na b c\n";
    let declarations = vec![decl("", PartKind::NotesWithLyrics)];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(
        notes_track(&tracks, "")
            .lyrics
            .as_ref()
            .unwrap()
            .measure_syllables[0]
            .len(),
        3
    );
}

#[test]
fn cross_measure_tie_continuation_needs_fewer_lyrics() {
    let content = concat!(
        "(time=4/4 key=C4 bpm=120)\n0 0 0 (3\na\n",
        "\n",
        "3) 0 0 0\n",
        "_\n",
    );
    let declarations = vec![decl("", PartKind::NotesWithLyrics)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let lyrics = notes_track(&tracks, "").lyrics.as_ref().unwrap();
    assert_eq!(lyrics.measure_syllables.len(), 2);
    assert_eq!(lyrics.measure_syllables[0].len(), 1);
    assert!(lyrics.measure_syllables[1].is_empty());
}

#[test]
fn spaced_open_group_cross_measure_lyrics() {
    let content = concat!(
        "(time=4/4 key=C4 bpm=120)\n",
        "1 - 6m -\n",
        "(6- 7-\n",
        "慈 -\n",
        "\n",
        "1 - 6m -\n",
        "7) 1 2 3\n",
        "光 - 光\n",
    );
    let declarations = vec![
        decl("main", PartKind::Chord),
        decl("S1", PartKind::NotesWithLyrics),
    ];
    let tracks = parse(content, 0, &declarations).unwrap();
    let s1 = notes_track(&tracks, "S1");
    assert_eq!(total_lyrics_syllables(s1), 5);
}

#[test]
fn omitted_trailing_lyrics_without_precedent_is_recoverable() {
    // Measure 2 has no lyrics and no preceding lyrics in the same group to ditto from.
    // Parsing must succeed; the missing lyrics become an empty (no-lyrics) measure.
    let content = concat!(
        "(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c d\n",
        "\n",
        "5 6 7 1\n",
    );
    let declarations = vec![decl("", PartKind::NotesWithLyrics)];
    let tracks = parse(content, 0, &declarations).expect("missing lyrics must not abort parsing");
    let crate::ast::parsed::ParsedTrack::Timed(track) = &tracks[0];
    let lyrics = track.lyrics.as_ref().expect("track should have lyrics");
    assert_eq!(lyrics.measure_syllables.len(), 2);
    assert_eq!(
        lyrics.measure_syllables[1].len(),
        0,
        "measure 2 should have no syllables (treated as no lyrics)"
    );
}

#[test]
fn omitted_notes_row_is_recoverable() {
    // Part kind "lyrics notes" puts the lyrics row before the notes row in the score.
    // The score has only a lyrics row; the missing notes row must not abort parsing.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "A = lyrics notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "la la\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu")
        .expect("missing notes row must not abort parsing");
    let crate::ast::parsed::ParsedTrack::Timed(track) = &doc.tracks[0];
    let note_events: Vec<_> = track
        .score
        .events
        .iter()
        .filter(|e| {
            matches!(
                e.value,
                ScoreEvent::Note(_) | ScoreEvent::Rest(_) | ScoreEvent::Extension
            )
        })
        .collect();
    assert_eq!(
        note_events.len(),
        0,
        "measure with missing notes row should have no note events"
    );
}

#[test]
fn omitted_chord_row_is_recoverable() {
    // Part kind "notes chord" puts the notes row before the chord row in the score.
    // With only a notes row provided, the missing chord row is now a recoverable error.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "A = notes chord\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
    );
    assert!(
        crate::parser::parse(input, "test.jianpu").is_ok(),
        "missing chord row must not abort parsing"
    );
}

#[test]
fn partial_measure_still_needs_ditto_before_diverging_middle_columns() {
    let content = concat!(
        "(time=4/4 key=C4 bpm=120)\n",
        "1 - 6m -\n",
        "6. 6= 6= 6_ 5_ 3= (2_=2_)\n",
        "a b c d e f g\n",
        "4. 4= 4= 4_ 3_ 1= (2_=2_)\n",
        "\"\n",
        "6- 5-\n",
        "alto lyrics\n",
    );
    let declarations = vec![
        decl("main", PartKind::Chord),
        decl("A1", PartKind::NotesWithLyrics),
        decl("A2", PartKind::NotesWithLyrics),
        decl("S1", PartKind::NotesWithLyrics),
        decl("S2", PartKind::NotesWithLyrics),
    ];
    let tracks = parse(content, 0, &declarations).unwrap();
    let s1 = notes_track(&tracks, "S1");
    assert_eq!(
        s1.lyrics.as_ref().unwrap().measure_syllables[0][0].text,
        "alto"
    );
}

#[test]
fn implicit_trailing_ditto_matches_explicit_ditto() {
    let explicit = concat!(
        "(time=4/4 key=C4 bpm=120)\n",
        "1 - - -\n",
        "1 2 3 4\n",
        "do re mi fa\n",
        "\"\n",
        "\"\n",
    );
    let implicit = concat!(
        "(time=4/4 key=C4 bpm=120)\n",
        "1 - - -\n",
        "1 2 3 4\n",
        "do re mi fa\n",
    );
    let declarations = vec![
        decl("main", PartKind::Chord),
        decl("A", PartKind::NotesWithLyrics),
        decl("B", PartKind::NotesWithLyrics),
    ];
    let explicit_tracks = parse(explicit, 0, &declarations).unwrap();
    let implicit_tracks = parse(implicit, 0, &declarations).unwrap();
    let explicit_a = notes_track(&explicit_tracks, "A");
    let implicit_a = notes_track(&implicit_tracks, "A");
    let explicit_b = notes_track(&explicit_tracks, "B");
    let implicit_b = notes_track(&implicit_tracks, "B");
    assert_eq!(explicit_a.score.events.len(), implicit_a.score.events.len());
    assert_eq!(explicit_b.score.events.len(), implicit_b.score.events.len());
    assert_eq!(
        explicit_a.lyrics.as_ref().unwrap().measure_syllables,
        implicit_a.lyrics.as_ref().unwrap().measure_syllables
    );
    assert_eq!(
        explicit_b.lyrics.as_ref().unwrap().measure_syllables,
        implicit_b.lyrics.as_ref().unwrap().measure_syllables
    );
}

#[test]
fn measure_no_data_lines_is_recoverable() {
    // A directive-only group (no note lines) must not abort parsing.
    let content = "(time=4/4 key=C4 bpm=120)\n\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "measure with no data lines must not abort parsing"
    );
}

#[test]
fn measure_too_many_lines_is_recoverable() {
    // Single-part declaration but two data lines — extra line must be silently ignored.
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4\n5 6 7 1\n";
    let declarations = vec![decl("", PartKind::Notes)];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "measure with too many lines must not abort parsing"
    );
}

#[test]
fn measure_missing_chord_line_is_recoverable() {
    // [notes, chord] parts but only the notes line is provided.
    // The chord role cannot be filled implicitly, so this previously errored.
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4\n";
    let declarations = vec![
        decl("Melody", PartKind::Notes),
        decl("chord", PartKind::Chord),
    ];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "measure missing a chord role line must not abort parsing"
    );
}

#[test]
fn no_notes_track_is_recoverable() {
    // A parts declaration with only a chord track (no notes track) must not abort parsing.
    let content = "(time=4/4 key=C4 bpm=120)\n1 2 3 4\n";
    let declarations = vec![decl("Chord", PartKind::Chord)];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "parts declaration with no notes track must not abort parsing"
    );
}

#[test]
fn lex_unexpected_char_in_notes_line_is_recoverable() {
    // `@` at a word boundary in a notes line triggers LexUnexpectedChar — must not abort parsing.
    let content = "(time=4/4 key=C4 bpm=120)\n1 @ 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations)
        .expect("LexUnexpectedChar in notes line must not abort parsing");
    let track = notes_track(&tracks, "");
    assert_eq!(track.per_measure_lex_errors.len(), 1);
    assert!(
        track.per_measure_lex_errors[0].is_some(),
        "lex error must be recorded for the failing measure"
    );
}
