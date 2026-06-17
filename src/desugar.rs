use crate::ast::parsed::{flatten_score_line_slots, PartDecl, ScoreLineRole};
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span};
use crate::parser::score::measure_group;

type SourceLine = (String, usize);
type MeasureGroup = Vec<SourceLine>;

/// Resolves `"` ditto lines within each measure group.
///
/// A `"` on a data line means "same content as the closest preceding line of
/// the same score line role in this group." The directive line (starts with `(`)
/// is never a ditto source or target.
///
/// Returns `(groups, per_group_errors)`. Hard failures remain `Err`; recoverable
/// layout issues (missing lines, ditto with no precedent) produce `Ok` with a
/// `Some` error entry and a placeholder so rendering can continue.
pub fn desugar_groups(
    groups: Vec<MeasureGroup>,
    declarations: &[PartDecl],
    base_offset: usize,
) -> Result<(Vec<MeasureGroup>, Vec<Option<RecoverableError>>), IrrecoverableError> {
    let slots = flatten_score_line_slots(declarations);
    let mut desugared = Vec::with_capacity(groups.len());
    let mut per_group_errors = Vec::with_capacity(groups.len());
    for group in groups {
        let (padded, pad_error) =
            pad_implicit_ditto_group(&group, declarations, &slots, base_offset)?;
        let (desugared_group, desugar_error) =
            desugar_group(&padded, declarations, &slots, base_offset)?;
        desugared.push(desugared_group);
        per_group_errors.push(pad_error.or(desugar_error));
    }
    Ok((desugared, per_group_errors))
}

fn pad_implicit_ditto_group(
    group: &[SourceLine],
    declarations: &[PartDecl],
    slots: &[crate::ast::parsed::ScoreLineSlot],
    base_offset: usize,
) -> Result<(MeasureGroup, Option<RecoverableError>), IrrecoverableError> {
    let directive_count = measure_group::directive_line_count(group);

    let directive_lines = group.get(..directive_count).unwrap_or(&[]);
    let data_lines = group.get(directive_count..).unwrap_or(&[]);

    let span = data_lines
        .last()
        .or(group.last())
        .map(|(_, off)| Span::new(base_offset + *off, base_offset + *off + 1))
        .unwrap_or(Span::new(base_offset, base_offset + 1));

    let mut recoverable_error: Option<RecoverableError> = None;

    let effective_data_lines: Vec<(String, usize)> = if data_lines.is_empty() {
        recoverable_error = Some(RecoverableError::new(
            span,
            "measure has no data lines; treating all parts as empty".to_string(),
        ));
        Vec::new()
    } else if data_lines.len() > slots.len() {
        let part_list = declarations
            .iter()
            .map(|d| d.abbreviation.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        recoverable_error = Some(RecoverableError::new(
            span,
            format!(
                "this measure has {} lines but only {} expected (declared parts: {}); extra lines ignored",
                data_lines.len(),
                slots.len(),
                part_list,
            ),
        ));
        data_lines.get(..slots.len()).unwrap_or(data_lines).to_vec()
    } else {
        data_lines.to_vec()
    };

    let pad_offset = effective_data_lines
        .last()
        .map(|(_, off)| *off)
        .unwrap_or(0);
    let mut result_data: Vec<(String, usize)> = effective_data_lines.clone();

    for i in effective_data_lines.len()..slots.len() {
        let slot = slots.get(i).ok_or_else(|| {
            IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
                Span::new(0, 0),
                "score line slot missing for implicit ditto padding",
            ))
        })?;
        let role = slot.role;
        let has_precedent =
            (0..result_data.len()).any(|j| slots.get(j).map(|s| s.role == role).unwrap_or(false));

        if has_precedent {
            result_data.push(("\"".to_string(), pad_offset));
        } else if role == ScoreLineRole::Lyrics {
            let abbrev = track_abbreviation(declarations, slot.track_index);
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::new(
                    Span::new(base_offset + pad_offset, base_offset + pad_offset + 1),
                    format!("missing lyrics line for '{abbrev}'; treating as no lyrics"),
                )
            });
            result_data.push(("_".to_string(), pad_offset));
        } else if role == ScoreLineRole::Notes {
            let abbrev = track_abbreviation(declarations, slot.track_index);
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::new(
                    Span::new(base_offset + pad_offset, base_offset + pad_offset + 1),
                    format!("missing notes line for '{abbrev}'; treating as empty"),
                )
            });
            result_data.push(("_".to_string(), pad_offset));
        } else {
            // ScoreLineRole::Chord (the only remaining variant)
            let abbrev = track_abbreviation(declarations, slot.track_index);
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::new(
                    Span::new(base_offset + pad_offset, base_offset + pad_offset + 1),
                    format!("missing chord line for '{abbrev}'; treating as empty"),
                )
            });
            result_data.push(("_".to_string(), pad_offset));
        }
    }

    let mut result = directive_lines.to_vec();
    result.extend(result_data);
    Ok((result, recoverable_error))
}

