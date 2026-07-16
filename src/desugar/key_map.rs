use crate::ast::parsed::{PartDecl, PartKind};
use crate::error::{RecoverableError, Span};
use crate::parser::group_parser::ResolvedGroup;

use super::{GroupContext, KeyedLine, SourceLine};

pub(super) type KeyMap = Vec<(String, Vec<SourceLine>)>;

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

pub(super) fn filter_keyed_into_key_map(
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
            // `NotesWithLyrics` parts accept any number of lyric-verse lines after
            // the notes line, so only fixed-schema kinds are capacity-checked here.
            if matches!(decl.kind, PartKind::NotesWithLyrics) {
                continue;
            }
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
