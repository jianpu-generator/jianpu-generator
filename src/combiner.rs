use crate::ast::grouped::{
    GroupedMeasure, GroupedScore, GroupedTrack, Lyrics, MeasureDirectives, MultiPartMeasure, Notes,
    PartRow, PartSlice,
};
use crate::ast::parsed::PartKind;
use crate::error::{Diagnostic, RecoverableError, Span};

fn collect_part_measure_diagnostics(m: Option<&GroupedMeasure>) -> Vec<Diagnostic> {
    [
        m.and_then(|m| m.lyrics_error.clone())
            .map(Diagnostic::Warning),
        m.and_then(|m| m.beat_overflow_error.clone())
            .map(Diagnostic::Warning),
        m.and_then(|m| m.dash_after_rest_error.clone())
            .map(Diagnostic::Error),
        m.and_then(|m| m.lex_error.clone()).map(Diagnostic::Error),
        m.and_then(|m| m.lyrics_parse_error.clone())
            .map(Diagnostic::Error),
        m.and_then(|m| m.extension_no_preceding_event_error.clone())
            .map(Diagnostic::Error),
    ]
    .into_iter()
    .flatten()
    .chain(
        m.into_iter()
            .flat_map(|m| m.dotted_eighth_errors.iter().cloned()),
    )
    .chain(m.into_iter().flat_map(|m| m.chord_errors.iter().cloned()))
    .collect()
}

fn measure_has_error(m: &GroupedMeasure) -> bool {
    m.dash_after_rest_error.is_some()
        || m.lex_error.is_some()
        || m.lyrics_parse_error.is_some()
        || m.extension_no_preceding_event_error.is_some()
        || m.dotted_eighth_errors
            .iter()
            .any(|d| matches!(d, Diagnostic::Error(_)))
        || m.chord_errors
            .iter()
            .any(|d| matches!(d, Diagnostic::Error(_)))
}

fn combine_measure(
    grouped_score: &GroupedScore,
    measure_idx: usize,
    directives_fallback: &MeasureDirectives,
) -> MultiPartMeasure {
    let (directives, directives_error) = match grouped_score.measure_directives.get(measure_idx) {
        Some(d) => (d, None),
        None => (
            directives_fallback,
            Some(Diagnostic::Error(
                RecoverableError::measure_directives_missing(Span::new(0, 0)),
            )),
        ),
    };
    let (part_rows, part_row_diagnostics) = build_part_rows(&grouped_score.parts, measure_idx);
    let (source_span, source_span_error) = grouped_score
        .parts
        .iter()
        .filter_map(|track| match track {
            GroupedTrack::Timed(part) => part.measures.get(measure_idx),
        })
        .fold(None, |acc: Option<Span>, m| {
            Some(match acc {
                None => m.source_span,
                Some(prev) => Span::new(
                    prev.start.min(m.source_span.start),
                    prev.end.max(m.source_span.end),
                ),
            })
        })
        .map(|span| (span, None))
        .unwrap_or_else(|| {
            (
                Span::new(0, 0),
                Some(Diagnostic::Error(RecoverableError::source_span_missing(
                    Span::new(0, 0),
                    measure_idx,
                ))),
            )
        });
    let parse_error = grouped_score
        .per_measure_parse_errors
        .get(measure_idx)
        .and_then(|e| e.clone())
        .map(Diagnostic::Error);
    let measure_diagnostics: Vec<Diagnostic> = directives_error
        .into_iter()
        .chain(source_span_error)
        .chain(parse_error)
        .chain(grouped_score.parts.iter().flat_map(|track| match track {
            GroupedTrack::Timed(part) => {
                collect_part_measure_diagnostics(part.measures.get(measure_idx))
            }
        }))
        .chain(part_row_diagnostics)
        .collect();
    MultiPartMeasure {
        time_signature: directives.time_signature.clone(),
        bpm: directives.bpm,
        key: directives.key.clone(),
        label: directives.label.clone(),
        parts: part_rows,
        source_span,
        diagnostics: measure_diagnostics,
    }
}

pub(crate) fn combine(grouped_score: &GroupedScore) -> (Vec<MultiPartMeasure>, Vec<Diagnostic>) {
    if grouped_score.parts.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let expected_len = grouped_score
        .parts
        .first()
        .map(GroupedTrack::measure_count)
        .unwrap_or(0);
    let max_len = grouped_score
        .parts
        .iter()
        .map(GroupedTrack::measure_count)
        .max()
        .unwrap_or(0);

    let mismatch_diagnostics: Vec<Diagnostic> = grouped_score
        .parts
        .iter()
        .skip(1)
        .filter(|track| track.measure_count() != expected_len)
        .map(|track| {
            Diagnostic::Error(RecoverableError::part_measure_count_mismatch(
                Span::new(0, 0),
                format!("{:?}", track.track_name()),
                track.measure_count(),
                expected_len,
            ))
        })
        .collect();

    let directives_fallback = MeasureDirectives {
        time_signature: None,
        bpm: None,
        key: None,
        label: None,
    };
    let combined = (0..max_len)
        .map(|measure_idx| combine_measure(grouped_score, measure_idx, &directives_fallback))
        .collect();

    (combined, mismatch_diagnostics)
}

