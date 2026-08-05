//! Phase 3c-3d: pads every repacked part's bucket lists to the same measure
//! count, then reassembles raw measure groups for a final
//! `desugar::desugar_groups` pass, and renders that pass's output back into
//! `# score`-section text.

use crate::ast::parsed::{PartDecl, PartKind, ScoreLineRole};
use crate::desugar;
use crate::parser;

use super::capacity::scan_time_signatures;
use super::repack::VerseBuckets;
use super::{RawSourceLine, UnzippedDocumentContext};

/// Whether declared part `decl` contributes nothing at all to `measure_index`
/// — neither primary content nor any verse content — used both to decide the
/// "every declared part is blank" reassembly special case and, per part,
/// whether to omit its lines from that measure entirely.
fn part_is_blank_at_measure(
    decl: &PartDecl,
    primary_text: &str,
    verse_buckets: &[VerseBuckets],
    measure_index: usize,
) -> bool {
    if !primary_text.is_empty() {
        return false;
    }
    match decl.kind {
        PartKind::NotesWithLyrics | PartKind::Lyrics => verse_buckets.iter().all(|verse| {
            verse
                .buckets
                .get(measure_index)
                .is_none_or(|text| text.is_empty())
        }),
        _ => true,
    }
}

/// The score lines (without `[Abbrev]` key prefixes) this declared part
/// contributes at `measure_index`, honoring the on-disk positional
/// constraint that a verse can't exist without every lower verse also having
/// a line (backfilled with `desugar::implicit_fill` placeholders where a
/// lower verse/the notes line itself has no real content at this measure).
/// Empty when the part contributes nothing at all here (see
/// [`part_is_blank_at_measure`]).
fn lines_for_part_at_measure(
    decl: &PartDecl,
    primary_text: &str,
    verse_buckets: &[VerseBuckets],
    measure_index: usize,
    time_num: u8,
) -> Vec<String> {
    let verse_text = |verse_number: usize| -> Option<&str> {
        verse_buckets
            .iter()
            .find(|verse| verse.verse_number == verse_number)
            .and_then(|verse| verse.buckets.get(measure_index))
            .map(String::as_str)
            .filter(|text| !text.is_empty())
    };
    let highest_nonempty_verse = verse_buckets
        .iter()
        .filter(|verse| {
            verse
                .buckets
                .get(measure_index)
                .is_some_and(|text| !text.is_empty())
        })
        .map(|verse| verse.verse_number)
        .max()
        .unwrap_or(0);

    match decl.kind {
        PartKind::NotesWithLyrics => {
            if highest_nonempty_verse == 0 {
                return if primary_text.is_empty() {
                    Vec::new()
                } else {
                    vec![primary_text.to_string()]
                };
            }
            let mut lines = Vec::with_capacity(1 + highest_nonempty_verse);
            lines.push(if primary_text.is_empty() {
                desugar::implicit_fill(ScoreLineRole::Notes, time_num)
            } else {
                primary_text.to_string()
            });
            for verse_number in 1..=highest_nonempty_verse {
                lines.push(
                    verse_text(verse_number)
                        .map(str::to_string)
                        .unwrap_or_else(|| desugar::implicit_fill(ScoreLineRole::Lyrics, time_num)),
                );
            }
            lines
        }
        PartKind::Lyrics => {
            let total = highest_nonempty_verse.max(usize::from(!primary_text.is_empty()));
            (1..=total)
                .map(|verse_number| {
                    if verse_number == 1 {
                        if primary_text.is_empty() {
                            desugar::implicit_fill(ScoreLineRole::Lyrics, time_num)
                        } else {
                            primary_text.to_string()
                        }
                    } else {
                        verse_text(verse_number)
                            .map(str::to_string)
                            .unwrap_or_else(|| {
                                desugar::implicit_fill(ScoreLineRole::Lyrics, time_num)
                            })
                    }
                })
                .collect()
        }
        _ => {
            if primary_text.is_empty() {
                Vec::new()
            } else {
                vec![primary_text.to_string()]
            }
        }
    }
}

