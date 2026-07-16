use crate::ast::parsed::{KeyChange, Offset, Syllable};
use crate::error::{Diagnostic, RecoverableError, Span, Warning};

// ── Public final types ────────────────────────────────────────────────────────

/// Default `row_height` in points, used when unset in `# metadata`.
pub const DEFAULT_ROW_HEIGHT: u32 = 24;
/// Default `max_measures_per_system`, used when unset in `# metadata`.
pub const DEFAULT_MAX_MEASURES_PER_SYSTEM: u32 = 4;
/// Default `label_width` in points, used when unset in `# metadata`.
pub const DEFAULT_LABEL_WIDTH: u32 = 40;
/// Default `note_number_width` in points, used when unset in `# metadata`.
pub const DEFAULT_NOTE_NUMBER_WIDTH: u32 = 8;
/// Default `parts_list_columns`, used when unset in `# metadata`.
pub const DEFAULT_PARTS_LIST_COLUMNS: u32 = 4;
/// Default `merge_duplicate_measures_across_parts`, used when unset in `# metadata`.
pub const DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS: bool = true;
/// Default `hide_resting_parts`, used when unset in `# metadata`.
pub const DEFAULT_HIDE_RESTING_PARTS: bool = true;
/// Default `hide_system_dividers`, used when unset in `# metadata`.
pub const DEFAULT_HIDE_SYSTEM_DIVIDERS: bool = false;
/// Default `section_label_offset`, used when unset in `# metadata`.
pub const DEFAULT_SECTION_LABEL_OFFSET: Offset = Offset { x: 0, y: 0 };

/// Default `lyrics_font_size` in points: 60% of `row_height`, used when unset in `# metadata`.
pub fn default_lyrics_font_size(row_height: u32) -> u32 {
    (row_height as f32 * 0.6).round() as u32
}

#[derive(Clone)]
pub struct Metadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    /// Row height in points. Controls font sizes, dot radii, and all vertical spacing. Default: 24.
    pub row_height: u32,
    /// Maximum number of measures per system line before wrapping. Default: 4.
    pub max_measures_per_system: u32,
    /// Left margin reserved for part labels in points. Default: 40.
    pub label_width: u32,
    /// Estimated rendered width of a single digit note number (0–9) in points. Default: 8.
    pub note_number_width: u32,
    /// Number of columns in the parts list header. Default: 4.
    pub parts_list_columns: u32,
    /// Lyrics font size in points. Default: 60% of row_height.
    pub lyrics_font_size: u32,
    /// When `false`, identical measure rows from different parts are no longer merged
    /// into one row (see `consolidator::consolidate`). Default: `true`.
    pub merge_duplicate_measures_across_parts: bool,
    /// When `false`, an all-rest part is no longer omitted from a measure that has other
    /// parts with real content (see `compiler::compile_measure`). Default: `true`.
    pub hide_resting_parts: bool,
    /// When `true`, the horizontal divider line drawn between systems is omitted (see
    /// `grid_layout::layout`). Default: `false`.
    pub hide_system_dividers: bool,
    /// Translation in points applied to a rendered section label, after layout (see
    /// `renderer::new_renderer::render_directive_line`). Default: `(0, 0)`.
    pub section_label_offset: Offset,
}

#[derive(Clone)]
pub struct Notes {
    pub events: Vec<NoteEvent>,
}

/// One verse's syllables for a single measure.
#[derive(Clone)]
pub struct Lyrics {
    pub syllables: Vec<Syllable>,
}

#[derive(Clone)]
pub struct PartSlice {
    pub name: Option<String>,
    pub kind: crate::ast::parsed::PartKind,
    pub soundfont: crate::ast::parsed::Soundfont,
    pub volume: u8,
    pub octave_offset: i8,
    pub notes: Notes,
    /// One entry per verse, in order. Empty when this part has no lyrics this measure.
    pub lyrics: Vec<Lyrics>,
    /// True when this slice's source measure had at least one `Diagnostic::Error`.
    /// The compiler uses this to drop incoming cross-measure tie/slur arcs.
    pub has_error: bool,
    /// Abbreviation of the group whose `[GroupAbbrev]` broadcast produced this
    /// measure's content, when this part didn't override it with its own line.
    pub group_provenance: Option<String>,
}

#[derive(Clone)]
pub struct MultiPartMeasure {
    pub time_signature: Option<TimeSignature>,
    pub bpm: Option<u32>,
    pub key: Option<KeyChange>,
    pub label: Option<String>,
    /// `dcalcoda` on this measure: after playing it, playback restarts from measure 0.
    pub dc_al_coda: bool,
    /// `tocoda` on this measure: on the second pass only, playback cuts to the `coda` measure.
    pub to_coda: bool,
    /// `coda` on this measure: playback resumes here on the second pass.
    pub coda: bool,
    /// `segno` on this measure: marks the measure `dsalcoda`/`dsalfine` jumps back to.
    pub segno: bool,
    /// `dsalcoda` on this measure: after playing it, playback restarts from the `segno` measure.
    pub ds_al_coda: bool,
    /// `dcalfine` on this measure: after playing it, playback restarts from measure 0 and stops at `fine`.
    pub dc_al_fine: bool,
    /// `fine` on this measure: on the second pass only, playback stops here.
    pub fine: bool,
    /// `dsalfine` on this measure: after playing it, playback restarts from the `segno` measure and stops at `fine`.
    pub ds_al_fine: bool,
    pub parts: Vec<PartRow>,
    /// Byte range of this measure's note events in the original source.
    /// Used to map editor cursor position to a measure index.
    pub source_span: Span,
    /// Diagnostics collected during grouping for this measure.
    /// Non-empty triggers a colored overlay in the SVG renderer.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone)]
pub enum PartRow {
    Timed(PartSlice),
}

impl PartRow {
    pub fn name(&self) -> Option<&String> {
        self.slice().name.as_ref()
    }

