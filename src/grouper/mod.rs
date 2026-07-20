use crate::ast::grouped::{
    default_lyrics_font_size, GroupedScore, GroupedTrack, Metadata, Score,
    DEFAULT_DIRECTIVE_ROW_OFFSET, DEFAULT_HIDE_RESTING_PARTS, DEFAULT_HIDE_SYSTEM_DIVIDERS,
    DEFAULT_MAX_MEASURES_PER_SYSTEM, DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
    DEFAULT_NOTE_NUMBER_WIDTH, DEFAULT_PARTS_LIST_COLUMNS, DEFAULT_PART_LABEL_WIDTH_PT,
    DEFAULT_ROW_HEIGHT,
};
use crate::ast::parsed::{ParsedDocument, ParsedMetadata, ParsedTrack};
use crate::combiner;
use crate::error::{Diagnostic, IrrecoverableError};

#[path = "empty_note_measures.rs"]
mod empty_note_measures;

mod directive_grouper;
mod lyrics_pairing;
mod part_grouper;
mod sequence_resolution;
mod tie_validation;

use directive_grouper::DirectiveGrouper;
use part_grouper::group_timed_track;
use sequence_resolution::resolve_sequence;
use tie_validation::validate_ties;

pub fn group(doc: ParsedDocument) -> Result<Score, IrrecoverableError> {
    let metadata = doc.metadata;
    let sequence = doc.sequence;
    let sequence_parse_errors = doc.sequence_parse_errors;
    let declarations = doc.declarations;
    let group = doc.group;
    let document_diagnostics: Vec<Diagnostic> = doc
        .section_structure_errors
        .into_iter()
        .chain(doc.metadata_parse_errors)
        .chain(doc.parts_parse_errors)
        .chain(doc.group_parse_errors)
        .map(Diagnostic::Error)
        .collect();
    let mut grouped_tracks = Vec::new();
    for track in doc.tracks {
        grouped_tracks.push(match track {
            ParsedTrack::Timed(part) => GroupedTrack::Timed(group_timed_track(part)?),
        });
    }

    let measure_directives = DirectiveGrouper::new(
        metadata
            .merge_duplicate_measures_across_parts
            .unwrap_or(DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS),
        metadata
            .hide_resting_parts
            .unwrap_or(DEFAULT_HIDE_RESTING_PARTS),
    )
    .process_all(&doc.directive_events_per_measure);

    let grouped_score = GroupedScore {
        measure_directives,
        parts: grouped_tracks,
        per_measure_parse_errors: doc.per_measure_parse_errors,
    };

    let (measures, combiner_diagnostics) = combiner::combine(&grouped_score);

    let mut score = Score {
        metadata: resolve_metadata(metadata),
        measures,
        document_diagnostics: document_diagnostics
            .into_iter()
            .chain(combiner_diagnostics)
            .collect(),
        sequence: None,
    };
    validate_ties(&mut score);
    resolve_sequence(
        &mut score,
        sequence,
        sequence_parse_errors,
        &declarations,
        group.as_ref(),
    );
    Ok(score)
}

/// Fills in each unset `metadata` field with its documented default.
fn resolve_metadata(metadata: ParsedMetadata) -> Metadata {
    let row_height = metadata.row_height.unwrap_or(DEFAULT_ROW_HEIGHT);
    Metadata {
        title: metadata.title,
        subtitle: metadata.subtitle,
        author: metadata.author,
        row_height,
        max_measures_per_system: metadata
            .max_measures_per_system
            .unwrap_or(DEFAULT_MAX_MEASURES_PER_SYSTEM),
        note_number_width: metadata
            .note_number_width
            .unwrap_or(DEFAULT_NOTE_NUMBER_WIDTH),
        part_label_width_pt: metadata
            .part_label_width_pt
            .unwrap_or(DEFAULT_PART_LABEL_WIDTH_PT),
        parts_list_columns: metadata
            .parts_list_columns
            .unwrap_or(DEFAULT_PARTS_LIST_COLUMNS),
        lyrics_font_size: metadata
            .lyrics_font_size
            .unwrap_or_else(|| default_lyrics_font_size(row_height)),
        merge_duplicate_measures_across_parts: metadata
            .merge_duplicate_measures_across_parts
            .unwrap_or(DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS),
        hide_resting_parts: metadata
            .hide_resting_parts
            .unwrap_or(DEFAULT_HIDE_RESTING_PARTS),
        hide_system_dividers: metadata
            .hide_system_dividers
            .unwrap_or(DEFAULT_HIDE_SYSTEM_DIVIDERS),
        directive_row_offset: metadata
            .directive_row_offset
            .unwrap_or(DEFAULT_DIRECTIVE_ROW_OFFSET),
    }
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
