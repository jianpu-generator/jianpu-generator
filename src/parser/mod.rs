use crate::ast::parsed::{ParsedDocument, ParsedMetadata, PartDecl};
use crate::error::{DocumentSection, IrrecoverableError, RecoverableError, Span};

pub mod anyfix;
pub mod lyrics;
pub mod metadata_parser;
pub mod parts_parser;
pub mod preset_parser;
pub mod score;
pub mod section_splitter;
pub mod sequence_parser;

pub(crate) struct DocumentSectionContents {
    pub parts: (String, usize),
    pub score: (String, usize),
    pub metadata: (String, usize),
    pub sequence: Option<(String, usize)>,
    pub presets: Vec<preset_parser::PresetSection>,
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
    presets: Vec<preset_parser::PresetSection>,
}

fn partition_sections(
    sections: Vec<section_splitter::RawSection>,
    doc_span: Span,
    errors: &mut Vec<RecoverableError>,
) -> RawSections {
    use section_splitter::SectionKind;

    let mut raw = RawSections::default();
    for section in sections.into_iter() {
        if let (SectionKind::Preset, Some(name)) = (section.kind, section.preset_name) {
            raw.presets.push(preset_parser::PresetSection {
                name,
                header_span: section.header_span,
                content: section.content,
                content_offset: section.content_offset,
            });
            continue;
        }
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
            // Taken above: a preset section always carries its name.
            SectionKind::Preset => {}
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
            presets: raw.presets,
        },
        errors,
    )
}

/// A document's score, as the text the score parser reads, and where that text
/// starts in the document. Spans in it are spans in the document.
pub(crate) struct ScoreText {
    pub text: String,
    pub offset: usize,
}

/// Everything in a document but the score's content, read whichever syntax it
/// is written in.
pub(crate) struct ReadDocument {
    pub metadata: ParsedMetadata,
    pub metadata_parse_errors: Vec<RecoverableError>,
    pub declarations: Vec<PartDecl>,
    pub parts_parse_errors: Vec<RecoverableError>,
    pub section_structure_errors: Vec<RecoverableError>,
    pub score: ScoreText,
    pub presets: preset_parser::ParsedPresets,
    pub sequence: Option<sequence_parser::SequenceSection>,
    pub sequence_parse_errors: Vec<RecoverableError>,
}

/// Reads `input`, in either syntax. `instruments` names the sound a part picks.
pub(crate) fn read_document(
    input: &str,
    instruments: &[parts_parser::InstrumentInfo],
) -> ReadDocument {
    if crate::document::is_anyfix(input) {
        anyfix::read(input, instruments)
    } else {
        read_section_document(input, instruments)
    }
}

fn read_section_document(
    input: &str,
    instruments: &[parts_parser::InstrumentInfo],
) -> ReadDocument {
    let (sections, section_structure_errors) = load_document_sections(input);
    let (meta_content, meta_offset) = sections.metadata;
    let (parts_content, parts_offset) = sections.parts;
    let (score_text, score_offset) = sections.score;
    let (metadata, metadata_parse_errors) =
        metadata_parser::parse_metadata(&meta_content, meta_offset);
    let (declarations, parts_parse_errors) =
        parts_parser::parse_parts(&parts_content, parts_offset, instruments);
    let presets = preset_parser::parse_presets(sections.presets, &declarations);
    let (sequence, sequence_parse_errors) = match sections.sequence {
        Some((sequence_content, sequence_offset)) => {
            sequence_parser::parse_sequence(&sequence_content, sequence_offset)
        }
        None => (None, Vec::new()),
    };
    ReadDocument {
        metadata,
        metadata_parse_errors,
        declarations,
        parts_parse_errors,
        section_structure_errors,
        score: ScoreText {
            text: score_text,
            offset: score_offset,
        },
        presets,
        sequence,
        sequence_parse_errors,
    }
}

pub fn parse(
    input: &str,
    filename: &str,
    instruments: &[parts_parser::InstrumentInfo],
) -> Result<ParsedDocument, IrrecoverableError> {
    let path = std::path::Path::new(filename);
    let read = read_document(input, instruments);

    let (tracks, directive_events_per_measure, per_measure_parse_errors, abbreviation_references) =
        if read.declarations.is_empty() {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        } else {
            score::interleaved_parser::parse(
                &read.score.text,
                read.score.offset,
                &read.declarations,
            )
            .map_err(|error| error.with_path(path))?
        };

    Ok(ParsedDocument {
        metadata: read.metadata,
        declarations: read.declarations,
        tracks,
        directive_events_per_measure,
        abbreviation_references,
        per_measure_parse_errors,
        metadata_parse_errors: read.metadata_parse_errors,
        parts_parse_errors: read.parts_parse_errors,
        section_structure_errors: read.section_structure_errors,
        sequence: read.sequence,
        sequence_parse_errors: read.sequence_parse_errors,
        presets: read.presets.presets,
        preset_diagnostics: read.presets.diagnostics,
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod relative_key_tests;
