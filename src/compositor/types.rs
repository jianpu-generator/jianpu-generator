use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compiler::types::ArcKind;

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
        accidental: Accidental,
        octave: i8,
        dotted: bool,
    },
    Rest {
        dotted: bool,
    },
    MultiMeasureRest {
        count: u32,
        width: f32,
    },
    ChordSymbol {
        text: String,
        dotted: bool,
    },
    NoteDash {
        dotted: bool,
    },
    PercussionHit,
    Underline {
        width: f32,
        level: u32,
    },
    TieOrSlur {
        kind: ArcKind,
        width: f32,
    },
    /// Tuplet bracket: short vertical ticks + horizontal line spanning
    /// `width`, with `label` (the tuplet digit, e.g. `"3"`) centered above
    /// the midpoint. See `GridContent::TupletBracket`.
    TupletBracket {
        label: String,
        width: f32,
    },
    BarLine {
        height: f32,
    },
    HorizontalLine {
        width: f32,
    },
    Lyric(String),
    /// A standalone `lyrics` part's whole verse line, left-aligned starting
    /// at the element's `x` rather than centered like [`AbsoluteContent::Lyric`].
    LyricLine(String),
    Text {
        content: String,
        font_size: f32,
        anchor: TextAnchor,
        baseline: DominantBaseline,
        font: FontFamily,
        weight: FontWeight,
        italic: bool,
    },
    MeasureHighlight {
        width: f32,
        height: f32,
    },
    /// Red semi-transparent overlay drawn over a measure with recoverable errors.
    ErrorHighlight {
        width: f32,
        height: f32,
    },
    MeasureClickTarget {
        width: f32,
        height: f32,
        measure_index: usize,
        measure_index_end: usize,
    },
    /// Background rect behind one part's sounding note/rest, toggled at
    /// playback time by the frontend rather than filled here (see
    /// `renderer::new_types::SvgKind::PlaybackCursorRect`).
    PlaybackCursorTarget {
        width: f32,
        height: f32,
        source_part_index: usize,
        note_id: usize,
    },
    /// Invisible click/drag hit target layered above `PlaybackCursorTarget`
    /// for the same note/rest, since that rect is `pointer-events: none`
    /// (its `fill` is owned exclusively by playback highlighting — see
    /// `renderer::new_types::TransparentRectRole::NoteClickTarget`).
    NoteClickTarget {
        width: f32,
        height: f32,
        source_part_index: usize,
        note_id: usize,
    },
    /// Invisible click/drag hit target laid over a part's `RowLabel` text
    /// (see `grid_layout::types::PartLabelClickTarget`), spanning that
    /// part's own sub-rows within the fixed-width label region. Clicking or
    /// drag-selecting it selects every note/rest that part sounds across
    /// `measure_index_start..=measure_index_end` (the whole system the
    /// label sits in).
    PartLabelClickTarget {
        width: f32,
        height: f32,
        source_part_index: usize,
        measure_index_start: usize,
        measure_index_end: usize,
    },
    DirectiveLine {
        /// Bar-number span, rendered as its own text element pinned to the
        /// line's start (offset 0) so it always precedes `label` and
        /// `spans`, regardless of their widths.
        bar_number: Option<TextSpan>,
        /// Section-label text, rendered as its own text/box element
        /// independent of `spans` (see `label_x_offset`) rather than as one
        /// of `spans`'s tspans, so it doesn't need to know their combined
        /// rendered width.
        label: Option<String>,
        /// Key/bpm/time-signature spans, i.e. everything on the line except
        /// `bar_number` and `label`.
        spans: Vec<TextSpan>,
        /// X offset (in points, from the line's start) where `spans`
        /// begins: right after `bar_number` when there is no `label`, or
        /// past `label`'s bounding box when there is, so the three
        /// elements never overlap regardless of their measured widths.
        spans_x_offset: f32,
        /// X offset (in points, from the line's start) where the
        /// independent `label` text element begins: past `bar_number`'s
        /// measured width when one is present, zero otherwise.
        label_x_offset: f32,
        /// Whether `directive_row_offset` should be applied to this line.
        /// `true` for ordinary directive lines; `false` for the `# sequence`
        /// summary header, which must not move.
        apply_row_offset: bool,
    },
}

#[derive(Debug, Clone)]
pub struct TextSpan {
    pub content: String,
    pub bold: bool,
    pub italic: bool,
    pub font_size: f32,
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
