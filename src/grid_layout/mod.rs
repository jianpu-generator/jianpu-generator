pub(crate) mod click_targets;
pub(crate) mod expand;
pub(crate) mod highlight;
pub mod layout;
pub(crate) mod playback_cursor;
pub(crate) mod slur_placement;
pub(crate) mod system_walk;
pub(crate) mod tuplet_placement;
pub mod types;

pub use layout::layout;
pub use types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, Header, PostArcGridContent, VAlign,
};

pub(crate) const PAGE_MARGIN: f32 = 25.0;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_accidental_spacing;
#[cfg(test)]
mod tests_measure_spacing;
