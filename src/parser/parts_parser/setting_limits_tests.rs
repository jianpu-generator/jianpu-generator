use super::{parse_parts, PART_SETTING_LIMITS};
use crate::error::{RecoverableErrorKind, Span};

#[test]
fn volume_at_either_bound_is_accepted() {
    let content = "A = notes 0%\nB = notes 100%\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].volume, PART_SETTING_LIMITS.volume.min);
    assert_eq!(decls[1].volume, PART_SETTING_LIMITS.volume.max);
}

#[test]
fn volume_above_max_clamps_with_diagnostic_on_the_volume_token() {
    let content = "B = notes 150%\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert_eq!(decls[0].volume, 100);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors[0].kind,
        RecoverableErrorKind::PartsVolumeOutOfRange { volume: 150 }
    );
    assert_eq!(errors[0].span, Span::new(10, 14));
    assert_eq!(
        errors[0].kind.message(),
        "volume 150% is out of range; valid range is 0% to 100%; clamped to 100%"
    );
}

#[test]
fn volume_too_wide_for_a_byte_still_clamps() {
    let content = "B = notes 999%\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert_eq!(decls[0].volume, 100);
    assert_eq!(
        errors[0].kind,
        RecoverableErrorKind::PartsVolumeOutOfRange { volume: 999 }
    );
}

#[test]
fn octave_offset_at_either_bound_is_accepted() {
    let content = "A = notes -4\nB = notes +4\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(
        decls[0].octave_offset,
        PART_SETTING_LIMITS.octave_offset.min
    );
    assert_eq!(
        decls[1].octave_offset,
        PART_SETTING_LIMITS.octave_offset.max
    );
}

#[test]
fn octave_offset_above_max_clamps_with_diagnostic() {
    let content = "B = notes +5\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert_eq!(decls[0].octave_offset, 4);
    assert_eq!(
        errors[0].kind,
        RecoverableErrorKind::PartsOctaveOffsetOutOfRange { offset: 5 }
    );
    assert_eq!(errors[0].span, Span::new(10, 12));
    assert_eq!(
        errors[0].kind.message(),
        "octave offset +5 is out of range; valid range is -4 to +4; clamped to +4"
    );
}

#[test]
fn octave_offset_below_min_clamps_with_diagnostic() {
    let content = "B = notes -200\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert_eq!(decls[0].octave_offset, -4);
    assert_eq!(
        errors[0].kind.message(),
        "octave offset -200 is out of range; valid range is -4 to +4; clamped to -4"
    );
}

#[test]
fn follow_part_inherits_its_targets_clamped_volume() {
    let content = "A = notes 150%\nB = follow[A]\n";
    let (decls, _) = parse_parts(content, 0, &[]);
    assert_eq!(decls[1].volume, 100);
}
