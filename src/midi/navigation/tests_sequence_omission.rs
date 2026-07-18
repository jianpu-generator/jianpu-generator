use super::expand_navigation_with_origins;
use crate::ast::grouped::{
    Metadata, MultiPartMeasure, Notes, PartRow, PartSlice, Score, SequenceSpan,
};
use crate::ast::parsed::{Offset, PartKind, Soundfont};
use crate::error::Span;

fn metadata() -> Metadata {
    Metadata {
        title: None,
        subtitle: None,
        author: None,
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        part_label_width_pt: 40,
        parts_list_columns: 3,
        lyrics_font_size: 14,
        merge_duplicate_measures_across_parts: true,
        hide_resting_parts: true,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
    }
}

fn bare_measure(index: usize) -> MultiPartMeasure {
    MultiPartMeasure {
        time_signature: None,
        bpm: None,
        key: None,
        label: None,
        merge_duplicate_measures_across_parts: true,
        hide_resting_parts: true,
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

fn score_with_sequence(measures: Vec<MultiPartMeasure>, sequence: Vec<SequenceSpan>) -> Score {
    Score {
        metadata: metadata(),
        measures,
        document_diagnostics: vec![],
        sequence: Some(sequence),
    }
}

fn measure_with_parts(index: usize, part_names: &[&str]) -> MultiPartMeasure {
    MultiPartMeasure {
        parts: part_names
            .iter()
            .map(|name| {
                PartRow::Timed(PartSlice {
                    name: Some(name.to_string()),
                    kind: PartKind::Notes,
                    soundfont: Soundfont::default(),
                    volume: 100,
                    octave_offset: 0,
                    notes: Notes { events: vec![] },
                    lyrics: vec![],
                    has_error: false,
                    group_provenance: None,
                })
            })
            .collect(),
        ..bare_measure(index)
    }
}

#[test]
fn sequence_omit_parts_drops_named_parts_per_occurrence() {
    // A single measure "Chorus" with three parts (S, A2, T). Replaying it
    // twice with different `(-abbrev ...)` omissions should drop only the
    // named parts on each occurrence, leaving the written measure untouched.
    let measures = vec![measure_with_parts(0, &["S", "A2", "T"])];
    let score = score_with_sequence(
        measures,
        vec![
            SequenceSpan {
                label: "Chorus".to_string(),
                start: 0,
                end: 0,
                omit_parts: vec!["S".to_string(), "A2".to_string()],
                omit_parts_display: Vec::new(),
            },
            SequenceSpan {
                label: "Chorus".to_string(),
                start: 0,
                end: 0,
                omit_parts: vec!["A2".to_string()],
                omit_parts_display: Vec::new(),
            },
        ],
    );
    let (expanded, _) = expand_navigation_with_origins(&score).unwrap();
    assert_eq!(expanded.measures.len(), 2);
    fn names(measure: &MultiPartMeasure) -> Vec<&str> {
        measure
            .parts
            .iter()
            .filter_map(|p| p.name().map(String::as_str))
            .collect()
    }
    assert_eq!(names(&expanded.measures[0]), vec!["T"]);
    assert_eq!(names(&expanded.measures[1]), vec!["S", "T"]);
    // The original written score is untouched.
    assert_eq!(score.measures[0].parts.len(), 3);
}
