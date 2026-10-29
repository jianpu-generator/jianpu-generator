use crate::ast::grouped::Score;
use crate::error::{Diagnostic, Warning, WarningKind};
use crate::grouper::group;
use crate::parser;

fn parse_and_group(parts: &str, score: &str) -> Score {
    let input = format!("# parts\n{parts}\n\n# score\ntime=4/4 key=C4 bpm=120\n{score}\n");
    let doc = parser::parse(&input, "test.jianpu", &[]).unwrap();
    group(doc).unwrap()
}

fn range_warnings(score: &Score) -> Vec<&Warning> {
    score
        .measures
        .iter()
        .flat_map(|measure| &measure.diagnostics)
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::Warning(warning) if warning.kind == WarningKind::NoteOutOfRange => {
                Some(warning)
            }
            _ => None,
        })
        .collect()
}

fn range_messages(score: &Score) -> Vec<String> {
    range_warnings(score)
        .into_iter()
        .map(|warning| warning.message.clone())
        .collect()
}

#[test]
fn part_without_instrument_gets_no_warnings() {
    let score = parse_and_group("Violin [V] = notes", "[V] 1,, 2,, 3,, 4,,");
    assert!(range_warnings(&score).is_empty());
}

#[test]
fn notes_inside_the_range_get_no_warnings() {
    let score = parse_and_group("Violin [V] = notes violin", "[V] 5, 1 3 5");
    assert!(range_warnings(&score).is_empty());
}

#[test]
fn each_note_below_the_range_gets_its_own_warning() {
    let score = parse_and_group("Violin [V] = notes violin", "[V] 1 4, 5, 3,");
    assert_eq!(
        range_messages(&score),
        vec![
            "V: note sounds as F3, below the violin's lowest note G3",
            "V: note sounds as E3, below the violin's lowest note G3",
        ]
    );
}

#[test]
fn warning_points_at_the_note() {
    let input = "[V] 1 4, 5 5";
    let score = parse_and_group("Violin [V] = notes violin", input);
    let warnings = range_warnings(&score);
    let [warning] = warnings.as_slice() else {
        panic!("expected one warning, got {}", warnings.len());
    };
    let source = format!(
        "# parts\nViolin [V] = notes violin\n\n# score\ntime=4/4 key=C4 bpm=120\n{input}\n"
    );
    assert_eq!(&source[warning.span.start..warning.span.end], "4,");
}

#[test]
fn note_above_the_range_names_the_highest_note() {
    let score = parse_and_group("Soprano [S] = notes soprano", "[S] 1 1' 6' 1");
    assert_eq!(
        range_messages(&score),
        vec!["S: note sounds as A5, above the soprano's highest note G5"]
    );
}

#[test]
fn octave_offset_shifts_the_sounding_pitch() {
    let score = parse_and_group("Cello [C] = notes cello -1", "[C] 1 1 1 1'");
    assert!(range_warnings(&score).is_empty());

    let score = parse_and_group("Cello [C] = notes cello -2", "[C] 1, 1 1 1");
    assert_eq!(
        range_messages(&score),
        vec!["C: note sounds as C1, below the cello's lowest note C2"]
    );
}

#[test]
fn key_shifts_the_sounding_pitch() {
    let input = "# parts\nAlto [A] = notes alto\n\n# score\ntime=4/4 key=A3 bpm=120\n[A] 1 3 5 3\n";
    let score = group(parser::parse(input, "test.jianpu", &[]).unwrap()).unwrap();
    assert!(range_warnings(&score).is_empty());

    let input =
        "# parts\nAlto [A] = notes alto\n\n# score\ntime=4/4 key=Eb4 bpm=120\n[A] 1 3 5 1'\n";
    let score = group(parser::parse(input, "test.jianpu", &[]).unwrap()).unwrap();
    assert_eq!(
        range_messages(&score),
        vec!["A: note sounds as Eb5, above the alto's highest note D5"]
    );
}

#[test]
fn follow_part_inherits_its_targets_instrument() {
    let score = parse_and_group(
        "Violin [V] = notes violin\nViolin 2 [V2] = follow[V] -1",
        "[V] 1 2 3 4",
    );
    assert_eq!(range_messages(&score).len(), 4);
    assert!(range_messages(&score)
        .iter()
        .all(|message| message.starts_with("V2: ")));
}

#[test]
fn follow_part_can_override_its_targets_instrument() {
    let score = parse_and_group(
        "Violin [V] = notes violin\nCello [C] = follow[V] cello -1",
        "[V] 1 2 3 4",
    );
    assert!(range_warnings(&score).is_empty());
}

#[test]
fn chords_part_cannot_name_an_instrument() {
    let input = "# parts\nChords [C] = chords guitar\n\n# score\n[C] 1 - - -\n";
    let doc = parser::parse(input, "test.jianpu", &[]).unwrap();
    assert!(doc
        .parts_parse_errors
        .iter()
        .any(|error| error.message().contains("only notes parts")));
}

#[test]
fn unknown_instrument_is_reported_and_the_part_kept() {
    let input = "# parts\nViolin [V] = notes viola\n\n# score\n[V] 1 2 3 4\n";
    let doc = parser::parse(input, "test.jianpu", &[]).unwrap();
    assert!(doc
        .parts_parse_errors
        .iter()
        .any(|error| error.message().contains("unknown instrument \"viola\"")));
    assert_eq!(doc.declarations.len(), 1);
}
