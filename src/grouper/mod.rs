use crate::ast::grouped::{GroupedScore, GroupedTrack, Metadata, Score};
use crate::ast::parsed::{ParsedDocument, ParsedTrack};
use crate::combiner;
use crate::error::{Diagnostic, IrrecoverableError};

#[path = "empty_note_measures.rs"]
mod empty_note_measures;

mod directive_grouper;
mod lyrics_pairing;
mod navigation_validation;
mod part_grouper;
mod sequence_resolution;
mod tie_validation;

use directive_grouper::DirectiveGrouper;
use navigation_validation::validate_navigation_markers;
use part_grouper::group_timed_track;
use sequence_resolution::resolve_sequence;
use tie_validation::validate_ties;

pub fn group(doc: ParsedDocument) -> Result<Score, IrrecoverableError> {
    let metadata = doc.metadata;
    let sequence = doc.sequence;
    let sequence_parse_errors = doc.sequence_parse_errors;
    let document_diagnostics: Vec<Diagnostic> = doc
        .section_structure_errors
        .into_iter()
        .chain(doc.metadata_parse_errors)
        .chain(doc.parts_parse_errors)
        .map(Diagnostic::Error)
        .collect();
    let mut grouped_tracks = Vec::new();
    for track in doc.tracks {
        grouped_tracks.push(match track {
            ParsedTrack::Timed(part) => GroupedTrack::Timed(group_timed_track(part)?),
        });
    }

    let measure_directives = DirectiveGrouper::new().process_all(&doc.directive_events_per_measure);

    let grouped_score = GroupedScore {
        measure_directives,
        parts: grouped_tracks,
        per_measure_parse_errors: doc.per_measure_parse_errors,
    };

    let (measures, combiner_diagnostics) = combiner::combine(&grouped_score);

    let mut score = Score {
        metadata: Metadata {
            title: metadata.title,
            subtitle: metadata.subtitle,
            author: metadata.author,
            row_height: metadata.row_height.unwrap_or(24),
            max_columns: metadata.max_columns.unwrap_or(28),
            label_width: metadata.label_width.unwrap_or(40),
            note_number_width: metadata.note_number_width.unwrap_or(8),
            parts_list_columns: metadata.parts_list_columns.unwrap_or(4),
        },
        measures,
        document_diagnostics: document_diagnostics
            .into_iter()
            .chain(combiner_diagnostics)
            .collect(),
        sequence: None,
    };
    validate_ties(&mut score);
    resolve_sequence(&mut score, sequence, sequence_parse_errors);
    if score.sequence.is_none() {
        validate_navigation_markers(&mut score);
    }
    Ok(score)
}

#[cfg(test)]
mod percussion_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests_lyrics.rs"]
mod tests_lyrics;

#[cfg(test)]
#[path = "tests_tie.rs"]
mod tests_tie;
