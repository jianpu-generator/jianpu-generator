use super::expand_navigation;
use crate::ast::grouped::{Metadata, MultiPartMeasure, Score};
use crate::error::Span;

fn metadata() -> Metadata {
    Metadata {
        title: None,
        subtitle: None,
        author: None,
        row_height: 24,
        max_columns: 28,
        label_width: 40,
        note_number_width: 8,
        parts_list_columns: 3,
    }
}

fn bare_measure(index: usize) -> MultiPartMeasure {
    MultiPartMeasure {
        time_signature: None,
        bpm: None,
        key: None,
        label: None,
        dc_al_coda: false,
        to_coda: false,
        coda: false,
        parts: vec![],
        source_span: Span::new(index, index + 1),
        diagnostics: vec![],
    }
}

fn score_with(measures: Vec<MultiPartMeasure>) -> Score {
    Score {
        metadata: metadata(),
        measures,
        document_diagnostics: vec![],
    }
}

#[test]
fn no_markers_is_identity() {
    let measures: Vec<_> = (0..4).map(bare_measure).collect();
    let score = score_with(measures);
    let expanded = expand_navigation(&score).unwrap();
    assert_eq!(expanded.measures.len(), 4);
    for (i, m) in expanded.measures.iter().enumerate() {
        assert_eq!(m.source_span, Span::new(i, i + 1));
    }
}

#[test]
fn valid_markers_expand_to_expected_sequence() {
    // 4 measures: tocoda on 1, coda on 2, dcalcoda on 3.
    let mut measures: Vec<_> = (0..4).map(bare_measure).collect();
    measures[1].to_coda = true;
    measures[2].coda = true;
    measures[3].dc_al_coda = true;
    let score = score_with(measures);
    let expanded = expand_navigation(&score).unwrap();
    let sequence: Vec<usize> = expanded
        .measures
        .iter()
        .map(|m| m.source_span.start)
        .collect();
    assert_eq!(sequence, vec![0, 1, 2, 3, 0, 1, 2, 3]);
}

#[test]
fn partial_set_is_error() {
    let mut measures: Vec<_> = (0..3).map(bare_measure).collect();
    measures[1].to_coda = true;
    // no coda, no dcalcoda
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn duplicate_marker_is_error() {
    let mut measures: Vec<_> = (0..4).map(bare_measure).collect();
    measures[0].dc_al_coda = true;
    measures[1].dc_al_coda = true;
    measures[2].to_coda = true;
    measures[3].coda = true;
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn tocoda_after_coda_is_error() {
    let mut measures: Vec<_> = (0..4).map(bare_measure).collect();
    measures[3].dc_al_coda = true;
    measures[2].to_coda = true;
    measures[1].coda = true;
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn empty_score_is_identity() {
    let score = score_with(vec![]);
    let expanded = expand_navigation(&score).unwrap();
    assert!(expanded.measures.is_empty());
}
