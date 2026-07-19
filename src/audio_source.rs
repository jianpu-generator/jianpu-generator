use crate::error::IrrecoverableError;
use crate::parser::parts_parser::InstrumentInfo;
use crate::{apply_track_filter, compile};
use crate::{compiler, consolidator, grid_layout};

/// Parse, group, optionally filter tracks, and synthesize WAV bytes.
///
/// When `enabled_tracks` is `None`, all parts are included.
/// When `Some(tracks)` is empty, no parts are included.
#[cfg(feature = "wav")]
pub fn write_wav_from_source_filtered(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    sf2_bytes: &[u8],
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let score = crate::midi::expand_navigation(&score)?;
    let midi_bytes = crate::midi::write_midi(&score)?;
    crate::wav::write_wav(&midi_bytes, sf2_bytes)
}

/// Parse, group, optionally filter tracks, and synthesize WAV for a single measure.
///
/// BPM and key context is accumulated from all preceding measures so
/// that mid-piece measures sound correct even without explicit directives.
#[cfg(feature = "wav")]
pub fn write_wav_for_measure_from_source(
    source: &str,
    filename: &str,
    measure_index: usize,
    enabled_tracks: Option<&[String]>,
    sf2_bytes: &[u8],
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let (score, measure_index) = crate::midi::expand_for_measure(&score, measure_index)?;
    let midi_bytes = crate::midi::write_midi_for_measure(&score, measure_index)?;
    crate::wav::write_wav(&midi_bytes, sf2_bytes)
}

/// A measure range to synthesize, plus how its end measure should be
/// resolved when it recurs later in the performance (due to a repeat/jump).
///
/// When `extend_to_last_occurrence` is `true`, the end measure is extended
/// through its last occurrence — this is what the web app's "play from
/// current measure" (which always passes the score's literal last written
/// measure as the range end) needs to follow the repeat to the true end.
/// When `false`, the range stops at the end measure's first occurrence at or
/// after the start — what an exact range selection (e.g. "play current
/// measure") needs to avoid overrunning into a later repeat/jump pass.
pub struct MeasureRangeSelection {
    pub range: std::ops::RangeInclusive<usize>,
    pub extend_to_last_occurrence: bool,
}

/// Parse, group, optionally filter tracks, and synthesize WAV for a consecutive measure range.
///
/// BPM and key context is accumulated from all measures before the range's start.
#[cfg(feature = "wav")]
pub fn write_wav_for_measure_range_from_source(
    source: &str,
    filename: &str,
    selection: &MeasureRangeSelection,
    enabled_tracks: Option<&[String]>,
    sf2_bytes: &[u8],
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let (score, start, end) = crate::midi::expand_for_measure_range(
        &score,
        *selection.range.start(),
        *selection.range.end(),
        selection.extend_to_last_occurrence,
    )?;
    let midi_bytes = crate::midi::write_midi_for_measure_range(&score, start, end)?;
    crate::wav::write_wav(&midi_bytes, sf2_bytes)
}

/// Parse, group, optionally filter tracks, and compute the elapsed-seconds
/// offset of each measure boundary (length = `measures + 1`; the last entry
/// is the total duration). Used to sync a UI playhead against WAV audio
/// returned by [`write_wav_from_source_filtered`].
#[cfg(feature = "midi")]
pub fn measure_start_times_from_source(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<f64>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let score = crate::midi::expand_navigation(&score)?;
    crate::midi::measure_start_times_seconds(&score)
}

/// Parse, group, optionally filter tracks, and compute the elapsed-seconds
/// start/end of every sounding note/rest, keyed by `(source_part_index,
/// note_id)`. Used to drive the SVG preview's per-part, per-note playback
/// highlight (see [`crate::midi::NoteTiming`]), replacing the old
/// measure-level playhead built from [`measure_start_times_from_source`].
///
/// Follows the same `# sequence`/D.C.-al-Coda-Fine playback order as the
/// actual audio (see [`crate::midi::note_timings_seconds`]), so a repeated
/// or reordered written note is highlighted every time it's actually heard,
/// not just its first pass — while `note_id`s themselves still agree with
/// `ColumnElement::note_id`/the rendered SVG's `data-note-id`, since those
/// are computed from the written score, not the expanded timeline.
#[cfg(feature = "midi")]
pub fn note_timings_from_source(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<crate::midi::NoteTiming>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    crate::midi::note_timings_seconds(&score)
}

