use crate::ast::grouped::{
    default_author_font_size, default_lyrics_font_size, default_page_number_font_size,
    default_part_legend_font_size, default_subtitle_font_size, default_title_font_size,
    resolve_text_style, GroupedScore, GroupedTrack, Metadata, Score, TextStyle, TextStyleDefaults,
    DEFAULT_CHORDS_HORIZONTAL_PADDING_PT, DEFAULT_DIRECTIVE_ROW_OFFSET, DEFAULT_HIDE_RESTING_PARTS,
    DEFAULT_HIDE_SYSTEM_DIVIDERS, DEFAULT_LYRICS_HORIZONTAL_PADDING_PT,
    DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT, DEFAULT_MAX_MEASURES_PER_SYSTEM,
    DEFAULT_MEASURE_NUMBER_FONT_SIZE, DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
    DEFAULT_NOTES_HORIZONTAL_PADDING_PT, DEFAULT_NOTE_DASH_HORIZONTAL_PADDING_PT,
    DEFAULT_NOTE_NUMBER_WIDTH, DEFAULT_PARTS_LIST_COLUMNS, DEFAULT_PART_LABEL_FONT_SIZE,
    DEFAULT_PART_LABEL_WIDTH_PT, DEFAULT_ROW_HEIGHT, DEFAULT_SECTION_LABEL_FONT_SIZE,
    DEFAULT_SEQUENCE_FONT_SIZE,
};
use crate::ast::parsed::{ParsedDocument, ParsedMeasureSlot, ParsedMetadata, ParsedTrack};
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
    let document_diagnostics: Vec<Diagnostic> = doc
        .section_structure_errors
        .into_iter()
        .chain(doc.metadata_parse_errors)
        .chain(doc.parts_parse_errors)
        .map(Diagnostic::Error)
        .collect();
    let global_resolution_multipliers = compute_global_resolution_multipliers(&doc.tracks);
    let mut grouped_tracks = Vec::new();
    for track in doc.tracks {
        grouped_tracks.push(match track {
            ParsedTrack::Timed(part) => {
                GroupedTrack::Timed(group_timed_track(part, &global_resolution_multipliers)?)
            }
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
    resolve_sequence(&mut score, sequence, sequence_parse_errors, &declarations);
    Ok(score)
}

/// For each measure index, the tuplet-rescale multiplier every part must use at that
/// measure: `lcm` of every part's own `resolution_multiplier_of` at that index. Parts
/// share a measure's column space (see `MeasureBlock` in `grid_layout`), so if each part
/// rescaled tuplets against only its own content, a triplet in one part would desync
/// that measure's grid from a sibling part with no tuplet (or a different tuplet ratio),
/// leaving their notes misaligned at matching beats. Computing one multiplier per
/// measure index up front, before any part is grouped, keeps every part on the same
/// rescaled grid for that measure.
fn compute_global_resolution_multipliers(tracks: &[ParsedTrack]) -> Vec<u32> {
    let measure_count = tracks
        .iter()
        .map(|track| match track {
            ParsedTrack::Timed(part) => part.measure_slots.len(),
        })
        .max()
        .unwrap_or(0);
    (0..measure_count)
        .map(|slot_index| {
            tracks
                .iter()
                .filter_map(|track| match track {
                    ParsedTrack::Timed(part) => part.measure_slots.get(slot_index),
                })
                .filter_map(|slot| match slot {
                    ParsedMeasureSlot::Real { events } => {
                        Some(crate::tuplet::resolution_multiplier_of(events))
                    }
                    ParsedMeasureSlot::EmptyNote { .. } => None,
                })
                .fold(1, crate::tuplet::lcm)
        })
        .collect()
}

/// The thirteen `TextStyle` fields of `Metadata`, resolved separately from
/// `resolve_metadata`'s other (non-text-style) fields to keep each function
/// under clippy's line-count limit.
struct ResolvedTextStyles {
    title_style: TextStyle,
    subtitle_style: TextStyle,
    author_style: TextStyle,
    sequence: TextStyle,
    part_legend: TextStyle,
    measure_number: TextStyle,
    section_label: TextStyle,
    part_label: TextStyle,
    page_number: TextStyle,
    lyrics: TextStyle,
    notes: TextStyle,
    chords: TextStyle,
    note_dash: TextStyle,
}

/// `resolve_text_style` for the common case of a kind with no non-zero
/// horizontal/vertical padding default (every kind except `lyrics`, `notes`,
/// `chords`, `note_dash` — see `resolve_text_styles`).
fn simple_text_style(
    parsed: crate::ast::parsed::TextStyle,
    default_font_size: u32,
    defaults: TextStyleDefaults,
) -> TextStyle {
    resolve_text_style(parsed, default_font_size, 0, 0, defaults)
}

/// Resolves the `TextStyle` fields whose only non-zero default is `font_size`
/// (see `simple_text_style`). Split out from `resolve_text_styles` to stay
/// under clippy's line-count limit.
fn resolve_simple_text_styles(
    metadata: &ParsedMetadata,
    row_height: u32,
) -> (
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
) {
    use crate::compositor::types::FontFamily;
    let defaults = |bold: bool, italic: bool, font_family: FontFamily| TextStyleDefaults {
        bold,
        italic,
        font_family,
    };
    (
        simple_text_style(
            metadata.title_style,
            default_title_font_size(row_height),
            defaults(false, false, FontFamily::Title),
        ),
        simple_text_style(
            metadata.subtitle_style,
            default_subtitle_font_size(row_height),
            defaults(false, true, FontFamily::Title),
        ),
        simple_text_style(
            metadata.author_style,
            default_author_font_size(row_height),
            defaults(false, false, FontFamily::Title),
        ),
        simple_text_style(
            metadata.sequence_style,
            DEFAULT_SEQUENCE_FONT_SIZE,
            defaults(false, false, FontFamily::SansSerif),
        ),
        simple_text_style(
            metadata.part_legend_style,
            default_part_legend_font_size(row_height),
            defaults(false, false, FontFamily::SansSerif),
        ),
        simple_text_style(
            metadata.measure_number_style,
            DEFAULT_MEASURE_NUMBER_FONT_SIZE,
            defaults(false, false, FontFamily::SansSerif),
        ),
        simple_text_style(
            metadata.section_label_style,
            DEFAULT_SECTION_LABEL_FONT_SIZE,
            defaults(true, true, FontFamily::SansSerif),
        ),
        simple_text_style(
            metadata.page_number_style,
            default_page_number_font_size(row_height),
            defaults(false, false, FontFamily::SansSerif),
        ),
        simple_text_style(
            metadata.part_label_style,
            DEFAULT_PART_LABEL_FONT_SIZE,
            defaults(false, false, FontFamily::SansSerif),
        ),
    )
}

/// Resolves every text kind's `TextStyle`, applying each documented default
/// (see `Metadata`'s per-kind field docs) to whichever components the
/// `# metadata` section left unset.
fn resolve_text_styles(metadata: &ParsedMetadata, row_height: u32) -> ResolvedTextStyles {
    let (
        title_style,
        subtitle_style,
        author_style,
        sequence,
        part_legend,
        measure_number,
        section_label,
        page_number,
        part_label,
    ) = resolve_simple_text_styles(metadata, row_height);
    let lyrics_font_size = metadata
        .lyrics_style
        .font_size
        .unwrap_or_else(|| default_lyrics_font_size(row_height));
    let notes_font_size = metadata.notes_style.font_size.unwrap_or(lyrics_font_size);
    ResolvedTextStyles {
        title_style,
        subtitle_style,
        author_style,
        sequence,
        part_legend,
        measure_number,
        section_label,
        part_label,
        page_number,
        lyrics: resolve_text_style(
            metadata.lyrics_style,
            lyrics_font_size,
            DEFAULT_LYRICS_HORIZONTAL_PADDING_PT,
            DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT,
            TextStyleDefaults {
                bold: false,
                italic: false,
                font_family: crate::compositor::types::FontFamily::Title,
            },
        ),
        notes: resolve_text_style(
            metadata.notes_style,
            notes_font_size,
            DEFAULT_NOTES_HORIZONTAL_PADDING_PT,
            0,
            TextStyleDefaults {
                bold: false,
                italic: false,
                font_family: crate::compositor::types::FontFamily::Monospace,
            },
        ),
        chords: resolve_text_style(
            metadata.chords_style,
            lyrics_font_size,
            DEFAULT_CHORDS_HORIZONTAL_PADDING_PT,
            0,
            TextStyleDefaults {
                bold: false,
                italic: false,
                font_family: crate::compositor::types::FontFamily::Monospace,
            },
        ),
        note_dash: resolve_text_style(
            metadata.note_dash_style,
            notes_font_size,
            DEFAULT_NOTE_DASH_HORIZONTAL_PADDING_PT,
            0,
            TextStyleDefaults {
                bold: false,
                italic: false,
                font_family: crate::compositor::types::FontFamily::Monospace,
            },
        ),
    }
}

/// Fills in each unset `metadata` field with its documented default.
fn resolve_metadata(metadata: ParsedMetadata) -> Metadata {
    let row_height = metadata.row_height.unwrap_or(DEFAULT_ROW_HEIGHT);
    let styles = resolve_text_styles(&metadata, row_height);
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
        parts_list_columns: metadata
            .parts_list_columns
            .unwrap_or(DEFAULT_PARTS_LIST_COLUMNS),
        part_label_width_pt: metadata
            .part_label_width_pt
            .unwrap_or(DEFAULT_PART_LABEL_WIDTH_PT),
        title_style: styles.title_style,
        subtitle_style: styles.subtitle_style,
        author_style: styles.author_style,
        sequence: styles.sequence,
        part_legend: styles.part_legend,
        measure_number: styles.measure_number,
        section_label: styles.section_label,
        part_label: styles.part_label,
        page_number: styles.page_number,
        lyrics: styles.lyrics,
        notes: styles.notes,
        chords: styles.chords,
        note_dash: styles.note_dash,
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
