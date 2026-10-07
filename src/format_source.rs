//! [`format_score`]: a best-effort formatter for the Zipped (normal) editor
//! view. Three independent cleanups run per `# score` measure group:
//!
//! 1. **Redundant-line removal.** A `[Key]` data line is dropped when it is
//!    exactly what the existing implicit-fill machinery
//!    (`desugar::implicit_fill`) would already produce if that key were not
//!    mentioned at all in this measure group — an all-rest line for a
//!    `Notes`/`Chord`-role occurrence, or an all-`_` line for a `Lyrics`-role
//!    occurrence. Only a key's *trailing* removable lines are eligible: an
//!    earlier all-rest verse can't be dropped out from under a later verse
//!    with real content, since that would shift the later verse into the
//!    earlier one's slot. `follow[X]` parts are never touched (their
//!    implicit fill is the follow target's content, not rest).
//! 2. **Part-order sorting.** A key's surviving line is moved to the position
//!    matching its part's order in `# parts` (a lyric part's line right after
//!    its target's notes line, in verse order), regardless of how the lines
//!    were interleaved in the source. A key
//!    that can't be resolved to a declared part keeps its original relative
//!    position, ordered after every recognised part.
//! 3. **Whitespace normalization.** Every directive line and surviving data
//!    line has its runs of internal whitespace collapsed to one space
//!    (quote-aware for directive lines, so `label="Two Words"` survives as a
//!    single token).
//!
//! The redundant-line removal reuses `desugar::desugar_groups` rather than
//! re-implementing "how would this resolve": once the eligible trailing
//! lines are stripped from the raw input, feeding the result back through
//! `desugar_groups` fills the dropped slots correctly for free.
//!
//! Infallible / best-effort, mirroring `source_edit::update_part_declaration`'s
//! "not found -> unchanged" convention: a missing `# parts`/`# score` section,
//! or any internal parse failure, returns `source` unchanged.

use crate::ast::parsed::{PartDecl, ScoreLineRole};
use crate::desugar;
use crate::parser;

/// A raw, not-yet-desugared score line paired with its byte offset within
/// its containing section, matching `measure_group::collect_groups`'s output.
type RawSourceLine = (String, usize);

/// Formats `source`'s `# score` section: drops `[Key]` data lines that are
/// entirely redundant with implicit-fill, and collapses whitespace to single
/// spaces on every surviving directive/data line. Returns `source` unchanged
/// if `# parts`/`# score` can't be resolved.
pub fn format_score(source: &str) -> String {
    let (sections, _section_errors) = parser::load_document_sections(source);
    let (parts_content, parts_offset) = sections.parts;
    let (score_content, score_offset) = sections.score;
    if score_content.trim().is_empty() {
        return source.to_string();
    }

    let (declarations, _parts_errors) =
        parser::parts_parser::parse_parts(&parts_content, parts_offset, &[]);
    if declarations.is_empty() {
        return source.to_string();
    }

    let raw_groups = parser::score::measure_group::collect_groups(&score_content);
    let filtered_groups: Vec<Vec<RawSourceLine>> = raw_groups
        .iter()
        .map(|group| format_group(group, &declarations))
        .collect();

    // Validate that the filtered/reordered groups still desugar cleanly
    // (best-effort safety net — `desugar_groups` is not used for rendering
    // here: it always materializes exactly one line per declared part per
    // group regardless of whether that part had an explicit line, which
    // would put every dropped fixed-schema (Notes/Chords/Percussion) rest
    // line right back. The actual output is built directly from
    // `filtered_groups` below, so intentionally-dropped lines stay dropped
    // and real `.jianpu` parsing's own implicit-fill covers them, exactly as
    // it already does for any measure group that omits a part's line.
    if desugar::desugar_groups(filtered_groups.clone(), &declarations, score_offset).is_err() {
        return source.to_string();
    }

    let group_texts: Vec<String> = filtered_groups
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|(content, _offset)| content.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect();
    let new_score_content = if group_texts.is_empty() {
        String::new()
    } else {
        format!("{}\n", group_texts.join("\n\n"))
    };

    let mut result = String::with_capacity(
        score_offset
            + new_score_content.len()
            + source
                .len()
                .saturating_sub(score_offset + score_content.len()),
    );
    result.push_str(source.get(..score_offset).unwrap_or(source));
    result.push_str(&new_score_content);
    result.push_str(
        source
            .get(score_offset + score_content.len()..)
            .unwrap_or(""),
    );
    result
}

