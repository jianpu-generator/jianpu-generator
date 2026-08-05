//! [`format_unzipped_text`]: a best-effort formatter for the Unzipped Edit
//! view, breaking each measure onto its own line for readability. Newlines
//! within a block are purely cosmetic — `parse::split_unzipped_blocks`
//! collapses them back to single spaces on merge — so this is a pure
//! readability pass, not a semantic one.
//!
//! Implemented as merge-then-re-extract rather than inserting line breaks
//! into `unzipped_text` directly: only `merge_unzipped_text`'s repack/re-bar
//! pass actually knows where one measure ends and the next begins in a flat,
//! already-edited token stream (the same reason `merge_unzipped_text` itself
//! re-bars by beat/token capacity instead of trusting whitespace in the
//! input). Re-extracting afterwards also means the returned text/ranges are
//! already consistent with the merged `source`, exactly like a real edit
//! followed by a fresh `extract_unzipped_text` call.

use super::extract::extract_unzipped_text_with_separator;
use super::merge::merge_unzipped_text;
use super::{UnzippedEditError, UnzippedExtractOutput};

/// Formats `unzipped_text` (as currently shown in the Unzipped Edit view of
/// `source`) by breaking each measure onto its own line: merges it back into
/// `source` exactly as a real edit would, then re-extracts with each block's
/// measures newline-joined instead of space-joined. Errors identically to
/// [`super::merge_unzipped_text`] (an unknown/malformed block header, or an
/// internal parse failure).
pub fn format_unzipped_text(
    source: &str,
    unzipped_text: &str,
) -> Result<UnzippedExtractOutput, UnzippedEditError> {
    let merged_source = merge_unzipped_text(source, unzipped_text)?;
    extract_unzipped_text_with_separator(&merged_source, '\n')
}
