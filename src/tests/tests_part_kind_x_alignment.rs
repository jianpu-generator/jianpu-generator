use super::*;

/// Recursively collects the visual leading-glyph x of every `Text` element
/// carrying `variant`, in document order. The leading-glyph x is where the
/// first character actually starts painting, accounting for `anchor`
/// (`Middle`/`End` shift the paint origin away from the box x), so it's the
/// coordinate readers actually compare when eyeballing column alignment.
fn collect_leading_glyph_x(
    elements: &[renderer::new_types::SvgElement],
    variant: renderer::new_types::SvgVariant,
    out: &mut Vec<(f32, String)>,
) {
    use compositor::types::TextAnchor;
    use renderer::new_types::SvgKind;

    for elem in elements {
        if elem.variant == Some(variant) {
            if let SvgKind::Text {
                content,
                font_size,
                anchor,
                ..
            } = &elem.kind
            {
                let text_width = font_metrics::monospace_text_width(content, *font_size);
                let leading_x = match anchor {
                    TextAnchor::Start => elem.x,
                    TextAnchor::Middle => elem.x - text_width / 2.0,
                    TextAnchor::End => elem.x - text_width,
                };
                out.push((leading_x, content.clone()));
            }
        }
        if let SvgKind::Group { children, .. } = &elem.kind {
            collect_leading_glyph_x(children, variant, out);
        }
    }
}

/// A chord symbol wider than a single digit (e.g. `2m`) must still have its
/// root digit line up with the note it annotates — a reader visually
/// compares `[c] 1 2m 3m 4` against `[n] 1 2 3 4` column by column, not box
/// center against box center.
#[test]
fn wide_chord_symbols_align_their_root_digit_with_the_note() {
    let input = concat!(
        "# parts\n",
        "c = chords\n",
        "n = notes\n",
        "\n",
        "# score\n",
        "[c] 1 2m 3m 4\n",
        "[n] 1 2 3 4\n",
    );

    let output =
        render_documents_from_source_filtered_with_lyrics(input, "test.jianpu", None, None, &[])
            .unwrap();

    let mut chords = Vec::new();
    let mut notes = Vec::new();
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::ChordSymbol,
        &mut chords,
    );
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::NoteHead,
        &mut notes,
    );

    assert_eq!(chords.len(), 4, "expected 4 chord symbols, got {chords:?}");
    assert_eq!(notes.len(), 4, "expected 4 note heads, got {notes:?}");

    for ((chord_x, chord_text), (note_x, note_text)) in chords.iter().zip(notes.iter()) {
        assert!(
            (chord_x - note_x).abs() < 0.01,
            "chord {chord_text:?} (leading x={chord_x}) should align with note \
             {note_text:?} (leading x={note_x})",
        );
    }
}

#[test]
fn notes_and_chords_parts_align_on_the_same_beat() {
    let input = concat!(
        "# parts\n",
        "a = notes\n",
        "b = chords\n",
        "\n",
        "# score\n",
        "[a] 1\n",
        "[b] 1\n",
    );

    let output =
        render_documents_from_source_filtered_with_lyrics(input, "test.jianpu", None, None, &[])
            .unwrap();

    let mut chords = Vec::new();
    let mut notes = Vec::new();
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::ChordSymbol,
        &mut chords,
    );
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::NoteHead,
        &mut notes,
    );

    let (chord_x, _) = chords.first().expect("expected a chord symbol");
    let (note_x, _) = notes.first().expect("expected a note head");

    assert!(
        (chord_x - note_x).abs() < 0.01,
        "the note head and the chord symbol occupying the same beat should share \
         the same visual x position (note at x={note_x}, chord at x={chord_x})"
    );
}
