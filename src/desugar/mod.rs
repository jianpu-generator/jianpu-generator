use crate::ast::parsed::{AbbreviationReference, PartDecl, PartKind, ScoreLineRole, ScoreLineSlot};
use crate::error::{IrrecoverableError, RecoverableError, Span};
use crate::parser::group_parser::ResolvedGroup;
use crate::parser::score::measure_group;
use key_map::KeyMap;

mod key_map;

/// A raw, not-yet-desugared score line: `[Key] content`, paired with its byte offset.
type RawSourceLine = (String, usize);

/// A desugared score line, positionally bound to one `PartDecl`'s score-line slot.
#[derive(Debug, Clone)]
pub(crate) struct SourceLine {
    pub(crate) content: String,
    pub(crate) offset: usize,
    /// Abbreviation of the group whose `[GroupAbbrev]` broadcast produced this line's
    /// content, when this member did not override it with its own `[MemberAbbrev]` line.
    pub(crate) group: Option<String>,
}

type MeasureGroup = Vec<SourceLine>;
type DesugarGroupsResult = Result<
    (
        Vec<MeasureGroup>,
        Vec<Vec<ScoreLineSlot>>,
        Vec<Option<RecoverableError>>,
        Vec<AbbreviationReference>,
    ),
    IrrecoverableError,
>;
type ExpandMeasureGroupResult = Result<
    (
        MeasureGroup,
        Vec<ScoreLineSlot>,
        Option<RecoverableError>,
        Vec<AbbreviationReference>,
    ),
    IrrecoverableError,
>;

fn extract_time_numerator(group: &[RawSourceLine]) -> Option<u8> {
    let (first_line, _) = group.first()?;
    first_line
        .split_whitespace()
        .find(|t| t.starts_with("time="))?
        .strip_prefix("time=")?
        .split('/')
        .next()?
        .parse::<u8>()
        .ok()
}

pub(crate) fn desugar_groups(
    groups: Vec<Vec<RawSourceLine>>,
    declarations: &[PartDecl],
    resolved_groups: &[ResolvedGroup],
    base_offset: usize,
) -> DesugarGroupsResult {
    let mut desugared = Vec::with_capacity(groups.len());
    let mut slots_per_group = Vec::with_capacity(groups.len());
    let mut per_group_errors = Vec::with_capacity(groups.len());
    let mut abbreviation_references = Vec::new();
    let mut current_time_num: u8 = 4;
    for group in groups {
        if let Some(num) = extract_time_numerator(&group) {
            current_time_num = num;
        }
        let (expanded, slots, error, references) = expand_measure_group(
            &group,
            declarations,
            resolved_groups,
            base_offset,
            current_time_num,
        )?;
        desugared.push(expanded);
        slots_per_group.push(slots);
        per_group_errors.push(error);
        abbreviation_references.extend(references);
    }
    Ok((
        desugared,
        slots_per_group,
        per_group_errors,
        abbreviation_references,
    ))
}

pub(crate) fn parse_key_prefix(line: &str) -> Option<(&str, &str)> {
    line.strip_prefix('[')
        .and_then(|s| s.find(']').map(|i| (s[..i].trim(), s[i + 1..].trim())))
}

/// Produces the implicit-fill content (rest line / all-`_` lyrics line) for a
/// declaration that's absent from an otherwise-nonempty keyed group.
pub(crate) fn implicit_fill(role: ScoreLineRole, time_num: u8) -> String {
    match role {
        ScoreLineRole::Lyrics => "_".to_string(),
        ScoreLineRole::Notes | ScoreLineRole::Chord => {
            itertools::join(std::iter::repeat_n("0", time_num as usize), " ")
        }
    }
}

struct GroupContext {
    span: Span,
    pad_offset: usize,
    base_offset: usize,
    time_num: u8,
}

struct KeyedLine {
    key: String,
    content: String,
    content_offset: usize,
    key_prefix_span: Span,
    /// Span of just the trimmed abbreviation text (excluding `[`/`]` and inner
    /// whitespace), distinct from `key_prefix_span` which covers the whole
    /// bracketed prefix and must keep doing so for `part_key_unknown`'s error span.
    key_span: Span,
}

