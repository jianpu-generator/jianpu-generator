use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compiler::types::ArcKind;

#[path = "click_target_types.rs"]
mod click_target_types;
pub use click_target_types::{BarNumberClickTarget, MeasureClickTarget, PartLabelClickTarget};

#[derive(Debug, Clone)]
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
    pub measure_highlights: Vec<MeasureHighlight>,
    pub error_highlights: Vec<MeasureHighlight>,
    pub measure_click_targets: Vec<MeasureClickTarget>,
    pub playback_cursor_targets: Vec<PlaybackCursorTarget>,
    pub part_label_click_targets: Vec<PartLabelClickTarget>,
    pub lyric_click_targets: Vec<LyricClickTarget>,
    pub lyric_label_click_targets: Vec<LyricLabelClickTarget>,
    pub bar_number_click_targets: Vec<BarNumberClickTarget>,
}

/// One measure's extent within a `GridRow`'s musical column range, plus its
/// weight data — the raw material `column_geometry` uses to give denser
/// measures more pixel width than sparser ones, and to give individual
/// columns within a measure (e.g. a notehead vs. the dash after it)
/// different widths too. Shared by every row of a system (built once in
/// `expand::expand_system_to_rows`).
#[derive(Debug, Clone)]
pub struct MeasureColumnLayout {
    /// First musical grid column of this measure (absolute, i.e. already
    /// offset past the label region).
    pub start_col: u32,
    /// Number of grid columns this measure occupies (`block_column_width`).
    pub col_count: u32,
    /// Relative note-density weight of the whole measure vs. other measures
    /// in the system (`measure_note_weight`); higher = wider. Independent of
    /// `column_weights` below — dashes and bar lines don't affect this.
    pub weight: f32,
    /// Per-column weight within this measure (`measure_column_weights`),
    /// one entry per column in `start_col..start_col + col_count`; higher =
    /// wider relative to other columns in the same measure. Its sum is
    /// unrelated to `weight` — used only to split the measure's own pixel
    /// width across its columns, never to compare against other measures.
    pub column_weights: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct GridRow {
    pub height_pt: f32,
    pub column_count: u32,
    /// True for rows belonging to a system (note/lyric/decoration rows),
    /// which reserve `LABEL_COLS` columns at the start for the part label
    /// (see [`GridRow::column_geometry`]). False for header/footer/separator
    /// rows, which divide their full width evenly across `column_count`.
    pub has_label_region: bool,
    /// Per-measure column layout for this row's system, shared by every row
    /// in the system. Empty for rows with `has_label_region: false`.
    pub measure_layout: Vec<MeasureColumnLayout>,
    pub elements: Vec<GridElement>,
}

#[path = "column_geometry.rs"]
mod geometry;
pub use geometry::ColumnGeometry;

#[derive(Debug, Clone)]
pub struct GridElement {
    pub column: u32,
    pub column_span: u32,
    pub halign: HAlign,
    pub valign: VAlign,
    pub content: GridContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone)]
pub enum GridContent {
    /// Note head. `octave > 0` = dots above, `octave < 0` = dots below,
    /// `octave.abs()` = dot count. Octave rendered inline by the renderer;
    /// OctaveDot sub-rows exist for vertical spacing only.
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
    /// all-rest source measures.
    MultiMeasureRest {
        count: u32,
    },
    NoteDash {
        dotted: bool,
        double_dotted: bool,
    },
    /// Spacing-only row for octave dots. Resolver emits nothing for this.
    OctaveDot,
    ChordSymbol {
        text: String,
        dotted: bool,
        double_dotted: bool,
    },
    /// Percussion hit glyph (unpitched GM drum key), centered like a note head.
    PercussionHit,
    /// Durational underline. `level=0` half-beat, `level=1` quarter-beat.
    Underline {
        level: u32,
    },
    /// Same-system tie/slur arc: from center of from-column to center of to-column.
    TieOrSlur {
        kind: ArcKind,
    },
    /// Cross-system arc, first system: center of from-column to right edge of system.
    TieOrSlurTail {
        kind: ArcKind,
    },
    /// Cross-system arc, last system: left edge of system to center of to-column.
    TieOrSlurHead {
        kind: ArcKind,
    },
    /// Tuplet bracket spanning a contiguous run of tuplet-tagged notes/rests
    /// within one measure: from center of from-column to center of
    /// to-column, like `TieOrSlur`. Never crosses a measure or system
    /// boundary (see **Tuplet** in `ARCHITECTURE.md`), so unlike
    /// `TieOrSlur` there is no `Tail`/`Head` cross-system variant. `label`
    /// is the tuplet's printed digit, e.g. `"3"` for a triplet.
    TupletBracket {
        label: String,
    },
    /// Vertical bar line. `height_pt` baked in by grid layout layer.
    BarLine {
        height_pt: f32,
    },
    /// Full-width horizontal system separator.
    HorizontalLine,
    /// Part name at column=0, column_span=4 in the note-head sub-row.
    RowLabel(String),
    /// `source_part_index`/`note_id` identify the note this syllable is sung
    /// on, and `verse` (0-indexed) disambiguates which verse line it belongs
    /// to when a part has more than one (see
    /// `compiler::types::ElementContent::Lyric`), letting the SVG preview
    /// give this syllable its own click target independent of its note's —
    /// see `renderer::new_types::Tag::Lyric`.
    LyricSyllable {
        text: String,
        source_part_index: usize,
        note_id: usize,
        verse: usize,
    },
    /// One verse's full text line for a standalone `lyrics` part, rendered as
    /// a single left-aligned block spanning the whole measure (via
    /// `GridElement::column_span`) rather than positioned per note like
    /// `LyricSyllable`.
    LyricLine(String),
    DirectiveLine {
        label: Option<String>,
        bar_number: Option<u32>,
        key: Option<String>,
        bpm: Option<u32>,
        time_signature: Option<(u32, u32)>,
    },
    /// Generic styled text for header and footer rows.
    Text {
        content: String,
        font_size: f32,
        bold: bool,
        italic: bool,
    },
    /// The resolved `# sequence` playback order, rendered as "Sequence: "
    /// followed by each label (styled like an inline section label) joined
    /// by " › ".
    SequenceLine {
        entries: Vec<SequenceEntryInfo>,
        font_size: f32,
    },
}

