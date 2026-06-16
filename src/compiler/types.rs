use crate::ast::parsed::JianPuPitch;
use crate::error::JianPuError;

#[derive(Debug, Clone)]
pub struct MeasureBlock {
    pub rows: Vec<MeasureRow>,
    pub decorations: Vec<Decoration>,
    /// Errors collected during grouping for this measure.
    /// Non-empty when the measure's source had recoverable parse errors.
    pub errors: Vec<JianPuError>,
}

impl PartialEq for MeasureBlock {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.decorations == other.decorations
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureRow {
    pub id: RowId,
    pub label: String,
    pub elements: Vec<ColumnElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RowId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnElement {
    pub column: u32,
    pub content: ElementContent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementContent {
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
        from_column: u32,
        to_column: u32,
        last_head_column: u32,
        level: u32,
    },
    BarLine,
    /// Visual dash rendered after a note head for each extra beat of duration (e.g. `1-`).
    NoteDash,
    Lyric(String),
}

/// The full logical extent of one slur or tie arc across measures.
/// Resolved into grid arc elements by the layout stage.
#[derive(Debug, Clone, PartialEq)]
pub struct SlurSpan {
    pub part_index: usize,
    pub from_measure: usize, // 0-indexed global measure index
    pub from_column: u32,    // measure-relative column of the opening note
    pub to_measure: usize,
    pub to_column: u32, // measure-relative column of the closing note
}

/// Return value of `compiler::compile`.
#[derive(Debug, Clone, PartialEq)]
pub struct CompileResult {
    pub blocks: Vec<MeasureBlock>,
    pub slur_spans: Vec<SlurSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decoration {
    Bpm(u32),
    TimeSignature { numerator: u32, denominator: u32 },
    SectionLabel(String),
    BarNumber(u32),
}