fn key_prefix_span_in_line(line: &str, line_offset: usize, base_offset: usize) -> Span {
    let end = line
        .find(']')
        .map(|index| index + 1)
        .unwrap_or_else(|| line.len().min(1));
    Span::new(base_offset + line_offset, base_offset + line_offset + end)
}

/// Span of just the trimmed abbreviation text inside a `[Key]` prefix, e.g. for
/// `[ Sop ] 1 2 3 4` this is the span of `Sop`, excluding brackets/whitespace.
fn key_span_in_line(line: &str, line_offset: usize, base_offset: usize) -> Option<Span> {
    let inner = line.strip_prefix('[')?;
    let close = inner.find(']')?;
    let raw_key = &inner[..close];
    let leading_ws = raw_key.len() - raw_key.trim_start().len();
    let trimmed = raw_key.trim();
    let key_start = base_offset + line_offset + 1 + leading_ws;
    Some(Span::new(key_start, key_start + trimmed.len()))
}

fn expand_measure_group(
    group: &[RawSourceLine],
    declarations: &[PartDecl],
    resolved_groups: &[ResolvedGroup],
    base_offset: usize,
    time_num: u8,
) -> ExpandMeasureGroupResult {
    let directive_count = measure_group::directive_line_count(group);
    let directive_lines = group.get(..directive_count).unwrap_or(&[]);
    let data_lines = group.get(directive_count..).unwrap_or(&[]);

    let span = data_lines
        .last()
        .or(group.last())
        .map(|(_, off)| Span::new(base_offset + *off, base_offset + *off + 1))
        .unwrap_or(Span::new(base_offset, base_offset + 1));

    let pad_offset = data_lines.last().map(|(_, off)| *off).unwrap_or(0);
    let context = GroupContext {
        span,
        pad_offset,
        base_offset,
        time_num,
    };

    let mut recoverable_error: Option<RecoverableError> = None;
    let mut keyed: Vec<KeyedLine> = Vec::new();

    for (line, offset) in data_lines {
        if let Some((key, content)) = parse_key_prefix(line) {
            let prefix_length = line.len().saturating_sub(content.len());
            let key_span = key_span_in_line(line, *offset, base_offset)
                .unwrap_or_else(|| key_prefix_span_in_line(line, *offset, base_offset));
            keyed.push(KeyedLine {
                key: key.to_string(),
                content: content.to_string(),
                content_offset: *offset + prefix_length,
                key_prefix_span: key_prefix_span_in_line(line, *offset, base_offset),
                key_span,
            });
        } else {
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::score_line_missing_key_prefix(Span::new(
                    base_offset + offset,
                    base_offset + offset + 1,
                ))
            });
        }
    }

    let abbreviation_references: Vec<AbbreviationReference> = keyed
        .iter()
        .map(|line| AbbreviationReference {
            abbreviation: line.key.clone(),
            span: line.key_span,
        })
        .collect();

    let (result_data, slots) = if keyed.is_empty() {
        recoverable_error.replace(RecoverableError::measure_no_data_lines(context.span));
        // No `[Key]`-prefixed line could be attributed to any declaration, so
        // there is no real per-group role list to compute. Fall back to each
        // declaration's static default role list (the same shape
        // `interleaved_parser`'s degenerate single-synthesized-line repair
        // expects), so slot routing doesn't go out of range downstream.
        let default_slots = declarations
            .iter()
            .enumerate()
            .flat_map(|(track_index, decl)| {
                roles_for_group(decl, None, None)
                    .into_iter()
                    .map(move |role| ScoreLineSlot { track_index, role })
            })
            .collect();
        (Vec::new(), default_slots)
    } else {
        expand_keyed(
            keyed,
            declarations,
            resolved_groups,
            &context,
            &mut recoverable_error,
        )
    };

    let mut result: Vec<SourceLine> = directive_lines
        .iter()
        .map(|(content, offset)| SourceLine {
            content: content.clone(),
            offset: *offset,
            group: None,
        })
        .collect();
    result.extend(result_data);
    Ok((result, slots, recoverable_error, abbreviation_references))
}

