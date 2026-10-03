use itertools::Itertools;

use super::spelling::Interval;

const WHITE_KEY_WIDTH: i16 = 22;
const WHITE_KEY_HEIGHT: i16 = 64;
const BLACK_KEY_WIDTH: i16 = 14;
const BLACK_KEY_HEIGHT: i16 = 40;
const LABEL_FONT_SIZE: i16 = 9;
/// White keys shown beyond the outermost chord tone on each side, so the
/// viewer can tell which key a chord tone is from its neighbours.
const PADDING_WHITE_KEYS: i16 = 2;
const HIGHLIGHT_FILL: &str = "#3b82f6";

/// One sounding chord tone on the keyboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PianoTone {
    /// MIDI note number; only its relative position matters.
    pub(crate) midi: i16,
    /// Interval from the chord root, e.g. `"1"`, `"b3"`, `"#5"`.
    pub(crate) label: String,
    /// The tone's function in the chord, e.g. `"Root"`, `"Minor 3rd"`.
    pub(crate) function_name: String,
}

/// Names the role `interval` plays in a chord: `"Root"`, `"Minor 3rd"`,
/// `"Perfect 5th"`, ...
pub(crate) fn interval_function_name(interval: Interval) -> String {
    let name = match (interval.letter_steps, interval.semitones) {
        (0, 0) => "Root",
        (1, 1) => "Minor 2nd",
        (1, 2) => "Major 2nd",
        (2, 3) => "Minor 3rd",
        (2, 4) => "Major 3rd",
        (3, 5) => "Perfect 4th",
        (4, 6) => "Diminished 5th",
        (4, 7) => "Perfect 5th",
        (4, 8) => "Augmented 5th",
        (6, 10) => "Minor 7th",
        (6, 11) => "Major 7th",
        _ => return interval_label(interval),
    };
    name.to_string()
}

/// Names `interval` by its scale degree above the root, with a `b`/`#` when
/// it differs from the major scale's: a minor third is `"b3"`, an augmented
/// fifth `"#5"`, a minor seventh `"b7"`, the root `"1"`.
pub(crate) fn interval_label(interval: Interval) -> String {
    const MAJOR_SCALE_SEMITONES: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];
    let major_semitones = MAJOR_SCALE_SEMITONES
        .get(usize::from(interval.letter_steps))
        .copied()
        .unwrap_or(0);
    let alteration = interval.semitones - major_semitones;
    let accidental = if alteration >= 0 { "#" } else { "b" };
    format!(
        "{}{}",
        accidental.repeat(usize::from(alteration.unsigned_abs())),
        interval.letter_steps + 1
    )
}

fn is_black_key(midi: i16) -> bool {
    matches!(midi.rem_euclid(12), 1 | 3 | 6 | 8 | 10)
}

/// Extends `edge` by `PADDING_WHITE_KEYS` white keys in `step` direction,
/// then on to the next white key so the keyboard never ends on a black one.
fn padded_edge(edge: i16, step: i16) -> i16 {
    let mut current = edge;
    let mut white_keys_added = 0;
    while white_keys_added < PADDING_WHITE_KEYS || is_black_key(current) {
        current += step;
        if !is_black_key(current) {
            white_keys_added += 1;
        }
    }
    current
}

/// Renders a keyboard spanning just the chord tones plus padding. The tones
/// are filled and labelled with their interval from the root; every other key
/// is left blank. The root `<svg>` carries `data-piano-tones` (e.g.
/// `"1:0 b3:3 5:7"`, label and semitones above the lowest tone) for tests.
pub(crate) fn render_piano_diagram_svg(tones: &[PianoTone], chord_name: &str) -> String {
    let Some((lowest, highest)) = tones.iter().map(|tone| tone.midi).minmax().into_option() else {
        return String::new();
    };
    let first_key = padded_edge(lowest, -1);
    let last_key = padded_edge(highest, 1);
    let white_keys = (first_key..=last_key)
        .filter(|&midi| !is_black_key(midi))
        .collect_vec();
    let width = WHITE_KEY_WIDTH * white_keys.len() as i16;
    let tone_at = |midi: i16| tones.iter().find(|tone| tone.midi == midi);

    let white_key_shapes = white_keys
        .iter()
        .enumerate()
        .map(|(index, &midi)| {
            render_key(
                index as i16 * WHITE_KEY_WIDTH,
                WHITE_KEY_WIDTH,
                WHITE_KEY_HEIGHT,
                tone_at(midi),
                false,
            )
        })
        .join("");
    let black_key_shapes = (first_key..=last_key)
        .filter(|&midi| is_black_key(midi))
        .map(|midi| {
            let white_keys_before = white_keys.iter().filter(|&&white| white < midi).count() as i16;
            render_key(
                white_keys_before * WHITE_KEY_WIDTH - BLACK_KEY_WIDTH / 2,
                BLACK_KEY_WIDTH,
                BLACK_KEY_HEIGHT,
                tone_at(midi),
                true,
            )
        })
        .join("");
    let tones_attribute = tones
        .iter()
        .map(|tone| format!("{}:{}", tone.label, tone.midi - lowest))
        .join(" ");
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {WHITE_KEY_HEIGHT}" width="{width}" height="{WHITE_KEY_HEIGHT}" role="img" aria-label="Piano keyboard for {chord_name}" data-piano-tones="{tones_attribute}" stroke="currentColor">{white_key_shapes}{black_key_shapes}</svg>"#
    )
}

fn render_key(x: i16, width: i16, height: i16, tone: Option<&PianoTone>, is_black: bool) -> String {
    let fill = match (tone, is_black) {
        (Some(_), _) => HIGHLIGHT_FILL,
        (None, true) => "currentColor",
        (None, false) => "none",
    };
    let rectangle = format!(
        r#"<rect x="{x}" y="0" width="{width}" height="{height}" fill="{fill}" stroke-width="1"/>"#
    );
    let Some(tone) = tone else {
        return rectangle;
    };
    // `<title>` is the native mouse-hover tooltip; `data-tone-function` lets
    // the page show the same text on touch.
    format!(
        r##"<g data-tone-function="{function_name}"><title>{function_name}</title>{rectangle}<text x="{center}" y="{y}" font-size="{LABEL_FONT_SIZE}" text-anchor="middle" fill="#ffffff" stroke="none">{label}</text></g>"##,
        function_name = tone.function_name,
        center = x + width / 2,
        y = height - 4,
        label = tone.label,
    )
}