/// One `# sequence` entry as rendered on the "Sequence: ..." header line:
/// a label, plus any part abbreviations that entry's `(-abbrev ...)` suffix
/// omits from that occurrence's MIDI/WAV playback (rendered parenthetically
/// next to the label; empty when the entry has no omissions).
#[derive(Debug, Clone)]
pub struct SequenceEntryInfo {
    pub label: String,
    pub omit_parts: Vec<String>,
}

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
    },
    SequenceLine {
        entries: Vec<SequenceEntryInfo>,
        font_size: f32,
    },
}

#[derive(Debug, Clone)]
pub struct PartListEntry {
    pub abbreviation: String,
    pub display_name: String,
    /// Abbreviations of the parts a group resolves to, shown in the legend as
    /// `V — Vocal (S1,S2,A1,A2)`. Empty for a plain part entry.
    pub members: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub part_list: Vec<PartListEntry>,
    pub parts_list_columns: u32,
    /// The resolved `# sequence` playback order, one entry per label
    /// reference (e.g. `["A", "B", "A"]`), if present. Rendered as a line
    /// near the top of the score; does not affect SVG/PDF measure order.
    pub sequence: Option<Vec<SequenceEntryInfo>>,
    /// Font size in points for `title` (see `Metadata::title_font_size`).
    pub title_font_size: f32,
    /// Font size in points for `subtitle` (see `Metadata::subtitle_font_size`).
    pub subtitle_font_size: f32,
    /// Font size in points for `author` (see `Metadata::author_font_size`).
    pub author_font_size: f32,
    /// Font size in points for the `# sequence` summary line (see
    /// `Metadata::sequence_font_size`).
    pub sequence_font_size: f32,
    /// Font size in points for the part-name legend entries (see
    /// `Metadata::part_legend_font_size`).
    pub part_legend_font_size: f32,
}

#[derive(Debug, Clone)]
pub struct MeasureHighlight {
    pub row_start: usize,
    pub row_end: usize,
    pub column_start: f32,
    pub column_end: f32,
}

/// The screen extent of one sounding note/rest (or one contiguous piece of
/// one, when a tie splits it across measures/systems), keyed by
/// `(source_part_index, note_id)` — the same identity used by
/// [`crate::midi::timing::NoteTiming`], so playback can look up which grid
/// position(s) to highlight for a given part's currently-sounding note.
#[derive(Debug, Clone)]
pub struct PlaybackCursorTarget {
    pub row_start: usize,
    /// Bottom row of the *playback-cursor* rect — may extend past
    /// `click_row_end` to cover this part's lyric verse row(s), so the "now
    /// playing" highlight visually covers the lyric text sounding with the
    /// note. Read only by `resolve_playback_cursor_target`.
    pub row_end: usize,
    /// Bottom row of the note's own *click/selection* target — always just
    /// this note row's own sub-rows, never a following lyric verse row (a
    /// lyric syllable has its own independent [`LyricClickTarget`], selected
    /// independently of its note). Read only by `resolve_note_click_target`.
    pub click_row_end: usize,
    pub column_start: f32,
    pub column_end: f32,
    pub source_part_index: usize,
    pub note_id: usize,
}

/// Invisible click/drag hit target for one lyric syllable, in its own row,
/// starting at the syllable's own grid column but widened to match its
/// note's full written column span (attack plus any dash-continuation
/// columns — see `grid_layout::click_targets::compute_all_lyric_click_targets`)
/// so a multi-beat note's syllable box covers its whole duration —
/// independent of its note's own [`PlaybackCursorTarget`]-derived click
/// target, so a click on the syllable itself always wins hit-testing there
/// (see `renderer::new_types::Tag::Lyric`).
#[derive(Debug, Clone)]
pub struct LyricClickTarget {
    pub row: usize,
    pub column_start: f32,
    pub column_end: f32,
    pub source_part_index: usize,
    pub note_id: usize,
    pub verse: usize,
}

/// Invisible hit target laid over one verse's `RowLabel` text (e.g.
/// "M:v1"), spanning that one verse row within the fixed-width label region
/// (columns `0..LABEL_COLS`) — the lyric-side mirror of
/// [`PartLabelClickTarget`]. Clicking or drag-selecting it is a shortcut for
/// selecting every syllable that verse sings across the whole system the
/// label sits in — `measure_index_start`/`measure_index_end` give that
/// system's full measure range, same as `PartLabelClickTarget`'s.
#[derive(Debug, Clone)]
pub struct LyricLabelClickTarget {
    pub row: usize,
    pub source_part_index: usize,
    pub verse: usize,
    pub measure_index_start: usize,
    pub measure_index_end: usize,
}