fn desugar_group(
    group: &[SourceLine],
    _declarations: &[PartDecl],
    slots: &[crate::ast::parsed::ScoreLineSlot],
    base_offset: usize,
) -> Result<(MeasureGroup, Option<RecoverableError>), IrrecoverableError> {
    let directive_count = measure_group::directive_line_count(group);

    let directive_lines = group.get(..directive_count).unwrap_or(&[]).to_vec();
    let data_lines = group.get(directive_count..).unwrap_or(&[]);

    let mut resolved: Vec<(String, usize)> = Vec::with_capacity(data_lines.len());
    let mut recoverable_error: Option<RecoverableError> = None;

    for (i, (line, offset)) in data_lines.iter().enumerate() {
        if line == "\"" {
            if i >= slots.len() {
                resolved.push((line.clone(), *offset));
                continue;
            }
            let role = slots.get(i).map(|s| s.role).ok_or_else(|| {
                IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
                    Span::new(0, 0),
                    "score line slot missing for ditto line",
                ))
            })?;
            let source = (0..resolved.len())
                .rev()
                .find(|&j| slots.get(j).map(|s| s.role == role).unwrap_or(false))
                .and_then(|j| resolved.get(j).map(|r| r.0.clone()));

            match source {
                Some(src_content) => resolved.push((src_content, *offset)),
                None => {
                    let span = Span::new(base_offset + *offset, base_offset + *offset + 1);
                    recoverable_error.get_or_insert_with(|| {
                        RecoverableError::new(
                            span,
                            format!(
                                "ditto '\"' has no preceding {} line in this measure group",
                                role_name(role)
                            ),
                        )
                    });
                    resolved.push((ditto_no_precedent_placeholder(role), *offset));
                }
            }
        } else {
            resolved.push((line.clone(), *offset));
        }
    }

    let mut result = directive_lines;
    result.extend(resolved);
    Ok((result, recoverable_error))
}

fn ditto_no_precedent_placeholder(role: ScoreLineRole) -> String {
    match role {
        ScoreLineRole::Notes | ScoreLineRole::Lyrics | ScoreLineRole::Chord => "_".to_string(),
    }
}

fn role_name(role: ScoreLineRole) -> &'static str {
    match role {
        ScoreLineRole::Notes => "notes",
        ScoreLineRole::Lyrics => "lyrics",
        ScoreLineRole::Chord => "chord",
    }
}

