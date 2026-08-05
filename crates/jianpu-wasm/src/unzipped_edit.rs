use crate::types::{LyricsVerseRangesOut, PartMeasureRangesOut, SpanOut, UnzippedEditResponse};
use jianpu_generator::unzipped_edit::{self as unzipped_edit_impl, UnzippedEditError};

/// Fetches declared part abbreviations, in `# parts` declaration order, to
/// zip positionally with the core crate's `UnzippedExtractOutput::part_measure_ranges`
/// (one inner `Vec` per declared part, indexed by declaration order — see
/// the core crate's `extract_unzipped_text` doc comment). Reuses the same
/// public `list_part_declarations_from_source` helper `part_declarations.rs`
/// already calls elsewhere in this crate, rather than threading declarations
/// through the core crate's (crate-private) `resolve_document_context`.
fn part_abbreviations(source: &str) -> Vec<String> {
    jianpu_generator::list_part_declarations_from_source(source, "input.jianpu", &[])
        .map(|declarations| {
            declarations
                .into_iter()
                .map(|declaration| declaration.abbreviation)
                .collect()
        })
        .unwrap_or_default()
}

fn part_measure_ranges_out(
    source: &str,
    part_measure_ranges: Vec<Vec<(usize, usize)>>,
) -> Vec<PartMeasureRangesOut> {
    part_abbreviations(source)
        .into_iter()
        .zip(part_measure_ranges)
        .map(|(abbreviation, ranges)| PartMeasureRangesOut {
            abbreviation,
            ranges: ranges
                .into_iter()
                .map(|(start, end)| SpanOut { start, end })
                .collect(),
        })
        .collect()
}

/// Flattens the core crate's per-part `Vec<Vec<LyricsVerseRanges>>` (one
/// inner `Vec` per declared part, in declaration order — matching
/// [`part_measure_ranges_out`]'s zip) into a flat, self-describing list, one
/// entry per tagged `[Abbrev:lyrics:N]` verse block actually present.
fn lyrics_verse_ranges_out(
    source: &str,
    lyrics_verse_ranges: Vec<Vec<unzipped_edit_impl::LyricsVerseRanges>>,
) -> Vec<LyricsVerseRangesOut> {
    part_abbreviations(source)
        .into_iter()
        .zip(lyrics_verse_ranges)
        .flat_map(|(abbreviation, verses)| {
            verses.into_iter().map(move |verse| LyricsVerseRangesOut {
                abbreviation: abbreviation.clone(),
                verse_number: verse.verse_number,
                ranges: verse
                    .measure_ranges
                    .into_iter()
                    .map(|(start, end)| SpanOut { start, end })
                    .collect(),
            })
        })
        .collect()
}

pub(crate) fn extract_unzipped_text_response(source: &str) -> UnzippedEditResponse {
    match unzipped_edit_impl::extract_unzipped_text(source) {
        Ok(output) => UnzippedEditResponse::Ok {
            part_measure_ranges: part_measure_ranges_out(source, output.part_measure_ranges),
            lyrics_verse_ranges: lyrics_verse_ranges_out(source, output.lyrics_verse_ranges),
            text: output.text,
        },
        Err(UnzippedEditError::UnknownPart) => UnzippedEditResponse::UnknownPart,
        // `MalformedHeader`/`UnexpectedLyricsBlock` have no dedicated wasm
        // response status per the Phase 4 design; fold them into the
        // generic `Err` variant.
        Err(
            UnzippedEditError::ParseFailed
            | UnzippedEditError::MalformedHeader
            | UnzippedEditError::UnexpectedLyricsBlock,
        ) => UnzippedEditResponse::Err,
    }
}

/// Unzipped-view "Format" action: breaks each measure onto its own line
/// within every block. Returns fresh `part_measure_ranges`/
/// `lyrics_verse_ranges` against the reformatted `text`, exactly like
/// [`extract_unzipped_text_response`].
pub(crate) fn format_unzipped_text_response(
    source: &str,
    unzipped_text: &str,
) -> UnzippedEditResponse {
    match unzipped_edit_impl::format_unzipped_text(source, unzipped_text) {
        Ok(output) => UnzippedEditResponse::Ok {
            part_measure_ranges: part_measure_ranges_out(source, output.part_measure_ranges),
            lyrics_verse_ranges: lyrics_verse_ranges_out(source, output.lyrics_verse_ranges),
            text: output.text,
        },
        Err(UnzippedEditError::UnknownPart) => UnzippedEditResponse::UnknownPart,
        Err(
            UnzippedEditError::ParseFailed
            | UnzippedEditError::MalformedHeader
            | UnzippedEditError::UnexpectedLyricsBlock,
        ) => UnzippedEditResponse::Err,
    }
}

/// `merge_unzipped_text` returns the full updated `.jianpu` source (not
/// unzipped text), so there is no unzipped-text byte space for
/// `part_measure_ranges`/`lyrics_verse_ranges` to index into here — callers
/// re-fetch ranges via a follow-up `extract_unzipped_text` call against the
/// merged source (per the frontend design in the Unzipped View plan's Phase
/// 6, where the render-request hook already re-extracts on every source
/// change). Always empty for this response, unlike
/// `extract_unzipped_text_response`'s.
pub(crate) fn merge_unzipped_text_response(
    source: &str,
    unzipped_text: &str,
) -> UnzippedEditResponse {
    match unzipped_edit_impl::merge_unzipped_text(source, unzipped_text) {
        Ok(text) => UnzippedEditResponse::Ok {
            text,
            part_measure_ranges: Vec::new(),
            lyrics_verse_ranges: Vec::new(),
        },
        Err(UnzippedEditError::UnknownPart) => UnzippedEditResponse::UnknownPart,
        Err(
            UnzippedEditError::ParseFailed
            | UnzippedEditError::MalformedHeader
            | UnzippedEditError::UnexpectedLyricsBlock,
        ) => UnzippedEditResponse::Err,
    }
}