/// The slot a `[Key]` addresses: its declaration, and which of that part's
/// lines it is (0 = its notes/chords line, `n` = verse `n`).
#[derive(Clone, Copy)]
struct KeySlot {
    declaration_index: usize,
    slot_index: usize,
}

fn key_slot(key: &str, declarations: &[PartDecl]) -> Option<KeySlot> {
    declarations
        .iter()
        .enumerate()
        .find_map(|(declaration_index, decl)| {
            if decl.abbreviation == key {
                return Some(KeySlot {
                    declaration_index,
                    slot_index: 0,
                });
            }
            decl.verses
                .iter()
                .position(|verse| verse.abbreviation == key)
                .map(|verse_position| KeySlot {
                    declaration_index,
                    slot_index: verse_position + 1,
                })
        })
}

/// Which data lines are redundant with implicit fill. Per part, walks its
/// lines from the highest slot down and stops at the first one that isn't:
/// an earlier no-lyrics verse can't be dropped out from under a later verse
/// with real content, and a rest notes line stays while any verse is kept.
/// `follow[X]` notes lines are never removable (their implicit fill is the
/// target's content, not rest).
fn removable_lines(parsed: &[Option<(&str, &str)>], declarations: &[PartDecl]) -> Vec<bool> {
    let slots: Vec<Option<KeySlot>> = parsed
        .iter()
        .map(|entry| entry.and_then(|(key, _)| key_slot(key, declarations)))
        .collect();
    let mut removable = vec![false; parsed.len()];
    for (declaration_index, decl) in declarations.iter().enumerate() {
        let mut part_lines: Vec<(usize, usize)> = slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                slot.filter(|slot| slot.declaration_index == declaration_index)
                    .map(|slot| (index, slot.slot_index))
            })
            .collect();
        part_lines.sort_by_key(|&(_, slot_index)| std::cmp::Reverse(slot_index));
        for (index, slot_index) in part_lines {
            let role = match (slot_index, decl.follow_target.is_some()) {
                (0, true) => break,
                (0, false) => decl.score_line_roles().first().copied(),
                _ => Some(ScoreLineRole::Lyrics),
            };
            let content = parsed.get(index).copied().flatten().map(|(_, c)| c);
            match (role, content) {
                (Some(role), Some(content)) if is_removable(role, content) => {
                    if let Some(flag) = removable.get_mut(index) {
                        *flag = true;
                    }
                }
                _ => break,
            }
        }
    }
    removable
}

/// Every whitespace-split token is a rest: `0` optionally followed by a run
/// of `_`/`=`/`.`/`-` suffix characters, or a bare `-`/`-.` dash-only
/// extension atom (extends a preceding rest without repeating `0`).
fn is_all_rest(content: &str) -> bool {
    let tokens: Vec<&str> = content.split_whitespace().collect();
    !tokens.is_empty()
        && tokens
            .iter()
            .all(|token| *token == "-" || *token == "-." || is_rest_token(token))
}

fn is_rest_token(token: &str) -> bool {
    token
        .strip_prefix('0')
        .is_some_and(|rest| rest.chars().all(|c| matches!(c, '_' | '=' | '.' | '-')))
}

/// Every whitespace-split token is exactly `_` (no lyrics for this measure).
fn is_all_no_lyrics(content: &str) -> bool {
    let tokens: Vec<&str> = content.split_whitespace().collect();
    !tokens.is_empty() && tokens.iter().all(|token| *token == "_")
}

fn is_removable(role: ScoreLineRole, content: &str) -> bool {
    match role {
        ScoreLineRole::Notes | ScoreLineRole::Chord => is_all_rest(content),
        ScoreLineRole::Lyrics => is_all_no_lyrics(content),
    }
}

