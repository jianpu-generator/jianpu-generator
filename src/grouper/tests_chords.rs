use super::*;
use crate::ast::parsed::PartKind;

#[test]
fn chord_invalid_token_is_recoverable() {
    use crate::error::{Diagnostic, RecoverableErrorKind};
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nChords = chords\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Chords] @ 0 0 0\n[Melody] 1 2 3 4\n",
    ));
    assert!(score.measures[0].diagnostics.iter().any(|d| matches!(
        d,
        Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::ChordExpectedDegreeDigit { .. })
    )));
    assert!(!score.measures.is_empty());
}

#[test]
fn chord_expected_degree_digit_is_recoverable() {
    use crate::error::{Diagnostic, RecoverableErrorKind};
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nChords = chords\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Chords] 8 2 3 4\n[Melody] 1 2 3 4\n",
    ));
    assert!(score.measures[0].diagnostics.iter().any(|d| matches!(
        d,
        Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::ChordExpectedDegreeDigit { .. })
    )));
    let chord_row = score.measures[0]
        .parts
        .iter()
        .find_map(|row| match row {
            crate::ast::grouped::PartRow::Timed(part) if part.kind == PartKind::Chords => {
                Some(part)
            }
            crate::ast::grouped::PartRow::Timed(_) => None,
        })
        .expect("chord part");
    assert_eq!(chord_row.notes.events.len(), 3);
}

#[test]
fn chord_unknown_suffix_is_recoverable() {
    use crate::error::{Diagnostic, WarningKind};
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nChords = chords\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Chords] 1z 2 3 4\n[Melody] 1 2 3 4\n",
    ));
    assert!(score.measures[0]
        .diagnostics
        .iter()
        .any(|d| matches!(d, Diagnostic::Warning(w) if w.kind == WarningKind::ChordUnknownSuffix)));
}

#[test]
fn chord_invalid_bass_is_recoverable() {
    use crate::error::{Diagnostic, WarningKind};
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nChords = chords\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Chords] 1/X 2 3 4\n[Melody] 1 2 3 4\n",
    ));
    assert!(score.measures[0]
        .diagnostics
        .iter()
        .any(|d| matches!(d, Diagnostic::Warning(w) if w.kind == WarningKind::ChordInvalidBass)));
}

#[test]
fn chord_bass_unexpected_char_is_recoverable() {
    use crate::error::{Diagnostic, WarningKind};
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nChords = chords\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Chords] 1/5x 2 3 4\n[Melody] 1 2 3 4\n",
    ));
    assert!(score.measures[0].diagnostics.iter().any(
        |d| matches!(d, Diagnostic::Warning(w) if w.kind == WarningKind::ChordBassUnexpectedChar)
    ));
}

#[test]
fn chord_bass_trailing_chars_is_recoverable() {
    use crate::error::{Diagnostic, WarningKind};
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nChords = chords\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Chords] 1/5bb 2 3 4\n[Melody] 1 2 3 4\n",
    ));
    assert!(score.measures[0].diagnostics.iter().any(
        |d| matches!(d, Diagnostic::Warning(w) if w.kind == WarningKind::ChordBassTrailingChars)
    ));
}

#[test]
fn chord_extension_no_preceding_event_is_recoverable() {
    use crate::error::{Diagnostic, RecoverableErrorKind};
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nc = chords\nn = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[c] - 1 - -\n[n] 1 2 3 4\n",
    ));
    assert!(
        score.measures[0].diagnostics.iter().any(|d| matches!(
            d,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::ExtensionNoPrecedingEvent { chord_track: true, .. })
        )),
        "expected ExtensionNoPrecedingEvent error on measure 0"
    );
    assert_eq!(
        score.measures.len(),
        1,
        "render should continue past the error"
    );
}

#[test]
fn chord_part_produces_one_chord_event_per_measure() {
    use crate::ast::grouped::PartRow;
    let input = "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nchord = chords\nMelody = notes\n\n# score\ntime=4/4 key=C4 bpm=120\n[chord] 1 - - -\n[Melody] 1---\n";
    let doc = parser::parse(input, "test.jianpu", &[]).unwrap();
    let score = group(doc).unwrap();
    let measure = &score.measures[0];
    let chord_row = measure
        .parts
        .iter()
        .find(|r| {
            matches!(
                r,
                PartRow::Timed(p) if p.kind == PartKind::Chords
            )
        })
        .unwrap();
    let slice = chord_row.slice();
    assert_eq!(slice.notes.events.len(), 1);
    match &slice.notes.events[0] {
        NoteEvent::Chord(c) => {
            assert_eq!(c.duration, 16); // 4 tokens * 4 quarter-beats
        }
        NoteEvent::Note(_) | NoteEvent::Rest(_) | NoteEvent::Percussion(_) => {
            panic!("expected Chord event")
        }
    }
}
