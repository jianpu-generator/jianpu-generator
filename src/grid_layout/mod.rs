pub(crate) mod click_targets;
pub(crate) mod expand;
pub(crate) mod highlight;
pub mod layout;
pub(crate) mod playback_cursor;
pub(crate) mod slur_placement;
pub(crate) mod system_walk;
pub(crate) mod tuplet_placement;
pub mod types;

pub use layout::{layout, LayoutOutput};
pub use types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, Header, MeasureRange, PostArcGridContent,
    VAlign,
};

pub(crate) const PAGE_MARGIN: f32 = 25.0;

/// The music region's usable width in points: the page width minus its
/// margins on both sides, minus the fixed part-label column. Shared by
/// `layout::layout`'s overflow check and (via `GridRow::column_geometry`'s
/// own `usable_width_pt - label_width_pt` step, fed `page_width_pt -
/// 2*PAGE_MARGIN` by `coordinate_resolver::resolve`) every row's real
/// rendered geometry, so the overflow check compares against the same
/// number columns actually get split across.
pub(crate) fn usable_music_width_pt(page_width_pt: f32, label_width_pt: f32) -> f32 {
    page_width_pt - 2.0 * PAGE_MARGIN - label_width_pt
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_accidental_spacing;
#[cfg(test)]
mod tests_column_rod_spring;
#[cfg(test)]
mod tests_column_span_weight;
#[cfg(test)]
mod tests_dot_spacing;
#[cfg(test)]
mod tests_measure_spacing;
#[cfg(test)]
mod tests_measure_spacing_multi_measure_rest;
