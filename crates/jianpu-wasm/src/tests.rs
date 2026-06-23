use super::*;
use types::DiagnosticSeverity;

#[test]
fn ok_response_has_svgs() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Melody = notes lyrics\n",
        "\n",
        "[score]\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        "a b c d\n",
    );
    let resp = render_response(input, None, None);
    match resp {
        RenderResponse::Ok { svgs, .. } => {
            assert_eq!(svgs.len(), 1);
            assert!(svgs[0].starts_with("<svg"));
        }
        RenderResponse::Err { .. } => panic!("expected ok"),
    }
}

#[test]
fn list_parts_response_returns_declarations() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
    );
    let resp = list_parts_response(input);
    match resp {
        ListPartsResponse::Ok { parts } => {
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
fn render_with_disabled_lyrics_hides_lyrics_for_part() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes lyrics\n",
        "Alto = notes lyrics\n",
        "\n",
        "[score]\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        "sop sop sop sop\n",
        "5 6 7 1\n",
        "alt alt alt alt\n",
    );
    let all = match render_response(input, None, None) {
        RenderResponse::Ok { svgs, .. } => svgs,
        RenderResponse::Err { .. } => panic!("expected ok"),
    };
    let alto_lyrics_hidden =
        match render_response(input, None, Some(vec!["Alto".into()]).as_deref()) {
            RenderResponse::Ok { svgs, .. } => svgs,
            RenderResponse::Err { .. } => panic!("expected ok"),
        };
    assert!(all[0].contains("sop"));
    assert!(all[0].contains("alt"));
    assert!(alto_lyrics_hidden[0].contains("sop"));
    assert!(!alto_lyrics_hidden[0].contains("alt"));
}

#[test]
fn render_with_enabled_tracks_filters_parts() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
    );
    let all = match render_response(input, None, None) {
        RenderResponse::Ok { svgs, .. } => svgs,
        RenderResponse::Err { .. } => panic!("expected ok"),
    };
    let soprano_only = match render_response(input, Some(vec!["Soprano".into()]).as_deref(), None) {
        RenderResponse::Ok { svgs, .. } => svgs,
        RenderResponse::Err { .. } => panic!("expected ok"),
    };
    assert_ne!(all[0], soprano_only[0]);
}

#[test]
fn err_response_has_structured_diagnostic() {
    // Missing sections are now recoverable; render returns Ok with error diagnostics.
    let resp = render_response("not valid jianpu", None, None);
    let diagnostics = match resp {
        RenderResponse::Err { diagnostics, .. } | RenderResponse::Ok { diagnostics, .. } => {
            diagnostics
        }
    };
    assert!(!diagnostics.is_empty());
    let d = &diagnostics[0];
    assert!(!d.message.is_empty());
}

#[test]
fn recoverable_error_produces_warning_severity_view_zone() {
    // lyrics underflow is a recoverable error
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "[parts]\nMelody = notes lyrics\n\n",
        "[score]\ntime=4/4 key=C4 bpm=120\n1 2 3 4\na b\n",
    );
    let resp = render_response(input, None, None);
    match resp {
        RenderResponse::Ok {
            diagnostics,
            diagnostic_view_zones,
            ..
        } => {
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
            assert_eq!(diagnostic_view_zones.len(), 1);
            assert_eq!(
                diagnostic_view_zones[0].severity,
                DiagnosticSeverity::Warning
            );
            assert_eq!(diagnostic_view_zones[0].messages.len(), 1);
        }
        RenderResponse::Err { .. } => panic!("expected ok"),
    }
}

#[test]
fn reference_jianpu_renders() {
    let source = include_str!("../../../reference.jianpu");
    let resp = render_response(source, None, None);
    match resp {
        RenderResponse::Ok { svgs, .. } => {
            assert!(
                !svgs.is_empty(),
                "reference.jianpu should render in the wasm path used by the web editor"
            );
        }
        RenderResponse::Err { diagnostics, .. } => {
            panic!(
                "reference.jianpu failed in wasm render path: {}",
                diagnostics[0].message
            );
        }
    }
}

#[cfg(feature = "pdf")]
fn test_pdf_fonts() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    (
        include_bytes!("../../../fonts/SourceHanSansSC-Regular.otf").to_vec(),
        include_bytes!("../../../fonts/SourceHanSansTC-Regular.otf").to_vec(),
        include_bytes!("../../../fonts/NotoSansMono-Regular.ttf").to_vec(),
    )
}

#[cfg(feature = "pdf")]
#[test]
fn reference_jianpu_generates_pdf() {
    let source = include_str!("../../../reference.jianpu");
    let (sc, tc, mono) = test_pdf_fonts();
    let resp = generate_pdf_response(source, None, None, sc, tc, mono);
    match resp {
        GeneratePdfResponse::Ok { pdf } => {
            assert!(pdf.len() > 4);
            assert_eq!(&pdf[0..4], b"%PDF");
        }
        GeneratePdfResponse::Err { diagnostics } => {
            panic!(
                "reference.jianpu failed in wasm pdf path: {}",
                diagnostics[0].message
            );
        }
    }
}

