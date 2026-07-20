use crate::responses::{generate_wav_for_measure_range_response, generate_wav_response};
use crate::types::GenerateWavResponse;

#[test]
fn generate_wav_for_measure_range_response_returns_riff_wav() {
    let source = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n",
    );
    let soundfont = include_bytes!("../../../fonts/GeneralUser_GS.sf2").to_vec();
    let resp =
        generate_wav_for_measure_range_response(source, 0, 0, false, true, None, None, soundfont);
    match resp {
        GenerateWavResponse::Ok { wav } => {
            assert!(wav.len() > 4);
            assert_eq!(&wav[0..4], b"RIFF");
        }
        GenerateWavResponse::Err { diagnostics } => {
            panic!("expected Ok: {}", diagnostics[0].message);
        }
    }
}

#[test]
fn reference_jianpu_generates_wav() {
    let soundfont = include_bytes!("../../../fonts/GeneralUser_GS.sf2").to_vec();
    for path in super::demo_file_paths() {
        let source = super::read_demo_file(&path);
        let resp = generate_wav_response(&source, None, soundfont.clone());
        match resp {
            GenerateWavResponse::Ok { wav } => {
                assert!(wav.len() > 4);
                assert_eq!(&wav[0..4], b"RIFF");
            }
            GenerateWavResponse::Err { diagnostics } => {
                panic!(
                    "{path:?} failed in wasm wav path: {}",
                    diagnostics[0].message
                );
            }
        }
    }
}
