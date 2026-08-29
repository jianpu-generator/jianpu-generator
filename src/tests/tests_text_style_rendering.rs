use super::*;
use compositor::types::AbsoluteContent;

#[test]
fn title_width_pt_reserves_a_minimum_box_width() {
    // A short title ("Hi") whose real rendered width is far below 300pt —
    // the box-width computation must widen it to the configured minimum
    // rather than passing the real text width straight through.
    let source = concat!(
        "# metadata\ntitle = \"Hi\"\ntitle = { width_pt: 300 }\n\n",
        "# parts\nS = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[S] 1\n",
    );
    let score = compile(source, "test", &[]).unwrap();
    assert_eq!(score.metadata.title_style.width_pt, 300);
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: score.metadata.title_style.font_size as f32,
        title_min_width_pt: score.metadata.title_style.width_pt as f32,
        subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
        author_font_size: score.metadata.author_style.font_size as f32,
        sequence_font_size: score.metadata.sequence.font_size as f32,
        part_legend_font_size: score.metadata.part_legend.font_size as f32,
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
                section_label_vertical_padding_pt: config.section_label_vertical_padding_pt(),
                part_label: config.part_label_font_size as f32,
            },
            paddings: config.element_paddings(),
            page_number_vertical_padding_pt: config.page_number_vertical_padding_pt(),
        },
    )
    .expect("coordinate resolver should not fail in tests");

    let reserved_width_pt = abs[0]
        .elements
        .iter()
        .find_map(|e| match &e.content {
            AbsoluteContent::Text {
                content,
                reserved_width_pt,
                ..
            } if content == "Hi" => Some(*reserved_width_pt),
            _ => None,
        })
        .expect("title text element should be present");

    assert!(
        reserved_width_pt >= 300.0,
        "title's reserved box width should be at least 300pt, got {reserved_width_pt}"
    );
}

/// Lays out a minimal single-note-part score, returning the total height in
/// points of its one system's `GridRow`s — used by
/// `notes_vertical_padding_pt_grows_the_note_head_sub_row` to compare a
/// padded score's system height against an unpadded one.
fn single_note_system_height_pt(notes_metadata_line: &str) -> f32 {
    let source = format!(
        "# metadata\n{notes_metadata_line}\n\n# parts\nS = notes\n\n# score\ntime=4/4 key=C4 bpm=120\n[S] 1\n",
    );
    let score = compile(&source, "test", &[]).unwrap();
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: score.metadata.title_style.font_size as f32,
        title_min_width_pt: score.metadata.title_style.width_pt as f32,
        subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
        author_font_size: score.metadata.author_style.font_size as f32,
        sequence_font_size: score.metadata.sequence.font_size as f32,
        part_legend_font_size: score.metadata.part_legend.font_size as f32,
    };
    let compile_result = compiler::compile(&score);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_pages =
        grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None).pages;
    // Excludes the last row: the footer's `remaining_height` always expands
    // to fill whatever the body doesn't use, so summing it in would cancel
    // out exactly the body-height difference this test is measuring.
    let rows = &grid_pages[0].rows;
    rows[..rows.len() - 1].iter().map(|r| r.height_pt).sum()
}

