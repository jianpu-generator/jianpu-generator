use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;

/// Rebuilds `score.measures` to reflect a `# sequence` section's resolved
/// playback order, so downstream MIDI/WAV generation replays measures in
/// actual playback order instead of written order.
///
/// - No `# sequence` section present: returns the score unchanged.
/// - Otherwise: returns a score whose measures are the expanded playback
///   sequence, with each occurrence's `(-abbrev ...)` part omissions applied.
pub fn expand_navigation(score: &Score) -> Result<Score, IrrecoverableError> {
    expand_navigation_with_origins(score).map(|(score, _)| score)
}

/// Same as [`expand_navigation`], but also returns a same-length `Vec<usize>`
/// mapping each measure in the expanded score back to its index in the
/// original written `score.measures`.
pub fn expand_navigation_with_origins(
    score: &Score,
) -> Result<(Score, Vec<usize>), IrrecoverableError> {
    let (expanded, origins) = expand_navigation_with_note_positions(score)?;
    Ok((
        expanded,
        origins
            .into_iter()
            .map(|o| o.written_measure_index)
            .collect(),
    ))
}

/// One measure of the expanded (playback-order) score, tracing back to where
/// it came from in the written score.
pub struct ExpandedMeasureOrigin {
    /// Index into the original written `score.measures`.
    pub written_measure_index: usize,
    /// `part_written_indices[i]` is the index this expanded measure's
    /// `parts[i]` had in the written measure's `parts`, before any
    /// `(-abbrev ...)` omission dropped some entries and shifted the rest —
    /// so callers keying data by the written `(part_index, note_id)` (e.g.
    /// `ColumnElement::note_id`) can still look it up after expansion.
    pub part_written_indices: Vec<usize>,
}

/// Same as [`expand_navigation_with_origins`], but the origins additionally
/// carry, per expanded measure, the written index of each surviving part —
/// needed to key note-level playback timing (see
/// [`super::timing::note_timings_seconds`]) back to the written
/// `(source_part_index, note_id)` identity after `(-abbrev ...)` omissions
/// have shifted part positions.
pub fn expand_navigation_with_note_positions(
    score: &Score,
) -> Result<(Score, Vec<ExpandedMeasureOrigin>), IrrecoverableError> {
    if score.measures.is_empty() {
        return Ok((score.clone(), Vec::new()));
    }

    if let Some(spans) = &score.sequence {
        let idx: Vec<(usize, &[String])> = spans
            .iter()
            .flat_map(|span| (span.start..=span.end).map(move |i| (i, span.omit_parts.as_slice())))
            .collect();
        return Ok(build_expanded(score, &idx));
    }

    let origins = score
        .measures
        .iter()
        .enumerate()
        .map(|(i, measure)| ExpandedMeasureOrigin {
            written_measure_index: i,
            part_written_indices: (0..measure.parts.len()).collect(),
        })
        .collect();
    Ok((score.clone(), origins))
}

/// Clones `score.measures` at each `(index, omit_parts)` pair (playback
/// order) into a new `Score`, dropping any part whose abbreviation appears
/// in that occurrence's `omit_parts`, alongside origin info for each
/// resulting measure (its written index, and the written index of each
/// surviving part).
fn build_expanded(
    score: &Score,
    idx: &[(usize, &[String])],
) -> (Score, Vec<ExpandedMeasureOrigin>) {
    let mut origins = Vec::with_capacity(idx.len());
    let measures = idx
        .iter()
        .filter_map(|&(i, omit_parts)| {
            let mut measure = score.measures.get(i).cloned()?;
            let mut part_written_indices = Vec::with_capacity(measure.parts.len());
            let mut written_idx = 0usize;
            measure.parts.retain(|part| {
                let omitted = !omit_parts.is_empty()
                    && part.name().is_some_and(|name| omit_parts.contains(name));
                if !omitted {
                    part_written_indices.push(written_idx);
                }
                written_idx += 1;
                !omitted
            });
            origins.push(ExpandedMeasureOrigin {
                written_measure_index: i,
                part_written_indices,
            });
            Some(measure)
        })
        .collect();

    (
        Score {
            metadata: score.metadata.clone(),
            measures,
            document_diagnostics: score.document_diagnostics.clone(),
            sequence: None,
        },
        origins,
    )
}

mod range;
pub use range::{earliest_playback_position, expand_for_measure_range};

#[cfg(test)]
mod tests_sequence_omission;
