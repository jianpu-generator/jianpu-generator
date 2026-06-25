use crate::ast::parsed::{flatten_score_line_slots, PartDecl, ScoreLineRole};
use crate::error::{IrrecoverableError, RecoverableError, Span};
use crate::parser::score::measure_group;

type SourceLine = (String, usize);
type MeasureGroup = Vec<SourceLine>;
type KeyMap = Vec<(String, Vec<SourceLine>)>;
type DesugarGroupsResult =
    Result<(Vec<MeasureGroup>, Vec<Option<RecoverableError>>), IrrecoverableError>;

fn extract_time_numerator(group: &[SourceLine]) -> Option<u8> {
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

pub fn desugar_groups(
    groups: Vec<MeasureGroup>,
    declarations: &[PartDecl],
    base_offset: usize,
) -> DesugarGroupsResult {
    let slots = flatten_score_line_slots(declarations);
    let mut desugared = Vec::with_capacity(groups.len());
    let mut per_group_errors = Vec::with_capacity(groups.len());
    let mut current_time_num: u8 = 4;
    for group in groups {
        if let Some(num) = extract_time_numerator(&group) {
            current_time_num = num;
        }
        let (expanded, error) =
            expand_measure_group(&group, declarations, &slots, base_offset, current_time_num)?;
        desugared.push(expanded);
        per_group_errors.push(error);
    }
    Ok((desugared, per_group_errors))
}

pub(crate) fn parse_key_prefix(line: &str) -> Option<(&str, &str)> {
    line.strip_prefix('[')
        .and_then(|s| s.find(']').map(|i| (s[..i].trim(), s[i + 1..].trim())))
}

fn implicit_fill(role: ScoreLineRole, time_num: u8) -> String {
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
    group: &[SourceLine],
    declarations: &[PartDecl],
    _slots: &[crate::ast::parsed::ScoreLineSlot],
    base_offset: usize,
    time_num: u8,
) -> Result<(MeasureGroup, Option<RecoverableError>), IrrecoverableError> {
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
    let mut keyed: Vec<(String, String, usize)> = Vec::new();

    for (line, offset) in data_lines {
        if let Some((key, content)) = parse_key_prefix(line) {
            let prefix_length = line.len().saturating_sub(content.len());
            keyed.push((key.to_string(), content.to_string(), offset + prefix_length));
        } else {
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::score_line_missing_key_prefix(Span::new(
                    base_offset + offset,
                    base_offset + offset + 1,
                ))
            });
        }
    }

    let result_data = if keyed.is_empty() {
        recoverable_error.replace(RecoverableError::measure_no_data_lines(context.span));
        Vec::new()
    } else {
        expand_keyed(keyed, declarations, &context, &mut recoverable_error)
    };

    let mut result = directive_lines.to_vec();
    result.extend(result_data);
    Ok((result, recoverable_error))
}

fn expand_keyed(
    keyed: Vec<(String, String, usize)>,
    declarations: &[PartDecl],
    context: &GroupContext,
    recoverable_error: &mut Option<RecoverableError>,
) -> Vec<SourceLine> {
    let key_map = filter_keyed_into_key_map(keyed, declarations, context, recoverable_error);
    resolve_tracks(&key_map, declarations, context)
}

fn filter_keyed_into_key_map(
    keyed: Vec<(String, String, usize)>,
    declarations: &[PartDecl],
    context: &GroupContext,
    recoverable_error: &mut Option<RecoverableError>,
) -> KeyMap {
    let valid_keyed: Vec<_> = keyed
        .into_iter()
        .filter(|(key, _, offset)| {
            let line_span = Span::new(
                context.base_offset + offset,
                context.base_offset + offset + 1,
            );
            if !declarations.iter().any(|d| &d.abbreviation == key) {
                recoverable_error
                    .get_or_insert_with(|| RecoverableError::part_key_unknown(line_span, key));
                false
            } else {
                true
            }
        })
        .collect();

    let mut key_map: KeyMap = Vec::new();
    for (key, content, offset) in valid_keyed {
        if let Some(entry) = key_map.iter_mut().find(|(k, _)| k == &key) {
            entry.1.push((content, offset));
        } else {
            key_map.push((key, vec![(content, offset)]));
        }
    }

    for (abbrev, lines) in &key_map {
        if let Some(decl) = declarations.iter().find(|d| &d.abbreviation == abbrev) {
            let slot_count = decl.score_line_roles().len();
            if let Some((_, excess_offset)) = lines.get(slot_count) {
                let line_span = Span::new(
                    context.base_offset + excess_offset,
                    context.base_offset + excess_offset + 1,
                );
                recoverable_error.get_or_insert_with(|| {
                    RecoverableError::general(
                        line_span,
                        format!(
                            "part [{}] has {} lines but only {} slot(s)",
                            abbrev,
                            lines.len(),
                            slot_count
                        ),
                    )
                });
            }
        }
    }

    key_map
}

fn resolve_tracks(
    key_map: &KeyMap,
    declarations: &[PartDecl],
    context: &GroupContext,
) -> Vec<SourceLine> {
    let mut resolved_per_track: Vec<Vec<SourceLine>> = Vec::with_capacity(declarations.len());

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

        let track_lines: Vec<SourceLine> = decl
            .score_line_roles()
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
                (implicit_fill(role, context.time_num), context.pad_offset)
            })
            .collect();
        resolved_per_track.push(track_lines);
    }

    resolved_per_track.into_iter().flatten().collect()
}

#[cfg(test)]
#[path = "desugar_tests.rs"]
mod tests;
