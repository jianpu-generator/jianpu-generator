//! Phase 3a: [`extract_unzipped_text`] flattens every declared part's
//! resolved score lines into `[Abbrev]`-headed blocks. See the parent
//! module's doc comment for the overall Unzipped Edit design.

use crate::ast::parsed::{PartKind, ScoreLineRole, ScoreLineSlot};
use crate::desugar::{self, SourceLine};
use crate::parser::{
    self,
    score::token_parser::{parse_chord_line, parse_notes_line, parse_percussion_line, GroupStack},
};

use super::capacity::scan_measure_capacities;
use super::{
    capacity_at, extract_part_line, fold_extensions, resolve_document_context, LyricsVerseRanges,
    UnzippedEditError, UnzippedExtractOutput,
};

/// The greatest number of Lyrics-role slots this part has in any single
/// measure group, across the whole document — i.e. the highest verse number
/// this part ever writes.
fn max_lyrics_verses(slots_per_group: &[Vec<ScoreLineSlot>], target_index: usize) -> usize {
    slots_per_group
        .iter()
        .map(|slots| {
            slots
                .iter()
                .filter(|slot| {
                    slot.track_index == target_index && slot.role == ScoreLineRole::Lyrics
                })
                .count()
        })
        .max()
        .unwrap_or(0)
}

/// Rewrites one measure's extracted line text to be explicit about its own
/// true beat weight, so `merge::repack_part`'s greedy bucket-filler — which
/// only ever sees a flattened, boundary-free token stream and re-derives
/// measure boundaries purely from each token's literal weight — never has to
/// rely on `syntax.md`'s implicit shortfall-extension rule (a lone `0`
/// filling a 4/4 measure has *true* weight 16, not the literal 4 a bare rest
/// token parses to). That padding machinery lives in
/// `parser::score::interleaved_beat_padding` but is only invoked by the real
/// compile pipeline (off `desugar_groups`'s pre-padding output), never by
/// `extract_unzipped_text`, so any token here that implicitly relied on it
/// would otherwise be under-weighted and cause the repacker to drift a
/// measure out of phase with the original document.
///
/// Mirrors `interleaved_beat_padding::can_implicitly_pad`'s eligibility rule
/// inline (deficit divisible by 4, last cluster's weight >= 4, and the sum of
/// every earlier cluster's weight itself divisible by 4) rather than
/// broadening that module's visibility for it — same precedent as
/// `capacity::beats_per_measure`. Only ever called for a Notes/Chords/
/// Percussion/NotesWithLyrics part's non-Lyrics-role primary block; `line`
/// unparseable, already at or over `capacity`, or not implicitly-fillable is
/// returned unchanged (those are pre-existing diagnostic cases unrelated to
/// this padding).
fn pad_to_explicit_weight(line: &str, kind: PartKind, capacity: u32) -> String {
    if line.is_empty() {
        return line.to_string();
    }
    let parsed = match kind {
        PartKind::Notes | PartKind::NotesWithLyrics => {
            parse_notes_line(line, 0, &mut GroupStack::default())
        }
        PartKind::Chords => parse_chord_line(line, 0, &mut GroupStack::default()),
        PartKind::Percussion => parse_percussion_line(line, 0, &mut GroupStack::default()),
        PartKind::Lyrics => return line.to_string(),
    };
    let Ok(parsed) = parsed else {
        return line.to_string();
    };
    let clusters = fold_extensions(parsed.events);
    let total: u32 = clusters.iter().map(|(_, weight)| *weight).sum();
    if total >= capacity {
        return line.to_string();
    }
    let deficit = capacity - total;
    if deficit % 4 != 0 {
        return line.to_string();
    }
    let Some((_, last_weight)) = clusters.last() else {
        return line.to_string();
    };
    let before_last: u32 = clusters
        .iter()
        .rev()
        .skip(1)
        .map(|(_, weight)| *weight)
        .sum();
    if *last_weight < 4 || before_last % 4 != 0 {
        return line.to_string();
    }

    let mut padded = line.to_string();
    for _ in 0..(deficit / 4) {
        padded.push_str(" -");
    }
    padded
}

/// The `(target_index, role, occurrence)` slot [`append_block_ranges`] reads
/// from each measure group — see [`extract_part_line`] for what each field
/// selects.
struct BlockSlot {
    target_index: usize,
    role: ScoreLineRole,
    occurrence: usize,
}

