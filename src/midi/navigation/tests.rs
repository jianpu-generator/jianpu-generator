use super::{expand_navigation, expand_navigation_with_origins};
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
        segno: false,
        ds_al_coda: false,
        dc_al_fine: false,
        fine: false,
        ds_al_fine: false,
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

#[test]
fn no_markers_origins_are_identity() {
    let measures: Vec<_> = (0..4).map(bare_measure).collect();
    let score = score_with(measures);
    let (_, origins) = expand_navigation_with_origins(&score).unwrap();
    assert_eq!(origins, vec![0, 1, 2, 3]);
}

#[test]
fn valid_markers_origins_match_expected_sequence() {
    // 4 measures: tocoda on 1, coda on 2, dcalcoda on 3.
    let mut measures: Vec<_> = (0..4).map(bare_measure).collect();
    measures[1].to_coda = true;
    measures[2].coda = true;
    measures[3].dc_al_coda = true;
    let score = score_with(measures);
    let (_, origins) = expand_navigation_with_origins(&score).unwrap();
    assert_eq!(origins, vec![0, 1, 2, 3, 0, 1, 2, 3]);
}

#[test]
fn dead_zone_written_index_is_absent_from_origins() {
    // 5 measures: tocoda on 1, dcalcoda on 2 (before coda), a dead-zone
    // measure at 3 (between dcalcoda and coda, so reachable by neither the
    // first pass 0..=dc nor the second pass's coda..=last segment), coda on 4.
    let mut measures: Vec<_> = (0..5).map(bare_measure).collect();
    measures[1].to_coda = true;
    measures[2].dc_al_coda = true;
    measures[4].coda = true;
    let score = score_with(measures);
    let (_, origins) = expand_navigation_with_origins(&score).unwrap();
    assert!(!origins.contains(&3));
}

#[test]
fn segno_markers_expand_to_expected_sequence() {
    // 5 measures: segno on 1, tocoda on 2, coda on 3, dsalcoda on 4.
    let mut measures: Vec<_> = (0..5).map(bare_measure).collect();
    measures[1].segno = true;
    measures[2].to_coda = true;
    measures[3].coda = true;
    measures[4].ds_al_coda = true;
    let score = score_with(measures);
    let expanded = expand_navigation(&score).unwrap();
    let sequence: Vec<usize> = expanded
        .measures
        .iter()
        .map(|m| m.source_span.start)
        .collect();
    // Pass 1: 0..=4 (through dsalcoda). Pass 2: restart from segno (1)
    // through tocoda (2), then jump to coda (3) through the end (4).
    assert_eq!(sequence, vec![0, 1, 2, 3, 4, 1, 2, 3, 4]);
}

#[test]
fn segno_partial_set_is_error() {
    let mut measures: Vec<_> = (0..3).map(bare_measure).collect();
    measures[0].segno = true;
    // no dsalcoda, no tocoda, no coda
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn dcalcoda_and_segno_together_is_error() {
    let mut measures: Vec<_> = (0..4).map(bare_measure).collect();
    measures[0].dc_al_coda = true;
    measures[0].segno = true;
    measures[1].to_coda = true;
    measures[2].coda = true;
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn segno_after_dsalcoda_is_error() {
    // segno (1) occurs after dsalcoda (0), which is invalid: dsalcoda must
    // jump back to a measure at or before itself.
    let mut measures: Vec<_> = (0..4).map(bare_measure).collect();
    measures[0].ds_al_coda = true;
    measures[1].segno = true;
    measures[2].to_coda = true;
    measures[3].coda = true;
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn dcalfine_markers_expand_to_expected_sequence() {
    // 4 measures: fine on 1, dcalfine on 3.
    let mut measures: Vec<_> = (0..4).map(bare_measure).collect();
    measures[1].fine = true;
    measures[3].dc_al_fine = true;
    let score = score_with(measures);
    let expanded = expand_navigation(&score).unwrap();
    let sequence: Vec<usize> = expanded
        .measures
        .iter()
        .map(|m| m.source_span.start)
        .collect();
    // Pass 1: 0..=3 (through dcalfine). Pass 2: restart from the start
    // through fine (1), then stop.
    assert_eq!(sequence, vec![0, 1, 2, 3, 0, 1]);
}

#[test]
fn dsalfine_markers_expand_to_expected_sequence() {
    // 5 measures: segno on 1, fine on 3, dsalfine on 4.
    let mut measures: Vec<_> = (0..5).map(bare_measure).collect();
    measures[1].segno = true;
    measures[3].fine = true;
    measures[4].ds_al_fine = true;
    let score = score_with(measures);
    let expanded = expand_navigation(&score).unwrap();
    let sequence: Vec<usize> = expanded
        .measures
        .iter()
        .map(|m| m.source_span.start)
        .collect();
    // Pass 1: 0..=4 (through dsalfine). Pass 2: restart from segno (1)
    // through fine (3), then stop.
    assert_eq!(sequence, vec![0, 1, 2, 3, 4, 1, 2, 3]);
}

#[test]
fn dcalfine_partial_set_is_error() {
    let mut measures: Vec<_> = (0..3).map(bare_measure).collect();
    measures[2].dc_al_fine = true;
    // no fine
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn dsalfine_partial_set_is_error() {
    let mut measures: Vec<_> = (0..3).map(bare_measure).collect();
    measures[0].segno = true;
    measures[2].ds_al_fine = true;
    // no fine
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn fine_before_segno_is_error() {
    // fine (0) occurs before segno (1), so pass 2 (restarting at segno)
    // could never reach it.
    let mut measures: Vec<_> = (0..3).map(bare_measure).collect();
    measures[0].fine = true;
    measures[1].segno = true;
    measures[2].ds_al_fine = true;
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn tocoda_before_segno_is_error() {
    // tocoda (1) occurs before segno (3), so pass 2 (restarting at segno
    // through tocoda) would form an empty/backwards range instead of being
    // rejected.
    let mut measures: Vec<_> = (0..7).map(bare_measure).collect();
    measures[1].to_coda = true;
    measures[3].segno = true;
    measures[5].ds_al_coda = true;
    measures[6].coda = true;
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn dcalfine_and_dcalcoda_together_is_error() {
    let mut measures: Vec<_> = (0..4).map(bare_measure).collect();
    measures[0].dc_al_fine = true;
    measures[1].fine = true;
    measures[2].to_coda = true;
    measures[3].coda = true;
    measures[3].dc_al_coda = true;
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}

#[test]
fn fine_combined_with_tocoda_coda_is_error() {
    // dcalfine present alongside a stray tocoda/coda pair (belonging to no
    // scheme) must be rejected rather than silently ignored.
    let mut measures: Vec<_> = (0..4).map(bare_measure).collect();
    measures[0].to_coda = true;
    measures[1].coda = true;
    measures[2].fine = true;
    measures[3].dc_al_fine = true;
    let score = score_with(measures);
    assert!(expand_navigation(&score).is_err());
}