/// Phase 3c: pad every part's every bucket list (primary and each verse) to
/// the same, largest measure count — the max across the original document's
/// measure count and every repacked bucket's own length.
pub(super) fn reconcile_bucket_lengths(
    original_measure_count: usize,
    part_buckets: &mut [Vec<String>],
    verse_buckets_per_part: &mut [Vec<VerseBuckets>],
) -> usize {
    let new_total = [
        original_measure_count,
        part_buckets.iter().map(Vec::len).max().unwrap_or(0),
        verse_buckets_per_part
            .iter()
            .flatten()
            .map(|verse| verse.buckets.len())
            .max()
            .unwrap_or(0),
    ]
    .into_iter()
    .max()
    .unwrap_or(0);
    for buckets in part_buckets.iter_mut() {
        buckets.resize(new_total, String::new());
    }
    for verses in verse_buckets_per_part.iter_mut() {
        for verse in verses {
            verse.buckets.resize(new_total, String::new());
        }
    }
    new_total
}

/// Phase 3d: reassemble `new_total` raw measure groups (directive line, if
/// any, plus one `[Abbrev] <content>` line per part per contributed line) for
/// a final `desugar::desugar_groups` pass to consume. Directive lines are
/// reused verbatim from the original groups by measure index and never
/// regenerated; a part contributing nothing at a given measure omits its
/// lines entirely so `desugar_groups`'s own implicit-fill fallback covers it
/// — *except* when every declared part is blank at some measure, which is
/// hand-filled here the same way it always has been (see
/// [`part_is_blank_at_measure`]/[`lines_for_part_at_measure`] doc comments).
pub(super) fn build_raw_groups_for_desugar(
    context: &UnzippedDocumentContext,
    raw_groups: &[Vec<RawSourceLine>],
    part_buckets: &[Vec<String>],
    verse_buckets_per_part: &[Vec<VerseBuckets>],
    new_total: usize,
) -> Vec<Vec<RawSourceLine>> {
    let directive_lines: Vec<Option<String>> = raw_groups
        .iter()
        .map(|group| {
            (parser::score::measure_group::directive_line_count(group) == 1)
                .then(|| group.first().map(|(content, _)| content.clone()))
                .flatten()
        })
        .collect();
    let time_numerators: Vec<u8> = scan_time_signatures(&context.score_content)
        .into_iter()
        .map(|(numerator, _denominator)| numerator)
        .collect();
    let time_num_at = |measure_index: usize| -> u8 {
        time_numerators
            .get(measure_index)
            .or_else(|| time_numerators.last())
            .copied()
            .unwrap_or(4)
    };

    (0..new_total)
        .map(|measure_index| {
            let mut lines: Vec<RawSourceLine> = Vec::new();
            if let Some(Some(directive)) = directive_lines.get(measure_index) {
                lines.push((directive.clone(), 0));
            }
            let all_parts_blank =
                context
                    .declarations
                    .iter()
                    .enumerate()
                    .all(|(part_index, decl)| {
                        let primary_text = part_buckets
                            .get(part_index)
                            .and_then(|b| b.get(measure_index));
                        let verses = verse_buckets_per_part
                            .get(part_index)
                            .map_or(&[][..], Vec::as_slice);
                        part_is_blank_at_measure(
                            decl,
                            primary_text.map_or("", String::as_str),
                            verses,
                            measure_index,
                        )
                    });
            for (part_index, decl) in context.declarations.iter().enumerate() {
                let primary_text = part_buckets
                    .get(part_index)
                    .and_then(|b| b.get(measure_index))
                    .map_or("", String::as_str);
                let verses = verse_buckets_per_part
                    .get(part_index)
                    .map_or(&[][..], Vec::as_slice);
                let part_lines = lines_for_part_at_measure(
                    decl,
                    primary_text,
                    verses,
                    measure_index,
                    time_num_at(measure_index),
                );
                if !part_lines.is_empty() {
                    for line in part_lines {
                        lines.push((format!("[{}] {line}", decl.abbreviation), 0));
                    }
                } else if all_parts_blank {
                    let role = decl
                        .score_line_roles()
                        .first()
                        .copied()
                        .unwrap_or(ScoreLineRole::Notes);
                    let filled = desugar::implicit_fill(role, time_num_at(measure_index));
                    lines.push((format!("[{}] {filled}", decl.abbreviation), 0));
                }
            }
            lines
        })
        .collect()
}
