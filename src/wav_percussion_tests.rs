use super::write_percussion_preview_wav;

static SF2_BYTES: &[u8] = include_bytes!("../fonts/GeneralUser_GS.sf2");

#[test]
fn write_percussion_preview_wav_returns_riff_wav() {
    let bytes = write_percussion_preview_wav(38, SF2_BYTES).unwrap();
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
}
