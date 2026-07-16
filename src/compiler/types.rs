use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::error::Diagnostic;

#[derive(Debug, Clone, PartialEq)]
pub enum ArcKind {
    Slur,
    Tie,
}

/// Column width (in grid-layout columns) reserved for a collapsed
/// `MultiMeasureRest` block, regardless of how many source measures it
/// represents — wide enough to read visually as "more than one measure",
/// with the printed count communicating the actual number. Shared between
/// the compiler (which positions the block's `BarLine`) and the grid-layout
/// expansion step (which spans the `MultiMeasureRest` glyph to match).
pub const MULTI_MEASURE_REST_WIDTH: u32 = 8;

#[derive(Debug, Clone)]
pub struct MeasureBlock {
    pub rows: Vec<MeasureRow>,
    pub decorations: Vec<Decoration>,
    /// Diagnostics collected during grouping for this measure.
    pub diagnostics: Vec<Diagnostic>,
    /// Number of original source measures this block stands in for. `1` for
    /// every normal block; > 1 when a run of all-rest measures has been
    /// folded into a single `MultiMeasureRest` block.
    pub represents_measures: usize,
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
    /// Abbreviation of the group whose `[GroupAbbrev]` broadcast produced this
    /// row's content, when the source part didn't override it with its own line.
    /// Used by the consolidator to label a fully-merged unison row with the
    /// group's abbreviation instead of concatenating member labels.
    pub group_provenance: Option<String>,
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
    /// A single wide rest bar standing in for `count` consecutive
    /// all-rest source measures (cross-measure collapsing).
    MultiMeasureRest {
        count: usize,
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
    /// A syllable for one verse (0-indexed) of a `notes+lyrics` part.
    Lyric {
        text: String,
        verse: usize,
    },
}

/// The full logical extent of one slur or tie arc across measures.
/// Resolved into grid arc elements by the layout stage.
#[derive(Debug, Clone, PartialEq)]
pub struct SlurSpan {
    pub kind: ArcKind,
    pub part_index: usize,
    pub from_measure: usize, // 0-indexed position in the final `CompileResult.blocks` list, after rest-run merging
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
        segno: bool,
        ds_al_coda: bool,
        dc_al_fine: bool,
        fine: bool,
        ds_al_fine: bool,
    },
}
