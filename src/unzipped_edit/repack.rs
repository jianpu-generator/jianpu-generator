//! Phase 3b: repacks each declared part's flat Unzipped Edit text (already
//! split into per-part blocks by `parse`) into measures against the original
//! document's beat/token capacities.

use crate::ast::parsed::{PartDecl, PartKind, ScoreLineRole};
use crate::parser::score::token_parser::{
    parse_chord_line, parse_notes_line, parse_percussion_line, GroupStack,
};

use super::capacity::scan_measure_token_counts;
use super::parse::UnzippedBlocks;
use super::{capacity_at, fold_extensions, UnzippedDocumentContext, UnzippedEditError};

/// Greedily re-bars `units` (each a raw text slice paired with its weight —
/// quarter-beats for timed parts, one token = one unit for `Lyrics` parts)
/// against `capacities`, extending past `capacities`'s length with its last
/// entry once the original measure count is exceeded (this is the "shift
/// content, auto re-bar" growth behavior). If `capacities` is empty (a score
/// with zero original measures), every unit falls into a single measure 0
/// rather than looping forever, since there is no capacity to compare against.
///
/// Matches the plan's pseudocode exactly for genuine capacity exhaustion: a
/// single `if` check per unit (not a `while` loop), so a single unit that
/// overflows a measure's capacity is not split across measures — it spills
/// into that measure's text and surfaces as the existing beat-overflow
/// diagnostic on the next real parse.
///
/// A *zero*-capacity measure is skipped unconditionally, before the overflow
/// check: it arises only from a lyrics verse's per-measure occurrence
/// capacity (`scan_measure_token_counts`), where a measure structurally has
/// no slot for that verse at all — not from beat exhaustion — so a run of
/// several such measures in a row must all be skipped for one unit, not one
/// measure per unit. Both checks re-run in a loop (rather than the zero-skip
/// running once up front) because advancing past a *non-zero*-capacity
/// measure via the overflow branch can itself land on a zero-capacity
/// measure, which must also be skipped before this unit is placed — e.g. a
/// lyrics verse present in measures 0 and 2 but absent from measure 1: after
/// measure 0 fills up, overflow advances to measure 1 (capacity 0), which
/// must be skipped through to measure 2 before this token is placed, not
/// wrongly deposited into measure 1's empty slot. The loop is still bounded:
/// each iteration either consumes one entry of the zero-or-more extension
/// tail past `capacities.len()` (impossible, since only capacities with an
/// actual index below `capacities.len()` can be zero) or advances
/// `measure_index` past an original, finite `capacities` entry, and a
/// non-zero-capacity measure's overflow branch always terminates the loop by
/// resetting `current_beat` to 0.
fn repack_into_measures(units: &[(&str, u32)], capacities: &[u32]) -> Vec<String> {
    let mut buckets: Vec<Vec<&str>> = vec![Vec::new()];
    let mut current_beat: u32 = 0;
    let mut measure_index: usize = 0;
    for (text, weight) in units {
        loop {
            if measure_index < capacities.len() && capacity_at(capacities, measure_index) == 0 {
                measure_index += 1;
                buckets.push(Vec::new());
                continue;
            }
            if current_beat >= capacity_at(capacities, measure_index) {
                measure_index += 1;
                current_beat = 0;
                buckets.push(Vec::new());
                continue;
            }
            break;
        }
        // `measure_index` only ever advances past the bucket(s) just pushed
        // above, so it's always in bounds.
        if let Some(bucket) = buckets.get_mut(measure_index) {
            bucket.push(text);
        }
        current_beat += weight;
    }

    buckets.into_iter().map(|tokens| tokens.join(" ")).collect()
}

/// Repacks a Lyrics-role flat text (whitespace-tokenized, one token = one
/// unit) into measures by token-count `capacities`. Shared by `Lyrics`-kind
/// primary blocks and every lyrics verse of `NotesWithLyrics`/`Lyrics` parts.
fn repack_lyrics_text(flat_text: &str, capacities: &[u32]) -> Vec<String> {
    let units: Vec<(&str, u32)> = flat_text.split_whitespace().map(|tok| (tok, 1)).collect();
    repack_into_measures(&units, capacities)
}

