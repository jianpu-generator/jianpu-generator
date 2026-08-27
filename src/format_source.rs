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
//! 2. **Part-order sorting.** A key's surviving lines are moved together as
//!    one block to the position matching that part's order in `# parts`,
//!    regardless of how the lines were interleaved in the source. A key
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

use crate::ast::parsed::{PartDecl, PartKind, ScoreLineRole};
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

/// Whether an occurrence at `occurrence_index` (0-based) of `key`'s lines in
/// this measure group is a `Notes`/`Chord`-role or `Lyrics`-role slot, for a
/// key that resolves to `decl`. Mirrors `desugar::roles_for_group`'s
/// static-kind branch, computed directly from the raw per-key line count
/// rather than through full slot resolution: for `Chords`/`Notes`,
/// occurrence 0 is Notes/Chord-role and every later occurrence (a positional
/// lyrics verse) is Lyrics-role; `Percussion` only has occurrence 0, which is
/// Notes-role.
fn role_at_occurrence(decl: &PartDecl, occurrence_index: usize) -> Option<ScoreLineRole> {
    match decl.kind {
        PartKind::Chords | PartKind::Notes if occurrence_index > 0 => Some(ScoreLineRole::Lyrics),
        PartKind::Chords | PartKind::Notes | PartKind::Percussion => (occurrence_index == 0)
            .then(|| decl.score_line_roles().first().copied())
            .flatten(),
    }
}

/// The part declaration a raw `[Key]` line's role list should be read from: a
/// direct, non-`follow[X]` part declaration. `None` for an unknown key or a
/// `follow[X]` part (excluded entirely: its implicit fill is the follow
/// target's content, not rest, so an explicit rest line there is real
/// content).
fn decl_for_key<'a>(key: &str, declarations: &'a [PartDecl]) -> Option<&'a PartDecl> {
    let decl = declarations.iter().find(|d| d.abbreviation == key)?;
    decl.follow_target.is_none().then_some(decl)
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

/// One key's parsed lines within a measure group, in file order.
struct KeyLines<'a> {
    key: String,
    /// Index into the group's data-line list, per occurrence (file order).
    indices: Vec<usize>,
    decl: Option<&'a PartDecl>,
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

    let mut key_lines: Vec<KeyLines> = Vec::new();
    for (index, entry) in parsed.iter().enumerate() {
        let Some((key, _content)) = entry else {
            continue;
        };
        if let Some(existing) = key_lines.iter_mut().find(|k| k.key == *key) {
            existing.indices.push(index);
        } else {
            key_lines.push(KeyLines {
                key: key.to_string(),
                indices: vec![index],
                decl: decl_for_key(key, declarations),
            });
        }
    }

    let mut removable: Vec<bool> = vec![false; data_lines.len()];
    for entry in &key_lines {
        let Some(decl) = entry.decl else { continue };
        for (occurrence_index, &data_index) in entry.indices.iter().enumerate().rev() {
            let Some(role) = role_at_occurrence(decl, occurrence_index) else {
                break;
            };
            let Some((_, content)) = parsed.get(data_index).copied().flatten() else {
                break;
            };
            if !is_removable(role, content) {
                break;
            }
            if let Some(slot) = removable.get_mut(data_index) {
                *slot = true;
            }
        }
    }

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
    let attribution = positional_attribution_keys(parsed);

    // Blocks in first-seen order; a linear scan per line is fine since a
    // measure group only ever has a handful of distinct keys.
    let mut blocks: Vec<(DataLineBlockKey<'a>, Vec<RawSourceLine>)> = Vec::new();
    for (index, ((line, drop), attributed_key)) in normalized_data_lines
        .into_iter()
        .zip(removable.iter())
        .zip(attribution.iter())
        .enumerate()
    {
        if *drop {
            continue;
        }
        let block_key = match attributed_key {
            Some(key) => DataLineBlockKey::Key(key),
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
        DataLineBlockKey::Key(key) => declaration_index(key, declarations).unwrap_or(usize::MAX),
        DataLineBlockKey::Unparsed(_) => usize::MAX,
    });

    blocks.into_iter().flat_map(|(_, lines)| lines).collect()
}

/// A key's position in `# parts` declaration order, for sorting purposes.
/// Unlike `decl_for_key`, this includes `follow[X]` parts (they still have a
/// declared position to sort by) and returns `None` only for a genuinely
/// unrecognised key.
fn declaration_index(key: &str, declarations: &[PartDecl]) -> Option<usize> {
    declarations.iter().position(|d| d.abbreviation == key)
}

/// Per data line, the key its content is attributed to for sort-block
/// membership: an explicit `[Key]`-prefixed line's own key, or — for a bare
/// (positional) line — whichever `[Key]` line most recently preceded it in
/// this measure group, mirroring `desugar::attribution::attribute_data_lines`'s
/// nearest-preceding-key rule. `None` for a bare line with no preceding
/// `[Key]` line yet (a standalone lyrics caption): it isn't resolved to its
/// target part here, so it stays its own unattached block.
fn positional_attribution_keys<'a>(parsed: &[Option<(&'a str, &'a str)>]) -> Vec<Option<&'a str>> {
    let mut current_key: Option<&'a str> = None;
    parsed
        .iter()
        .map(|entry| match entry {
            Some((key, _)) => {
                current_key = Some(key);
                Some(*key)
            }
            None => current_key,
        })
        .collect()
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