#[test]
fn section_label_vertical_padding_pt_grows_the_label_box_height() {
    // `label_box_height` (see `AbsoluteContent::DirectiveLine`) is the
    // section-label background box's real drawn height — must grow by at
    // least `vertical_padding_pt` over the unpadded default.
    fn label_box_height(padding_line: &str) -> f32 {
        let source = format!(
            "# metadata\n{padding_line}\n\n# parts\nS = notes\n\n# score\ntime=4/4 key=C4 bpm=120 label=\"Verse 1\"\n[S] 1\n",
        );
        let score = compile(&source, "test", &[]).unwrap();
        let config = render_config::RenderConfig::from_metadata(&score.metadata);
        let header = grid_layout::types::Header {
            title: score.metadata.title.clone(),
            subtitle: score.metadata.subtitle.clone(),
            author: score.metadata.author.clone(),
            part_list: vec![],
            parts_list_columns: 3,
            sequence: None,
            title_font_size: score.metadata.title_style.font_size as f32,
            title_min_width_pt: score.metadata.title_style.width_pt as f32,
            subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
            author_font_size: score.metadata.author_style.font_size as f32,
            sequence_font_size: score.metadata.sequence.font_size as f32,
            part_legend_font_size: score.metadata.part_legend.font_size as f32,
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
                    section_label_vertical_padding_pt: config.section_label_vertical_padding_pt(),
                    part_label: config.part_label_font_size as f32,
                },
                paddings: config.element_paddings(),
                page_number_vertical_padding_pt: config.page_number_vertical_padding_pt(),
            },
        )
        .expect("coordinate resolver should not fail in tests");

        abs[0]
            .elements
            .iter()
            .find_map(|e| match &e.content {
                AbsoluteContent::DirectiveLine {
                    label,
                    label_box_height,
                    ..
                } if label.as_deref() == Some("Verse 1") => Some(*label_box_height),
                _ => None,
            })
            .expect("directive line with a section label should be present")
    }

    let unpadded = label_box_height("section_label = { vertical_padding_pt: 0 }");
    let padded = label_box_height("section_label = { vertical_padding_pt: 8 }");
    assert!(
        padded >= unpadded + 8.0,
        "padded label box height ({padded}) should be at least 8pt taller than unpadded ({unpadded})"
    );
}

#[test]
fn page_number_vertical_padding_pt_pushes_the_footer_text_up() {
    // `make_footer_row` always sizes the footer row to whatever's left of
    // the page — a row-height change wouldn't be visible — so the real
    // effect is offsetting the page-number text upward from the bottom
    // edge (see `resolve_row_element`'s `bottom_padding`), i.e. its
    // resolved `y` decreases as padding grows.
    fn footer_text_y(padding_line: &str) -> f32 {
        let source = format!(
            "# metadata\n{padding_line}\n\n# parts\nS = notes\n\n# score\ntime=4/4 key=C4 bpm=120\n[S] 1\n",
        );
        let score = compile(&source, "test", &[]).unwrap();
        let config = render_config::RenderConfig::from_metadata(&score.metadata);
        let header = grid_layout::types::Header {
            title: score.metadata.title.clone(),
            subtitle: score.metadata.subtitle.clone(),
            author: score.metadata.author.clone(),
            part_list: vec![],
            parts_list_columns: 3,
            sequence: None,
            title_font_size: score.metadata.title_style.font_size as f32,
            title_min_width_pt: score.metadata.title_style.width_pt as f32,
            subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
            author_font_size: score.metadata.author_style.font_size as f32,
            sequence_font_size: score.metadata.sequence.font_size as f32,
            part_legend_font_size: score.metadata.part_legend.font_size as f32,
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
                    section_label_vertical_padding_pt: config.section_label_vertical_padding_pt(),
                    part_label: config.part_label_font_size as f32,
                },
                paddings: config.element_paddings(),
                page_number_vertical_padding_pt: config.page_number_vertical_padding_pt(),
            },
        )
        .expect("coordinate resolver should not fail in tests");

        abs[0]
            .elements
            .iter()
            .find_map(|e| match &e.content {
                AbsoluteContent::Text { content, .. } if content == "1 / 1" => Some(e.y),
                _ => None,
            })
            .expect("footer page-number text element should be present")
    }

    let unpadded = footer_text_y("page_number = { vertical_padding_pt: 0 }");
    let padded = footer_text_y("page_number = { vertical_padding_pt: 4 }");
    assert!(
        padded <= unpadded - 4.0,
        "padded footer text y ({padded}) should sit at least 4pt above unpadded ({unpadded})"
    );
}

#[test]
fn notes_vertical_padding_pt_grows_the_note_head_sub_row() {
    // The note-head sub-row is one flat `base` per `note_part_sub_row_heights`
    // regardless of how many measures/columns the system has, so any nonzero
    // `notes.vertical_padding_pt` must grow the system's total row height by
    // at least that amount — a direct, config-only effect, not something
    // that depends on the particular notes in this minimal fixture.
    let unpadded = single_note_system_height_pt("notes = { vertical_padding_pt: 0 }");
    let padded = single_note_system_height_pt("notes = { vertical_padding_pt: 5 }");
    assert!(
        padded >= unpadded + 5.0,
        "padded system height ({padded}) should be at least 5pt taller than unpadded ({unpadded})"
    );
}
