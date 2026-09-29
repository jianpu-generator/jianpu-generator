use crate::parser::parts_parser::PART_SETTING_LIMITS;

/// The largest value a MIDI data byte can hold.
const MAX_MIDI_DATA_BYTE: u32 = 127;

/// A part's volume percentage as a CC7 (channel volume) data byte. A volume
/// above the part-setting maximum is clamped, so the result is always a valid
/// MIDI data byte.
pub(super) fn channel_volume_control_value(volume: u8) -> u8 {
    let max_volume = u32::from(PART_SETTING_LIMITS.volume.max);
    let scaled = u32::from(volume) * MAX_MIDI_DATA_BYTE / max_volume;
    scaled.min(MAX_MIDI_DATA_BYTE) as u8
}
