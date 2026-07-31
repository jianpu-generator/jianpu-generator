use crate::ast::parsed::{Offset, ParsedMetadata};
use crate::error::{RecoverableError, Span};

fn span_of_key_in_line(byte_offset: usize, line: &str, key_raw: &str, key: &str) -> Span {
    let leading_whitespace = line.len() - line.trim_start().len();
    let key_start_in_key_raw = key_raw.len() - key_raw.trim_start().len();
    let key_start = byte_offset + leading_whitespace + key_start_in_key_raw;
    Span::new(key_start, key_start + key.len())
}

fn span_of_value_in_line(byte_offset: usize, line: &str, key_raw: &str, value_raw: &str) -> Span {
    let leading_whitespace_in_line = line.len() - line.trim_start().len();
    let value_leading_whitespace = value_raw.len() - value_raw.trim_start().len();
    let value_start =
        byte_offset + leading_whitespace_in_line + key_raw.len() + 1 + value_leading_whitespace;
    let value_trimmed = value_raw.trim();
    Span::new(value_start, value_start + value_trimmed.len())
}

fn parse_positive_u32(key: &str, value: &str, value_span: &Span) -> Result<u32, RecoverableError> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| RecoverableError::metadata_invalid_integer(*value_span, key, value))?;
    if parsed == 0 {
        return Err(RecoverableError::metadata_must_be_positive(
            *value_span,
            key,
        ));
    }
    Ok(parsed)
}

fn parse_numeric_field(
    target: &mut Option<u32>,
    key: &str,
    value: &str,
    value_span: &Span,
    errors: &mut Vec<RecoverableError>,
) {
    match parse_positive_u32(key, value, value_span) {
        Ok(v) => *target = Some(v),
        Err(e) => errors.push(e),
    }
}

fn parse_offset_field(
    target: &mut Option<Offset>,
    key: &str,
    value: &str,
    value_span: &Span,
    errors: &mut Vec<RecoverableError>,
) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let parsed = match parts.as_slice() {
        [x, y] => x.parse::<i32>().ok().zip(y.parse::<i32>().ok()),
        _ => None,
    };
    match parsed {
        Some((x, y)) => *target = Some(Offset { x, y }),
        None => errors.push(RecoverableError::metadata_invalid_integer_pair(
            *value_span,
            key,
            value,
        )),
    }
}

fn parse_bool_field(
    target: &mut Option<bool>,
    key: &str,
    value: &str,
    value_span: &Span,
    errors: &mut Vec<RecoverableError>,
) {
    match value {
        "yes" => *target = Some(true),
        "no" => *target = Some(false),
        _ => errors.push(RecoverableError::metadata_invalid_boolean(
            *value_span,
            key,
            value,
        )),
    }
}

#[derive(Default)]
struct MetadataAccumulator {
    title: Option<String>,
    subtitle: Option<String>,
    author: Option<String>,
    row_height: Option<u32>,
    max_measures_per_system: Option<u32>,
    note_number_width: Option<u32>,
    part_label_width_pt: Option<u32>,
    parts_list_columns: Option<u32>,
    lyrics_font_size: Option<u32>,
    notes_font_size: Option<u32>,
    chords_font_size: Option<u32>,
    title_font_size: Option<u32>,
    subtitle_font_size: Option<u32>,
    author_font_size: Option<u32>,
    sequence_font_size: Option<u32>,
    merge_duplicate_measures_across_parts: Option<bool>,
    hide_resting_parts: Option<bool>,
    hide_system_dividers: Option<bool>,
    directive_row_offset: Option<Offset>,
}

