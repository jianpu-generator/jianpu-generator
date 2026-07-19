use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compiler::types::ArcKind;

#[derive(Debug, Clone)]
pub struct AbsolutePage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub elements: Vec<AbsoluteElement>,
}

#[derive(Debug, Clone)]
pub struct AbsoluteElement {
    pub x: f32,
    pub y: f32,
    pub content: AbsoluteContent,
}

#[derive(Debug, Clone)]
pub enum AbsoluteContent {
    NoteHead {
        pitch: JianPuPitch,
        accidental: Accidental,
        octave: i8,
        dotted: bool,
    },
    Rest {
        dotted: bool,
    },
    MultiMeasureRest {
        count: u32,
        width: f32,
    },
    ChordSymbol(String),
    PercussionHit,
    Underline {
        width: f32,
        level: u32,
    },
    TieOrSlur {
        kind: ArcKind,
        width: f32,
    },
    BarLine {
        height: f32,
    },
    HorizontalLine {
        width: f32,
    },
    Lyric(String),
    Text {
        content: String,
        font_size: f32,
        anchor: TextAnchor,
        baseline: DominantBaseline,
        font: FontFamily,
        weight: FontWeight,
        italic: bool,
    },
    MeasureHighlight {
        width: f32,
        height: f32,
    },
    /// Red semi-transparent overlay drawn over a measure with recoverable errors.
    ErrorHighlight {
        width: f32,
        height: f32,
    },
    MeasureClickTarget {
        width: f32,
        height: f32,
        measure_index: usize,
        measure_index_end: usize,
    },
    /// Background rect behind one part's sounding note/rest, toggled at
    /// playback time by the frontend rather than filled here (see
    /// `renderer::new_types::SvgKind::NoteHighlightRect`).
    NoteHighlightTarget {
        width: f32,
        height: f32,
        source_part_index: usize,
        note_id: usize,
    },
    DirectiveLine {
        label: Option<String>,
        spans: Vec<TextSpan>,
        /// X offset (in points, from the line's start) where the vector
        /// Segno glyph should be drawn, if a Segno marker is present. `None`
        /// when there is no Segno marker on this line.
        segno_icon_offset: Option<f32>,
        /// Whether `directive_row_offset` should be applied to this line.
        /// `true` for ordinary directive lines; `false` for the `# sequence`
        /// summary header, which must not move.
        apply_row_offset: bool,
    },
}

#[derive(Debug, Clone)]
pub struct TextSpan {
    pub content: String,
    pub bold: bool,
    pub italic: bool,
    pub font_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DominantBaseline {
    Middle,
    Hanging,
    Ideographic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontFamily {
    Monospace,
    SansSerif,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
}