#[cfg(feature = "pdf")]
#[test]
fn reference_jianpu_generates_split_pdf_zip() {
    use std::io::Read;
    use zip::ZipArchive;

    let source = include_str!("../../../reference.jianpu");
    let (sc, tc, mono) = test_pdf_fonts();
    let resp = generate_split_pdfs_response(source, "reference", sc, tc, mono);
    match resp {
        GenerateSplitPdfsResponse::Ok { zip } => {
            assert!(zip.len() > 4);
            assert_eq!(&zip[0..2], b"PK");
            let cursor = std::io::Cursor::new(zip);
            let mut archive = ZipArchive::new(cursor).unwrap();
            assert!(archive.len() >= 1);
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).unwrap();
                let name = file.name().to_string();
                assert!(
                    name.starts_with("reference - ") && name.ends_with(".pdf"),
                    "unexpected zip entry: {name}"
                );
                let mut buf = [0u8; 4];
                file.read_exact(&mut buf).unwrap();
                assert_eq!(&buf, b"%PDF");
            }
        }
        GenerateSplitPdfsResponse::Err { diagnostics } => {
            panic!(
                "reference.jianpu failed in wasm split pdf path: {}",
                diagnostics[0].message
            );
        }
    }
}

#[test]
fn get_measure_at_offset_ok_for_note_in_measure() {
    let source = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes\n\n",
        "[score]\ntime=4/4 key=C4 bpm=120\n1 2 3 4\n",
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
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes\n\n",
        "[score]\ntime=4/4 key=C4 bpm=120\n1 2 3 4\n",
    );
    let resp = get_measure_at_offset_response(source, 0);
    assert!(
        matches!(resp, MeasureAtOffsetResponse::NotInMeasure),
        "expected NotInMeasure"
    );
}

#[cfg(feature = "wav")]
#[test]
fn generate_wav_for_measure_range_response_returns_riff_wav() {
    let source = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes\n\n",
        "[score]\ntime=4/4 key=C4 bpm=120\n1 2 3 4\n",
    );
    let resp = generate_wav_for_measure_range_response(source, 0, 0, None);
    match resp {
        GenerateWavResponse::Ok { wav } => {
            assert!(wav.len() > 4);
            assert_eq!(&wav[0..4], b"RIFF");
        }
        GenerateWavResponse::Err { diagnostics } => {
            panic!("expected Ok: {}", diagnostics[0].message);
        }
    }
}

#[cfg(feature = "wav")]
#[test]
fn reference_jianpu_generates_wav() {
    let source = include_str!("../../../reference.jianpu");
    let resp = generate_wav_response(source, None);
    match resp {
        GenerateWavResponse::Ok { wav } => {
            assert!(wav.len() > 4);
            assert_eq!(&wav[0..4], b"RIFF");
        }
        GenerateWavResponse::Err { diagnostics } => {
            panic!(
                "reference.jianpu failed in wasm wav path: {}",
                diagnostics[0].message
            );
        }
    }
}

#[test]
fn diagnostic_span_is_utf8_byte_offset() {
    // 'x' in a notes line is a recoverable error (LexUnexpectedChar),
    // so render returns Ok with a warning diagnostic.
    let source = concat!(
        "[metadata]\n",
        "title = \"你好\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Melody = notes lyrics\n",
        "\n",
        "[score]\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 x 4\n",
        "a b c d\n",
    );
    let token_byte_start = source.find('x').expect("error token in source");
    let resp = render_response(source, None, None);
    let diagnostics = match resp {
        RenderResponse::Ok { diagnostics, .. } => diagnostics,
        RenderResponse::Err { diagnostics, .. } => diagnostics,
    };
    assert!(!diagnostics.is_empty());
    // The diagnostic span must overlap with the 'x' token — verify the span
    // is anchored absolutely in the source (not line-locally, which would be < 4).
    let d = &diagnostics[0];
    assert!(
        d.span.start >= token_byte_start || d.span.end > token_byte_start,
        "span ({}, {}) does not cover token at {token_byte_start}",
        d.span.start,
        d.span.end
    );
    assert!(
        d.span.start > 4,
        "span.start {} should be absolute in source, not line-local",
        d.span.start
    );
}

#[test]
fn list_measure_spans_returns_one_span_per_measure() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Melody = notes\n",
        "\n",
        "[score]\n",
        "1 2 3 4\n",
        "\n",
        "5 6 7 1\n",
    );
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok { spans } => {
            assert_eq!(spans.len(), 2);
            assert!(spans[0].start < spans[1].start);
            assert_eq!(spans[0].view_zone_start, spans[0].start);
        }
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}

