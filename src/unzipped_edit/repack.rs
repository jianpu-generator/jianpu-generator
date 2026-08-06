//! Phase 3b: repacks each declared part's flat Unzipped Edit text (already
//! split into per-part blocks by `parse`) into measures against the original
//! document's beat/token capacities.
//!
//! Notes/Chords/Percussion (and the Notes half of `notes+lyrics`) repack
//! against real, intrinsic beat capacity via [`repack_into_measures`].
//! Lyrics-role content has no such intrinsic capacity, so it's repacked by
//! [`repack_lyrics_via_diff`] instead: a diff of the edited text against the
//! occurrence's *original* per-measure tokens, which reproduces untouched
//! stretches exactly and only consults a capacity ceiling for genuinely new
//! content. See the parent module's doc comment for why a pure
//! "recompute capacity differently" approach can't work for lyrics.

use crate::ast::parsed::{PartDecl, PartKind, ScoreLineRole};
use crate::parser::score::token_parser::{
    parse_chord_line, parse_notes_line, parse_percussion_line, GroupStack,
};

use super::capacity::scan_measure_tokens;
use super::diff::{diff_tokens, DiffToken};
use super::parse::UnzippedBlocks;
use super::{capacity_at, fold_extensions, UnzippedDocumentContext, UnzippedEditError};

/// Advances `measure_index`/`current_weight`, pushing a fresh empty bucket
/// onto `buckets` for each measure advanced through, until `measure_index`
/// names a measure that can legally receive the next unit — i.e. until
/// `capacity_at(measure_index)` is non-zero and `current_weight` hasn't
/// already reached it. Capacity-source-agnostic (`capacity_of` is any `Fn`),
/// shared by [`repack_into_measures`] (beat/token capacities) and
/// [`repack_lyrics_via_diff`]'s diff-driven walk (existence/ceiling
/// capacities).
///
/// A *zero*-capacity measure is skipped unconditionally, before the overflow
/// check: it arises only from a lyrics verse's per-measure occurrence
/// capacity, where a measure structurally has no slot for that occurrence at
/// all — not from beat exhaustion — so a run of several such measures in a
/// row must all be skipped for one unit, not one measure per unit.
/// `original_len` bounds this zero-check to the occurrence's actual
/// (finite) original measure count, since capacity sources that extend
/// indefinitely past their own end (the "shift content, auto re-bar" growth
/// behavior) must never report `0` there — see `capacity_at`'s doc comment.
///
/// Both checks re-run in a loop (rather than the zero-skip running once up
/// front) because advancing past a non-zero-capacity measure via the
/// overflow branch can itself land on a zero-capacity measure, which must
/// also be skipped before this unit is placed. The loop is still bounded:
/// each iteration either consumes one entry of the zero-or-more extension
/// tail past `original_len` (impossible, since only measures with an actual
/// index below `original_len` can be zero) or advances `measure_index` past
/// an original, finite entry, and a non-zero-capacity measure's overflow
/// branch always terminates the loop by resetting `current_weight` to 0.
fn advance_to_placeable_measure<T>(
    buckets: &mut Vec<Vec<T>>,
    measure_index: &mut usize,
    current_weight: &mut u32,
    original_len: usize,
    capacity_of: &impl Fn(usize) -> u32,
) {
    loop {
        if *measure_index < original_len && capacity_of(*measure_index) == 0 {
            *measure_index += 1;
            buckets.push(Vec::new());
            continue;
        }
        if *current_weight >= capacity_of(*measure_index) {
            *measure_index += 1;
            *current_weight = 0;
            buckets.push(Vec::new());
            continue;
        }
        break;
    }
}

/// Greedily re-bars `units` (each a raw text slice paired with its weight —
/// quarter-beats for timed parts) against `capacities`, extending past
/// `capacities`'s length with its last entry once the original measure count
/// is exceeded (this is the "shift content, auto re-bar" growth behavior).
/// If `capacities` is empty (a score with zero original measures), every
/// unit falls into a single measure 0 rather than looping forever, since
/// there is no capacity to compare against.
///
/// A single unit that overflows a measure's capacity is not split across
/// measures — it spills into that measure's text and surfaces as the
/// existing beat-overflow diagnostic on the next real parse. See
/// [`advance_to_placeable_measure`] for the zero-capacity-skip/overflow
/// mechanics this delegates to.
fn repack_into_measures(units: &[(&str, u32)], capacities: &[u32]) -> Vec<String> {
    let mut buckets: Vec<Vec<&str>> = vec![Vec::new()];
    let mut current_beat: u32 = 0;
    let mut measure_index: usize = 0;
    let capacity_of = |index: usize| capacity_at(capacities, index);
    for (text, weight) in units {
        advance_to_placeable_measure(
            &mut buckets,
            &mut measure_index,
            &mut current_beat,
            capacities.len(),
            &capacity_of,
        );
        // `measure_index` only ever advances past the bucket(s) just pushed
        // above, so it's always in bounds.
        if let Some(bucket) = buckets.get_mut(measure_index) {
            bucket.push(text);
        }
        current_beat += weight;
    }

    buckets.into_iter().map(|tokens| tokens.join(" ")).collect()
}

