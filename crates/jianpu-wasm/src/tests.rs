use super::*;

#[path = "tests_parts_and_measures.rs"]
mod tests_parts_and_measures;

#[cfg(feature = "pdf")]
#[path = "tests_pdf.rs"]
mod tests_pdf;

#[path = "tests_render.rs"]
mod tests_render;

#[path = "tests_share.rs"]
mod tests_share;

#[cfg(feature = "wav")]
#[path = "tests_wav.rs"]
mod tests_wav;

#[path = "group_diagnostics_tests.rs"]
mod group_diagnostics_tests;
