use super::*;
use types::DiagnosticSeverity;

#[test]
fn share_payload_round_trips_through_brotli() {
    let fixtures = [
        include_str!("../../../reference.jianpu"),
        include_str!("../../../simple.jianpu"),
        include_str!("../../../fixtures/follow_and_key.jianpu"),
        include_str!("../../../彌勒淨土鄉.jianpu"),
    ];
    for fixture in fixtures {
        let compressed = compress_share_payload(fixture);
        let decompressed = decompress_share_payload(&compressed);
        assert_eq!(decompressed.as_deref(), Some(fixture));
    }
}

#[test]
fn decompress_share_payload_rejects_garbage() {
    assert_eq!(decompress_share_payload(b"not brotli"), None);
}

#[test]
fn ok_response_has_svgs() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Melody = notes+lyrics\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Melody] 1 2 3 4\n",
        "[Melody] a b c d\n",
    );
    let resp = render_response(input, None, None, &[]);
    match resp {
        RenderResponse::Ok { documents, .. } => {
            assert_eq!(documents.len(), 1);
            assert!(!documents[0].elements.is_empty());
        }
        RenderResponse::Err { .. } => panic!("expected ok"),
    }
}

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
fn render_with_disabled_lyrics_hides_lyrics_for_part() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes+lyrics\n",
        "Alto = notes+lyrics\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Soprano] 1 2 3 4\n",
        "[Soprano] sop sop sop sop\n",
        "[Alto] 5 6 7 1\n",
        "[Alto] alt alt alt alt\n",
    );
    let all = match render_response(input, None, None, &[]) {
        RenderResponse::Ok { documents, .. } => documents,
        RenderResponse::Err { .. } => panic!("expected ok"),
    };
    let alto_lyrics_hidden =
        match render_response(input, None, Some(vec!["Alto".into()]).as_deref(), &[]) {
            RenderResponse::Ok { documents, .. } => documents,
            RenderResponse::Err { .. } => panic!("expected ok"),
        };
    // With lyrics, both parts render more elements than without
    assert!(all[0].elements.len() > alto_lyrics_hidden[0].elements.len());
}

#[test]
fn render_with_enabled_tracks_filters_parts() {
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
    let all = match render_response(input, None, None, &[]) {
        RenderResponse::Ok { documents, .. } => documents,
        RenderResponse::Err { .. } => panic!("expected ok"),
    };
    let soprano_only =
        match render_response(input, Some(vec!["Soprano".into()]).as_deref(), None, &[]) {
            RenderResponse::Ok { documents, .. } => documents,
            RenderResponse::Err { .. } => panic!("expected ok"),
        };
    // Rendering both parts produces more elements than rendering one
    assert_ne!(all[0].elements.len(), soprano_only[0].elements.len());
}

#[test]
fn err_response_has_structured_diagnostic() {
    // Missing sections are now recoverable; render returns Ok with error diagnostics.
    let resp = render_response("not valid jianpu", None, None, &[]);
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
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b\n",
    );
    let resp = render_response(input, None, None, &[]);
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
    let resp = render_response(source, None, None, &[]);
    match resp {
        RenderResponse::Ok {
            documents,
            diagnostics,
            ..
        } => {
            assert!(
                !documents.is_empty(),
                "reference.jianpu should render in the wasm path used by the web editor"
            );
            assert!(
                !documents[0].elements.is_empty(),
                "first page should have elements"
            );
            assert!(
                diagnostics.is_empty(),
                "reference.jianpu should have no errors or warnings, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
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

#[cfg(feature = "wav")]
#[test]
fn generate_wav_for_measure_range_response_returns_riff_wav() {
    let source = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n",
    );
    let soundfont = include_bytes!("../../../fonts/GeneralUser_GS.sf2").to_vec();
    let resp = generate_wav_for_measure_range_response(source, 0, 0, None, soundfont);
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
    let soundfont = include_bytes!("../../../fonts/GeneralUser_GS.sf2").to_vec();
    let resp = generate_wav_response(source, None, soundfont);
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
    // 'z' in a notes line is a recoverable error (LexUnexpectedChar),
    // so render returns Ok with a warning diagnostic.
    let source = concat!(
        "# metadata\n",
        "title = \"你好\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Melody = notes+lyrics\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Melody] 1 2 z 4\n",
        "[Melody] a b c d\n",
    );
    let token_byte_start = source.find('z').expect("error token in source");
    let resp = render_response(source, None, None, &[]);
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

#[test]
fn follow_part_with_explicit_override_in_some_sections_renders_ok() {
    // p2 follows p1 but has an explicit override ([p2]12) in the first section.
    // s2 follows s1 but has an explicit override ([s2]67) in the first section.
    // In the second section (time=4/4), only [s1] and [a] appear; the follow
    // parts (p2, s2) and their leaders (p1) are absent and should be filled
    // implicitly / via follow resolution.
    let input = concat!(
        "# metadata\n",
        "title = \"\"\n",
        "author = \"\"\n",
        "\n",
        "# parts\n",
        "Pluck [p1] = notes\n",
        "Pluck 2 [p2] = follow[p1]\n",
        "String [s1] = notes\n",
        "String 2 [s2] = follow[s1]\n",
        "Accompaniment [a] = chords\n",
        "\n",
        "\n",
        "# score\n",
        "time=2/4\n",
        "[p1]12\n",
        "[p2]12\n",
        "[s1]1'2'\n",
        "[s2]67\n",
        "[a]4 5 \n",
        "\n",
        "time=4/4\n",
        "[s1] 5---\n",
        "[a] 1---\n",
    );
    let resp = render_response(input, None, None, &[]);
    match resp {
        RenderResponse::Ok {
            documents,
            diagnostics,
            ..
        } => {
            assert!(!documents.is_empty(), "expected at least one page");
            assert!(
                diagnostics.is_empty(),
                "expected no diagnostics, got: {}",
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        RenderResponse::Err { diagnostics, .. } => {
            panic!(
                "expected ok but got error: {}",
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
    }
}

#[test]
fn two_parts_with_non_four_four_first_section_has_no_diagnostics() {
    // Minimal reproducer: two parts, time=2/4 first section, both parts
    // present in the second section.  The renderer must not emit any
    // diagnostics — the bug caused spurious "measure count mismatch" and
    // "beat overflow" errors whenever the first section was not 4/4.
    let input = concat!(
        "# metadata\n",
        "title = \"\"\n",
        "author = \"\"\n",
        "\n",
        "# parts\n",
        "A [a] = notes\n",
        "B [b] = notes\n",
        "\n",
        "# score\n",
        "time=2/4\n",
        "[a]12\n",
        "[b]12\n",
        "\n",
        "time=4/4\n",
        "[a] 5\n",
        "[b] 1\n",
    );
    let resp = render_response(input, None, None, &[]);
    match resp {
        RenderResponse::Ok {
            documents,
            diagnostics,
            ..
        } => {
            assert!(!documents.is_empty(), "expected at least one page");
            assert!(
                diagnostics.is_empty(),
                "expected no diagnostics, got: {}",
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        RenderResponse::Err { diagnostics, .. } => {
            panic!(
                "expected ok but got error: {}",
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
    }
}

#[path = "group_diagnostics_tests.rs"]
mod group_diagnostics_tests;
