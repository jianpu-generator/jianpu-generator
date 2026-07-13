use super::*;

#[path = "tests_render_filtering.rs"]
mod tests_render_filtering;
#[path = "tests_render_rendering.rs"]
mod tests_render_rendering;
#[cfg(feature = "pdf")]
#[path = "tests_render_split_pdf.rs"]
mod tests_render_split_pdf;