/// Repacks one Lyrics-role occurrence's edited flat text into measures by
/// diffing it against that occurrence's own original per-measure tokens
/// (`original_tokens_per_measure`), rather than by greedily re-filling
/// against a flattened token-count capacity the way
/// [`repack_into_measures`] does. See the parent module's doc comment for
/// why: a verse's own per-measure token count isn't an intrinsic capacity
/// the way a time signature's beat budget is, so recomputing it differently
/// can't simultaneously reproduce exact original boundaries for unedited
/// content and allow local growth — only keeping the original boundaries
/// as ground truth and diffing against them can.
///
/// `ceiling` bounds how much room is available for genuinely new (inserted,
/// or reflowed-forward) content per measure — the corresponding Notes
/// line's onset count for a `notes+lyrics` verse, or verse 1's own token
/// count for a standalone `Lyrics`-kind part (including verse 1 itself).
/// `original_tokens_per_measure`'s own per-measure lengths independently
/// gate *existence*: `0` there means this occurrence has no slot in that
/// measure at all, regardless of what `ceiling` says.
pub(super) fn repack_lyrics_via_diff(
    original_tokens_per_measure: &[Vec<String>],
    ceiling: &[u32],
    edited_text: &str,
) -> Vec<String> {
    let original_flat: Vec<&str> = original_tokens_per_measure
        .iter()
        .flat_map(|tokens| tokens.iter().map(String::as_str))
        .collect();
    let owner_measure: Vec<usize> = original_tokens_per_measure
        .iter()
        .enumerate()
        .flat_map(|(measure_index, tokens)| tokens.iter().map(move |_| measure_index))
        .collect();
    let edited_tokens: Vec<&str> = edited_text.split_whitespace().collect();
    let script = diff_tokens(&original_flat, &owner_measure, &edited_tokens);

    let original_len = original_tokens_per_measure.len();
    // `0` only within the occurrence's own (finite) original range and only
    // where it structurally has no slot there; past that range, or wherever
    // it does have a slot, defer to `ceiling` (extending past `ceiling`'s
    // own end with `u32::MAX`, matching `capacity_at`'s empty-slice
    // fallback) — never merged into one capacity slice fed through
    // `capacity_at`, since both `original_tokens_per_measure` and `ceiling`
    // can legitimately end in a `0` entry, which `capacity_at`'s "extend
    // with the last entry" behavior would turn into an unbounded `0` tail.
    let capacity_of = |index: usize| -> u32 {
        let has_slot = original_tokens_per_measure
            .get(index)
            .map(|tokens| !tokens.is_empty())
            .unwrap_or(true);
        if !has_slot {
            0
        } else {
            ceiling.get(index).copied().unwrap_or(u32::MAX)
        }
    };

    let mut buckets: Vec<Vec<&str>> = vec![Vec::new()];
    let mut measure_index: usize = 0;
    let mut current_weight: u32 = 0;

    for token in &script {
        let edited_index = match *token {
            DiffToken::Equal {
                edited_index,
                owner_measure: token_owner,
            } if token_owner >= measure_index => {
                // Still-unedited-so-far stretch: snap forward to this
                // token's original home (only actually advancing/resetting
                // when `token_owner` is strictly past the current measure —
                // a run of several tokens sharing one original measure must
                // all land there without re-zeroing `current_weight`
                // between them) and place it unconditionally, with no
                // capacity check. This is what guarantees byte-for-byte
                // reproduction of untouched content — including a measure
                // whose own original token count already exceeds the
                // ceiling — and it's the only branch a fully unedited round
                // trip ever takes.
                if token_owner > measure_index {
                    measure_index = token_owner;
                    current_weight = 0;
                    while buckets.len() <= measure_index {
                        buckets.push(Vec::new());
                    }
                }
                current_weight += 1;
                edited_index
            }
            DiffToken::Equal { edited_index, .. } | DiffToken::Insert { edited_index } => {
                // Either genuinely new content, or original content whose
                // home measure has already been passed (earlier overflow
                // pushed `measure_index` beyond it) — both reflow forward
                // against the capacity ceiling like Notes/Chords already do.
                advance_to_placeable_measure(
                    &mut buckets,
                    &mut measure_index,
                    &mut current_weight,
                    original_len,
                    &capacity_of,
                );
                current_weight += 1;
                edited_index
            }
        };
        if let (Some(bucket), Some(&text)) = (
            buckets.get_mut(measure_index),
            edited_tokens.get(edited_index),
        ) {
            bucket.push(text);
        }
    }

    buckets.into_iter().map(|tokens| tokens.join(" ")).collect()
}

