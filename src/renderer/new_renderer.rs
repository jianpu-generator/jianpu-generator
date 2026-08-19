use crate::ast::parsed::Offset;
use crate::compositor::types::{
    AbsoluteContent, AbsoluteElement, AbsolutePage, DominantBaseline, TextAnchor,
};
use crate::render_config::RenderConfig;
use crate::renderer::new_types::{SvgDocument, SvgElement, SvgKind, SvgVariant};
use click_targets::{
    render_measure_click_target, render_note_click_target, render_playback_cursor_target,
    render_secondary_click_target,
};
use directive_line::{render_directive_line, DirectiveLineArgs};
use glyph_renderers::{
    render_bar_line, render_chord_symbol, render_horizontal_line, render_lyric, render_lyric_line,
    render_multi_measure_rest, render_note_dash, render_note_head, render_percussion_hit,
    render_rest, render_tie_or_slur, render_tuplet_bracket, render_underline, DotState,
    NoteRenderParams,
};

mod click_targets;
mod directive_line;
mod glyph_renderers;

pub fn render_new(pages: &[AbsolutePage], config: &RenderConfig) -> Vec<SvgDocument> {
    pages.iter().map(|page| render_page(page, config)).collect()
}

fn render_page(page: &AbsolutePage, config: &RenderConfig) -> SvgDocument {
    let row_height = config.row_height as f32;
    let lyric_font_size = config.lyric_font_size();
    let cjk_font_size = config.lyric_cjk_font_size();
    let notes_font_size = config.notes_font_size();
    let chords_font_size = config.chords_font_size();
    let note_number_width = config.note_number_width as f32;

    let params = RenderElementParams {
        row_height,
        lyric_font_size,
        cjk_font_size,
        notes_font_size,
        chords_font_size,
        note_number_width,
        directive_row_offset: config.directive_row_offset,
    };

    let elements = page
        .elements
        .iter()
        .flat_map(|elem| render_element(elem, &params))
        .collect();

    SvgDocument {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements,
    }
}

struct RenderElementParams {
    row_height: f32,
    lyric_font_size: f32,
    cjk_font_size: f32,
    notes_font_size: f32,
    chords_font_size: f32,
    note_number_width: f32,
    directive_row_offset: Offset,
}

fn render_element(elem: &AbsoluteElement, params: &RenderElementParams) -> Vec<SvgElement> {
    let RenderElementParams {
        row_height,
        lyric_font_size,
        cjk_font_size,
        notes_font_size,
        chords_font_size,
        note_number_width,
        directive_row_offset,
    } = params;
    match &elem.content {
        AbsoluteContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
            double_dotted,
        } => render_note_head(
            elem,
            pitch,
            accidental,
            *octave,
            &DotState::new(*dotted, *double_dotted),
            &NoteRenderParams {
                base_font_size: notes_font_size,
                note_number_width,
            },
        ),
        AbsoluteContent::Rest {
            dotted,
            double_dotted,
        } => render_rest(
            elem,
            &DotState::new(*dotted, *double_dotted),
            notes_font_size,
            note_number_width,
        ),
        AbsoluteContent::NoteDash {
            dotted,
            double_dotted,
        } => render_note_dash(
            elem,
            &DotState::new(*dotted, *double_dotted),
            note_number_width,
        ),
        AbsoluteContent::MultiMeasureRest { count, width } => {
            render_multi_measure_rest(elem, *count, *width, row_height, notes_font_size)
        }
        AbsoluteContent::ChordSymbol {
            text,
            dotted,
            double_dotted,
        } => render_chord_symbol(
            elem,
            text,
            &DotState::new(*dotted, *double_dotted),
            chords_font_size,
        ),
        AbsoluteContent::PercussionHit => render_percussion_hit(elem, notes_font_size),
        AbsoluteContent::Underline { width, level: _ } => render_underline(elem, width),
        AbsoluteContent::TieOrSlur { kind: _, width } => {
            render_tie_or_slur(elem, width, row_height)
        }
        AbsoluteContent::TupletBracket { label, width } => {
            render_tuplet_bracket(elem, label, *width, row_height, notes_font_size)
        }
        AbsoluteContent::BarLine { height } => render_bar_line(elem, height),
        AbsoluteContent::HorizontalLine { width } => render_horizontal_line(elem, width),
        AbsoluteContent::Lyric { text, .. } => {
            render_lyric(elem, text, lyric_font_size, cjk_font_size)
        }
        AbsoluteContent::LyricLine(s) => render_lyric_line(elem, s, lyric_font_size, cjk_font_size),
        content => render_overlay_element(elem, content, *directive_row_offset),
    }
}

