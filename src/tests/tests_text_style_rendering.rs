use super::*;
use compositor::types::AbsoluteContent;

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
        subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
        author_font_size: score.metadata.author_style.font_size as f32,
        sequence_font_size: score.metadata.sequence.font_size as f32,
        part_legend_font_size: score.metadata.part_legend.font_size as f32,
        ..Default::default()
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
            subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
            author_font_size: score.metadata.author_style.font_size as f32,
            sequence_font_size: score.metadata.sequence.font_size as f32,
            part_legend_font_size: score.metadata.part_legend.font_size as f32,
            ..Default::default()
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
                    ..Default::default()
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
            subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
            author_font_size: score.metadata.author_style.font_size as f32,
            sequence_font_size: score.metadata.sequence.font_size as f32,
            part_legend_font_size: score.metadata.part_legend.font_size as f32,
            ..Default::default()
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
                    ..Default::default()
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

/// Renders a minimal single-note-part score (with a title, so `title`'s own
/// text element is present) with `metadata_line` spliced into `# metadata`,
/// returning the concatenated raw SVG(s).
fn render_with_metadata_line(metadata_line: &str) -> String {
    let source = format!(
        "# metadata\ntitle = \"T\"\n{metadata_line}\n\n# parts\nS = notes\n\n# score\ntime=4/4 key=C4 bpm=120\n[S] 1\n",
    );
    render_svgs_from_source(&source, "test", &[])
        .unwrap_or_else(|err| panic!("compile returned an irrecoverable error: {}", err.message()))
        .svgs
        .join("")
}

/// The `font-family='...'` attribute value of the first `<text ...>T</text>`
/// element in `svg` (the title, given `render_with_metadata_line`'s fixture).
fn title_font_family_attr(svg: &str) -> String {
    let needle = ">T</text>";
    let body_end = svg
        .find(needle)
        .expect("expected the title's <text>T</text> element in the rendered SVG");
    let open = svg[..body_end]
        .rfind("<text")
        .expect("expected an opening <text before the title's body");
    let tag = &svg[open..body_end];
    let attr_start = tag
        .find("font-family='")
        .expect("expected a font-family attribute on the title's <text> element")
        + "font-family='".len();
    let attr_end = tag[attr_start..]
        .find('\'')
        .expect("expected a closing quote on font-family");
    tag[attr_start..attr_start + attr_end].to_string()
}

#[test]
fn title_font_family_override_changes_the_rendered_font_family() {
    // `title` defaults to the `Title` role; overriding it to `sans_serif`
    // must change the rendered `font-family` attribute, proving the
    // metadata field has a real effect rather than being parsed and
    // ignored.
    let default_svg = render_with_metadata_line("");
    let overridden_svg = render_with_metadata_line("title = { font_family: sans_serif }");
    let default_family = title_font_family_attr(&default_svg);
    let overridden_family = title_font_family_attr(&overridden_svg);
    assert_ne!(
        default_family, overridden_family,
        "expected title's font-family to change when font_family: sans_serif overrides its Title default"
    );
}

#[test]
fn font_family_is_rejected_as_a_metadata_error_on_notes_chords_and_note_dash() {
    for kind in ["notes", "chords", "note_dash"] {
        let source = format!(
            "# metadata\n{kind} = {{ font_family: monospace }}\n\n# parts\nS = notes\n\n# score\ntime=4/4 key=C4 bpm=120\n[S] 1\n",
        );
        let score = compile(&source, "test", &[]).expect("compile should not irrecoverably fail");
        assert!(
            score
                .document_diagnostics
                .iter()
                .any(|d| d.message().contains(&format!("{kind}.font_family"))),
            "expected a metadata error mentioning `{kind}.font_family`, got: {:?}",
            score.document_diagnostics
        );
    }
}
