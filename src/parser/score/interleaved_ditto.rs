use crate::ast::parsed::{PartDecl, ScoreLineRole};
use crate::desugar::parse_key_prefix;
use crate::parser::score::measure_group;
use std::collections::HashMap;

/// Ditto flags per track, per measure group, computed from the raw groups
/// before desugaring erases the distinction. A line is a ditto when it is an
/// explicit `"` or an omitted trailing line (which desugaring pads as
/// implicit ditto).
pub(crate) struct NotMentionedMeasures {
    /// `[track][measure]`: every score line of the track was a ditto.
    pub full: Vec<Vec<bool>>,
    /// `[track][measure]`: the track's lyric line was a ditto. Always false
    /// for tracks without a lyrics line.
    pub lyrics: Vec<Vec<bool>>,
}

pub(crate) fn compute_not_mentioned_measures(
    groups: &[Vec<(String, usize)>],
    declarations: &[PartDecl],
) -> NotMentionedMeasures {
    let mut full = vec![Vec::with_capacity(groups.len()); declarations.len()];
    let mut lyrics = vec![Vec::with_capacity(groups.len()); declarations.len()];

    let global_slot_starts: Vec<usize> = declarations
        .iter()
        .scan(0usize, |acc, decl| {
            let start = *acc;
            *acc += decl.score_line_roles().len();
            Some(start)
        })
        .collect();

    for group in groups {
        let directive_count = measure_group::directive_line_count(group);
        let data_lines = group.get(directive_count..).unwrap_or(&[]);

        let mut key_counts: HashMap<&str, usize> = HashMap::new();
        let mut positional_count = 0usize;
        let mut has_key_lines = false;

        for (line, _) in data_lines {
            if let Some((abbreviation, _content)) = parse_key_prefix(line) {
                has_key_lines = true;
                *key_counts.entry(abbreviation).or_insert(0) += 1;
            } else {
                positional_count += 1;
            }
        }

        for (track_index, (track_full, track_lyrics)) in
            full.iter_mut().zip(lyrics.iter_mut()).enumerate()
        {
            let (not_mentioned, lyrics_not_mentioned) = if track_index == 0 {
                (false, false)
            } else if let (Some(decl), Some(&gs)) = (
                declarations.get(track_index),
                global_slot_starts.get(track_index),
            ) {
                let roles = decl.score_line_roles();
                let lyrics_local_idx = roles.iter().position(|r| *r == ScoreLineRole::Lyrics);

                if has_key_lines {
                    let key_count = key_counts
                        .get(decl.abbreviation.as_str())
                        .copied()
                        .unwrap_or(0);
                    let not_mentioned = key_count == 0;
                    let lyrics_not_mentioned = lyrics_local_idx
                        .map(|idx| key_count <= idx)
                        .unwrap_or(false);
                    (not_mentioned, lyrics_not_mentioned)
                } else {
                    let not_mentioned = gs >= positional_count;
                    let lyrics_not_mentioned = lyrics_local_idx
                        .map(|idx| (gs + idx) >= positional_count)
                        .unwrap_or(false);
                    (not_mentioned, lyrics_not_mentioned)
                }
            } else {
                (false, false)
            };

            track_full.push(not_mentioned);
            track_lyrics.push(lyrics_not_mentioned);
        }
    }

    NotMentionedMeasures { full, lyrics }
}
