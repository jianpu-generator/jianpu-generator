//! A second surface syntax for `.poentaro` documents, written in anyfix
//! expressions and saved as `.anyfix` (see `anyfix-syntax.md`).
//!
//! - [`read`] reads a document's anyfix tree into the document it describes.
//! - [`convert`] goes the other way, from the current syntax to `.anyfix`.
//!
//! Notes, lyrics and chords stay text in both directions: an `.anyfix`
//! score holds them in raw strings.

mod convert;
mod read;
pub(crate) mod tree;
mod words;

pub use convert::convert;
pub(crate) use convert::{key_value_field as field_text, part_kind_text, PartKindText};
pub(crate) use read::{read, read_metadata_field, source_part_declarations};
pub(crate) use words::{abbreviation, label, quoted};

/// The file extension of a document written in anyfix expressions.
pub const ANYFIX_EXTENSION: &str = ".anyfix";

#[cfg(test)]
mod convert_tests;
#[cfg(test)]
mod demo_svg_tests;
#[cfg(test)]
mod read_tests;
