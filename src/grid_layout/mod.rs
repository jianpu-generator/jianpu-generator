pub(crate) mod expand;
pub mod layout;
pub mod types;

pub use layout::layout;
pub use types::{GridContent, GridElement, GridPage, GridRow, HAlign, Header, VAlign};

pub(crate) const PAGE_MARGIN: f32 = 25.0;

#[cfg(test)]
mod tests;
