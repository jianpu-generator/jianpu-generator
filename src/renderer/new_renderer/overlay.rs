use crate::ast::parsed::Offset;
use crate::compositor::types::{AbsoluteContent, AbsoluteElement, DominantBaseline, TextAnchor};
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

use super::click_targets::{
    render_measure_click_target, render_note_click_target, render_playback_cursor_target,
    render_secondary_click_target,
};
use super::directive_line::{render_directive_line, DirectiveLineArgs};

/// The text/overlay half of [`render_element`]'s dispatch, split out for length.
pub(super) fn render_overlay_element(
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
            underline,
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
                underline: *underline,
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
        | AbsoluteContent::LyricLabelClickTarget { .. }
        | AbsoluteContent::BarNumberClickTarget { .. }
        | AbsoluteContent::BarLineClickTarget { .. } => {
            render_secondary_click_target(elem, content)
        }
        AbsoluteContent::DirectiveLine { .. } => {
            render_directive_line_overlay(elem, content, directive_row_offset)
        }
        // See the matching comment in `render_note_glyph`: `render_element`'s
        // outer match exhaustively routes this fixed set of variants here.
        _ => Vec::new(),
    }
}

/// The [`AbsoluteContent::DirectiveLine`] arm of [`render_overlay_element`]'s
/// dispatch — split out to keep that function under clippy's line-count
/// limit.
fn render_directive_line_overlay(
    elem: &AbsoluteElement,
    content: &AbsoluteContent,
    directive_row_offset: Offset,
) -> Vec<SvgElement> {
    let AbsoluteContent::DirectiveLine {
        bar_number,
        bar_number_font_family,
        label,
        label_font_size,
        label_bold,
        label_italic,
        label_underline,
        label_font_family,
        label_box_height,
        spans,
        spans_font_family,
        spans_x_offset,
        label_x_offset,
        apply_row_offset,
    } = content
    else {
        return Vec::new();
    };
    render_directive_line(
        elem,
        &DirectiveLineArgs {
            bar_number,
            bar_number_font_family: *bar_number_font_family,
            label,
            label_font_size: *label_font_size,
            label_bold: *label_bold,
            label_italic: *label_italic,
            label_underline: *label_underline,
            label_font_family: *label_font_family,
            label_box_height: *label_box_height,
            spans,
            spans_font_family: *spans_font_family,
            spans_x_offset: *spans_x_offset,
            label_x_offset: *label_x_offset,
            apply_row_offset: *apply_row_offset,
            directive_row_offset,
        },
    )
}

#[derive(Clone, Copy)]
struct TextContentStyle {
    font_size: f32,
    anchor: TextAnchor,
    baseline: DominantBaseline,
    font: crate::compositor::types::FontFamily,
    weight: crate::compositor::types::FontWeight,
    italic: bool,
    underline: bool,
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
            underline: style.underline,
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