fn expand_keyed(
    keyed: Vec<KeyedLine>,
    declarations: &[PartDecl],
    resolved_groups: &[ResolvedGroup],
    context: &GroupContext,
    recoverable_error: &mut Option<RecoverableError>,
) -> (Vec<SourceLine>, Vec<ScoreLineSlot>) {
    let key_map = key_map::filter_keyed_into_key_map(
        keyed,
        declarations,
        resolved_groups,
        context,
        recoverable_error,
    );
    resolve_tracks(&key_map, declarations, context)
}

/// The score-line roles this part contributes to this specific measure group.
/// For `NotesWithLyrics`, the number of `Lyrics` roles is the number of
/// consecutive `[Part]` lyric lines actually written after the notes line in
/// this group (verses 1..N), defaulting to a single implicit-fill verse when
/// no lyrics line was written at all. Other kinds keep their static role list.
fn roles_for_group(
    decl: &PartDecl,
    key_lines: Option<&[SourceLine]>,
    follow_target_roles: Option<&[ScoreLineRole]>,
) -> Vec<ScoreLineRole> {
    match (decl.kind, key_lines) {
        (PartKind::NotesWithLyrics, Some(lines)) => {
            let verse_count = lines.len().saturating_sub(1).max(1);
            std::iter::once(ScoreLineRole::Notes)
                .chain(itertools::repeat_n(ScoreLineRole::Lyrics, verse_count))
                .collect()
        }
        (PartKind::NotesWithLyrics, None) => follow_target_roles
            .map(|roles| roles.to_vec())
            .unwrap_or_else(|| decl.score_line_roles().to_vec()),
        (PartKind::Lyrics, Some(lines)) => {
            let verse_count = lines.len().max(1);
            itertools::repeat_n(ScoreLineRole::Lyrics, verse_count).collect()
        }
        _ => decl.score_line_roles().to_vec(),
    }
}

fn resolve_tracks(
    key_map: &KeyMap,
    declarations: &[PartDecl],
    context: &GroupContext,
) -> (Vec<SourceLine>, Vec<ScoreLineSlot>) {
    let mut resolved_per_track: Vec<Vec<SourceLine>> = Vec::with_capacity(declarations.len());
    let mut roles_per_track: Vec<Vec<ScoreLineRole>> = Vec::with_capacity(declarations.len());

    for i in 0..declarations.len() {
        let Some(decl) = declarations.get(i) else {
            continue;
        };
        let key_lines = key_map
            .iter()
            .find(|(k, _)| k == &decl.abbreviation)
            .map(|(_, v)| v.as_slice());
        let follow_target_index = decl.follow_target.as_ref().and_then(|target| {
            declarations
                .get(..i)
                .unwrap_or(&[])
                .iter()
                .position(|d| &d.abbreviation == target)
        });

        let roles = roles_for_group(
            decl,
            key_lines,
            follow_target_index
                .and_then(|t| roles_per_track.get(t))
                .map(Vec::as_slice),
        );

        let track_lines: Vec<SourceLine> = roles
            .iter()
            .enumerate()
            .map(|(slot_index, &role)| {
                if let Some(line) = key_lines.and_then(|ls| ls.get(slot_index)) {
                    return line.clone();
                }
                if let Some(line) = follow_target_index
                    .and_then(|t| resolved_per_track.get(t))
                    .and_then(|track| track.get(slot_index))
                {
                    return line.clone();
                }
                SourceLine {
                    content: implicit_fill(role, context.time_num),
                    offset: context.pad_offset,
                    group: None,
                }
            })
            .collect();
        resolved_per_track.push(track_lines);
        roles_per_track.push(roles);
    }

    let lines = resolved_per_track.into_iter().flatten().collect();
    let slots = roles_per_track
        .into_iter()
        .enumerate()
        .flat_map(|(track_index, roles)| {
            roles
                .into_iter()
                .map(move |role| ScoreLineSlot { track_index, role })
        })
        .collect();
    (lines, slots)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
