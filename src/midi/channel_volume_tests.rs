use super::channel_volume_control_value;
use crate::parser::parts_parser::PART_SETTING_LIMITS;

#[test]
fn maximum_volume_is_full_scale() {
    assert_eq!(
        channel_volume_control_value(PART_SETTING_LIMITS.volume.max),
        127
    );
}

#[test]
fn zero_volume_is_silent() {
    assert_eq!(channel_volume_control_value(0), 0);
}

#[test]
fn volume_above_the_maximum_never_exceeds_a_midi_data_byte() {
    assert_eq!(channel_volume_control_value(255), 127);
}