/// Repack one declared part's flat unzipped-edit text into measures, dispatching
/// the token grammar by [`PartKind`].
fn repack_part(
    decl: &PartDecl,
    flat_text: &str,
    capacities: &[u32],
) -> Result<Vec<String>, UnzippedEditError> {
    match decl.kind {
        PartKind::Notes | PartKind::NotesWithLyrics => {
            let parsed = parse_notes_line(flat_text, 0, &mut GroupStack::default())
                .map_err(|_| UnzippedEditError::ParseFailed)?;
            let clusters = fold_extensions(parsed.events);
            let units: Vec<(&str, u32)> = clusters
                .iter()
                .map(|(span, duration)| (&flat_text[span.start..span.end], *duration))
                .collect();
            Ok(repack_into_measures(&units, capacities))
        }
        PartKind::Chords => {
            let parsed = parse_chord_line(flat_text, 0, &mut GroupStack::default())
                .map_err(|_| UnzippedEditError::ParseFailed)?;
            let clusters = fold_extensions(parsed.events);
            let units: Vec<(&str, u32)> = clusters
                .iter()
                .map(|(span, duration)| (&flat_text[span.start..span.end], *duration))
                .collect();
            Ok(repack_into_measures(&units, capacities))
        }
        PartKind::Percussion => {
            let parsed = parse_percussion_line(flat_text, 0, &mut GroupStack::default())
                .map_err(|_| UnzippedEditError::ParseFailed)?;
            let clusters = fold_extensions(parsed.events);
            let units: Vec<(&str, u32)> = clusters
                .iter()
                .map(|(span, duration)| (&flat_text[span.start..span.end], *duration))
                .collect();
            Ok(repack_into_measures(&units, capacities))
        }
        PartKind::Lyrics => Ok(repack_lyrics_text(flat_text, capacities)),
    }
}

/// One declared part's repacked lyrics verse: a specific 1-based
/// `verse_number` and its per-measure buckets, already resized to the
/// reconciled total measure count by the time reassembly reads it.
pub(super) struct VerseBuckets {
    pub(super) verse_number: usize,
    pub(super) buckets: Vec<String>,
}

/// `(part_buckets, verse_buckets_per_part)`, both indexed parallel to
/// `UnzippedDocumentContext::declarations` — see [`repack_all_parts`].
pub(super) type RepackedParts = (Vec<Vec<String>>, Vec<Vec<VerseBuckets>>);

/// Repack every declared part's flat primary text, and every tagged lyrics
/// verse present in `blocks`, into measures against the original document's
/// beat/token capacities.
pub(super) fn repack_all_parts(
    context: &UnzippedDocumentContext,
    blocks: &UnzippedBlocks,
    capacities: &[u32],
) -> Result<RepackedParts, UnzippedEditError> {
    let mut part_buckets: Vec<Vec<String>> = Vec::with_capacity(context.declarations.len());
    let mut verse_buckets_per_part: Vec<Vec<VerseBuckets>> =
        Vec::with_capacity(context.declarations.len());
    for (part_index, decl) in context.declarations.iter().enumerate() {
        let flat_text = blocks
            .primary
            .get(&decl.abbreviation)
            .map(String::as_str)
            .unwrap_or("");
        let part_capacities = if decl.kind == PartKind::Lyrics {
            scan_measure_token_counts(
                &context.score_content,
                &context.declarations,
                &context.resolved_groups,
                part_index,
                ScoreLineRole::Lyrics,
                0,
            )
        } else {
            capacities.to_vec()
        };
        part_buckets.push(repack_part(decl, flat_text, &part_capacities)?);

        let mut verses = Vec::new();
        if let Some(verse_texts) = blocks.lyrics_verses.get(&decl.abbreviation) {
            for (&verse_number, flat_verse_text) in verse_texts {
                let verse_capacities = scan_measure_token_counts(
                    &context.score_content,
                    &context.declarations,
                    &context.resolved_groups,
                    part_index,
                    ScoreLineRole::Lyrics,
                    verse_number - 1,
                );
                let buckets = repack_lyrics_text(flat_verse_text, &verse_capacities);
                verses.push(VerseBuckets {
                    verse_number,
                    buckets,
                });
            }
        }
        verse_buckets_per_part.push(verses);
    }
    Ok((part_buckets, verse_buckets_per_part))
}
