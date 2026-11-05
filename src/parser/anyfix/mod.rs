//! A second surface syntax for `.poentaro` documents, written in anyfix
//! expressions and saved as `.anyfix` (see `anyfix-syntax.md`).
//!
//! - [`lower`] reads an `.anyfix` document and writes the same document in
//!   the current syntax, which the existing parser then reads.
//! - [`convert`] goes the other way, from the current syntax to `.anyfix`.
//!
//! Notes, lyrics and chords stay text in both directions: an `.anyfix`
//! score holds them in raw strings.

mod convert;
mod lower;
mod source_map;
mod words;

pub use convert::convert;
pub use lower::{lower, Lowered, LoweringError};
pub use source_map::{Segment, SourceMap};

/// The file extension of a document written in anyfix expressions.
pub const ANYFIX_EXTENSION: &str = ".anyfix";

#[cfg(test)]
mod convert_tests;
#[cfg(test)]
mod demo_svg_tests;
#[cfg(test)]
mod lower_tests;
#[cfg(test)]
mod source_map_tests;
