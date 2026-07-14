use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::error::Diagnostic;

#[derive(Debug, Clone, PartialEq)]
pub enum ArcKind {
    Slur,
    Tie,
}

#[derive(Debug, Clone)]
pub struct MeasureBlock {
    pub rows: Vec<MeasureRow>,
    pub decorations: Vec<Decoration>,
    /// Diagnostics collected during grouping for this measure.
    pub diagnostics: Vec<Diagnostic>,
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
    /// The original part index this row was compiled from, before consolidation.
    /// Used to look up slur arcs keyed by original part index.
    pub source_part_index: usize,
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
        accidental: Accidental,
        octave: i8,
        dotted: bool,
    },
    Rest {
        dotted: bool,
    },
    ChordSymbol(String),
    PercussionHit,
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
    pub kind: ArcKind,
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
    DirectiveLine {
        label: Option<String>,
        bar_number: Option<u32>,
        key: Option<String>,
        bpm: Option<u32>,
        time_signature: Option<(u32, u32)>,
        dc_al_coda: bool,
        to_coda: bool,
        coda: bool,
    },
}