/// The text/overlay half of [`render_element`]'s dispatch, split out for length.
fn render_overlay_element(
    elem: &AbsoluteElement,
    content: &AbsoluteContent,
    directive_row_offset: Offset,
) -> Vec<SvgElement> {
    match content {
        AbsoluteContent::Text {
            content,
            font_size,
            anchor,
            baseline,
            font,
            weight,
            italic,
        } => vec![render_text_content(
            elem,
            content,
            TextContentStyle {
                font_size: *font_size,
                anchor: *anchor,
                baseline: *baseline,
                font: *font,
                weight: *weight,
                italic: *italic,
            },
        )],
        AbsoluteContent::MeasureHighlight { width, height } => {
            vec![render_highlight_rect(elem, *width, *height, false)]
        }
        AbsoluteContent::ErrorHighlight { width, height } => {
            vec![render_highlight_rect(elem, *width, *height, true)]
        }
        AbsoluteContent::MeasureClickTarget {
            width,
            height,
            measure_index,
            measure_index_end,
        } => render_measure_click_target(elem, *width, *height, *measure_index, *measure_index_end),
        AbsoluteContent::PlaybackCursorTarget {
            width,
            height,
            source_part_index,
            note_id,
        } => render_playback_cursor_target(elem, *width, *height, *source_part_index, *note_id),
        AbsoluteContent::NoteClickTarget {
            width,
            height,
            source_part_index,
            note_id,
        } => render_note_click_target(elem, *width, *height, *source_part_index, *note_id),
        AbsoluteContent::PartLabelClickTarget { .. }
        | AbsoluteContent::LyricClickTarget { .. }
        | AbsoluteContent::LyricLabelClickTarget { .. } => {
            render_secondary_click_target(elem, content)
        }
        AbsoluteContent::DirectiveLine {
            bar_number,
            label,
            spans,
            spans_x_offset,
            label_x_offset,
            apply_row_offset,
        } => render_directive_line(
            elem,
            &DirectiveLineArgs {
                bar_number,
                label,
                spans,
                spans_x_offset: *spans_x_offset,
                label_x_offset: *label_x_offset,
                apply_row_offset: *apply_row_offset,
                directive_row_offset,
            },
        ),
        _ => Vec::new(),
    }
}

#[derive(Clone, Copy)]
struct TextContentStyle {
    font_size: f32,
    anchor: TextAnchor,
    baseline: DominantBaseline,
    font: crate::compositor::types::FontFamily,
    weight: crate::compositor::types::FontWeight,
    italic: bool,
}

fn render_text_content(
    elem: &AbsoluteElement,
    content: &str,
    style: TextContentStyle,
) -> SvgElement {
    SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Text),
        kind: SvgKind::Text {
            content: content.to_string(),
            font_size: style.font_size,
            anchor: style.anchor,
            baseline: style.baseline,
            font: style.font,
            weight: style.weight,
            italic: style.italic,
        },
    }
}

fn render_highlight_rect(
    elem: &AbsoluteElement,
    width: f32,
    height: f32,
    is_error: bool,
) -> SvgElement {
    SvgElement {
        x: elem.x,
        y: elem.y,
        variant: None,
        kind: if is_error {
            SvgKind::ErrorRect { width, height }
        } else {
            SvgKind::Rect { width, height }
        },
    }
}
