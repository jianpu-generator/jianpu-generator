//! Real bodies for the `Guest` methods `guest_impl.rs` dispatches to. Kept
//! at the exact by-value parameter types the `Guest` trait requires (see
//! `mod.rs`'s doc comment), so `needless_pass_by_value` is relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;

// Phase 3, group 7 of PLAN-wit-bindgen-migration.md: closes the real
// porting gap group 6's "fully complete" claim missed — these 17
// functions have live `web/src` call sites but no `#[wasm_bindgen] fn`
// counterpart existed anywhere in Phase 3's ordering list. Same
// underlying-function-reuse pattern as every prior group; see each
// method below for which existing `crate::` function it delegates to,
// and this file's/`wit/world.wit`'s comments for the handful that had no
// separate `crate::` function to delegate to at all (their logic was
// inline in the old `#[wasm_bindgen] fn` itself, called here directly
// rather than through a newly-extracted function, since duplicating a
// one-line call into a `jianpu_generator::` function is not meaningfully
// riskier than extracting it).

pub(super) fn parse_metadata_fields(source: String) -> MetadataFields {
    metadata_fields_to_wit(jianpu_generator::source_edit::parse_metadata_fields(
        &source,
    ))
}

pub(super) fn update_metadata_field(source: String, edit: MetadataEdit) -> String {
    jianpu_generator::source_edit::update_metadata_field(&source, metadata_edit_from_wit(edit))
}

pub(super) fn highlight_tokens(source: String) -> Vec<HighlightToken> {
    jianpu_generator::highlight::highlight_tokens(&source)
        .into_iter()
        .map(highlight_token_to_wit)
        .collect()
}

pub(super) fn section_headers(source: String) -> Vec<SectionHeader> {
    jianpu_generator::highlight::section_headers(&source)
        .into_iter()
        .map(section_header_to_wit)
        .collect()
}

// Same underlying `jianpu_generator` calls as the old
// `wasm_boundary::set_layout_fonts`/`shift_part_octave`/`format_score`
// `#[wasm_bindgen] fn`s — their logic was already a direct call into
// `jianpu_generator`, not a separate `crate::` response function, so
// this duplicates that same one/three-line call rather than extracting
// a new shared function for it.

pub(super) fn set_layout_fonts(fonts: FontBytesByFamily) {
    jianpu_generator::set_layout_font_bytes(font_bytes_by_family_from_wit(fonts));
}

pub(super) fn shift_part_octave(source: String, abbreviation: String, delta: i32) -> String {
    jianpu_generator::source_edit::shift_part_octave(&source, &abbreviation, delta as i8)
}

pub(super) fn list_streamable_parts(source: String) -> Vec<StreamablePart> {
    jianpu_generator::part_stream::list_streamable_parts(&source)
        .into_iter()
        .map(|part| StreamablePart {
            abbreviation: part.abbreviation,
            display_name: part.display_name,
            kind: streamable_part_kind_to_wit(part.kind),
            ineligible_reason: part.ineligible_reason,
        })
        .collect()
}

pub(super) fn extract_part_stream(
    source: String,
    abbreviation: String,
) -> ExtractPartStreamResponse {
    use jianpu_generator::part_stream;
    match part_stream::extract_part_stream(&source, &abbreviation) {
        Ok(stream) => ExtractPartStreamResponse::Ok(PartStream {
            text: stream.text,
            measure_starts: offsets_to_wit(&stream.measure_starts),
        }),
        Err(error) => ExtractPartStreamResponse::Err(part_stream_failure_to_wit(error)),
    }
}

fn part_stream_failure_to_wit(
    error: jianpu_generator::part_stream::PartStreamError,
) -> PartStreamFailure {
    use jianpu_generator::part_stream;
    PartStreamFailure {
        message: error.describe(),
        reason: match error {
            part_stream::PartStreamError::SourceDoesNotParse { message } => {
                PartStreamError::SourceDoesNotParse(message)
            }
            part_stream::PartStreamError::UnknownPart { abbreviation } => {
                PartStreamError::UnknownPart(abbreviation)
            }
            part_stream::PartStreamError::Ineligible { reason } => {
                PartStreamError::Ineligible(reason)
            }
        },
    }
}

pub(super) fn format_part_stream(
    source: String,
    abbreviation: String,
    edited_stream: String,
    cursors: Vec<u32>,
) -> FormattedPartStream {
    use jianpu_generator::part_stream::StreamCursor;
    let cursors: Vec<StreamCursor> = cursors
        .iter()
        .map(|&offset| StreamCursor {
            offset: offset as usize,
        })
        .collect();
    let formatted = jianpu_generator::part_stream::format_part_stream(
        &source,
        &abbreviation,
        &edited_stream,
        &cursors,
    );
    FormattedPartStream {
        edits: formatted
            .edits
            .into_iter()
            .map(|edit| StreamTextEdit {
                span: Span {
                    start: edit.span.start as u32,
                    end: edit.span.end as u32,
                },
                replacement: edit.replacement,
            })
            .collect(),
        cursors: formatted
            .cursors
            .iter()
            .map(|cursor| cursor.offset as u32)
            .collect(),
    }
}