/// Builds the filtered/reordered/whitespace-normalized raw measure group for
/// `group`: directive line(s) normalized, unparseable data lines passed
/// through untouched, eligible trailing redundant `[Key]` lines dropped, and
/// surviving `[Key]` blocks sorted into `# parts` declaration order (see
/// module docs).
fn format_group(group: &[RawSourceLine], declarations: &[PartDecl]) -> Vec<RawSourceLine> {
    let directive_count = parser::score::measure_group::directive_line_count(group);
    let directive_lines = group.get(..directive_count).unwrap_or(&[]);
    let data_lines = group.get(directive_count..).unwrap_or(&[]);

    // Parsed `(key, content)` per data line, `None` for lines left untouched.
    let parsed: Vec<Option<(&str, &str)>> = data_lines
        .iter()
        .map(|(line, _offset)| desugar::parse_key_prefix(line))
        .collect();

    let mut removable = removable_lines(&parsed, declarations);

    let remaining_count = removable.iter().filter(|r| !**r).count();
    if remaining_count == 0 {
        if let Some(last_removable) = removable.iter().rposition(|r| *r) {
            if let Some(slot) = removable.get_mut(last_removable) {
                *slot = false;
            }
        }
    }

    let normalized_data_lines: Vec<RawSourceLine> = data_lines
        .iter()
        .zip(parsed.iter())
        .map(|((line, offset), entry)| {
            let normalized = match entry {
                Some((key, content)) => format!("[{key}] {}", normalize_data_line(content)),
                None => line.clone(),
            };
            (normalized, *offset)
        })
        .collect();

    let mut result: Vec<RawSourceLine> = directive_lines
        .iter()
        .map(|(content, offset)| (normalize_directive_line(content), *offset))
        .collect();
    result.extend(sort_data_lines_by_declaration(
        normalized_data_lines,
        &parsed,
        &removable,
        declarations,
    ));
    result
}

/// A key's surviving data lines, or one unparseable line, as a unit that
/// moves together when reordering.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum DataLineBlockKey<'a> {
    Key(&'a str),
    /// Carries the line's original index so each unparseable line is its
    /// own block, distinct from every other.
    Unparsed(usize),
}

/// Groups surviving data lines into blocks — every occurrence of a key
/// travels together, regardless of how interleaved with other keys' lines
/// it was in the source — and orders the blocks by the part's position in
/// `# parts`. A block for an unrecognised or unparseable key keeps its
/// original relative position, ordered after every recognised part (its
/// content isn't touched here; any resulting invalid document is caught by
/// `format_score`'s `desugar_groups` safety net).
fn sort_data_lines_by_declaration<'a>(
    normalized_data_lines: Vec<RawSourceLine>,
    parsed: &[Option<(&'a str, &'a str)>],
    removable: &[bool],
    declarations: &[PartDecl],
) -> Vec<RawSourceLine> {
    // Blocks in first-seen order; a linear scan per line is fine since a
    // measure group only ever has a handful of distinct keys.
    let mut blocks: Vec<(DataLineBlockKey<'a>, Vec<RawSourceLine>)> = Vec::new();
    for (index, ((line, drop), entry)) in normalized_data_lines
        .into_iter()
        .zip(removable.iter())
        .zip(parsed.iter())
        .enumerate()
    {
        if *drop {
            continue;
        }
        let block_key = match entry {
            Some((key, _)) => DataLineBlockKey::Key(key),
            None => DataLineBlockKey::Unparsed(index),
        };
        match blocks
            .iter_mut()
            .find(|(existing, _)| *existing == block_key)
        {
            Some((_, lines)) => lines.push(line),
            None => blocks.push((block_key, vec![line])),
        }
    }

    // `sort_by_key` is stable, so blocks with equal rank (including every
    // unrecognised-key block, which all rank `usize::MAX`) keep their
    // first-seen relative order.
    blocks.sort_by_key(|(block_key, _)| match block_key {
        DataLineBlockKey::Key(key) => key_slot(key, declarations).map_or((usize::MAX, 0), |slot| {
            (slot.declaration_index, slot.slot_index)
        }),
        DataLineBlockKey::Unparsed(_) => (usize::MAX, 0),
    });

    blocks.into_iter().flat_map(|(_, lines)| lines).collect()
}

/// Collapses whitespace to single spaces, leaving unparseable lines and
/// non-`[Key]`-prefixed content untouched by callers (this only runs on
/// lines already confirmed to parse).
fn normalize_data_line(content: &str) -> String {
    content.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Collapses whitespace to single spaces outside of `"..."` quoted spans, so
/// a `label="Two Words"` token survives as one token.
fn normalize_directive_line(content: &str) -> String {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in content.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
            current.push(c);
        } else if c.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens.join(" ")
}

#[cfg(test)]
#[path = "format_source_tests.rs"]
mod tests;