impl MetadataAccumulator {
    fn apply_field(
        &mut self,
        key: &str,
        value: &str,
        key_span: Span,
        value_span: &Span,
        errors: &mut Vec<RecoverableError>,
    ) {
        match key {
            "title" => self.title = Some(value.to_string()),
            "subtitle" => self.subtitle = Some(value.to_string()),
            "author" => self.author = Some(value.to_string()),
            "row_height" => {
                parse_numeric_field(&mut self.row_height, key, value, value_span, errors)
            }
            "max_measures_per_system" => parse_numeric_field(
                &mut self.max_measures_per_system,
                key,
                value,
                value_span,
                errors,
            ),
            "note_number_width" => {
                parse_numeric_field(&mut self.note_number_width, key, value, value_span, errors)
            }
            "part_label_width_pt" => parse_numeric_field(
                &mut self.part_label_width_pt,
                key,
                value,
                value_span,
                errors,
            ),
            "parts_list_columns" => {
                parse_numeric_field(&mut self.parts_list_columns, key, value, value_span, errors)
            }
            "lyrics_font_size" => {
                parse_numeric_field(&mut self.lyrics_font_size, key, value, value_span, errors)
            }
            "notes_font_size" => {
                parse_numeric_field(&mut self.notes_font_size, key, value, value_span, errors)
            }
            "chords_font_size" => {
                parse_numeric_field(&mut self.chords_font_size, key, value, value_span, errors)
            }
            "title_font_size" => {
                parse_numeric_field(&mut self.title_font_size, key, value, value_span, errors)
            }
            "subtitle_font_size" => {
                parse_numeric_field(&mut self.subtitle_font_size, key, value, value_span, errors)
            }
            "author_font_size" => {
                parse_numeric_field(&mut self.author_font_size, key, value, value_span, errors)
            }
            "sequence_font_size" => {
                parse_numeric_field(&mut self.sequence_font_size, key, value, value_span, errors)
            }
            "merge_duplicate_measures_across_parts" => parse_bool_field(
                &mut self.merge_duplicate_measures_across_parts,
                key,
                value,
                value_span,
                errors,
            ),
            "hide_resting_parts" => {
                parse_bool_field(&mut self.hide_resting_parts, key, value, value_span, errors)
            }
            "hide_system_dividers" => parse_bool_field(
                &mut self.hide_system_dividers,
                key,
                value,
                value_span,
                errors,
            ),
            "directive_row_offset" => parse_offset_field(
                &mut self.directive_row_offset,
                key,
                value,
                value_span,
                errors,
            ),
            _ => errors.push(RecoverableError::metadata_unknown_field(key_span, key)),
        }
    }
}

pub fn parse_metadata(
    content: &str,
    base_offset: usize,
) -> (ParsedMetadata, Vec<RecoverableError>) {
    let mut accumulator = MetadataAccumulator::default();
    let mut byte_offset = base_offset;
    let mut errors: Vec<RecoverableError> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            byte_offset += line.len() + 1;
            continue;
        }

        let Some((key_raw, value_raw)) = trimmed.split_once('=') else {
            errors.push(RecoverableError::metadata_malformed_line(
                Span::new(byte_offset, byte_offset + line.len()),
                trimmed,
            ));
            byte_offset += line.len() + 1;
            continue;
        };

        let key = key_raw.trim();
        let value = value_raw.trim().trim_matches('"');

        let key_span = span_of_key_in_line(byte_offset, line, key_raw, key);
        let value_span = span_of_value_in_line(byte_offset, line, key_raw, value_raw);

        accumulator.apply_field(key, value, key_span, &value_span, &mut errors);

        byte_offset += line.len() + 1;
    }

    (
        ParsedMetadata {
            title: accumulator.title,
            subtitle: accumulator.subtitle,
            author: accumulator.author,
            row_height: accumulator.row_height,
            max_measures_per_system: accumulator.max_measures_per_system,
            note_number_width: accumulator.note_number_width,
            part_label_width_pt: accumulator.part_label_width_pt,
            parts_list_columns: accumulator.parts_list_columns,
            lyrics_font_size: accumulator.lyrics_font_size,
            notes_font_size: accumulator.notes_font_size,
            chords_font_size: accumulator.chords_font_size,
            title_font_size: accumulator.title_font_size,
            subtitle_font_size: accumulator.subtitle_font_size,
            author_font_size: accumulator.author_font_size,
            sequence_font_size: accumulator.sequence_font_size,
            merge_duplicate_measures_across_parts: accumulator
                .merge_duplicate_measures_across_parts,
            hide_resting_parts: accumulator.hide_resting_parts,
            hide_system_dividers: accumulator.hide_system_dividers,
            directive_row_offset: accumulator.directive_row_offset,
        },
        errors,
    )
}

#[cfg(test)]
#[path = "metadata_parser_tests.rs"]
mod metadata_parser_tests;