pub(super) fn part_focus_measures(
    source: String,
    abbreviation: String,
    toggles: PartToggleState,
) -> PartFocusMeasuresResponse {
    match jianpu_generator::part_stream::part_focus_measures(
        &source,
        &abbreviation,
        &part_toggle_state_from_wit(toggles),
    ) {
        Ok(measures) => PartFocusMeasuresResponse::Ok(
            measures
                .into_iter()
                .map(|measure| FocusMeasure {
                    section_label: measure.section_label,
                    lines_above: measure.lines_above,
                    lines_below: measure.lines_below,
                })
                .collect(),
        ),
        Err(error) => PartFocusMeasuresResponse::Err(part_stream_failure_to_wit(error)),
    }
}

pub(super) fn apply_part_stream(
    source: String,
    abbreviation: String,
    edited_stream: String,
) -> PartStreamApplyResult {
    use jianpu_generator::part_stream::StreamDiagnosticSeverity;
    let result =
        jianpu_generator::part_stream::apply_part_stream(&source, &abbreviation, &edited_stream);
    PartStreamApplyResult {
        source: result.source,
        diagnostics: result
            .diagnostics
            .into_iter()
            .map(|diagnostic| Diagnostic {
                severity: match diagnostic.severity {
                    StreamDiagnosticSeverity::Error => DiagnosticSeverity::Error,
                    StreamDiagnosticSeverity::Warning => DiagnosticSeverity::Warning,
                },
                message: diagnostic.message,
                span: Span {
                    start: diagnostic.span.start as u32,
                    end: diagnostic.span.end as u32,
                },
            })
            .collect(),
        measure_starts: offsets_to_wit(&result.measure_starts),
        changed_measure_count: result.changed_measure_count as u32,
    }
}

fn streamable_part_kind_to_wit(
    kind: jianpu_generator::part_stream::StreamablePartKind,
) -> StreamablePartKind {
    use jianpu_generator::part_stream::StreamablePartKind as Kind;
    match kind {
        Kind::Notes => StreamablePartKind::Notes,
        Kind::Chords => StreamablePartKind::Chords,
        Kind::Percussion => StreamablePartKind::Percussion,
        Kind::Follow => StreamablePartKind::Follow,
        Kind::Lyrics => StreamablePartKind::Lyrics,
    }
}

fn offsets_to_wit(offsets: &[usize]) -> Vec<u32> {
    offsets.iter().map(|&offset| offset as u32).collect()
}

pub(super) fn shift_range_octave(
    source: String,
    ranges: Vec<ByteRange>,
    delta: i32,
) -> RangeEditResponse {
    let ranges = to_source_edit_ranges(ranges);
    let result = jianpu_generator::source_edit::shift_range_octave(&source, &ranges, delta as i8);
    to_range_edit_response(result)
}

pub(super) fn toggle_range_slur(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
    let ranges = to_source_edit_ranges(ranges);
    let result = jianpu_generator::source_edit::toggle_range_slur(&source, &ranges);
    to_range_edit_response(result)
}

pub(super) fn toggle_range_tie(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
    let ranges = to_source_edit_ranges(ranges);
    let result = jianpu_generator::source_edit::toggle_range_tie(&source, &ranges);
    to_range_edit_response(result)
}

pub(super) fn describe_selection(
    source: String,
    ranges: Vec<ByteRange>,
) -> Option<PitchDescription> {
    use jianpu_generator::pitch_description as description;
    let ranges = to_source_edit_ranges(ranges);
    description::describe_selection(&source, &ranges).map(|described| match described {
        description::PitchDescription::Note(note) => PitchDescription::Note(NoteDescription {
            letter_name: note.letter_name,
        }),
        description::PitchDescription::Chord(chord) => PitchDescription::Chord(ChordDescription {
            chord_name: chord.chord_name,
            tone_names: chord.tone_names,
            bass_note: chord.bass_note,
            guitar_diagram_svgs: chord.guitar_diagram_svgs,
            piano_diagram_svg: chord.piano_diagram_svg,
        }),
    })
}

fn to_source_edit_ranges(ranges: Vec<ByteRange>) -> Vec<jianpu_generator::source_edit::ByteRange> {
    ranges
        .into_iter()
        .map(|range| jianpu_generator::source_edit::ByteRange {
            start_byte: range.start_byte,
            end_byte: range.end_byte,
        })
        .collect()
}

