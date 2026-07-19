use crate::ast::grouped::Score;
use crate::ast::parsed::KeyChange;

pub fn build_single_measure_score(score: &Score, measure_index: usize) -> Option<Score> {
    let clamped_index = measure_index.min(score.measures.len().saturating_sub(1));
    let target = score.measures.get(clamped_index)?;

    // Accumulate BPM and key from all measures before the target
    let mut accumulated_bpm: Option<u32> = None;
    let mut accumulated_key: Option<KeyChange> = None;
    for measure in score.measures.iter().take(measure_index) {
        if let Some(bpm) = measure.bpm {
            accumulated_bpm = Some(bpm);
        }
        if let Some(key) = &measure.key {
            accumulated_key = Some(key.clone());
        }
    }

    // Clone target and inject accumulated context for fields the target doesn't override
    let mut patched = target.clone();
    if patched.bpm.is_none() {
        patched.bpm = accumulated_bpm;
    }
    if patched.key.is_none() {
        patched.key = accumulated_key;
    }

    Some(Score {
        metadata: score.metadata.clone(),
        measures: vec![patched],
        document_diagnostics: vec![],
        sequence: None,
    })
}

pub fn build_measure_range_score(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Option<Score> {
    if score.measures.is_empty() {
        return None;
    }
    let last = score.measures.len() - 1;
    let (start_index, end_index) = if start_index > end_index {
        (end_index.min(last), start_index.min(last))
    } else {
        (start_index.min(last), end_index.min(last))
    };
    let mut accumulated_bpm: Option<u32> = None;
    let mut accumulated_key: Option<KeyChange> = None;
    for measure in score.measures.iter().take(start_index) {
        if let Some(bpm) = measure.bpm {
            accumulated_bpm = Some(bpm);
        }
        if let Some(key) = &measure.key {
            accumulated_key = Some(key.clone());
        }
    }
    let count = end_index - start_index + 1;
    let mut measures: Vec<_> = score
        .measures
        .iter()
        .skip(start_index)
        .take(count)
        .cloned()
        .collect();
    if let Some(first) = measures.first_mut() {
        if first.bpm.is_none() {
            first.bpm = accumulated_bpm;
        }
        if first.key.is_none() {
            first.key = accumulated_key;
        }
    }
    Some(Score {
        metadata: score.metadata.clone(),
        measures,
        document_diagnostics: vec![],
        sequence: None,
    })
}
