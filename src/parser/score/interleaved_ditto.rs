use crate::ast::parsed::{flatten_score_line_slots, PartDecl, ScoreLineRole};

/// Ditto flags per track, per measure group, computed from the raw groups
/// before desugaring erases the distinction. A line is a ditto when it is an
/// explicit `"` or an omitted trailing line (which desugaring pads as
/// implicit ditto).
pub(crate) struct DittoMeasures {
    /// `[track][measure]`: every score line of the track was a ditto.
    pub full: Vec<Vec<bool>>,
    /// `[track][measure]`: the track's lyric line was a ditto. Always false
    /// for tracks without a lyrics line.
    pub lyrics: Vec<Vec<bool>>,
}

pub(crate) fn compute_ditto_measures(
    groups: &[Vec<(String, usize)>],
    declarations: &[PartDecl],
) -> DittoMeasures {
    let slots = flatten_score_line_slots(declarations);
    let mut full = vec![Vec::with_capacity(groups.len()); declarations.len()];
    let mut lyrics = vec![Vec::with_capacity(groups.len()); declarations.len()];

    for group in groups {
        let directive_count = usize::from(
            group
                .first()
                .map(|(l, _)| l.starts_with('('))
                .unwrap_or(false),
        );
        let data_lines = group.get(directive_count..).unwrap_or(&[]);
        let line_is_ditto = |slot_idx: usize| {
            data_lines
                .get(slot_idx)
                .map(|(line, _)| line == "\"")
                .unwrap_or(true)
        };

        for (track_index, (track_full, track_lyrics)) in
            full.iter_mut().zip(lyrics.iter_mut()).enumerate()
        {
            let mut all_lines_ditto = true;
            let mut lyric_line_ditto = false;
            for (slot_idx, slot) in slots.iter().enumerate() {
                if slot.track_index != track_index {
                    continue;
                }
                let is_ditto = line_is_ditto(slot_idx);
                all_lines_ditto &= is_ditto;
                if matches!(slot.role, ScoreLineRole::Lyrics) {
                    lyric_line_ditto = is_ditto;
                }
            }
            track_full.push(all_lines_ditto);
            track_lyrics.push(lyric_line_ditto);
        }
    }

    DittoMeasures { full, lyrics }
}
