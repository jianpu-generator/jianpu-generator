use super::*;

#[test]
fn render_svgs_from_source_smoke() {
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
    let svgs = render_svgs_from_source(input, "test.jianpu", &[])
        .unwrap()
        .svgs;
    assert_eq!(svgs.len(), 1);
    assert!(svgs[0].starts_with("<svg"));
    assert!(svgs[0].ends_with("</svg>"));
}

#[test]
fn lyrics_underflow_render_returns_svgs_and_non_empty_errors() {
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b\n",
    );
    let output = render_svgs_from_source(input, "test.jianpu", &[])
        .expect("underflow must not abort the render");
    assert!(
        !output.svgs.is_empty(),
        "should produce at least one SVG page"
    );
    assert_eq!(
        output.diagnostics.len(),
        1,
        "should report one underflow error"
    );
    assert!(output.diagnostics[0].message().contains("underflow"));
    assert!(
        output.svgs[0].contains(r#"data-testid="error-highlight""#),
        "SVG should contain an error-highlight rect"
    );
}

#[test]
fn follow_part_with_tied_note_produces_exactly_one_arc() {
    // p1 has 1~1 (a tied note), p2 follows p1, a is a chord track.
    // After consolidation p1 and p2 merge into one visual row.
    // The tie arc should appear exactly once, not twice (once for p1 and
    // once for p2 displaced above the chord row).
    let input = concat!(
        "# metadata\n",
        "title = \"\"\n",
        "author = \"\"\n",
        "\n",
        "# parts\n",
        "Pluck [p1] = notes\n",
        "Pluck 2 [p2] = follow[p1]\n",
        "Accompaniment [a] = chords\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[p1] 1~1\n",
        "[a] 3\n",
    );
    let svgs = render_svgs_from_source(input, "test.jianpu", &[])
        .unwrap()
        .svgs;
    let arc_count = svgs[0].matches("data-variant=\"tie-or-slur\"").count();
    assert_eq!(
        arc_count, 1,
        "expected exactly one tie arc; got {arc_count}. SVG: {}",
        &svgs[0]
    );
}

#[test]
fn lex_unexpected_char_renders_error_highlight_and_reports_error() {
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 @ 3 4\n",
    );
    let output = render_svgs_from_source(input, "test.jianpu", &[])
        .expect("LexUnexpectedChar must not abort the render");
    assert!(
        !output.svgs.is_empty(),
        "should produce at least one SVG page"
    );
    assert_eq!(output.diagnostics.len(), 1, "should report one lex error");
    assert!(
        output.diagnostics[0].message().contains("unexpected"),
        "error message should mention unexpected char, got: {}",
        output.diagnostics[0].message()
    );
    assert!(
        output.svgs[0].contains(r#"data-testid="error-highlight""#),
        "SVG should contain an error-highlight rect for the erroneous measure"
    );
}

#[test]
fn adjacent_beat_group_underlines_have_gap_between_them() {
    // "2_3=4=" is beat 2 and "6_7_" is beat 3 — both get a level-0 beam underline.
    // The underline for beat 2 must end strictly before the underline for beat 3 starts.
    let source = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nS = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[S] 0 2_3=4= 6_7_ 0\n",
    );
    let score = compile(source, "test", &[]).unwrap();
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
    };
    let compile_result = compiler::compile(&score);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    let abs = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.lyric_font_sizes(),
    )
    .expect("coordinate resolver should not fail in tests");

    let mut underlines: Vec<(f32, f32)> = abs[0]
        .elements
        .iter()
        .filter_map(|e| {
            if let compositor::types::AbsoluteContent::Underline { width, level: 0 } = &e.content {
                Some((e.x, *width))
            } else {
                None
            }
        })
        .collect();
    underlines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    assert_eq!(underlines.len(), 2, "expected two level-0 underlines");
    let (x1, w1) = underlines[0];
    let (x2, _) = underlines[1];
    assert!(
        x2 > x1 + w1,
        "underlines should have a gap but they touch: beat2 ends at {:.1}, beat3 starts at {:.1}",
        x1 + w1,
        x2
    );
}

#[test]
fn tie_operator_on_notes_renders_exactly_two_arcs() {
    // 7_6=5=~5~5 has two ~ tie operators, so exactly 2 arcs should be rendered.
    let input = concat!(
        "# metadata\ntitle = \"Untitled\"\nauthor = \"\"\n\n",
        "# parts\nMelody[m] = notes+lyrics\n\n",
        "# score\n[m]7_6=5=~5~5\n",
    );
    let svgs = render_svgs_from_source(input, "test.jianpu", &[])
        .unwrap()
        .svgs;
    let arc_count = svgs[0].matches(r#"data-variant="tie-or-slur""#).count();
    assert_eq!(arc_count, 2, "expected 2 arcs but got {arc_count}");
}
