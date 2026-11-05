use super::{convert, lower};
use crate::error::Span;

const DOCUMENT: &str = r#"metadata { title : "Hello", max measures per system : 4 },

parts [
  "Soprano" [S] : notes { volume : 80% },
  "Alto" [A] : notes,
],

score """
  bpm=92 label="Verse"
  [S] 1 2 3 4
  [A] 5 6 7 1'

  [S] 2 2 2 2
  [A] 3 3 3 3
  """,
"#;

fn offset_of(text: &str, needle: &str) -> usize {
    text.find(needle).unwrap()
}

#[test]
fn measure_lines_map_exactly_in_both_directions() {
    let lowered = lower(DOCUMENT, &[]);
    assert_eq!(lowered.errors, []);
    for needle in [
        "bpm=92 label=\"Verse\"",
        "[S] 1 2 3 4",
        "[A] 5 6 7 1'",
        "[A] 3 3 3 3",
    ] {
        let in_lowered = offset_of(&lowered.source, needle);
        let in_anyfix = offset_of(DOCUMENT, needle);
        assert_eq!(lowered.source_map.to_anyfix(in_lowered), in_anyfix);
        assert_eq!(lowered.source_map.to_lowered(in_anyfix), in_lowered);
        let span = lowered
            .source_map
            .span_to_anyfix(Span::new(in_lowered, in_lowered + needle.len()));
        assert_eq!(DOCUMENT.get(span.start..span.end), Some(needle));
    }
}

#[test]
fn the_end_of_a_measure_line_stays_on_that_line() {
    let lowered = lower(DOCUMENT, &[]);
    let needle = "[S] 1 2 3 4";
    let in_lowered = offset_of(&lowered.source, needle) + needle.len();
    let in_anyfix = offset_of(DOCUMENT, needle) + needle.len();
    assert_eq!(lowered.source_map.to_anyfix(in_lowered), in_anyfix);
}

#[test]
fn words_of_rewritten_lines_map_exactly() {
    let lowered = lower(DOCUMENT, &[]);
    for needle in ["Soprano", "Hello", "80"] {
        let in_lowered = offset_of(&lowered.source, needle);
        let span = lowered
            .source_map
            .span_to_anyfix(Span::new(in_lowered, in_lowered + needle.len()));
        assert_eq!(DOCUMENT.get(span.start..span.end), Some(needle));
    }
    let abbreviation = offset_of(&lowered.source, "[A]") + 1;
    let span = lowered
        .source_map
        .span_to_anyfix(Span::new(abbreviation, abbreviation + 1));
    assert_eq!(span.start, offset_of(DOCUMENT, "[A] :") + 1);
}

#[test]
fn a_rewritten_key_maps_to_the_field_that_wrote_it() {
    let lowered = lower(DOCUMENT, &[]);
    let start = offset_of(&lowered.source, "max_measures_per_system");
    let span = lowered
        .source_map
        .span_to_anyfix(Span::new(start, start + "max_measures_per_system".len()));
    let text = DOCUMENT.get(span.start..span.end).unwrap();
    assert!(text.contains("max measures per system"), "{text:?}");
}

#[test]
fn indentation_of_a_raw_string_maps_to_its_line() {
    let lowered = lower(DOCUMENT, &[]);
    let indentation = offset_of(DOCUMENT, "  [S] 1 2 3 4");
    assert_eq!(
        lowered.source_map.to_lowered(indentation),
        offset_of(&lowered.source, "[S] 1 2 3 4")
    );
}

#[test]
fn segments_keep_document_order_on_both_sides() {
    let lowered = lower(DOCUMENT, &[]);
    for pair in lowered.source_map.segments.windows(2) {
        let (Some(first), Some(second)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        assert!(first.lowered.end <= second.lowered.start, "{pair:?}");
        assert!(first.anyfix.end <= second.anyfix.start, "{pair:?}");
    }
}

#[test]
fn converted_demos_map_every_score_line() {
    let demo = include_str!("../../../fixtures/section-syntax-demos/10-multi-part.poentaro");
    let converted = convert(demo);
    let lowered = lower(&converted, &[]);
    assert_eq!(lowered.errors, []);
    let score = lowered.source.find("# score\n").unwrap() + "# score\n".len();
    let mut offset = score;
    for line in lowered.source.get(score..).unwrap().split_inclusive('\n') {
        let content = line.trim_end();
        if !content.is_empty() {
            let span = lowered
                .source_map
                .span_to_anyfix(Span::new(offset, offset + content.len()));
            assert_eq!(converted.get(span.start..span.end), Some(content));
        }
        offset += line.len();
    }
}
