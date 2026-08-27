use crate::ast::parsed::{AbbreviationReference, PartDecl, PartKind, ScoreLineRole, ScoreLineSlot};
use crate::error::{IrrecoverableError, RecoverableError, Span};
use crate::parser::score::measure_group;
use attribution::{attribute_data_lines, KeyedLine};
use key_map::KeyMap;

mod attribution;
mod key_map;

/// A raw, not-yet-desugared score line: `[Key] content`, paired with its byte offset.
type RawSourceLine = (String, usize);

/// A desugared score line, positionally bound to one `PartDecl`'s score-line slot.
#[derive(Debug, Clone)]
pub(crate) struct SourceLine {
    pub(crate) content: String,
    pub(crate) offset: usize,
    /// True when `content` was synthesized by [`implicit_fill`] to stand in for
    /// a declaration absent from this measure group, rather than written by
    /// the composer. Threaded through to `ParsedRest::implicit_fill` so an
    /// omitted part's filled-in rest renders with a distinct glyph.
    pub(crate) is_implicit_fill: bool,
    /// True when this line was synthesized from a bare (unprefixed) data line
    /// attributed to `key` by the positional-lyrics attribution algorithm,
    /// rather than carrying a literal `[Abbrev]` prefix written by the
    /// composer. Threaded through so `abbreviation_references` can exclude
    /// it (there is no literal abbreviation token in source to reference)
    /// and so `key_map.rs`'s fixed-schema capacity check can ignore it.
    pub(crate) is_positional: bool,
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
        let (expanded, slots, error, references) =
            expand_measure_group(&group, declarations, base_offset, current_time_num)?;
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

fn expand_measure_group(
    group: &[RawSourceLine],
    declarations: &[PartDecl],
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
    let keyed = attribute_data_lines(
        data_lines,
        declarations,
        base_offset,
        &mut recoverable_error,
    );

    let abbreviation_references: Vec<AbbreviationReference> = keyed
        .iter()
        .filter(|line| !line.is_positional)
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
        expand_keyed(keyed, declarations, &context, &mut recoverable_error)
    };

    let mut result: Vec<SourceLine> = directive_lines
        .iter()
        .map(|(content, offset)| SourceLine {
            content: content.clone(),
            offset: *offset,
            is_implicit_fill: false,
            is_positional: false,
        })
        .collect();
    result.extend(result_data);
    Ok((result, slots, recoverable_error, abbreviation_references))
}

fn expand_keyed(
    keyed: Vec<KeyedLine>,
    declarations: &[PartDecl],
    context: &GroupContext,
    recoverable_error: &mut Option<RecoverableError>,
) -> (Vec<SourceLine>, Vec<ScoreLineSlot>) {
    let key_map =
        key_map::filter_keyed_into_key_map(keyed, declarations, context, recoverable_error);
    resolve_tracks(&key_map, declarations, context)
}

/// The score-line roles this part contributes to this specific measure group.
/// For `NotesWithLyrics`, the number of `Lyrics` roles is the number of
/// consecutive `[Part]` lyric lines actually written after the notes line in
/// this group (verses 1..N), defaulting to a single implicit-fill verse when
/// no lyrics line was written at all.
///
/// A `Notes`/`Chords` part (any fixed-schema, notes-bearing kind except
/// `Percussion`, which is excluded from positional-lyrics eligibility — see
/// module docs) picks up extra `Lyrics` roles the same way, but *without* the
/// `NotesWithLyrics` floor: those extra lines only exist at all when the
/// composer wrote positionally-attached bare lines after this part's notes
/// line, so zero attached lines means zero verses, not one implicit-fill verse.
///
/// Other kinds keep their static role list.
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
        (PartKind::Notes | PartKind::Chords, Some(lines)) if lines.len() > 1 => {
            let verse_count = lines.len() - 1;
            let base_role = decl
                .score_line_roles()
                .first()
                .copied()
                .unwrap_or(ScoreLineRole::Notes);
            std::iter::once(base_role)
                .chain(itertools::repeat_n(ScoreLineRole::Lyrics, verse_count))
                .collect()
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
                    is_implicit_fill: true,
                    is_positional: false,
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