fn build_part_rows(
    grouped_tracks: &[GroupedTrack],
    measure_idx: usize,
) -> (Vec<PartRow>, Vec<Diagnostic>) {
    let mut part_rows = Vec::new();
    let mut diagnostics = Vec::new();

    for track in grouped_tracks.iter() {
        match track {
            GroupedTrack::Timed(part) => {
                let Some(measure) = part.measures.get(measure_idx) else {
                    diagnostics.push(Diagnostic::Error(
                        RecoverableError::timed_part_measure_missing(Span::new(0, 0)),
                    ));
                    continue;
                };
                let lyrics = match part.kind {
                    PartKind::NotesWithLyrics | PartKind::LyricsWithNotes => measure
                        .paired_lyrics
                        .clone()
                        .map(|syllables| Lyrics { syllables }),
                    PartKind::Chord | PartKind::Notes | PartKind::NotesWithChord => None,
                };
                let mut slice = PartSlice {
                    name: part.name.clone(),
                    kind: part.kind,
                    notes: Notes {
                        events: measure.notes.events.clone(),
                    },
                    lyrics,
                    has_error: measure_has_error(measure),
                };
                let is_ditto = part
                    .ditto_measures
                    .get(measure_idx)
                    .copied()
                    .unwrap_or(false);
                let lyrics_ditto = part
                    .lyrics_ditto_measures
                    .get(measure_idx)
                    .copied()
                    .unwrap_or(false);
                // A ditto'd lyric line duplicates the part above's lyrics, so
                // render this measure as a plain notes part: the copied
                // syllables are not shown and the lyric row is reclaimed.
                if lyrics_ditto
                    && !is_ditto
                    && matches!(
                        slice.kind,
                        PartKind::NotesWithLyrics | PartKind::LyricsWithNotes
                    )
                {
                    slice.kind = PartKind::Notes;
                    slice.lyrics = None;
                }
                part_rows.push(if is_ditto {
                    PartRow::Ditto(slice)
                } else {
                    PartRow::Timed(slice)
                });
            }
        }
    }

    (part_rows, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grouper, parser};

    fn make_two_part_score(soprano: &str, alto: &str) -> Vec<MultiPartMeasure> {
        let input = format!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nSoprano = notes\nAlto = notes\n\n[score]\ntime=4/4 key=C4 bpm=120\n{soprano}\n{alto}\n"
        );
        let doc = parser::parse(&input, "test.jianpu").unwrap();
        grouper::group(doc).unwrap().measures
    }

    #[test]
    fn combines_two_parts_into_measures() {
        let measures = make_two_part_score("1 2 3 4", "5 6 7 1");
        assert_eq!(measures.len(), 1);
        assert_eq!(measures[0].parts.len(), 2);
    }

    #[test]
    fn directives_come_from_first_part() {
        let measures = make_two_part_score("1 2 3 4", "5 6 7 1");
        assert_eq!(measures[0].bpm, Some(120));
        assert!(measures[0].time_signature.is_some());
    }

    #[test]
    fn beat_overflow_in_one_part_attaches_error_to_measure() {
        // Alto has 5 notes in a 4/4 bar — overflow is recoverable; the measure
        // gets an error and the 5th note is trimmed.
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nSoprano = notes\nAlto = notes\n\n",
            "[score]\n",
            "time=4/4 key=C4 bpm=120\n",
            "1 2 3 4\n",
            "5 6 7 1 5\n",
        );
        let doc =
            parser::parse(input, "test.jianpu").expect("beat overflow must not abort parsing");
        let score = grouper::group(doc).expect("grouping must succeed");
        assert_eq!(score.measures.len(), 1);
        assert_eq!(score.measures[0].diagnostics.len(), 1);
        assert!(
            score.measures[0].diagnostics[0]
                .message()
                .contains("beat overflow"),
            "got: {}",
            score.measures[0].diagnostics[0].message()
        );
    }

    #[test]
    fn missing_lyrics_line_in_first_measure_is_silently_filled() {
        // Omitted trailing lyrics with no ditto source: silently treated as no lyrics.
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
            "[parts]\nA = notes lyrics\n\n",
            "[score]\n",
            "1 2 3 4\n",
            "\n",
            "5 6 7 1\n",
            "la lo le li\n",
        );
        let doc =
            parser::parse(input, "test.jianpu").expect("missing lyrics must not abort parsing");
        let score = grouper::group(doc).expect("grouping must succeed");
        assert_eq!(score.measures.len(), 2);
        assert!(
            score.measures[0].diagnostics.is_empty(),
            "measure 1 should have no diagnostics when lyrics are silently omitted"
        );
        assert!(
            score.measures[1].diagnostics.is_empty(),
            "measure 2 should have no errors"
        );
    }

    #[test]
    fn measure_source_span_is_nonzero_after_combine() {
        let measures = make_two_part_score("1 2 3 4", "5 6 7 1");
        assert_eq!(measures.len(), 1);
        // span should not be the dummy (0, 0)
        assert!(
            measures[0].source_span.end > 0,
            "source_span.end should be > 0, got {:?}",
            measures[0].source_span
        );
    }
}
