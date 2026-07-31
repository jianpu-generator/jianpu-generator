use crate::ast::parsed::{KeyChange, Offset, Syllable};
use crate::error::{Diagnostic, RecoverableError, Span, Warning};

// ── Public final types ────────────────────────────────────────────────────────

/// Default `row_height` in points, used when unset in `# metadata`.
pub const DEFAULT_ROW_HEIGHT: u32 = 24;
/// Default `max_measures_per_system`, used when unset in `# metadata`.
pub const DEFAULT_MAX_MEASURES_PER_SYSTEM: u32 = 4;
/// Default `note_number_width` in points, used when unset in `# metadata`.
pub const DEFAULT_NOTE_NUMBER_WIDTH: u32 = 8;
/// Default `part_label_width_pt`, used when unset in `# metadata`.
pub const DEFAULT_PART_LABEL_WIDTH_PT: u32 = 40;
/// Default `parts_list_columns`, used when unset in `# metadata`.
pub const DEFAULT_PARTS_LIST_COLUMNS: u32 = 4;
/// Default `merge_duplicate_measures_across_parts`, used when unset in `# metadata`.
pub const DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS: bool = true;
/// Default `hide_resting_parts`, used when unset in `# metadata`.
pub const DEFAULT_HIDE_RESTING_PARTS: bool = true;
/// Default `hide_system_dividers`, used when unset in `# metadata`.
pub const DEFAULT_HIDE_SYSTEM_DIVIDERS: bool = false;
/// Default `directive_row_offset`, used when unset in `# metadata`.
pub const DEFAULT_DIRECTIVE_ROW_OFFSET: Offset = Offset { x: 0, y: 0 };

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
    /// Estimated rendered width of a single digit note number (0–9) in points. Default: 8.
    pub note_number_width: u32,
    /// Fixed width in points of the part-label column at the start of each system row,
    /// shared by every system in the score regardless of that system's musical density
    /// (see `grid_layout::types::GridRow::column_geometry`). Default: 40.
    pub part_label_width_pt: u32,
    /// Number of columns in the parts list header. Default: 4.
    pub parts_list_columns: u32,
    /// Lyrics font size in points. Default: 60% of row_height.
    pub lyrics_font_size: u32,
    /// Note head/rest/percussion-hit/tuplet-bracket font size in points. Default: `lyrics_font_size`.
    pub notes_font_size: u32,
    /// Chord symbol font size in points. Default: `lyrics_font_size`.
    pub chords_font_size: u32,
    /// Score-wide default for `merge_duplicate_measures_across_parts=`: when `false`,
    /// identical measure rows from different parts are no longer merged into one row
    /// (see `consolidator::consolidate`). Default: `true`. A `merge_duplicate_measures_across_parts=`
    /// directive line can override this from a given measure onward (see
    /// `MultiPartMeasure::merge_duplicate_measures_across_parts`).
    pub merge_duplicate_measures_across_parts: bool,
    /// Score-wide default for `hide_resting_parts=`: when `false`, an all-rest part is
    /// no longer omitted from a measure that has other parts with real content (see
    /// `compiler::compile_measure`). Default: `true`. A `hide_resting_parts=` directive
    /// line can override this from a given measure onward (see
    /// `MultiPartMeasure::hide_resting_parts`).
    pub hide_resting_parts: bool,
    /// When `true`, the horizontal divider line drawn between systems is omitted (see
    /// `grid_layout::layout`). Default: `false`.
    pub hide_system_dividers: bool,
    /// Translation in points applied to every rendered directive row (bar number, section
    /// label, key, bpm, time signature, nav markers), after layout (see
    /// `renderer::new_renderer::render_directive_line`). Not applied to the `# sequence`
    /// summary header. Default: `(0, 0)`.
    pub directive_row_offset: Offset,
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
    /// Copied from the source `GroupedMeasure::resolution_multiplier`: the factor
    /// every duration in `notes` was multiplied by during tuplet rescaling. `1`
    /// when the measure has no tuplets (the common case, a no-op). The compiler
    /// (`compiler::part_slice`) scales its column/underline arithmetic by this
    /// value so a rescaled measure still lays out correctly — see **Tuplet** in
    /// `ARCHITECTURE.md`.
    pub resolution_multiplier: u32,
    /// Copied from the source `GroupedMeasure::beat_group_size`: the quarter-beat
    /// width of one beam group under this measure's time signature (`4` for simple
    /// meters, `6` for compound meters like 6/8, 9/8, 12/8). The compiler scales
    /// this by `resolution_multiplier` before comparing against it, same as the
    /// other quarter-beat-grid constants.
    pub beat_group_size: u32,
}

#[derive(Clone)]
pub struct MultiPartMeasure {
    pub time_signature: Option<TimeSignature>,
    pub bpm: Option<u32>,
    pub key: Option<KeyChange>,
    pub label: Option<String>,
    /// Resolved value of `merge_duplicate_measures_across_parts=` in effect on this
    /// measure — either an override starting here or carried forward from an earlier
    /// measure or the `#metadata` default (see `consolidator::consolidate`).
    pub merge_duplicate_measures_across_parts: bool,
    /// Resolved value of `hide_resting_parts=` in effect on this measure — either an
    /// override starting here or carried forward from an earlier measure or the
    /// `#metadata` default (see `compiler::compile_measure`).
    pub hide_resting_parts: bool,
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
    /// in `measures`, in the order they should play.
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
    /// Individual part abbreviations to omit from this occurrence's MIDI/WAV
    /// playback (from a `# sequence` entry's `(-abbrev -abbrev ...)` suffix;
    /// any group abbreviation is expanded to its member parts here).
    pub omit_parts: Vec<String>,
    /// The `(-abbrev ...)` suffix's abbreviations as written — a group
    /// abbreviation is kept as-is, unexpanded — for display on the SVG/PDF
    /// "Sequence: ..." summary line, which shows the group's own label
    /// rather than spelling out its members.
    pub omit_parts_display: Vec<String>,
}

// ── Intermediate grouper types (not part of the public API) ─────────────────

pub(crate) struct MeasureDirectives {
    pub(crate) time_signature: Option<TimeSignature>,
    pub(crate) bpm: Option<u32>,
    pub(crate) key: Option<KeyChange>,
    pub(crate) label: Option<String>,
    pub(crate) merge_duplicate_measures_across_parts: bool,
    pub(crate) hide_resting_parts: bool,
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
    /// Factor by which every duration in this measure's events was multiplied by
    /// the tuplet-rescale pass so that tuplet ratios (e.g. 3-in-2) resolve to
    /// whole numbers. `1` when the measure has no tuplets (the common case).
    pub(crate) resolution_multiplier: u32,
    /// Quarter-beat width of one beam group under the time signature in effect
    /// for this measure — `4` (one quarter note) for simple meters, `6` (one
    /// dotted quarter) for compound meters like 6/8, 9/8, 12/8. Unscaled by
    /// `resolution_multiplier`; the compiler applies that scaling itself, the
    /// same way it does for the `4`/`6` constants used elsewhere.
    pub(crate) beat_group_size: u32,
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
