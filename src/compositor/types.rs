use crate::ast::parsed::JianPuPitch;

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
        octave: i8,
        dotted: bool,
    },
    Rest {
        dotted: bool,
    },
    ChordSymbol(String),
    Underline {
        width: f32,
        level: u32,
    },
    TieOrSlur {
        width: f32,
    },
    BarLine {
        height: f32,
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
