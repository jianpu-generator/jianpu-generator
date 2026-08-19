//! Click-target types split out of `types.rs` to keep it under the repo's
//! max-line-count lint — re-exported via `types.rs`'s `pub use`, so every
//! existing `crate::grid_layout::types::MeasureClickTarget`-style path still
//! resolves unchanged.

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

/// Invisible hit target laid over one measure's own rendered bar number,
/// which sits in its system's shared directive row above the musical rows
/// `MeasureClickTarget` already covers — without this, hovering/clicking a
/// bar number falls through to nothing. Only emitted where a bar number is
/// actually drawn (`make_decoration_row`'s `should_emit`). `column` is an
/// exact grid column (not a fractional bound like `MeasureClickTarget`'s)
/// since the bar number is a small text element, not a whole measure body —
/// its rendered width is measured at resolve time (`resolve_bar_number_click_target`).
#[derive(Debug, Clone)]
pub struct BarNumberClickTarget {
    pub row: usize,
    pub column: u32,
    pub measure_index: usize,
    pub measure_index_end: usize,
}

/// Invisible hit target laid over a part's `RowLabel` text, spanning that
/// part's own sub-rows (see `playback_cursor::part_row_ranges`) within the
/// fixed-width label region (columns `0..LABEL_COLS`). Clicking or
/// drag-selecting it is a shortcut for selecting every note/rest that part
/// sounds across the whole system the label sits in — `measure_index_start`/
/// `measure_index_end` give that system's full measure range, mirroring how
/// `MeasureClickTarget::measure_index`/`measure_index_end` scope a measure
/// click.
#[derive(Debug, Clone)]
pub struct PartLabelClickTarget {
    pub row_start: usize,
    pub row_end: usize,
    pub source_part_index: usize,
    pub measure_index_start: usize,
    pub measure_index_end: usize,
}
