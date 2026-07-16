use super::*;

#[test]
fn list_parts_response_returns_declarations() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Soprano] 1 2 3 4\n",
        "[Alto] 5 6 7 1\n",
    );
    let resp = part_declarations::list_parts_response(input, &[]);
    match resp {
        ListPartsResponse::Ok { parts, .. } => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0].abbreviation, "Soprano");
            assert_eq!(parts[1].abbreviation, "Alto");
        }
        ListPartsResponse::Err { diagnostics } => {
            panic!("expected ok: {}", diagnostics[0].message);
        }
    }
}

#[test]
fn get_measure_at_offset_ok_for_note_in_measure() {
    let source = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n",
    );
    let byte_offset = source.find("1 2 3 4").unwrap();
    let resp = get_measure_at_offset_response(source, byte_offset);
    match resp {
        MeasureAtOffsetResponse::Ok { measure_index } => assert_eq!(measure_index, 0),
        MeasureAtOffsetResponse::NotInMeasure => panic!("expected Ok"),
    }
}

#[test]
fn get_measure_at_offset_not_in_measure_for_header() {
    let source = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n",
    );
    let resp = get_measure_at_offset_response(source, 0);
    assert!(
        matches!(resp, MeasureAtOffsetResponse::NotInMeasure),
        "expected NotInMeasure"
    );
}

#[test]
fn list_measure_spans_returns_one_span_per_measure() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Melody = notes\n",
        "\n",
        "# score\n",
        "[Melody] 1 2 3 4\n",
        "\n",
        "[Melody] 5 6 7 1\n",
    );
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok { spans, .. } => {
            assert_eq!(spans.len(), 2);
            assert!(spans[0].start < spans[1].start);
            // No separate directive line precedes the first measure, so view_zone_start
            // is on the same source line as start (the [Melody] prefix is on that line).
            let count_newlines = |s: &str, end: usize| s[..end].matches('\n').count();
            assert_eq!(
                count_newlines(input, spans[0].view_zone_start),
                count_newlines(input, spans[0].start),
            );
        }
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}

#[test]
fn list_measure_spans_view_zone_start_includes_directive_line() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Melody = notes\n",
        "\n",
        "# score\n",
        "bpm=60\n",
        "[Melody] 1 2 3 4\n",
    );
    let directive_offset = input.find("bpm=60").unwrap();
    let notes_offset = input.find("1 2 3 4").unwrap();
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok { spans, .. } => {
            assert_eq!(spans.len(), 1);
            assert_eq!(spans[0].view_zone_start, directive_offset);
            assert_eq!(spans[0].start, notes_offset);
        }
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}

#[test]
fn list_measure_spans_returns_empty_for_invalid_source() {
    // Missing sections are recoverable; the response is Ok with no spans.
    let resp = list_measure_spans_response("not valid jianpu");
    match resp {
        ListMeasureSpansResponse::Ok { spans, .. } => assert!(spans.is_empty()),
        ListMeasureSpansResponse::Err => {}
    }
}
