use crate::ast::parsed::{flatten_score_line_slots, PartDecl};
use crate::parser::score::measure_group;

/// A pre-desugar score data line that should display a part inlay hint in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreLineHint {
    /// UTF-8 byte offset of the first character on the line in the full source file.
    pub line_start: usize,
    /// Part abbreviation from the `[parts]` declaration for this score line slot.
    pub abbreviation: String,
}

/// Build inlay-hint positions from raw measure groups before desugaring.
///
/// Only physical lines present in the source receive hints. Implicitly omitted
/// lines (padded later by desugar) are excluded.
pub fn score_line_hints(
    groups: &[Vec<(String, usize)>],
    score_offset: usize,
    declarations: &[PartDecl],
) -> Vec<ScoreLineHint> {
    let slots = flatten_score_line_slots(declarations);
    let mut hints = Vec::new();

    for group in groups {
        let directive_count = measure_group::directive_line_count(group);
        let data_lines = group.get(directive_count..).unwrap_or(&[]);

        for (slot_index, (_, line_offset)) in data_lines.iter().enumerate() {
            let Some(slot) = slots.get(slot_index) else {
                break;
            };
            let Some(declaration) = declarations.get(slot.track_index) else {
                continue;
            };
            hints.push(ScoreLineHint {
                line_start: score_offset + line_offset,
                abbreviation: declaration.abbreviation.clone(),
            });
        }
    }

    hints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::PartKind;

    fn decl(name: &str, kind: PartKind) -> PartDecl {
        PartDecl {
            abbreviation: name.to_string(),
            display_name: name.to_string(),
            kind,
        }
    }

    fn group(lines: &[&str], base_offset: usize) -> Vec<(String, usize)> {
        lines
            .iter()
            .enumerate()
            .map(|(index, line)| (line.to_string(), base_offset + index * 10))
            .collect()
    }

    #[test]
    fn hints_follow_slot_order_for_notes_with_lyrics() {
        let groups = vec![group(
            &[
                "(time=4/4 key=C4 bpm=120)",
                "1 - - -",
                "1 1 5 5",
                "twin- kle",
            ],
            100,
        )];
        let declarations = vec![
            decl("Chord", PartKind::Chord),
            decl("Melody", PartKind::NotesWithLyrics),
        ];
        let hints = score_line_hints(&groups, 50, &declarations);
        assert_eq!(
            hints,
            vec![
                ScoreLineHint {
                    line_start: 160,
                    abbreviation: "Chord".to_string(),
                },
                ScoreLineHint {
                    line_start: 170,
                    abbreviation: "Melody".to_string(),
                },
                ScoreLineHint {
                    line_start: 180,
                    abbreviation: "Melody".to_string(),
                },
            ]
        );
    }

    #[test]
    fn omitted_trailing_lines_have_no_hints() {
        let groups = vec![group(&["1 2 3 4"], 0)];
        let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
        let hints = score_line_hints(&groups, 0, &declarations);
        assert_eq!(
            hints,
            vec![ScoreLineHint {
                line_start: 0,
                abbreviation: "A".to_string(),
            }]
        );
    }

    #[test]
    fn explicit_ditto_line_gets_hint() {
        let groups = vec![group(&["1 2 3 4", "\""], 0)];
        let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
        let hints = score_line_hints(&groups, 0, &declarations);
        assert_eq!(hints.len(), 2);
        assert_eq!(hints[1].abbreviation, "B");
        assert_eq!(hints[1].line_start, 10);
    }

    #[test]
    fn lyrics_notes_order_uses_slot_mapping() {
        let groups = vec![group(&["fa fo fi", "1 2 3 4"], 0)];
        let declarations = vec![decl("Melody", PartKind::LyricsWithNotes)];
        let hints = score_line_hints(&groups, 0, &declarations);
        assert_eq!(hints.len(), 2);
        assert_eq!(hints[0].abbreviation, "Melody");
        assert_eq!(hints[1].abbreviation, "Melody");
    }
}