    pub fn slice(&self) -> &PartSlice {
        match self {
            PartRow::Timed(s) => s,
        }
    }

    pub fn slice_mut(&mut self) -> &mut PartSlice {
        match self {
            PartRow::Timed(s) => s,
        }
    }
}

pub(crate) enum GroupedTrack {
    Timed(GroupedPart),
}

impl GroupedTrack {
    pub(crate) fn measure_count(&self) -> usize {
        match self {
            GroupedTrack::Timed(part) => part.measures.len(),
        }
    }

    pub(crate) fn track_name(&self) -> &Option<String> {
        match self {
            GroupedTrack::Timed(part) => &part.name,
        }
    }
}

#[derive(Clone)]
pub struct Score {
    pub metadata: Metadata,
    pub measures: Vec<MultiPartMeasure>,
    /// Document-level diagnostics (e.g. metadata parse errors), not tied to any measure.
    pub document_diagnostics: Vec<Diagnostic>,
    /// Resolved playback order from a `# sequence` section, if present and
    /// valid: each span is a labeled section's inclusive measure-index range
    /// in `measures`, in the order they should play. Mutually exclusive with
    /// the D.C./D.S. al Coda/Fine navigation markers on `MultiPartMeasure`.
    pub sequence: Option<Vec<SequenceSpan>>,
}

/// A labeled section's resolved measure range, in written-score order: from
/// the labeled measure up to (but not including) the next labeled measure,
/// or the end of the score.
#[derive(Clone)]
pub struct SequenceSpan {
    pub label: String,
    /// Inclusive index into `Score.measures`.
    pub start: usize,
    /// Inclusive index into `Score.measures`.
    pub end: usize,
}

// ── Intermediate grouper types (not part of the public API) ─────────────────

pub(crate) struct MeasureDirectives {
    pub(crate) time_signature: Option<TimeSignature>,
    pub(crate) bpm: Option<u32>,
    pub(crate) key: Option<KeyChange>,
    pub(crate) label: Option<String>,
    pub(crate) dc_al_coda: bool,
    pub(crate) to_coda: bool,
    pub(crate) coda: bool,
    pub(crate) segno: bool,
    pub(crate) ds_al_coda: bool,
    pub(crate) dc_al_fine: bool,
    pub(crate) fine: bool,
    pub(crate) ds_al_fine: bool,
}

pub(crate) struct GroupedScore {
    pub(crate) measure_directives: Vec<MeasureDirectives>,
    pub(crate) parts: Vec<GroupedTrack>,
    pub(crate) per_measure_parse_errors: Vec<Option<RecoverableError>>,
}

pub(crate) struct GroupedMeasure {
    pub(crate) notes: Notes,
    pub(crate) source_span: Span,
    /// Tie-aware syllables paired to this measure's lyric slots, one entry per
    /// verse. Set for `NotesWithLyrics` parts during grouping.
    pub(crate) paired_lyrics: Vec<Vec<Syllable>>,
    /// Recoverable lyrics underflow/overflow for this measure, one per verse that has one.
    pub(crate) lyrics_error: Vec<Warning>,
    /// Recoverable beat overflow for this measure (notes trimmed), if any.
    pub(crate) beat_overflow_error: Option<Warning>,
    /// Recoverable error from `-` used after a rest in this measure, if any.
    pub(crate) dash_after_rest_error: Option<RecoverableError>,
    /// Grouping diagnostics: dotted-eighth RecoverableErrors and half-bar-boundary Warnings.
    pub(crate) dotted_eighth_errors: Vec<Diagnostic>,
    /// Chord parse diagnostics: promoted kinds are Error, others are Warning.
    pub(crate) chord_errors: Vec<Diagnostic>,
    /// Recoverable lex error from an unexpected character on the notes line, if any.
    pub(crate) lex_error: Option<RecoverableError>,
    /// Recoverable error from a malformed lyrics line (e.g. empty lyrics line), if any.
    pub(crate) lyrics_parse_error: Option<RecoverableError>,
    /// Recoverable error from `-` at the start of a measure with no preceding event, if any.
    pub(crate) extension_no_preceding_event_error: Option<RecoverableError>,
    /// Abbreviation of the group whose `[GroupAbbrev]` broadcast produced this
    /// measure's content, when this part didn't override it with its own line.
    pub(crate) group_provenance: Option<String>,
}

pub(crate) struct GroupedPart {
    pub(crate) name: Option<String>,
    pub(crate) kind: crate::ast::parsed::PartKind,
    pub(crate) soundfont: crate::ast::parsed::Soundfont,
    pub(crate) volume: u8,
    pub(crate) octave_offset: i8,
    pub(crate) measures: Vec<GroupedMeasure>,
}

// ── Shared note types ─────────────────────────────────────────────────────────

#[path = "grouped_notes.rs"]
mod grouped_notes;
pub use grouped_notes::{
    GroupedChordNote, GroupedNote, GroupedPercussionHit, GroupedRest, NoteEvent, TimeSignature,
};
