use super::*;

#[test]
fn triplet_of_sixteenth_notes_gets_double_underline_matching_written_duration() {
    // "3:{1=2=3=}" writes three sixteenth notes (`=`), which conventionally render
    // with a double underline (level 0 + level 1) regardless of the tuplet's 3-in-2
    // duration compression — the tuplet bracket/number is a separate overlay on top,
    // not a substitute for the beam reflecting each note's written value. Padded with
    // "2:{1=2=}" and "5:4:{1=1=1=1=1=} 2_3_4_" (from demo/12-tuplets.jianpu) so the
    // measure's nominal duration sum still fills the 4/4 bar (2 + 3 + 5 + 6 = 16).
    let source = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nS = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[S] 2:{1=2=} 3:{1=2=3=} 5:4:{1=1=1=1=1=} 2_3_4_\n",
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
        title_font_size: score.metadata.title_font_size as f32,
        subtitle_font_size: score.metadata.subtitle_font_size as f32,
        author_font_size: score.metadata.author_font_size as f32,
        sequence_font_size: score.metadata.sequence_font_size as f32,
        part_legend_font_size: score.metadata.part_legend_font_size as f32,
    };
    let compile_result = compiler::compile(&score);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_pages =
        grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None).pages;
    let abs = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.part_label_width_pt as f32,
        coordinate_resolver::ResolveFontSizes {
            lyric: config.lyric_font_sizes(),
            notes: config.notes_font_size(),
            chords: config.chords_font_size(),
            labels: coordinate_resolver::LabelFontSizes {
                measure_number: config.measure_number_font_size as f32,
                section_label: config.section_label_font_size as f32,
                part_label: config.part_label_font_size as f32,
            },
            paddings: config.element_paddings(),
        },
    )
    .expect("coordinate resolver should not fail in tests");

    let level1_underlines = abs[0]
        .elements
        .iter()
        .filter(|e| {
            matches!(
                e.content,
                compositor::types::AbsoluteContent::Underline { level: 1, .. }
            )
        })
        .count();

    assert!(
        level1_underlines > 0,
        "the sixteenth-note triplet 3:{{1=2=3=}} should still get a level-1 (double) \
         underline run reflecting its written sixteenth-note duration, independent of \
         the tuplet's rescaled duration"
    );
}