#[test]
fn list_measure_spans_view_zone_start_includes_directive_line() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Melody = notes\n",
        "\n",
        "[score]\n",
        "bpm=60\n",
        "1 2 3 4\n",
    );
    let directive_offset = input.find("bpm=60").unwrap();
    let notes_offset = input.find("1 2 3 4").unwrap();
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok { spans } => {
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
        ListMeasureSpansResponse::Ok { spans } => assert!(spans.is_empty()),
        ListMeasureSpansResponse::Err => {}
    }
}

mod group_diagnostics_tests {
    use crate::types::{
        group_diagnostics_into_view_zones, DiagnosticOut, DiagnosticSeverity, SpanOut,
    };

    fn make_diagnostic(
        severity: DiagnosticSeverity,
        message: &str,
        span_end: usize,
    ) -> DiagnosticOut {
        DiagnosticOut {
            severity,
            message: message.to_string(),
            span: SpanOut {
                start: 0,
                end: span_end,
            },
            report: None,
        }
    }

    #[test]
    fn single_error_produces_one_error_zone() {
        // "line1\nline2\n" — byte offset 10 is on line 2
        let source = "line1\nline2\n";
        let diagnostics = vec![make_diagnostic(DiagnosticSeverity::Error, "oops", 10)];
        let zones = group_diagnostics_into_view_zones(source, &diagnostics);
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].severity, DiagnosticSeverity::Error);
        assert_eq!(zones[0].after_line_number, 2);
        assert_eq!(zones[0].messages.len(), 1);
        assert_eq!(zones[0].messages[0].message, "oops");
    }

    #[test]
    fn single_warning_produces_one_warning_zone() {
        let source = "line1\n";
        let diagnostics = vec![make_diagnostic(DiagnosticSeverity::Warning, "note", 4)];
        let zones = group_diagnostics_into_view_zones(source, &diagnostics);
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(zones[0].after_line_number, 1);
    }

    #[test]
    fn two_errors_same_line_merge_into_one_zone() {
        let source = "line1\nline2\n";
        let diagnostics = vec![
            make_diagnostic(DiagnosticSeverity::Error, "first", 8),
            make_diagnostic(DiagnosticSeverity::Error, "second", 10),
        ];
        let zones = group_diagnostics_into_view_zones(source, &diagnostics);
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].messages.len(), 2);
        assert_eq!(zones[0].messages[0].message, "first");
        assert_eq!(zones[0].messages[1].message, "second");
    }

    #[test]
    fn error_and_warning_on_same_line_produce_two_zones_error_first() {
        let source = "line1\nline2\n";
        let diagnostics = vec![
            make_diagnostic(DiagnosticSeverity::Warning, "warn", 8),
            make_diagnostic(DiagnosticSeverity::Error, "err", 10),
        ];
        let zones = group_diagnostics_into_view_zones(source, &diagnostics);
        assert_eq!(zones.len(), 2);
        assert_eq!(zones[0].severity, DiagnosticSeverity::Error);
        assert_eq!(zones[1].severity, DiagnosticSeverity::Warning);
        assert_eq!(zones[0].after_line_number, 2);
        assert_eq!(zones[1].after_line_number, 2);
    }

    #[test]
    fn zones_sorted_by_line_number_ascending() {
        let source = "a\nb\nc\n";
        let diagnostics = vec![
            make_diagnostic(DiagnosticSeverity::Error, "line3", 5),
            make_diagnostic(DiagnosticSeverity::Error, "line1", 1),
        ];
        let zones = group_diagnostics_into_view_zones(source, &diagnostics);
        assert_eq!(zones.len(), 2);
        assert!(zones[0].after_line_number < zones[1].after_line_number);
    }

    #[test]
    fn empty_diagnostics_returns_empty_zones() {
        let zones = group_diagnostics_into_view_zones("source", &[]);
        assert!(zones.is_empty());
    }
}

#[test]
fn list_score_line_hints_returns_physical_line_offsets() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Chord = chord\n",
        "Melody = notes lyrics\n",
        "\n",
        "[score]\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 - - -\n",
        "1 1 5 5\n",
        "twin- kle\n",
    );
    let chord_offset = input.find("1 - - -").unwrap();
    let melody_lyrics_offset = input.find("twin- kle").unwrap();
    let resp = list_score_line_hints_response(input);
    match resp {
        ListScoreLineHintsResponse::Ok { hints } => {
            assert_eq!(hints.len(), 3);
            assert!(hints
                .iter()
                .any(|hint| { hint.line_start == chord_offset && hint.abbreviation == "Chord" }));
            assert!(hints.iter().any(|hint| {
                hint.line_start == melody_lyrics_offset && hint.abbreviation == "Melody"
            }));
        }
        ListScoreLineHintsResponse::Err { .. } => panic!("expected ok"),
    }
}
