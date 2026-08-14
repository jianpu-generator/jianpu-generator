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
    /// Resolved `merge_duplicate_measures_across_parts=` setting in effect on the
    /// measure this block was compiled from (see `consolidator::consolidate_block`).
    pub merge_duplicate_measures_across_parts: bool,
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

impl MeasureRow {
    /// The `note_id` of this row's first sounding element, if any — used to
    /// pick a single representative identity for a collapsed
    /// `MultiMeasureRest` row (see `merge_rest_run`) and by
    /// `midi::timing::note_timings_seconds`, which reads it back off the
    /// compiled block to stay in agreement with whatever the row carries.
    pub fn first_note_id(&self) -> Option<usize> {
        self.elements.iter().find_map(|el| el.note_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RowId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnElement {
    pub column: u32,
    pub content: ElementContent,
    /// Identity of the sounding note/rest this element belongs to, unique per
    /// `(source_part_index, note_id)` across the whole score (not reset per
    /// measure). Shared by a note's dash/tie-continuation columns so they
    /// highlight together during playback. `None` for elements that never
    /// sound on their own (lyrics, bar lines, underlines). A `MultiMeasureRest`
    /// collapsed-rest-run glyph reuses the `note_id` of the first underlying
    /// measure's rest for the part it stands in for (see `merge_rest_run`),
    /// so the whole run still highlights as one note during playback.
    pub note_id: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementContent {
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
    },
    /// A single wide rest bar standing in for `count` consecutive
    /// all-rest source measures (cross-measure collapsing).
    MultiMeasureRest {
        count: usize,
    },
    ChordSymbol {
        text: String,
        dotted: bool,
        double_dotted: bool,
    },
    PercussionHit,
    Underline {
        from_column: u32,
        to_column: u32,
        last_head_column: u32,
        level: u32,
    },
    BarLine,
    /// Visual dash rendered after a note head for each extra beat of duration (e.g. `1-`).
    /// `dotted`/`double_dotted` are true when the dash came from a `-.`/`-..`
    /// (dotted-beat/double-dotted-beat) extension rather than a plain `-`, and are
    /// rendered with trailing dot(s) to match.
    NoteDash {
        dotted: bool,
        double_dotted: bool,
    },
    /// A syllable for one verse (0-indexed) of a `notes+lyrics` part.
    Lyric {
        text: String,
        verse: usize,
    },
    /// One verse's (0-indexed) full text line for a standalone `lyrics` part —
    /// adurational, not tied to any note, rendered as a single left-aligned
    /// block spanning the whole measure rather than one syllable per column.
    LyricLine {
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

/// The full logical extent of one tuplet bracket over a contiguous run of
/// tuplet-tagged notes/rests sharing the same ratio. Unlike `SlurSpan`,
/// never crosses a measure or system boundary — tuplets can't span lines
/// (see **Tuplet** in `ARCHITECTURE.md`) — so one `measure_index` suffices.
#[derive(Debug, Clone, PartialEq)]
pub struct TupletSpan {
    pub part_index: usize,
    pub measure_index: usize, // 0-indexed position in the final `CompileResult.blocks` list, after rest-run merging
    pub from_column: u32,     // measure-relative column of the first tuplet-tagged note/rest
    pub to_column: u32,       // measure-relative column of the last tuplet-tagged note/rest
    pub label: String,        // the tuplet's printed digit, e.g. "3" for a triplet
}

/// Return value of `compiler::compile`.
#[derive(Debug, Clone, PartialEq)]
pub struct CompileResult {
    pub blocks: Vec<MeasureBlock>,
    pub slur_spans: Vec<SlurSpan>,
    pub tuplet_spans: Vec<TupletSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decoration {
    DirectiveLine {
        label: Option<String>,
        bar_number: Option<u32>,
        key: Option<String>,
        bpm: Option<u32>,
        time_signature: Option<(u32, u32)>,
    },
}
