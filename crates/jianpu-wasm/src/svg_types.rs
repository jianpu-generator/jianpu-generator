use jianpu_generator::{
    compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor},
    renderer::new_types::{SvgDocument, SvgElement, SvgKind, Tag, TransparentRectRole, TspanData},
};
use serde::Serialize;
use tsify::Tsify;

#[derive(Debug, Clone, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct SvgDocumentOut {
    pub width_pt: f32,
    pub height_pt: f32,
    pub elements: Vec<SvgElementOut>,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct SvgElementOut {
    pub x: f32,
    pub y: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    pub kind: SvgKindOut,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum TransparentRectRoleOut {
    MeasureClickTarget,
    SectionLabelBackground,
    SectionLabelClickTarget,
    NoteClickTarget,
    PartLabelClickTarget,
    LyricClickTarget,
    LyricLabelClickTarget,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum SvgKindOut {
    Text {
        content: String,
        font_size: f32,
        anchor: TextAnchorOut,
        baseline: DominantBaselineOut,
        font: FontFamilyOut,
        weight: FontWeightOut,
        italic: bool,
    },
    Line {
        x2: f32,
        y2: f32,
        stroke_width: f32,
    },
    Circle {
        r: f32,
    },
    Path {
        control_x: f32,
        control_y: f32,
        end_x: f32,
        end_y: f32,
        stroke_width: f32,
    },
    Rect {
        width: f32,
        height: f32,
    },
    ErrorRect {
        width: f32,
        height: f32,
    },
    PlaybackCursorRect {
        width: f32,
        height: f32,
    },
    TransparentRect {
        width: f32,
        height: f32,
        role: TransparentRectRoleOut,
    },
    TextWithTspans {
        font_size: f32,
        anchor: TextAnchorOut,
        baseline: DominantBaselineOut,
        spans: Vec<TspanOut>,
    },
    Group {
        children: Vec<SvgElementOut>,
        tag: Option<TagOut>,
    },
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct TspanOut {
    pub content: String,
    pub bold: bool,
    pub italic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum TagOut {
    Measure {
        index: usize,
        end: usize,
    },
    SectionLabel {
        label: String,
    },
    Note {
        source_part_index: usize,
        note_id: usize,
    },
    PartLabel {
        source_part_index: usize,
        measure_index_start: usize,
        measure_index_end: usize,
    },
    Lyric {
        source_part_index: usize,
        note_id: usize,
        verse: usize,
    },
    LyricLabel {
        source_part_index: usize,
        verse: usize,
        measure_index_start: usize,
        measure_index_end: usize,
    },
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum TextAnchorOut {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum DominantBaselineOut {
    Middle,
    Hanging,
    Ideographic,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum FontFamilyOut {
    Monospace,
    SansSerif,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum FontWeightOut {
    Normal,
    Bold,
}

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
        } => SvgKindOut::Text {
            content: content.clone(),
            font_size: *font_size,
            anchor: text_anchor_to_out(anchor),
            baseline: dominant_baseline_to_out(baseline),
            font: font_family_to_out(font),
            weight: font_weight_to_out(weight),
            italic: *italic,
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
            spans,
        } => SvgKindOut::TextWithTspans {
            font_size: *font_size,
            anchor: text_anchor_to_out(anchor),
            baseline: dominant_baseline_to_out(baseline),
            spans: spans.iter().map(tspan_to_out).collect(),
        },
        SvgKind::Group { children, tag } => SvgKindOut::Group {
            children: children.iter().map(svg_element_to_out).collect(),
            tag: tag.as_ref().map(tag_to_out),
        },
    }
}
