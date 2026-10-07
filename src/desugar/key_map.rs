use crate::ast::parsed::PartDecl;
use crate::error::{RecoverableError, Span};

use super::attribution::KeyedLine;
use super::{GroupContext, SourceLine};

/// The lines one part owns in a measure group: its notes/chords line, and one
/// line per verse (indexed by position in [`PartDecl::verses`]).
#[derive(Default)]
pub(super) struct PartLines {
    pub(super) base: Option<SourceLine>,
    pub(super) verses: Vec<Option<SourceLine>>,
}

/// One entry per declaration, aligned with `declarations`.
pub(super) type KeyMap = Vec<PartLines>;

/// Which slot of which declaration a `[Key]` addresses.
enum SlotAddress {
    Base {
        declaration_index: usize,
    },
    Verse {
        declaration_index: usize,
        verse_index: usize,
    },
}

fn address_of(key: &str, declarations: &[PartDecl]) -> Option<SlotAddress> {
    declarations
        .iter()
        .enumerate()
        .find_map(|(declaration_index, declaration)| {
            if declaration.abbreviation == key {
                return Some(SlotAddress::Base { declaration_index });
            }
            declaration
                .verses
                .iter()
                .position(|verse| verse.abbreviation == key)
                .map(|verse_index| SlotAddress::Verse {
                    declaration_index,
                    verse_index,
                })
        })
}

pub(super) fn filter_keyed_into_key_map(
    keyed: Vec<KeyedLine>,
    declarations: &[PartDecl],
    context: &GroupContext,
    recoverable_error: &mut Option<RecoverableError>,
) -> KeyMap {
    let mut key_map: KeyMap = declarations.iter().map(|_| PartLines::default()).collect();

    for line in keyed {
        let Some(address) = address_of(&line.key, declarations) else {
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::part_key_unknown(line.key_prefix_span, &line.key)
            });
            continue;
        };
        let source_line = SourceLine {
            content: line.content,
            offset: line.content_offset,
            is_implicit_fill: false,
        };
        let slot = match address {
            SlotAddress::Base { declaration_index } => key_map
                .get_mut(declaration_index)
                .map(|lines| &mut lines.base),
            SlotAddress::Verse {
                declaration_index,
                verse_index,
            } => key_map.get_mut(declaration_index).and_then(|lines| {
                if lines.verses.len() <= verse_index {
                    lines.verses.resize_with(verse_index + 1, || None);
                }
                lines.verses.get_mut(verse_index)
            }),
        };
        let Some(slot) = slot else { continue };
        if slot.is_some() {
            let line_span = Span::new(
                context.base_offset + source_line.offset,
                context.base_offset + source_line.offset + 1,
            );
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::general(
                    line_span,
                    format!("part [{}] has 2 lines but only 1 slot(s)", line.key),
                )
            });
        } else {
            *slot = Some(source_line);
        }
    }

    key_map
}
