//! Conversions from the internal [`jianpu_generator`] renderer types to the
//! `*Out` types in [`crate::svg_types`] that get serialized across the wasm
//! boundary. Split out from `svg_types.rs` to keep that file under the
//! repo's max-file-lines limit.
use jianpu_generator::{
    compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor},
    renderer::new_types::{SvgDocument, SvgElement, SvgKind, Tag, TransparentRectRole, TspanData},
};

use crate::svg_types::{
    DominantBaselineOut, FontFamilyOut, FontWeightOut, SvgDocumentOut, SvgElementOut, SvgKindOut,
    TagOut, TextAnchorOut, TransparentRectRoleOut, TspanOut,
};

pub(crate) fn svg_document_to_out(doc: &SvgDocument) -> SvgDocumentOut {
    SvgDocumentOut {
        width_pt: doc.width_pt,
        height_pt: doc.height_pt,
        elements: doc.elements.iter().map(svg_element_to_out).collect(),
    }
}

fn tspan_to_out(span: &TspanData) -> TspanOut {
    TspanOut {
        content: span.content.clone(),
        bold: span.bold,
        italic: span.italic,
        underline: span.underline,
        font_size: span.font_size,
    }
}

fn svg_element_to_out(el: &SvgElement) -> SvgElementOut {
    SvgElementOut {
        x: el.x,
        y: el.y,
        variant: el.variant.map(|variant| variant.as_str().to_string()),
        kind: svg_kind_to_out(&el.kind),
    }
}

fn text_anchor_to_out(anchor: &TextAnchor) -> TextAnchorOut {
    match anchor {
        TextAnchor::Start => TextAnchorOut::Start,
        TextAnchor::Middle => TextAnchorOut::Middle,
        TextAnchor::End => TextAnchorOut::End,
    }
}

fn dominant_baseline_to_out(baseline: &DominantBaseline) -> DominantBaselineOut {
    match baseline {
        DominantBaseline::Middle => DominantBaselineOut::Middle,
        DominantBaseline::Hanging => DominantBaselineOut::Hanging,
        DominantBaseline::Ideographic => DominantBaselineOut::Ideographic,
    }
}

fn font_family_to_out(font: &FontFamily) -> FontFamilyOut {
    match font {
        FontFamily::Monospace => FontFamilyOut::Monospace,
        FontFamily::SansSerif => FontFamilyOut::SansSerif,
        FontFamily::Title => FontFamilyOut::Title,
    }
}

fn font_weight_to_out(weight: &FontWeight) -> FontWeightOut {
    match weight {
        FontWeight::Normal => FontWeightOut::Normal,
        FontWeight::Bold => FontWeightOut::Bold,
    }
}

fn tag_to_out(tag: &Tag) -> TagOut {
    match tag {
        Tag::Measure { index, end } => TagOut::Measure {
            index: *index,
            end: *end,
        },
        Tag::BarNumber { index, end } => TagOut::BarNumber {
            index: *index,
            end: *end,
        },
        Tag::SectionLabel { label } => TagOut::SectionLabel {
            label: label.clone(),
        },
        Tag::Note {
            source_part_index,
            note_id,
        } => TagOut::Note {
            source_part_index: *source_part_index,
            note_id: *note_id,
        },
        Tag::PartLabel {
            source_part_index,
            measure_index_start,
            measure_index_end,
        } => TagOut::PartLabel {
            source_part_index: *source_part_index,
            measure_index_start: *measure_index_start,
            measure_index_end: *measure_index_end,
        },
        Tag::Lyric {
            source_part_index,
            note_id,
            verse,
        } => TagOut::Lyric {
            source_part_index: *source_part_index,
            note_id: *note_id,
            verse: *verse,
        },
        Tag::LyricLabel {
            source_part_index,
            verse,
            measure_index_start,
            measure_index_end,
        } => TagOut::LyricLabel {
            source_part_index: *source_part_index,
            verse: *verse,
            measure_index_start: *measure_index_start,
            measure_index_end: *measure_index_end,
        },
    }
}

