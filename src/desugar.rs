use crate::ast::parsed::{flatten_score_line_slots, PartDecl, ScoreLineRole};
use crate::error::{IrrecoverableError, RecoverableError, Span};
use crate::parser::group_parser::ResolvedGroup;
use crate::parser::score::measure_group;

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
type KeyMap = Vec<(String, Vec<SourceLine>)>;
type DesugarGroupsResult =
    Result<(Vec<MeasureGroup>, Vec<Option<RecoverableError>>), IrrecoverableError>;

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
    let slots = flatten_score_line_slots(declarations);
    let mut desugared = Vec::with_capacity(groups.len());
    let mut per_group_errors = Vec::with_capacity(groups.len());
    let mut current_time_num: u8 = 4;
    for group in groups {
        if let Some(num) = extract_time_numerator(&group) {
            current_time_num = num;
        }
        let (expanded, error) = expand_measure_group(
            &group,
            declarations,
            resolved_groups,
            &slots,
            base_offset,
            current_time_num,
        )?;
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

struct KeyedLine {
    key: String,
    content: String,
    content_offset: usize,
    key_prefix_span: Span,
}

fn key_prefix_span_in_line(line: &str, line_offset: usize, base_offset: usize) -> Span {
    let end = line
        .find(']')
        .map(|index| index + 1)
        .unwrap_or_else(|| line.len().min(1));
    Span::new(base_offset + line_offset, base_offset + line_offset + end)
}

fn expand_measure_group(
    group: &[RawSourceLine],
    declarations: &[PartDecl],
    resolved_groups: &[ResolvedGroup],
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
    let mut keyed: Vec<KeyedLine> = Vec::new();

    for (line, offset) in data_lines {
        if let Some((key, content)) = parse_key_prefix(line) {
            let prefix_length = line.len().saturating_sub(content.len());
            keyed.push(KeyedLine {
                key: key.to_string(),
                content: content.to_string(),
                content_offset: *offset + prefix_length,
                key_prefix_span: key_prefix_span_in_line(line, *offset, base_offset),
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

    let result_data = if keyed.is_empty() {
        recoverable_error.replace(RecoverableError::measure_no_data_lines(context.span));
        Vec::new()
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
    Ok((result, recoverable_error))
}

fn expand_keyed(
    keyed: Vec<KeyedLine>,
    declarations: &[PartDecl],
    resolved_groups: &[ResolvedGroup],
    context: &GroupContext,
    recoverable_error: &mut Option<RecoverableError>,
) -> Vec<SourceLine> {
    let key_map = filter_keyed_into_key_map(
        keyed,
        declarations,
        resolved_groups,
        context,
        recoverable_error,
    );
    resolve_tracks(&key_map, declarations, context)
}

/// Groups keyed lines that share the same key, in file order, appending each line's
/// content to that key's slot list (first line -> first slot, second line -> second slot, ...).
/// Lines built this way carry no group provenance (`group: None`): they are either a
/// part's own direct lines, or a group's own broadcast content prior to distribution.
fn group_by_key(lines: Vec<KeyedLine>) -> KeyMap {
    let mut map: KeyMap = Vec::new();
    for line in lines {
        let entry = SourceLine {
            content: line.content,
            offset: line.content_offset,
            group: None,
        };
        if let Some(existing) = map.iter_mut().find(|(k, _)| k == &line.key) {
            existing.1.push(entry);
        } else {
            map.push((line.key, vec![entry]));
        }
    }
    map
}

/// Fills each group broadcast's members with its content, one content line per slot index,
/// skipping any slot a member already has an explicit direct line for (direct lines win).
/// Lines filled this way are tagged with the group's abbreviation as their provenance.
fn merge_group_broadcasts(
    key_map: &mut KeyMap,
    group_map: KeyMap,
    resolved_groups: &[ResolvedGroup],
) {
    for (group_abbrev, contents) in group_map {
        let Some(resolved) = resolved_groups
            .iter()
            .find(|g| g.abbreviation == group_abbrev)
        else {
            continue;
        };
        for member in &resolved.members {
            if !key_map.iter().any(|(k, _)| k == member) {
                key_map.push((member.clone(), Vec::new()));
            }
            let Some((_, lines)) = key_map.iter_mut().find(|(k, _)| k == member) else {
                continue;
            };
            for (index, content) in contents.iter().enumerate() {
                if index == lines.len() {
                    lines.push(SourceLine {
                        content: content.content.clone(),
                        offset: content.offset,
                        group: Some(group_abbrev.clone()),
                    });
                }
            }
        }
    }
}

fn filter_keyed_into_key_map(
    keyed: Vec<KeyedLine>,
    declarations: &[PartDecl],
    resolved_groups: &[ResolvedGroup],
    context: &GroupContext,
    recoverable_error: &mut Option<RecoverableError>,
) -> KeyMap {
    let mut part_keyed = Vec::new();
    let mut group_keyed = Vec::new();
    for line in keyed {
        if declarations.iter().any(|d| d.abbreviation == line.key) {
            part_keyed.push(line);
        } else if resolved_groups.iter().any(|g| g.abbreviation == line.key) {
            group_keyed.push(line);
        } else {
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::part_key_unknown(line.key_prefix_span, &line.key)
            });
        }
    }

    let mut key_map = group_by_key(part_keyed);
    let group_map = group_by_key(group_keyed);
    merge_group_broadcasts(&mut key_map, group_map, resolved_groups);

    for (abbrev, lines) in &key_map {
        if let Some(decl) = declarations.iter().find(|d| &d.abbreviation == abbrev) {
            let slot_count = decl.score_line_roles().len();
            if let Some(excess) = lines.get(slot_count) {
                let line_span = Span::new(
                    context.base_offset + excess.offset,
                    context.base_offset + excess.offset + 1,
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
                SourceLine {
                    content: implicit_fill(role, context.time_num),
                    offset: context.pad_offset,
                    group: None,
                }
            })
            .collect();
        resolved_per_track.push(track_lines);
    }

    resolved_per_track.into_iter().flatten().collect()
}

#[cfg(test)]
#[path = "desugar_tests.rs"]
mod tests;