/// Same as [`measure_start_times_from_source`], but scoped to a measure range
/// and relative to the start of that range. Used to sync a playhead against
/// the audio clip returned by [`write_wav_for_measure_range_from_source`].
///
/// See [`write_wav_for_measure_range_from_source`] for `extend_to_last_occurrence`.
#[cfg(feature = "midi")]
pub fn measure_start_times_for_range_from_source(
    source: &str,
    filename: &str,
    measure_range: std::ops::RangeInclusive<usize>,
    extend_to_last_occurrence: bool,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<f64>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let (score, start, end) = crate::midi::expand_for_measure_range(
        &score,
        *measure_range.start(),
        *measure_range.end(),
        extend_to_last_occurrence,
    )?;
    crate::midi::measure_start_times_seconds_for_range(&score, start, end)
}

/// Same as [`note_timings_from_source`], but scoped to a measure range and
/// relative to the start of that range (`start_s`/`end_s` are seconds from
/// the start of the clip [`write_wav_for_measure_range_from_source`]
/// produces for the same range, not from the start of the whole piece).
///
/// See [`write_wav_for_measure_range_from_source`] for `extend_to_last_occurrence`.
#[cfg(feature = "midi")]
pub fn note_timings_for_range_from_source(
    source: &str,
    filename: &str,
    measure_range: std::ops::RangeInclusive<usize>,
    extend_to_last_occurrence: bool,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<crate::midi::NoteTiming>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let (_, start_pos, end_pos) = crate::midi::expand_for_measure_range(
        &score,
        *measure_range.start(),
        *measure_range.end(),
        extend_to_last_occurrence,
    )?;
    crate::midi::note_timings_seconds_for_range(&score, start_pos, end_pos)
}

/// Parse, group, optionally filter tracks, and return the written measure
/// index that each position in [`measure_start_times_from_source`]'s
/// playback-ordered timeline corresponds to (length = playback-sequence
/// length, i.e. `measure_start_times_from_source(...).len() - 1`).
///
/// Used to translate a playback position into the written measure to
/// highlight via [`crate::render_svgs_with_highlight_range`], so a UI
/// playhead follows D.C. al Coda navigation (repeats/jumps) instead of
/// assuming playback position and written measure index are the same thing.
#[cfg(feature = "midi")]
pub fn written_measure_indices_from_source(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<usize>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let (_, origins) = crate::midi::expand_navigation_with_origins(&score)?;
    Ok(origins)
}

/// Same as [`written_measure_indices_from_source`], but scoped to a measure
/// range: entries correspond 1:1 to [`measure_start_times_for_range_from_source`]'s
/// timeline for the same range.
///
/// See [`write_wav_for_measure_range_from_source`] for `extend_to_last_occurrence`.
#[cfg(feature = "midi")]
pub fn written_measure_indices_for_range_from_source(
    source: &str,
    filename: &str,
    measure_range: std::ops::RangeInclusive<usize>,
    extend_to_last_occurrence: bool,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<usize>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let (_, origins) = crate::midi::expand_navigation_with_origins(&score)?;
    let (_, start_pos, end_pos) = crate::midi::expand_for_measure_range(
        &score,
        *measure_range.start(),
        *measure_range.end(),
        extend_to_last_occurrence,
    )?;
    Ok(origins
        .get(start_pos..=end_pos)
        .map(<[usize]>::to_vec)
        .unwrap_or_default())
}

/// Parse, group, and optionally filter tracks, then return the cumulative
/// pixel-weight column boundaries of every rendered measure (one entry per
/// `data-measure-index`, see [`grid_layout::measure_column_boundaries`]).
///
/// Used to map a playhead's linear elapsed-time fraction within a measure
/// (from [`measure_start_times_from_source`]) onto the non-linear pixel
/// position notes actually render at, since measure/column width is
/// density-weighted rather than duration-proportional.
pub fn measure_column_boundaries_from_source(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<Vec<f32>>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let compile_result = compiler::compile(&score);
    let compile_result = consolidator::consolidate(compile_result);
    Ok(grid_layout::measure_column_boundaries(&compile_result))
}

/// Parse, group, optionally filter tracks, and generate MIDI (SMF) bytes.
///
/// When `enabled_tracks` is `None`, all parts are included.
/// When `Some(tracks)` is empty, no parts are included.
#[cfg(feature = "midi")]
pub fn write_midi_from_source_filtered(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let score = crate::midi::expand_navigation(&score)?;
    crate::midi::write_midi(&score)
}
