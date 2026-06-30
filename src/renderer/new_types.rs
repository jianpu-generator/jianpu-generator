use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgVariant {
    Text,
    NoteHead,
    NoteHeadAccidental,
    Rest,
    ChordSymbol,
    HorizontalLine,
    Underline,
    TieOrSlur,
    BarLine,
    Lyric,
    DirectiveLine,
}

impl SvgVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::NoteHead => "note-head",
            Self::NoteHeadAccidental => "note-head-accidental",
            Self::Rest => "rest",
            Self::ChordSymbol => "chord-symbol",
            Self::HorizontalLine => "horizontal-line",
            Self::Underline => "underline",
            Self::TieOrSlur => "tie-or-slur",
            Self::BarLine => "bar-line",
            Self::Lyric => "lyric",
            Self::DirectiveLine => "directive-line",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparentRectRole {
    MeasureClickTarget,
    SectionLabelBackground,
}

impl TransparentRectRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MeasureClickTarget => "measure-click-target-rect",
            Self::SectionLabelBackground => "section-label-bg",
        }
    }
}

#[derive(Debug)]
pub struct SvgDocument {
    pub width_pt: f32,
    pub height_pt: f32,
    pub elements: Vec<SvgElement>,
}

#[derive(Debug)]
pub struct SvgElement {
    pub x: f32,
    pub y: f32,
    pub variant: Option<SvgVariant>,
    pub kind: SvgKind,
}

#[derive(Debug)]
pub enum Tag {
    Measure { index: usize },
    SectionLabel { label: String },
}

#[derive(Debug)]

pub enum SvgKind {
    Text {
        content: String,
        font_size: f32,
        anchor: TextAnchor,
        baseline: DominantBaseline,
        font: FontFamily,
        weight: FontWeight,
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
        // Quadratic bezier: x/y from SvgElement; control and end vary
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
    /// Red semi-transparent overlay for erroneous measures (15% opacity).
    ErrorRect {
        width: f32,
        height: f32,
    },
    Group {
        children: Vec<SvgElement>,
        tag: Option<Tag>,
    },
    TransparentRect {
        width: f32,
        height: f32,
        role: TransparentRectRole,
    },
    TextWithTspans {
        font_size: f32,
        anchor: TextAnchor,
        baseline: DominantBaseline,
        spans: Vec<TspanData>,
    },
}

#[derive(Debug)]
pub struct TspanData {
    pub content: String,
    pub bold: bool,
    pub italic: bool,
    pub font_size: Option<f32>,
}
