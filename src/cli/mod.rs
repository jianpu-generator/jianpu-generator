use crate::error::{Diagnostic, IrrecoverableError, IrrecoverableErrorKind, Span};
use std::path::Path;

pub mod generate;

pub use generate::GenerateInput;

#[cfg(feature = "wav")]
pub static SF2_BYTES: &[u8] = include_bytes!("../../fonts/GeneralUser_GS.sf2");

#[cfg(feature = "pdf")]
pub fn default_pdf_fonts() -> crate::pdf::PdfFonts {
    crate::pdf::PdfFonts {
        sans_serif_sc: crate::fonts::TITLE_FONT_BYTES.to_vec(),
        sans_serif_tc: crate::fonts::SANS_SERIF_FONT_BYTES.to_vec(),
        monospace: crate::fonts::MONOSPACE_FONT_BYTES.to_vec(),
    }
}

/// Outcome of checking a `.jianpu` file for diagnostics.
pub struct CheckOutcome {
    /// `true` when the file parses with no errors (warnings are still allowed).
    pub ok: bool,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parse and group a `.jianpu` file, returning whether it has any errors and the
/// full list of collected diagnostics.
pub fn check(input: &Path) -> Result<CheckOutcome, IrrecoverableError> {
    let score = parse_and_group(input)?;
    let diagnostics = crate::collect_measure_diagnostics(&score);
    let ok = !diagnostics
        .iter()
        .any(|d| matches!(d, Diagnostic::Error(_)));
    Ok(CheckOutcome { ok, diagnostics })
}

pub(crate) fn read_source(input: &Path) -> Result<String, IrrecoverableError> {
    std::fs::read_to_string(input).map_err(|e| {
        IrrecoverableError::new(IrrecoverableErrorKind::IoReadFailed {
            span: Span::new(0, 0),
            path: input.to_path_buf(),
            source: e.to_string(),
        })
    })
}

pub(crate) fn parse_and_group(
    input: &Path,
) -> Result<crate::ast::grouped::Score, IrrecoverableError> {
    let content = read_source(input)?;
    let filename = input.to_string_lossy().to_string();
    let doc = crate::parser::parse(&content, &filename, &[]).map_err(|e| e.with_path(input))?;
    crate::grouper::group(doc).map_err(|e| e.with_path(input))
}

pub(crate) fn write_file(path: &Path, bytes: &[u8]) -> Result<(), IrrecoverableError> {
    std::fs::write(path, bytes).map_err(|e| {
        IrrecoverableError::new(IrrecoverableErrorKind::IoWriteFailed {
            span: Span::new(0, 0),
            path: path.to_path_buf(),
            source: e.to_string(),
        })
    })
}
