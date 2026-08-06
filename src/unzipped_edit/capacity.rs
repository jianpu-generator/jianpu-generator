//! Beat/token capacity scanning: [`scan_measure_capacities`],
//! [`scan_measure_tokens`], and [`scan_measure_token_counts`] give
//! [`super::merge::merge_unzipped_text`]'s repack algorithm the per-measure
//! capacities (and, for Lyrics-role content, original tokens) it re-bars
//! against.

use crate::ast::parsed::{PartDecl, ScoreEvent, ScoreLineRole};
use crate::desugar::{self, SourceLine};
use crate::parser::{self, group_parser::ResolvedGroup};

use super::extract_part_line;

/// Beats (quarter-beat units) in one measure of the given time signature: the same
/// `numerator * 16 / denominator` formula `interleaved_beat_padding::beats_per_measure`
/// uses, reimplemented locally since that helper is `pub(super)` to the interleaved
/// parser module and not worth broadening visibility for a one-line formula.
fn beats_per_measure(numerator: u8, denominator: u8) -> u32 {
    (numerator as u32) * (16 / denominator as u32)
}

/// Scan `score_content` (the raw, not-yet-desugared `# score` section) once and return
/// the active `(numerator, denominator)` time signature for each original measure-group
/// index, carrying the most recent `time=` directive forward across groups, mirroring
/// the `time_num`/`time_den` tracking loop `interleaved_parser::process_bar_group`
/// performs while parsing the same content for real. Shared by [`scan_measure_capacities`]
/// and by `merge::merge_unzipped_text`'s handling of measures where every declared part
/// comes up blank after a repack.
pub(super) fn scan_time_signatures(score_content: &str) -> Vec<(u8, u8)> {
    let raw_groups = parser::score::measure_group::collect_groups(score_content);

    let mut time_num: u8 = 4;
    let mut time_den: u8 = 4;
    raw_groups
        .iter()
        .map(|group| {
            let source_lines: Vec<SourceLine> = group
                .iter()
                .map(|(content, offset)| SourceLine {
                    content: content.clone(),
                    offset: *offset,
                    group: None,
                })
                .collect();
            let events =
                parser::score::interleaved_parser::directive_events_for_group(&source_lines, 0);
            for event in &events {
                if let ScoreEvent::TimeSignatureChange {
                    numerator,
                    denominator,
                } = &event.value
                {
                    time_num = *numerator;
                    time_den = *denominator;
                }
            }
            (time_num, time_den)
        })
        .collect()
}

/// Scan `score_content` (the raw, not-yet-desugared `# score` section) once and return
/// one beat-capacity per original measure-group index, in quarter-beat units.
pub fn scan_measure_capacities(score_content: &str) -> Vec<u32> {
    scan_time_signatures(score_content)
        .into_iter()
        .map(|(numerator, denominator)| beats_per_measure(numerator, denominator))
        .collect()
}

/// Scan `score_content` for `declarations[target_index]`'s `(role, occurrence)` score-line
/// slot and return its exact original whitespace tokens (syllables), one `Vec<String>` per
/// original measure-group index. Lyrics-role content has no beat/duration grammar, so the
/// merge-back repack works against these original tokens (and, for genuinely new content,
/// a token-count ceiling — see [`scan_measure_capacities`]) rather than a quarter-beat count.
/// `occurrence` is 0-based among same-role slots for this part (e.g. verse 2 is `role =
/// Lyrics, occurrence = 1`); a measure where this occurrence doesn't exist at all (fewer
/// verses written there than elsewhere) yields an empty `Vec`, not an error.
pub(super) fn scan_measure_tokens(
    score_content: &str,
    declarations: &[PartDecl],
    resolved_groups: &[ResolvedGroup],
    target_index: usize,
    role: ScoreLineRole,
    occurrence: usize,
) -> Vec<Vec<String>> {
    let raw_groups = parser::score::measure_group::collect_groups(score_content);
    let (desugared, slots_per_group, _errors, _references) =
        match desugar::desugar_groups(raw_groups, declarations, resolved_groups, 0) {
            Ok(result) => result,
            Err(_) => return Vec::new(),
        };

    desugared
        .iter()
        .zip(slots_per_group.iter())
        .map(|(group, slots)| {
            let line = extract_part_line(group, slots, target_index, role, occurrence);
            line.split_whitespace().map(str::to_string).collect()
        })
        .collect()
}

/// One token (syllable) count per original measure-group index — a thin wrapper over
/// [`scan_measure_tokens`] for callers that only need counts, not the tokens themselves.
pub fn scan_measure_token_counts(
    score_content: &str,
    declarations: &[PartDecl],
    resolved_groups: &[ResolvedGroup],
    target_index: usize,
    role: ScoreLineRole,
    occurrence: usize,
) -> Vec<u32> {
    scan_measure_tokens(
        score_content,
        declarations,
        resolved_groups,
        target_index,
        role,
        occurrence,
    )
    .iter()
    .map(|tokens| tokens.len() as u32)
    .collect()
}
