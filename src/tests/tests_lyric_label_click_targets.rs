use super::*;

/// Regression coverage for the lyric-verse-label feature: each verse row
/// (e.g. "M:v1", "M:v2") gets its own visible `RowLabel` text plus its own
/// invisible click target, mirroring how a part gets its own `RowLabel`
/// text plus `PartLabelClickTarget`.
fn resolve_test_score(input: &str) -> Vec<compositor::types::AbsolutePage> {
    let score = compile(input, "test", &[]).unwrap();
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
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.part_label_width_pt as f32,
        config.lyric_font_sizes(),
        config.notes_font_size(),
        config.chords_font_size(),
    )
    .expect("coordinate resolver should not fail in tests")
}

const TWO_VERSE_INPUT: &str = concat!(
    "# metadata\n",
    "title = \"t\"\n",
    "\n",
    "# parts\n",
    "Melody [M] = notes+lyrics\n",
    "\n",
    "# score\n",
    "time=4/4 key=C4 bpm=120\n",
    "[M] 1 2 3 4\n",
    "[M] do re mi fa\n",
    "[M] la ti da di\n",
);

#[test]
fn each_verse_row_renders_its_own_label_text() {
    let abs = resolve_test_score(TWO_VERSE_INPUT);
    let label_texts: Vec<&str> = abs[0]
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            compositor::types::AbsoluteContent::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        label_texts.contains(&"M:v1"),
        "expected a \"M:v1\" verse label, got {label_texts:?}"
    );
    assert!(
        label_texts.contains(&"M:v2"),
        "expected a \"M:v2\" verse label, got {label_texts:?}"
    );
}

#[test]
fn each_verse_row_gets_its_own_click_target() {
    let abs = resolve_test_score(TWO_VERSE_INPUT);
    let targets: Vec<(usize, usize, usize, usize)> = abs[0]
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            compositor::types::AbsoluteContent::LyricLabelClickTarget {
                source_part_index,
                verse,
                measure_index_start,
                measure_index_end,
                ..
            } => Some((
                *source_part_index,
                *verse,
                *measure_index_start,
                *measure_index_end,
            )),
            _ => None,
        })
        .collect();
    assert_eq!(
        targets,
        vec![(0, 0, 0, 0), (0, 1, 0, 0)],
        "expected one click target per verse row, scoped to the system's own measure range"
    );
}