fn transparent_rect_role_to_out(role: &TransparentRectRole) -> TransparentRectRoleOut {
    match role {
        TransparentRectRole::MeasureClickTarget => TransparentRectRoleOut::MeasureClickTarget,
        TransparentRectRole::BarNumberClickTarget => TransparentRectRoleOut::BarNumberClickTarget,
        TransparentRectRole::SectionLabelBackground => {
            TransparentRectRoleOut::SectionLabelBackground
        }
        TransparentRectRole::SectionLabelClickTarget => {
            TransparentRectRoleOut::SectionLabelClickTarget
        }
        TransparentRectRole::NoteClickTarget => TransparentRectRoleOut::NoteClickTarget,
        TransparentRectRole::PartLabelClickTarget => TransparentRectRoleOut::PartLabelClickTarget,
        TransparentRectRole::LyricClickTarget => TransparentRectRoleOut::LyricClickTarget,
        TransparentRectRole::LyricLabelClickTarget => TransparentRectRoleOut::LyricLabelClickTarget,
    }
}

fn svg_kind_to_out(kind: &SvgKind) -> SvgKindOut {
    match kind {
        SvgKind::Text {
            content,
            font_size,
            anchor,
            baseline,
            font,
            weight,
            italic,
            underline,
        } => SvgKindOut::Text {
            content: content.clone(),
            font_size: *font_size,
            anchor: text_anchor_to_out(anchor),
            baseline: dominant_baseline_to_out(baseline),
            font: font_family_to_out(font),
            weight: font_weight_to_out(weight),
            italic: *italic,
            underline: *underline,
        },
        SvgKind::Line {
            x2,
            y2,
            stroke_width,
        } => SvgKindOut::Line {
            x2: *x2,
            y2: *y2,
            stroke_width: *stroke_width,
        },
        SvgKind::Circle { r } => SvgKindOut::Circle { r: *r },
        SvgKind::Path {
            control_x,
            control_y,
            end_x,
            end_y,
            stroke_width,
        } => SvgKindOut::Path {
            control_x: *control_x,
            control_y: *control_y,
            end_x: *end_x,
            end_y: *end_y,
            stroke_width: *stroke_width,
        },
        SvgKind::Rect { width, height } => SvgKindOut::Rect {
            width: *width,
            height: *height,
        },
        SvgKind::ErrorRect { width, height } => SvgKindOut::ErrorRect {
            width: *width,
            height: *height,
        },
        SvgKind::PlaybackCursorRect { width, height } => SvgKindOut::PlaybackCursorRect {
            width: *width,
            height: *height,
        },
        SvgKind::TransparentRect {
            width,
            height,
            role,
        } => SvgKindOut::TransparentRect {
            width: *width,
            height: *height,
            role: transparent_rect_role_to_out(role),
        },
        SvgKind::TextWithTspans {
            font_size,
            anchor,
            baseline,
            font,
            spans,
        } => text_with_tspans_to_out(*font_size, anchor, baseline, font, spans),
        SvgKind::Group { children, tag } => SvgKindOut::Group {
            children: children.iter().map(svg_element_to_out).collect(),
            tag: tag.as_ref().map(tag_to_out),
        },
    }
}

/// The `SvgKind::TextWithTspans` arm of [`svg_kind_to_out`]'s dispatch, split
/// out to keep that function under clippy's line-count limit.
fn text_with_tspans_to_out(
    font_size: f32,
    anchor: &TextAnchor,
    baseline: &DominantBaseline,
    font: &FontFamily,
    spans: &[TspanData],
) -> SvgKindOut {
    SvgKindOut::TextWithTspans {
        font_size,
        anchor: text_anchor_to_out(anchor),
        baseline: dominant_baseline_to_out(baseline),
        font: font_family_to_out(font),
        spans: spans.iter().map(tspan_to_out).collect(),
    }
}
