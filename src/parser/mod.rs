use crate::ast::parsed::ParsedDocument;
use crate::error::{DocumentSection, IrrecoverableError, RecoverableError, Span};

pub mod lyrics;
pub mod metadata_parser;
pub mod parts_parser;
pub mod score;
pub mod section_splitter;

pub(crate) struct DocumentSectionContents {
    pub parts: (String, usize),
    pub score: (String, usize),
    pub metadata: (String, usize),
}

fn unwrap_or_missing(
    raw: Option<(String, usize)>,
    section: DocumentSection,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> (String, usize) {
    raw.unwrap_or_else(|| {
        errors.push(RecoverableError::section_missing(span, section));
        (String::new(), 0)
    })
}

pub(crate) fn load_document_sections(
    input: &str,
) -> (DocumentSectionContents, Vec<RecoverableError>) {
    use section_splitter::{split_sections, SectionKind};

    let (sections, mut errors) = split_sections(input);
    let doc_span = Span::new(0, input.len());

    let mut raw_metadata: Option<(String, usize)> = None;
    let mut raw_parts: Option<(String, usize)> = None;
    let mut raw_score: Option<(String, usize)> = None;

    // Track the order in which each section first appears to detect out-of-order.
    let mut metadata_order: Option<usize> = None;
    let mut parts_order: Option<usize> = None;
    let mut score_order: Option<usize> = None;

    for (index, section) in sections.into_iter().enumerate() {
        match section.kind {
            SectionKind::Metadata => {
                if raw_metadata.is_some() {
                    errors.push(RecoverableError::section_duplicate(
                        doc_span,
                        DocumentSection::Metadata,
                    ));
                } else {
                    metadata_order = Some(index);
                    raw_metadata = Some((section.content, section.content_offset));
                }
            }
            SectionKind::Parts => {
                if raw_parts.is_some() {
                    errors.push(RecoverableError::section_duplicate(
                        doc_span,
                        DocumentSection::Parts,
                    ));
                } else {
                    parts_order = Some(index);
                    raw_parts = Some((section.content, section.content_offset));
                }
            }
            SectionKind::Score => {
                if raw_score.is_some() {
                    errors.push(RecoverableError::section_duplicate(
                        doc_span,
                        DocumentSection::Score,
                    ));
                } else {
                    score_order = Some(index);
                    raw_score = Some((section.content, section.content_offset));
                }
            }
        }
    }

    // Detect out-of-order: any two present sections whose first-appearance indices
    // are not strictly ascending in canonical order (metadata < parts < score).
    let pairs = [
        (metadata_order, parts_order),
        (metadata_order, score_order),
        (parts_order, score_order),
    ];
    if pairs
        .iter()
        .any(|(earlier, later)| matches!((earlier, later), (Some(a), Some(b)) if a > b))
    {
        errors.push(RecoverableError::section_out_of_order(doc_span));
    }

    let metadata = raw_metadata.unwrap_or_else(|| (String::new(), 0));
    let parts = unwrap_or_missing(raw_parts, DocumentSection::Parts, doc_span, &mut errors);
    let score = unwrap_or_missing(raw_score, DocumentSection::Score, doc_span, &mut errors);
    (
        DocumentSectionContents {
            metadata,
            parts,
            score,
        },
        errors,
    )
}

pub fn parse(input: &str, filename: &str) -> Result<ParsedDocument, IrrecoverableError> {
    let path = std::path::Path::new(filename);
    let (sections, section_structure_errors) = load_document_sections(input);
    let (meta_content, meta_offset) = sections.metadata;
    let (parts_content, parts_offset) = sections.parts;
    let (score_content, score_offset) = sections.score;

    let (metadata, metadata_parse_errors) =
        metadata_parser::parse_metadata(&meta_content, meta_offset);
    let (declarations, parts_parse_errors) =
        parts_parser::parse_parts(&parts_content, parts_offset);
    let (tracks, directive_events_per_measure, per_measure_parse_errors) =
        if declarations.is_empty() {
            (Vec::new(), Vec::new(), Vec::new())
        } else {
            score::interleaved_parser::parse(&score_content, score_offset, &declarations)
                .map_err(|error| error.with_path(path))?
        };

    Ok(ParsedDocument {
        metadata,
        declarations,
        tracks,
        directive_events_per_measure,
        per_measure_parse_errors,
        metadata_parse_errors,
        parts_parse_errors,
        section_structure_errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::{ParsedTimedTrack, ParsedTrack};

    fn notes_track(doc: &ParsedDocument) -> &ParsedTimedTrack {
        doc.tracks
            .iter()
            .find_map(|t| match t {
                ParsedTrack::Timed(n) if n.lyrics.is_none() && n.abbreviation != "Chord" => Some(n),
                ParsedTrack::Timed(_) => None,
            })
            .or_else(|| {
                doc.tracks
                    .iter()
                    .map(|t| match t {
                        ParsedTrack::Timed(n) => n,
                    })
                    .next()
            })
            .expect("expected a notes track")
    }

    #[test]
    fn parses_full_document() {
        let input = concat!(
            "[metadata]\ntitle = \"hello world\"\nauthor = \"foo\"\n\n",
            "[parts]\nMelody = notes lyrics\n\n",
            "[score]\ntime=4/4 key=C4 bpm=120\n1 2 3 4\n你好wo rld\n"
        );
        let doc = parse(input, "test.jianpu").unwrap();
        assert_eq!(doc.metadata.title, "hello world");
        assert_eq!(doc.metadata.author, "foo");
        assert_eq!(doc.declarations.len(), 1);
        assert_eq!(doc.tracks.len(), 1);
        let notes = notes_track(&doc);
        assert_eq!(notes.score.events.len(), 7);
        assert_eq!(notes.lyrics.as_ref().unwrap().measure_syllables[0].len(), 4);
    }

    #[test]
    fn unknown_section_recoverable() {
        let input = "[unknown]\nfoo\n";
        let doc = parse(input, "test.jianpu").expect("unknown section must not abort parsing");
        assert!(doc
            .section_structure_errors
            .iter()
            .any(|e| matches!(&e.kind, crate::error::RecoverableErrorKind::SectionUnknown { name } if name == "unknown")));
    }

    #[test]
    fn duplicate_score_section_recoverable() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
            "[parts]\nMelody = notes\n\n",
            "[score]\ntime=4/4 key=C4 bpm=120\n1 2 3 4\n\n",
            "[score]\n5 6 7 1\n",
        );
        let doc =
            parse(input, "test.jianpu").expect("duplicate score section must not abort parsing");
        assert!(doc
            .section_structure_errors
            .iter()
            .any(|e| matches!(&e.kind, crate::error::RecoverableErrorKind::SectionDuplicate { section } if *section == DocumentSection::Score)));
    }

    #[test]
    fn missing_metadata_section_is_not_an_error() {
        let input = concat!(
            "[parts]\nMelody = notes\n\n",
            "[score]\ntime=4/4 key=C4 bpm=120\n1 2 3 4\n"
        );
        let doc =
            parse(input, "test.jianpu").expect("missing metadata section must not abort parsing");
        let has_metadata_missing = doc.section_structure_errors.iter().any(|e| {
            matches!(
                &e.kind,
                crate::error::RecoverableErrorKind::SectionMissing {
                    section
                } if *section == DocumentSection::Metadata
            )
        });
        debug_assert!(!has_metadata_missing);
    }

    #[test]
    fn parses_two_named_parts() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
            "[parts]\nSoprano = notes\nAlto = notes\n\n",
            "[score]\n",
            "time=4/4 key=C4 bpm=120\n",
            "1 2 3 4\n",
            "5 6 7 1\n",
        );
        let doc = parse(input, "test.jianpu").unwrap();
        assert_eq!(doc.tracks.len(), 2);
        let soprano = doc
            .tracks
            .iter()
            .find_map(|t| match t {
                ParsedTrack::Timed(n) if n.abbreviation == "Soprano" => Some(n),
                ParsedTrack::Timed(_) => None,
            })
            .unwrap();
        let alto = doc
            .tracks
            .iter()
            .find_map(|t| match t {
                ParsedTrack::Timed(n) if n.abbreviation == "Alto" => Some(n),
                ParsedTrack::Timed(_) => None,
            })
            .unwrap();
        assert!(soprano.lyrics.is_none());
        assert!(alto.lyrics.is_none());
    }

    #[test]
    fn too_many_lines_recoverable_error_span_points_to_absolute_file_position() {
        // One notes part but two data lines in a group → recoverable error.
        // The error span must point to the extra line's position in the *full* input.
        let input = concat!(
            "[metadata]\n",
            "title=\"t\"\n",
            "author=\"a\"\n",
            "\n",
            "[parts]\n",
            "Melody = notes\n",
            "\n",
            "[score]\n",
            "1 2 3 4\n",
            "5 6 7 1\n",
        );
        let expected_offset = input.find("5 6 7 1").unwrap();
        let doc = parse(input, "test.jianpu").expect("too-many-lines must not abort parsing");
        let error = doc.per_measure_parse_errors[0]
            .as_ref()
            .expect("recoverable error must be recorded for the measure");
        assert_eq!(
            error.span.start, expected_offset,
            "recoverable error span should point to the absolute file position of the extra line"
        );
    }

    #[test]
    fn too_many_lines_recoverable_error_lists_declared_parts() {
        // One notes part but two data lines → recoverable error should name the declared part.
        let input = concat!(
            "[metadata]\n",
            "title=\"t\"\n",
            "author=\"a\"\n",
            "\n",
            "[parts]\n",
            "Melody = notes\n",
            "\n",
            "[score]\n",
            "1 2 3 4\n",
            "5 6 7 1\n",
        );
        let doc = parse(input, "test.jianpu").expect("too-many-lines must not abort parsing");
        let error = doc.per_measure_parse_errors[0]
            .as_ref()
            .expect("recoverable error must be recorded for the measure");
        assert!(
            error.message().contains("Melody"),
            "recoverable error message should list the declared part 'Melody', got: {}",
            error.message()
        );
    }

    #[test]
    fn single_unnamed_part_remains_compatible() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
            "[parts]\nMelody = notes lyrics\n\n",
            "[score]\ntime=4/4 key=C4 bpm=120\n1 2 3 4\na b c d\n"
        );
        let doc = parse(input, "test.jianpu").unwrap();
        assert_eq!(doc.tracks.len(), 1);
        let notes = notes_track(&doc);
        assert_eq!(notes.abbreviation, "Melody");
        assert!(notes.lyrics.is_some());
    }
}
