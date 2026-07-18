mod content_conversion;
mod highlights;
pub mod resolve;
pub use resolve::{resolve, LyricFontSizes};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_multi_measure_rest;
#[cfg(test)]
mod tests_sequence_line;