fn to_range_edit_response(
    result: jianpu_generator::source_edit::RangeEditResult,
) -> RangeEditResponse {
    RangeEditResponse {
        source: result.source,
        ranges: result
            .ranges
            .into_iter()
            .map(|range| ByteRange {
                start_byte: range.start_byte,
                end_byte: range.end_byte,
            })
            .collect(),
    }
}

pub(super) fn format_score(source: String) -> String {
    jianpu_generator::format_source::format_score(&source)
}

// Same underlying `crate::part_declarations::list_part_declarations_response`/
// `update_part_declaration_source` the old `wasm_boundary::
// list_part_declarations`/`update_part_declaration` `#[wasm_bindgen] fn`s
// call — untouched, both mechanisms coexist.

pub(super) fn list_part_declarations(
    source: String,
    raw_instruments: Vec<InstrumentInfo>,
) -> ListPartDeclarationsResponse {
    let instruments: Vec<jianpu_generator::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    list_part_declarations_response_to_wit(
        crate::part_declarations::list_part_declarations_response(&source, &instruments),
    )
}

pub(super) fn update_part_declaration(
    source: String,
    abbreviation: String,
    settings: PartSettings,
    raw_instruments: Vec<InstrumentInfo>,
) -> String {
    let instruments: Vec<jianpu_generator::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    crate::part_declarations::update_part_declaration_source(
        &source,
        &abbreviation,
        &part_settings_from_wit(settings),
        &instruments,
    )
}

pub(super) fn list_sound_choices(raw_instruments: Vec<InstrumentInfo>) -> SoundChoices {
    let instruments: Vec<jianpu_generator::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    sound_choices_to_wit(jianpu_generator::sound_label::list_sound_choices(
        &instruments,
    ))
}

pub(super) fn get_part_setting_limits() -> PartSettingLimits {
    part_setting_limits_to_wit(jianpu_generator::parser::parts_parser::PART_SETTING_LIMITS)
}

// `extract_source_from_svg`/`extract_source_from_pdf`'s logic was
// already a direct call into `jianpu_generator::source_embed` (no
// separate `crate::` response function existed), so these duplicate that
// same call rather than extracting a new shared function for it — same
// judgment call as `set_layout_fonts`/`shift_part_octave`/`format_score`
// above.

pub(super) fn extract_source_from_svg(svg_bytes: Vec<u8>) -> Option<String> {
    let svg = std::str::from_utf8(&svg_bytes).ok()?;
    jianpu_generator::source_embed::extract_embedded_source(svg)
}

pub(super) fn extract_source_from_pdf(pdf_bytes: Vec<u8>) -> Option<String> {
    jianpu_generator::source_embed::extract_embedded_source_from_pdf(&pdf_bytes)
}

// Same underlying `crate::compress_share_payload_bytes`/
// `decompress_share_payload_bytes` the old `share_payload::
// compress_share_payload`/`decompress_share_payload` `#[wasm_bindgen]
// fn`s now also call — both extracted out of `share_payload.rs` (a
// whole-module-gated file) into `lib.rs` this group, mirroring group
// 5/6's `types_export.rs`/`trim_window` extraction pattern, so this
// boundary can reach them too.

pub(super) fn compress_share_payload(payload: String) -> Vec<u8> {
    crate::compress_share_payload_bytes(&payload)
}

pub(super) fn decompress_share_payload(bytes: Vec<u8>) -> Option<String> {
    crate::decompress_share_payload_bytes(&bytes)
}

// Same underlying `crate::selection_range::resolve_selection_range_response`
// the old `selection_range::resolve_selection_range` `#[wasm_bindgen]
// fn` calls — that function was already unconditional (not gated on
// `wasm-bindgen-boundary`), unlike this group's other findings, so no
// prerequisite-ungating fix was needed here, only widening
// `ClickableElementId`/`NoteCellOut`/`LyricCellOut`/
// `ResolveSelectionRangeResponse`'s reachability from `component.rs` (see
// `selection_range/mod.rs`'s own comment on its `pub(crate) use`).

pub(super) fn resolve_selection_range(
    note_spans: Vec<NoteSpan>,
    lyric_spans: Vec<LyricSpan>,
    anchor: ClickableElementId,
    current: ClickableElementId,
) -> ResolveSelectionRangeResponse {
    let note_spans: Vec<crate::types::NoteSpanOut> =
        note_spans.into_iter().map(note_span_from_wit).collect();
    let lyric_spans: Vec<crate::types::LyricSpanOut> =
        lyric_spans.into_iter().map(lyric_span_from_wit).collect();
    let anchor = clickable_element_id_from_wit(anchor);
    let current = clickable_element_id_from_wit(current);
    resolve_selection_range_response_to_wit(
        crate::selection_range::resolve_selection_range_response(
            &note_spans,
            &lyric_spans,
            &anchor,
            &current,
        ),
    )
}
