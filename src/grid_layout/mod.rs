pub(crate) mod expand;
pub(crate) mod highlight;
pub mod layout;
pub(crate) mod slur_placement;
pub mod types;

pub use layout::{layout, measure_column_boundaries};
pub use types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, Header, PostArcGridContent, VAlign,
};

pub(crate) const PAGE_MARGIN: f32 = 25.0;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_measure_spacing;