/// The onset count (folded, tie/extension-clustered — not raw whitespace
/// token count) of each already-repacked Notes-line measure string in
/// `notes_buckets`, for use as a `notes+lyrics` verse's capacity ceiling.
/// `0` on parse failure or a blank measure, rather than propagating an
/// error: a just-repacked measure is expected to always reparse cleanly, so
/// this is a defensive fallback, not a normal case.
pub(super) fn onset_counts_per_measure(notes_buckets: &[String]) -> Vec<u32> {
    notes_buckets
        .iter()
        .map(|measure_text| {
            parse_notes_line(measure_text, 0, &mut GroupStack::default())
                .map(|parsed| fold_extensions(parsed.events).len() as u32)
                .unwrap_or(0)
        })
        .collect()
}

/// Repack one declared Notes/Chords/Percussion part's flat unzipped-edit
/// text into measures, dispatching the token grammar by [`PartKind`].
/// Never called for `PartKind::Lyrics` — that kind's repack is
/// [`repack_lyrics_via_diff`], driven directly from [`repack_all_parts`]
/// since it needs the full `(original_tokens_per_measure, ceiling,
/// edited_text)` shape rather than a single flat capacity slice.
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
        // Unreachable in practice: `repack_all_parts` never calls
        // `repack_part` for a `Lyrics`-kind declaration (it calls
        // `repack_lyrics_via_diff` directly instead). Kept as a live match
        // arm rather than an `unreachable!()`/wildcard, since `PartKind`
        // must stay exhaustively matched here.
        PartKind::Lyrics => Err(UnzippedEditError::ParseFailed),
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
/// beat capacities (Notes/Chords/Percussion) or original tokens/ceiling
/// (Lyrics-role content — see [`repack_lyrics_via_diff`]).
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

        // `verse_ceiling`: the shared capacity ceiling every tagged verse of
        // this part anchors to — the Notes line's onset counts for
        // `notes+lyrics`, or verse 1's own token counts (self-referential)
        // for standalone `Lyrics`-kind parts.
        let (buckets, verse_ceiling) = if decl.kind == PartKind::Lyrics {
            let original_tokens = scan_measure_tokens(
                &context.score_content,
                &context.declarations,
                &context.resolved_groups,
                part_index,
                ScoreLineRole::Lyrics,
                0,
            );
            let ceiling: Vec<u32> = original_tokens
                .iter()
                .map(|tokens| tokens.len() as u32)
                .collect();
            let buckets = repack_lyrics_via_diff(&original_tokens, &ceiling, flat_text);
            (buckets, ceiling)
        } else {
            let buckets = repack_part(decl, flat_text, capacities)?;
            let ceiling = if decl.kind == PartKind::NotesWithLyrics {
                onset_counts_per_measure(&buckets)
            } else {
                Vec::new()
            };
            (buckets, ceiling)
        };
        part_buckets.push(buckets);

        let mut verses = Vec::new();
        if let Some(verse_texts) = blocks.lyrics_verses.get(&decl.abbreviation) {
            for (&verse_number, flat_verse_text) in verse_texts {
                let verse_tokens = scan_measure_tokens(
                    &context.score_content,
                    &context.declarations,
                    &context.resolved_groups,
                    part_index,
                    ScoreLineRole::Lyrics,
                    verse_number - 1,
                );
                let buckets =
                    repack_lyrics_via_diff(&verse_tokens, &verse_ceiling, flat_verse_text);
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

#[cfg(test)]
mod tests_repack;