fn track_abbreviation(declarations: &[PartDecl], track_index: usize) -> &str {
    declarations
        .get(track_index)
        .map(|d| d.abbreviation.as_str())
        .unwrap_or("unknown")
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

    fn group(lines: &[&str]) -> Vec<(String, usize)> {
        lines
            .iter()
            .enumerate()
            .map(|(i, l)| (l.to_string(), i * 10))
            .collect()
    }

    #[test]
    fn notes_ditto_copies_preceding_notes_line() {
        let groups = vec![group(&["1 2 3 4", "\""])];
        let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][1].0, "1 2 3 4");
    }

    #[test]
    fn lyrics_ditto_copies_preceding_lyrics_line() {
        let groups = vec![group(&["1 2 3 4", "hello world", "5 6 7 1", "\""])];
        let declarations = vec![
            decl("A", PartKind::NotesWithLyrics),
            decl("B", PartKind::NotesWithLyrics),
        ];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][3].0, "hello world");
    }

    #[test]
    fn chord_ditto_copies_preceding_chord_line() {
        let groups = vec![group(&["1 - - -", "1 2 3 4", "\"", "5 6 7 1"])];
        let declarations = vec![
            decl("main", PartKind::Chord),
            decl("A", PartKind::Notes),
            decl("main2", PartKind::Chord),
            decl("B", PartKind::Notes),
        ];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][2].0, "1 - - -");
    }

    #[test]
    fn notes_ditto_does_not_copy_lyrics_line() {
        let groups = vec![group(&["1 2 3 4", "hello world", "\""])];
        let declarations = vec![
            decl("A", PartKind::NotesWithLyrics),
            decl("B", PartKind::Notes),
        ];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][2].0, "1 2 3 4");
    }

    #[test]
    fn chained_ditto_resolves_transitively() {
        let groups = vec![group(&["1 2 3 4", "\"", "\""])];
        let declarations = vec![
            decl("A", PartKind::Notes),
            decl("B", PartKind::Notes),
            decl("C", PartKind::Notes),
        ];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][1].0, "1 2 3 4");
        assert_eq!(result[0][2].0, "1 2 3 4");
    }

    #[test]
    fn ditto_no_precedent_is_recoverable() {
        // Explicit `"` with no preceding same-role line must not abort desugaring.
        let groups = vec![group(&["\""])];
        let declarations = vec![decl("A", PartKind::Notes)];
        let (result, errors) = desugar_groups(groups, &declarations, 0)
            .expect("ditto with no precedent must not abort desugaring");
        let error = errors[0]
            .as_ref()
            .expect("should attach a recoverable error");
        assert!(
            error.message.contains("no preceding notes line"),
            "got: {}",
            error.message
        );
        assert_eq!(
            result[0][0].0, "_",
            "ditto without precedent should become an empty placeholder"
        );
    }

    #[test]
    fn ditto_with_no_preceding_line_of_same_type_is_recoverable() {
        let groups = vec![group(&["1 2 3 4", "\""])];
        let declarations = vec![decl("A", PartKind::NotesWithLyrics)];
        let (result, errors) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][1].0, "_");
        let error = errors[0]
            .as_ref()
            .expect("should attach a recoverable error");
        assert!(
            error.message.contains("no preceding lyrics line"),
            "got: {}",
            error.message
        );
    }

    #[test]
    fn directive_line_is_not_a_ditto_target() {
        let groups = vec![group(&["(time=4/4)", "\""])];
        let declarations = vec![decl("A", PartKind::Notes)];
        let (result, errors) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][1].0, "_");
        let error = errors[0]
            .as_ref()
            .expect("should attach a recoverable error");
        assert!(
            error.message.contains("no preceding notes line"),
            "got: {}",
            error.message
        );
    }

    #[test]
    fn directive_line_is_not_a_ditto_source() {
        let groups = vec![group(&["(time=4/4)", "1 2 3 4", "\""])];
        let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][0].0, "(time=4/4)");
        assert_eq!(result[0][2].0, "1 2 3 4");
    }

    #[test]
    fn non_ditto_lines_are_passed_through_unchanged() {
        let groups = vec![group(&["1 2 3 4", "hello"])];
        let declarations = vec![decl("A", PartKind::NotesWithLyrics)];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][0].0, "1 2 3 4");
        assert_eq!(result[0][1].0, "hello");
    }

    #[test]
    fn multiple_groups_are_desugared_independently() {
        let groups = vec![group(&["1 2 3 4"]), group(&["\""])];
        let declarations = vec![decl("A", PartKind::Notes)];
        let (result, errors) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][0].0, "1 2 3 4");
        assert_eq!(result[1][0].0, "_");
        assert!(errors[0].is_none());
        assert!(
            errors[1]
                .as_ref()
                .is_some_and(|error| error.message.contains("no preceding notes line")),
            "second group should carry a recoverable ditto error"
        );
    }

    #[test]
    fn omitted_trailing_notes_line_is_padded_as_implicit_ditto() {
        let groups = vec![group(&["1 2 3 4"])];
        let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][0].0, "1 2 3 4");
        assert_eq!(result[0][1].0, "1 2 3 4");
    }

    #[test]
    fn omitted_trailing_lines_pad_as_ditto_when_precedent_exists() {
        let groups = vec![group(&["1 - - -", "1 2 3 4", "hello"])];
        let declarations = vec![
            decl("main", PartKind::Chord),
            decl("A", PartKind::NotesWithLyrics),
            decl("B", PartKind::NotesWithLyrics),
        ];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][3].0, "1 2 3 4");
        assert_eq!(result[0][4].0, "hello");
    }

    #[test]
    fn omitted_trailing_lyrics_without_precedent_is_recoverable() {
        // Missing lyrics line with no precedent to ditto is now recoverable:
        // desugar fills in `_` (no lyrics) and returns a per-group error.
        let groups = vec![group(&["1 2 3 4"])];
        let declarations = vec![decl("A", PartKind::NotesWithLyrics)];
        let (result, errors) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][1].0, "_", "should fill in underscore placeholder");
        let error = errors[0].as_ref().expect("should have a recoverable error");
        assert!(
            error.message.contains("lyrics"),
            "error should mention lyrics, got: {}",
            error.message
        );
    }

    #[test]
    fn ditto_can_copy_underscore_no_lyrics_marker() {
        let groups = vec![group(&["1 2 3 4", "_", "\""])];
        let declarations = vec![
            decl("A", PartKind::NotesWithLyrics),
            decl("B", PartKind::NotesWithLyrics),
        ];
        let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
        assert_eq!(result[0][3].0, "_");
    }
}
