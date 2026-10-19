//! The part stream exports called through the `Guest` impl.
use super::*;

const SCORE: &str = r"# parts
Melody [M] = notes
Words [W] = lyrics[M]
Harmony [H] = chords
Chanted [C] = lyrics[H]

# score
[M] 1 2 3 4
[W] 你 好 世 界
[H] 1 4 5 1
[C] la la la la

[M] 5 6 7 1
[W] 一 二 三 四
";

fn row(part: &StreamablePart) -> String {
    format!(
        "{} {} {:?} {:?}",
        part.abbreviation, part.display_name, part.kind, part.ineligible_reason
    )
}

fn apply(abbreviation: &str, edited_stream: &str) -> PartStreamApplyResult {
    Component::apply_part_stream(
        SCORE.to_string(),
        abbreviation.to_string(),
        edited_stream.to_string(),
    )
}

#[test]
fn lists_eligible_and_ineligible_parts() {
    let rows: Vec<String> = Component::list_streamable_parts(SCORE.to_string())
        .iter()
        .map(row)
        .collect();
    assert_eq!(
        rows,
        vec![
            "M Melody StreamablePartKind::Notes None",
            "W Words StreamablePartKind::Lyrics None",
            "H Harmony StreamablePartKind::Chords None",
            "C Chanted StreamablePartKind::Lyrics \
             Some(\"lyrics under a chords part cannot be edited as a stream\")",
        ]
    );
}

#[test]
fn extracts_a_stream_with_byte_offset_measure_starts() {
    let ExtractPartStreamResponse::Ok(stream) =
        Component::extract_part_stream(SCORE.to_string(), "W".to_string())
    else {
        panic!("expected a stream");
    };
    assert_eq!(stream.text, "你 好 世 界\n一 二 三 四");
    assert_eq!(stream.measure_starts, vec![0, 16]);
}

#[test]
fn extracting_an_unknown_part_fails_with_a_message() {
    let ExtractPartStreamResponse::Err(failure) =
        Component::extract_part_stream(SCORE.to_string(), "X".to_string())
    else {
        panic!("expected a failure");
    };
    assert!(
        matches!(&failure.reason, PartStreamError::UnknownPart(abbreviation) if abbreviation == "X")
    );
    assert_eq!(failure.message, "no part [X] is declared");
}

#[test]
fn extracting_an_ineligible_part_fails_with_its_reason() {
    let ExtractPartStreamResponse::Err(failure) =
        Component::extract_part_stream(SCORE.to_string(), "C".to_string())
    else {
        panic!("expected a failure");
    };
    assert!(matches!(failure.reason, PartStreamError::Ineligible(_)));
    assert_eq!(
        failure.message,
        "lyrics under a chords part cannot be edited as a stream"
    );
}

#[test]
fn applying_the_unchanged_stream_returns_the_source_unchanged() {
    let result = apply("M", "1 2 3 4\n5 6 7 1");
    assert_eq!(result.source.as_deref(), Some(SCORE));
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.measure_starts, vec![0, 8]);
    assert_eq!(result.changed_measure_count, 0);
}

#[test]
fn applying_an_inserted_note_rebars_later_measures() {
    let result = apply("M", "0 1 2 3 4 5 6 7 1");
    assert_eq!(result.changed_measure_count, 3);
    assert_eq!(result.measure_starts, vec![0, 8, 16]);
    let source = result.source.unwrap();
    assert!(source.contains("[M] 0 1 2 3\n"), "{source}");
    assert!(source.contains("[M] 4 5 6 7\n"), "{source}");
    assert!(source.ends_with("[M] 1 0 0 0\n"), "{source}");
}

#[test]
fn a_barline_is_an_error_and_yields_no_source() {
    let result = apply("M", "1 2 3 4 | 5 6 7 1");
    assert_eq!(result.source, None);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert!(matches!(diagnostic.severity, DiagnosticSeverity::Error));
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (8, 9));
}

#[test]
fn a_leftover_cjk_syllable_is_spanned_in_utf8_bytes() {
    let result = apply("W", "你 好 世 界 一 二 三 四 五");
    assert_eq!(result.source, None);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert!(matches!(diagnostic.severity, DiagnosticSeverity::Error));
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (32, 35));
    assert!(diagnostic.message.contains("can't add measures"));
}
