use crate::error::IrrecoverableError;
use crate::parser::parts_parser::InstrumentInfo;
use crate::{apply_track_filter, compile};

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

/// Parse, group, optionally filter tracks, and synthesize WAV for a consecutive measure range.
///
/// BPM and key context is accumulated from all measures before `start_index`.
#[cfg(feature = "wav")]
pub fn write_wav_for_measure_range_from_source(
    source: &str,
    filename: &str,
    measure_range: std::ops::RangeInclusive<usize>,
    enabled_tracks: Option<&[String]>,
    sf2_bytes: &[u8],
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let (score, start, end) = crate::midi::expand_for_measure_range(
        &score,
        *measure_range.start(),
        *measure_range.end(),
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

/// Same as [`measure_start_times_from_source`], but scoped to a measure range
/// and relative to the start of that range. Used to sync a playhead against
/// the audio clip returned by [`write_wav_for_measure_range_from_source`].
#[cfg(feature = "midi")]
pub fn measure_start_times_for_range_from_source(
    source: &str,
    filename: &str,
    measure_range: std::ops::RangeInclusive<usize>,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<f64>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let (score, start, end) = crate::midi::expand_for_measure_range(
        &score,
        *measure_range.start(),
        *measure_range.end(),
    )?;
    crate::midi::measure_start_times_seconds_for_range(&score, start, end)
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
#[cfg(feature = "midi")]
pub fn written_measure_indices_for_range_from_source(
    source: &str,
    filename: &str,
    measure_range: std::ops::RangeInclusive<usize>,
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
    )?;
    Ok(origins
        .get(start_pos..=end_pos)
        .map(<[usize]>::to_vec)
        .unwrap_or_default())
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
