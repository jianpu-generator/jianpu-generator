pub(crate) mod expand;
pub(crate) mod highlight;
pub mod layout;
pub(crate) mod slur_placement;
pub mod types;

pub use layout::layout;
pub use types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, Header, LayoutOptions, PostArcGridContent,
    VAlign,
};

pub(crate) const PAGE_MARGIN: f32 = 25.0;

#[cfg(test)]
mod tests;
