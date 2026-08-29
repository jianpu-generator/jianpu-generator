use crate::ast::parsed::{Accidental, JianPuPitch};

use crate::grid_layout::types::SequenceEntryInfo;

/// `GridContent` after arc variants have been resolved.
/// Used in the coordinate-resolver layer; arc variants are handled before this point.
#[derive(Debug, Clone)]
pub enum PostArcGridContent {
    NoteHead {
        pitch: JianPuPitch,
        accidental: Accidental,
        octave: i8,
        dotted: bool,
        double_dotted: bool,
    },
    Rest {
        dotted: bool,
        double_dotted: bool,
        implicit_fill: bool,
    },
    /// A single wide rest bar standing in for `count` consecutive
    /// all-rest source measures.
    MultiMeasureRest {
        count: u32,
    },
    NoteDash {
        dotted: bool,
        double_dotted: bool,
    },
    OctaveDot,
    ChordSymbol {
        text: String,
        dotted: bool,
        double_dotted: bool,
    },
    PercussionHit,
    Underline {
        level: u32,
    },
    BarLine {
        height_pt: f32,
    },
    HorizontalLine,
    RowLabel(String),
    LyricSyllable {
        text: String,
        source_part_index: usize,
        note_id: usize,
        verse: usize,
    },
    LyricLine(String),
    DirectiveLine {
        label: Option<String>,
        bar_number: Option<u32>,
        key: Option<String>,
        bpm: Option<u32>,
        time_signature: Option<(u32, u32)>,
    },
    Text {
        content: String,
        font_size: f32,
        bold: bool,
        italic: bool,
        is_title: bool,
    },
    SequenceLine {
        entries: Vec<SequenceEntryInfo>,
        font_size: f32,
    },
}
