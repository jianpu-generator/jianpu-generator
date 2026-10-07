//! One-off migration from positional lyric lines (a bare line attached to the
//! `[Key]` line above it) to lyric parts (`Verse 1 [Mv1] = lyrics[M]`, written
//! as `[Mv1] ...` in the score).

use crate::parser::score::measure_group::{collect_groups, directive_line_count};
use crate::parser::section_splitter::{split_sections, SectionKind};

/// A part declared on one `# parts` line, as far as the migration needs it.
struct DeclaredPart {
    abbreviation: String,
    display_name: String,
    /// Byte offset in the source of the end of this declaration's line.
    line_end_offset: usize,
    /// Only notes, chords and follow parts can carry lyrics.
    can_carry_lyrics: bool,
}

/// A text insertion at a byte offset of the original source.
struct Insertion {
    offset: usize,
    text: String,
}

/// Rewrites `source` so every positional lyric line becomes a `[<verse>]`
/// line of a declared `lyrics[...]` part. Sources without positional lyric
/// lines are returned unchanged.
pub fn migrate_lyrics_syntax(source: &str) -> String {
    let (sections, _) = split_sections(source);
    let Some(parts_section) = sections.iter().find(|s| s.kind == SectionKind::Parts) else {
        return source.to_string();
    };
    let Some(score_section) = sections.iter().find(|s| s.kind == SectionKind::Score) else {
        return source.to_string();
    };

    let declared = declared_parts(&parts_section.content, parts_section.content_offset);
    let attachments = positional_attachments(&score_section.content, score_section.content_offset);
    let verse_counts = declared
        .iter()
        .map(|part| {
            attachments
                .iter()
                .filter(|attachment| attachment.target == part.abbreviation)
                .map(|attachment| attachment.verse_number)
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();

    let verse_abbreviations = unique_verse_abbreviations(&declared, &verse_counts);

    let declaration_insertions = declared
        .iter()
        .zip(&verse_counts)
        .zip(&verse_abbreviations)
        .filter(|((part, &count), _)| part.can_carry_lyrics && count > 0)
        .map(|((part, &count), abbreviations)| Insertion {
            offset: part.line_end_offset,
            text: (1..=count)
                .zip(abbreviations)
                .map(|(number, abbreviation)| {
                    format!(
                        "\n{} {} [{}] = lyrics[{}]",
                        part.display_name,
                        verse_label(count, number),
                        abbreviation,
                        part.abbreviation
                    )
                })
                .collect(),
        });

    let line_insertions = attachments.iter().filter_map(|attachment| {
        let index = declared
            .iter()
            .position(|part| part.abbreviation == attachment.target && part.can_carry_lyrics)?;
        let abbreviation = verse_abbreviations
            .get(index)?
            .get(attachment.verse_number - 1)?;
        Some(Insertion {
            offset: attachment.line_offset,
            text: format!("[{abbreviation}] "),
        })
    });

    let mut insertions: Vec<Insertion> = declaration_insertions.chain(line_insertions).collect();
    insertions.sort_by_key(|insertion| std::cmp::Reverse(insertion.offset));
    insertions
        .iter()
        .fold(source.to_string(), |mut text, insertion| {
            text.insert_str(insertion.offset, &insertion.text);
            text
        })
}

fn verse_label(verse_count: usize, verse_number: usize) -> String {
    if verse_count == 1 {
        "lyrics".to_string()
    } else {
        format!("verse {verse_number}")
    }
}

/// Per declared part, the abbreviation of each of its verses (verse 1 first),
/// avoiding every abbreviation already declared.
fn unique_verse_abbreviations(
    declared: &[DeclaredPart],
    verse_counts: &[usize],
) -> Vec<Vec<String>> {
    let mut taken: std::collections::HashSet<String> = declared
        .iter()
        .map(|part| part.abbreviation.clone())
        .collect();
    declared
        .iter()
        .zip(verse_counts)
        .map(|(part, &count)| {
            (1..=count)
                .map(|number| {
                    let mut candidate = format!("{}v{number}", part.abbreviation);
                    while !taken.insert(candidate.clone()) {
                        candidate.push('_');
                    }
                    candidate
                })
                .collect()
        })
        .collect()
}

fn declared_parts(content: &str, content_offset: usize) -> Vec<DeclaredPart> {
    content
        .split_inclusive('\n')
        .scan(content_offset, |line_start, raw_line| {
            let start = *line_start;
            *line_start += raw_line.len();
            Some((start, raw_line.trim_end_matches('\n')))
        })
        .filter_map(|(start, line)| {
            let equals = line.find('=')?;
            let (lhs, rhs) = (line[..equals].trim(), line[equals + 1..].trim());
            let (display_name, abbreviation) = match (lhs.rfind('['), lhs.ends_with(']')) {
                (Some(open), true) => (
                    lhs[..open].trim().to_string(),
                    lhs[open + 1..lhs.len() - 1].trim().to_string(),
                ),
                _ => (lhs.to_string(), lhs.to_string()),
            };
            Some(DeclaredPart {
                abbreviation,
                display_name,
                line_end_offset: start + line.trim_end_matches('\r').len(),
                can_carry_lyrics: !rhs.starts_with("percussion") && !rhs.starts_with("lyrics"),
            })
        })
        .collect()
}

/// A bare score line attached to the `[Key]` line above it.
struct PositionalAttachment {
    target: String,
    /// 1-based position among the bare lines under the same `[Key]` line.
    verse_number: usize,
    /// Byte offset in the source of the start of the line's text.
    line_offset: usize,
}

fn positional_attachments(content: &str, content_offset: usize) -> Vec<PositionalAttachment> {
    collect_groups(content)
        .into_iter()
        .flat_map(|group| {
            let data_start = directive_line_count(&group);
            let mut current_key: Option<(String, usize)> = None;
            let mut attachments = Vec::new();
            for (line, offset) in group.into_iter().skip(data_start) {
                match crate::desugar::parse_key_prefix(&line) {
                    Some((key, _)) => current_key = Some((key.to_string(), 0)),
                    None => {
                        if let Some((key, verses_so_far)) = current_key.as_mut() {
                            *verses_so_far += 1;
                            attachments.push(PositionalAttachment {
                                target: key.clone(),
                                verse_number: *verses_so_far,
                                line_offset: content_offset + offset,
                            });
                        }
                    }
                }
            }
            attachments
        })
        .collect()
}

#[cfg(test)]
mod tests;
