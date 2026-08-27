use crate::ast::parsed::PartDecl;
use crate::error::{RecoverableError, Span};

use super::attribution::KeyedLine;
use super::{GroupContext, SourceLine};

pub(super) type KeyMap = Vec<(String, Vec<SourceLine>)>;

/// Groups keyed lines that share the same key, in file order, appending each line's
/// content to that key's slot list (first line -> first slot, second line -> second slot, ...).
fn group_by_key(lines: Vec<KeyedLine>) -> KeyMap {
    let mut map: KeyMap = Vec::new();
    for line in lines {
        let entry = SourceLine {
            content: line.content,
            offset: line.content_offset,
            is_implicit_fill: false,
            is_positional: line.is_positional,
        };
        if let Some(existing) = map.iter_mut().find(|(k, _)| k == &line.key) {
            existing.1.push(entry);
        } else {
            map.push((line.key, vec![entry]));
        }
    }
    map
}

pub(super) fn filter_keyed_into_key_map(
    keyed: Vec<KeyedLine>,
    declarations: &[PartDecl],
    context: &GroupContext,
    recoverable_error: &mut Option<RecoverableError>,
) -> KeyMap {
    let mut part_keyed = Vec::new();
    for line in keyed {
        if declarations.iter().any(|d| d.abbreviation == line.key) {
            part_keyed.push(line);
        } else {
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::part_key_unknown(line.key_prefix_span, &line.key)
            });
        }
    }

    let key_map = group_by_key(part_keyed);

    for (abbrev, lines) in &key_map {
        if let Some(decl) = declarations.iter().find(|d| &d.abbreviation == abbrev) {
            // Positionally-attached bare lines don't count against a
            // fixed-schema part's slot count — they can be arbitrarily many
            // (one per lyric verse). Only genuinely duplicated `[Abbrev]`-
            // prefixed lines trip this check.
            let non_positional_count = lines.iter().filter(|l| !l.is_positional).count();
            let slot_count = decl.score_line_roles().len();
            if let Some(excess) = lines.iter().filter(|l| !l.is_positional).nth(slot_count) {
                let line_span = Span::new(
                    context.base_offset + excess.offset,
                    context.base_offset + excess.offset + 1,
                );
                recoverable_error.get_or_insert_with(|| {
                    RecoverableError::general(
                        line_span,
                        format!(
                            "part [{abbrev}] has {non_positional_count} lines but only {slot_count} slot(s)"
                        ),
                    )
                });
            }
        }
    }

    key_map
}
