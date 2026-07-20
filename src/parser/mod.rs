use crate::ast::parsed::ParsedDocument;
use crate::error::{DocumentSection, IrrecoverableError, RecoverableError, Span};

pub mod group_parser;
pub mod lyrics;
pub mod metadata_parser;
pub mod parts_parser;
pub mod score;
pub mod section_splitter;
pub mod sequence_parser;

pub(crate) struct DocumentSectionContents {
    pub parts: (String, usize),
    pub score: (String, usize),
    pub metadata: (String, usize),
    pub sequence: Option<(String, usize)>,
    pub group: Option<(String, usize)>,
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

#[derive(Default)]
struct RawSections {
    metadata: Option<(String, usize)>,
    parts: Option<(String, usize)>,
    score: Option<(String, usize)>,
    sequence: Option<(String, usize)>,
    group: Option<(String, usize)>,
}

fn partition_sections(
    sections: Vec<section_splitter::RawSection>,
    doc_span: Span,
    errors: &mut Vec<RecoverableError>,
) -> RawSections {
    use section_splitter::SectionKind;

    let mut raw = RawSections::default();
    for section in sections.into_iter() {
        let occurrence = SectionOccurrence {
            content: (section.content, section.content_offset),
            doc_span,
        };
        match section.kind {
            SectionKind::Metadata => record_section(
                &mut raw.metadata,
                DocumentSection::Metadata,
                occurrence,
                errors,
            ),
            SectionKind::Parts => {
                record_section(&mut raw.parts, DocumentSection::Parts, occurrence, errors)
            }
            SectionKind::Score => {
                record_section(&mut raw.score, DocumentSection::Score, occurrence, errors)
            }
            SectionKind::Sequence => record_section(
                &mut raw.sequence,
                DocumentSection::Sequence,
                occurrence,
                errors,
            ),
            SectionKind::Groups => {
                record_section(&mut raw.group, DocumentSection::Groups, occurrence, errors)
            }
        }
    }
    raw
}

struct SectionOccurrence {
    content: (String, usize),
    doc_span: Span,
}

fn record_section(
    slot: &mut Option<(String, usize)>,
    section: DocumentSection,
    occurrence: SectionOccurrence,
    errors: &mut Vec<RecoverableError>,
) {
    if slot.is_some() {
        errors.push(RecoverableError::section_duplicate(
            occurrence.doc_span,
            section,
        ));
    } else {
        *slot = Some(occurrence.content);
    }
}

pub(crate) fn load_document_sections(
    input: &str,
) -> (DocumentSectionContents, Vec<RecoverableError>) {
    use section_splitter::split_sections;

    let (sections, mut errors) = split_sections(input);
    let doc_span = Span::new(0, input.len());

    let raw = partition_sections(sections, doc_span, &mut errors);

    let metadata = raw.metadata.unwrap_or((String::new(), 0));
    let parts = unwrap_or_missing(raw.parts, DocumentSection::Parts, doc_span, &mut errors);
    let score = unwrap_or_missing(raw.score, DocumentSection::Score, doc_span, &mut errors);
    (
        DocumentSectionContents {
            metadata,
            parts,
            score,
            sequence: raw.sequence,
            group: raw.group,
        },
        errors,
    )
}

pub fn parse(
    input: &str,
    filename: &str,
    instruments: &[parts_parser::InstrumentInfo],
) -> Result<ParsedDocument, IrrecoverableError> {
    let path = std::path::Path::new(filename);
    let (sections, section_structure_errors) = load_document_sections(input);
    let (meta_content, meta_offset) = sections.metadata;
    let (parts_content, parts_offset) = sections.parts;
    let (score_content, score_offset) = sections.score;

    let (metadata, metadata_parse_errors) =
        metadata_parser::parse_metadata(&meta_content, meta_offset);
    let (declarations, parts_parse_errors) =
        parts_parser::parse_parts(&parts_content, parts_offset, instruments);

    let (group, mut group_parse_errors) = match sections.group {
        Some((group_content, group_offset)) => {
            group_parser::parse_group(&group_content, group_offset)
        }
        None => (None, Vec::new()),
    };
    let resolved_groups = match &group {
        Some(group_section) => {
            let (resolved, errors) =
                group_parser::resolve_and_validate_groups(group_section, &declarations);
            group_parse_errors.extend(errors);
            resolved
        }
        None => Vec::new(),
    };

    let (tracks, directive_events_per_measure, per_measure_parse_errors, abbreviation_references) =
        if declarations.is_empty() {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        } else {
            score::interleaved_parser::parse(
                &score_content,
                score_offset,
                &declarations,
                &resolved_groups,
            )
            .map_err(|error| error.with_path(path))?
        };
    let (sequence, sequence_parse_errors) = match sections.sequence {
        Some((sequence_content, sequence_offset)) => {
            sequence_parser::parse_sequence(&sequence_content, sequence_offset)
        }
        None => (None, Vec::new()),
    };

    Ok(ParsedDocument {
        metadata,
        declarations,
        tracks,
        directive_events_per_measure,
        abbreviation_references,
        per_measure_parse_errors,
        metadata_parse_errors,
        parts_parse_errors,
        section_structure_errors,
        sequence,
        sequence_parse_errors,
        group,
        group_parse_errors,
    })
}

#[cfg(test)]
mod tests;
