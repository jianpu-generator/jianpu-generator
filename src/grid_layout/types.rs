use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compiler::types::ArcKind;
use crate::grid_layout::layout::LABEL_COLS;

#[derive(Debug, Clone)]
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
    pub measure_highlights: Vec<MeasureHighlight>,
    pub error_highlights: Vec<MeasureHighlight>,
    pub measure_click_targets: Vec<MeasureClickTarget>,
}

#[derive(Debug, Clone)]
pub struct MeasureClickTarget {
    pub row_start: usize,
    pub row_end: usize,
    pub column_start: f32,
    pub column_end: f32,
    pub measure_index: usize,
    /// Last original source measure index this click target represents. Equal to
    /// `measure_index` for an ordinary measure block; greater than `measure_index`
    /// for a merged multi-measure rest, so clicking it can highlight the whole span.
    pub measure_index_end: usize,
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
    pub elements: Vec<GridElement>,
}

impl GridRow {
    /// Column geometry for this row, given the usable page width and the
    /// score-wide fixed part-label width. Rows with `has_label_region: true`
    /// (system rows) get a label column width independent of the row's own
    /// musical density; other rows (headers, footers) divide the full width
    /// evenly as before.
    pub fn column_geometry(&self, usable_width_pt: f32, label_width_pt: f32) -> ColumnGeometry {
        if self.has_label_region {
            let music_cols = self.column_count - LABEL_COLS;
            ColumnGeometry {
                label_cols: LABEL_COLS,
                label_col_width: label_width_pt / LABEL_COLS as f32,
                label_width_pt,
                music_col_width: (usable_width_pt - label_width_pt) / music_cols as f32,
            }
        } else {
            let col_width = usable_width_pt / self.column_count as f32;
            ColumnGeometry {
                label_cols: 0,
                label_col_width: col_width,
                label_width_pt: 0.0,
                music_col_width: col_width,
            }
        }
    }
}

/// Resolves a grid column index to a pixel x-offset (from the row's left
/// edge) and column width, so the fixed-width label region (columns
/// `0..label_cols`) and the variable-width music region (`label_cols..`)
/// can share a `GridRow` without the label's rendered width depending on
/// how many musical columns the row has.
#[derive(Debug, Clone, Copy)]
pub struct ColumnGeometry {
    label_cols: u32,
    label_col_width: f32,
    label_width_pt: f32,
    music_col_width: f32,
}

impl ColumnGeometry {
    /// x-offset of the start of `column`, relative to the row's left edge.
    /// `column` may be fractional (e.g. a highlight's `column_start`); it is
    /// never expected to straddle the label/music boundary.
    pub fn x_start(&self, column: f32) -> f32 {
        if column < self.label_cols as f32 {
            column * self.label_col_width
        } else {
            self.label_width_pt + (column - self.label_cols as f32) * self.music_col_width
        }
    }

    /// Width of a single column at `column`.
    pub fn col_width(&self, column: f32) -> f32 {
        if column < self.label_cols as f32 {
            self.label_col_width
        } else {
            self.music_col_width
        }
    }
}

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
    },
    Rest {
        dotted: bool,
    },
    /// A single wide rest bar standing in for `count` consecutive
    /// all-rest source measures.
    MultiMeasureRest {
        count: u32,
    },
    NoteDash,
    /// Spacing-only row for octave dots. Resolver emits nothing for this.
    OctaveDot,
    ChordSymbol(String),
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
    /// Vertical bar line. `height_pt` baked in by grid layout layer.
    BarLine {
        height_pt: f32,
    },
    /// Full-width horizontal system separator.
    HorizontalLine,
    /// Part name at column=0, column_span=4 in the note-head sub-row.
    RowLabel(String),
    LyricSyllable(String),
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
    /// Generic styled text for header and footer rows.
    Text {
        content: String,
        font_size: f32,
        bold: bool,
        italic: bool,
    },
    /// The resolved `# sequence` playback order, rendered as "Sequence: "
    /// followed by each label (styled like an inline section label) joined
    /// by " → ".
    SequenceLine {
        entries: Vec<String>,
    },
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
    },
    Rest {
        dotted: bool,
    },
    /// A single wide rest bar standing in for `count` consecutive
    /// all-rest source measures.
    MultiMeasureRest {
        count: u32,
    },
    NoteDash,
    OctaveDot,
    ChordSymbol(String),
    PercussionHit,
    Underline {
        level: u32,
    },
    BarLine {
        height_pt: f32,
    },
    HorizontalLine,
    RowLabel(String),
    LyricSyllable(String),
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
    Text {
        content: String,
        font_size: f32,
        bold: bool,
        italic: bool,
    },
    SequenceLine {
        entries: Vec<String>,
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
    pub sequence: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct MeasureHighlight {
    pub row_start: usize,
    pub row_end: usize,
    pub column_start: f32,
    pub column_end: f32,
}