/// Appends one block's flattened, space-joined measure tokens (no header) to
/// `text` and returns its per-measure byte ranges.
///
/// `pad_kind` is `Some(decl.kind)` only for the primary block of a
/// Notes/Chords/Percussion/NotesWithLyrics part (i.e. `role !=
/// ScoreLineRole::Lyrics`); when set, each measure's text is rewritten via
/// [`pad_to_explicit_weight`] against `capacities` before being appended.
/// Lyrics-role content has no beat/duration grammar (and thus no implicit
/// shortfall-extension to worry about), so callers pass `None` for it.
fn append_block_ranges(
    text: &mut String,
    desugared: &[Vec<SourceLine>],
    slots_per_group: &[Vec<ScoreLineSlot>],
    slot: &BlockSlot,
    pad_kind: Option<PartKind>,
    capacities: &[u32],
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::with_capacity(desugared.len());
    for (measure_index, (group, slots)) in desugared.iter().zip(slots_per_group.iter()).enumerate()
    {
        if measure_index > 0 {
            text.push(' ');
        }
        let start = text.len();
        let line = extract_part_line(group, slots, slot.target_index, slot.role, slot.occurrence);
        let line = match pad_kind {
            Some(kind) => {
                pad_to_explicit_weight(&line, kind, capacity_at(capacities, measure_index))
            }
            None => line,
        };
        text.push_str(&line);
        let end = text.len();
        ranges.push((start, end));
    }
    ranges
}

/// Extract every declared part's resolved score lines from `source`'s whole
/// document, flattened per part into one continuous token stream (measure
/// line breaks become insignificant single spaces).
///
/// Each part's primary block is emitted as `[Abbrev]\n<flattened tokens>`
/// (slot occurrence 0 of the part's first static role). A `NotesWithLyrics`
/// or `Lyrics` part additionally emits one `[Abbrev:lyrics:N]\n<flattened
/// tokens>` block per verse beyond what the primary block already covers
/// (verses 1..=max for `NotesWithLyrics`, since its primary block is Notes;
/// verses 2..=max for `Lyrics`, whose primary block already is verse 1).
/// Blocks are unzipped by a blank line, in declaration then verse order.
/// Implicit rests are filled in as explicit `0`s (or `_` for lyrics).
pub fn extract_unzipped_text(source: &str) -> Result<UnzippedExtractOutput, UnzippedEditError> {
    let context = resolve_document_context(source);
    let raw_groups = parser::score::measure_group::collect_groups(&context.score_content);
    let (desugared, slots_per_group, _errors, _references) = desugar::desugar_groups(
        raw_groups,
        &context.declarations,
        &context.resolved_groups,
        context.score_offset,
    )
    .map_err(|_| UnzippedEditError::ParseFailed)?;
    let capacities = scan_measure_capacities(&context.score_content);

    let mut text = String::new();
    let mut part_measure_ranges = Vec::with_capacity(context.declarations.len());
    let mut lyrics_verse_ranges = Vec::with_capacity(context.declarations.len());

    for (part_index, decl) in context.declarations.iter().enumerate() {
        let primary_role = decl
            .score_line_roles()
            .first()
            .copied()
            .unwrap_or(ScoreLineRole::Notes);
        let pad_kind = (primary_role != ScoreLineRole::Lyrics).then_some(decl.kind);

        text.push('[');
        text.push_str(&decl.abbreviation);
        text.push_str("]\n");
        let ranges = append_block_ranges(
            &mut text,
            &desugared,
            &slots_per_group,
            &BlockSlot {
                target_index: part_index,
                role: primary_role,
                occurrence: 0,
            },
            pad_kind,
            &capacities,
        );
        part_measure_ranges.push(ranges);

        let mut verse_blocks = Vec::new();
        if matches!(decl.kind, PartKind::NotesWithLyrics | PartKind::Lyrics) {
            let max_verses = max_lyrics_verses(&slots_per_group, part_index);
            let start_verse = if decl.kind == PartKind::Lyrics { 2 } else { 1 };
            for verse_number in start_verse..=max_verses {
                text.push_str("\n\n[");
                text.push_str(&decl.abbreviation);
                text.push_str(&format!(":lyrics:{verse_number}]\n"));
                let occurrence = verse_number - 1;
                let verse_ranges = append_block_ranges(
                    &mut text,
                    &desugared,
                    &slots_per_group,
                    &BlockSlot {
                        target_index: part_index,
                        role: ScoreLineRole::Lyrics,
                        occurrence,
                    },
                    None,
                    &capacities,
                );
                verse_blocks.push(LyricsVerseRanges {
                    verse_number,
                    measure_ranges: verse_ranges,
                });
            }
        }
        lyrics_verse_ranges.push(verse_blocks);

        if part_index + 1 < context.declarations.len() {
            text.push_str("\n\n");
        }
    }

    Ok(UnzippedExtractOutput {
        text,
        part_measure_ranges,
        lyrics_verse_ranges,
    })
}
