//! Experimental: export a grouped `Score` as MEI, for engraving as western
//! staff notation by Verovio.
//!
//! One staff per `notes` part, chord parts become `<harm>` symbols over the
//! top sounding staff, and lyric parts become verses on their target's
//! notes. Every note's `xml:id` encodes its source byte range
//! (`n{start}-{end}`), which Verovio carries through to its SVG output.

mod duration;
mod harmony;
mod layer;
mod lyrics;
mod pitch;
mod staves;

use crate::ast::grouped::{MultiPartMeasure, NoteEvent, PartSlice, Score, TimeSignature};
use crate::ast::parsed::{Accidental, KeyChange, NoteName, PartKind};
use harmony::{harm_xml, HarmonyPlacement};
use itertools::Itertools;
use layer::{layer_xml, lyrics_only_layer_xml, LayerContext, Slur, StaffState};
use lyrics::{collect_verses, measure_syllables, Verse};
use pitch::key_signature;
use staves::{collect_staves, StaffPart};
use std::collections::HashMap;

/// One `<measure>`'s pieces, kept apart until the slurs starting in it have
/// been added to `controls`.
struct MeasureXml {
    preamble: String,
    staves: String,
    controls: String,
}

/// What stays in effect from measure to measure.
struct ScoreState {
    key: KeyChange,
    time: TimeSignature,
    staves: Vec<StaffState>,
    slurs: Vec<Slur>,
}

/// Everything fixed for the whole score.
struct ScoreParts<'a> {
    staves: &'a [StaffPart],
    verses: &'a HashMap<String, Vec<Verse>>,
}

pub fn score_to_mei(score: &Score, display_names: &HashMap<String, String>) -> String {
    let staves = collect_staves(score, display_names);
    let verses = collect_verses(score);
    let parts = ScoreParts {
        staves: &staves,
        verses: &verses,
    };
    let first = score.measures.first();
    let mut state = ScoreState {
        key: first
            .and_then(|measure| measure.key.clone())
            .unwrap_or_else(default_key),
        time: first
            .and_then(|measure| measure.time_signature.clone())
            .unwrap_or(TimeSignature {
                numerator: 4,
                denominator: 4,
            }),
        staves: staves.iter().map(|_| StaffState::default()).collect(),
        slurs: Vec::new(),
    };
    let score_def = score_def_xml(&staves, &state.key, &state.time);
    let mut measures: Vec<MeasureXml> = score
        .measures
        .iter()
        .enumerate()
        .map(|(index, measure)| measure_xml(measure, index, &parts, &mut state))
        .collect();
    for slur in &state.slurs {
        if let Some(measure) = measures.get_mut(slur.start_measure) {
            measure.controls.push_str(&format!(
                "<slur staff=\"{}\" startid=\"#{}\" endid=\"#{}\"/>\n",
                slur.staff, slur.start_id, slur.end_id
            ));
        }
    }
    document_xml(score, &score_def, &measures)
}

pub(crate) fn default_key() -> KeyChange {
    KeyChange {
        note: crate::ast::parsed::Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    }
}

fn measure_xml(
    measure: &MultiPartMeasure,
    measure_index: usize,
    parts: &ScoreParts,
    state: &mut ScoreState,
) -> MeasureXml {
    let preamble = if measure_index == 0 {
        String::new()
    } else {
        measure_preamble(measure, state)
    };
    let staves = parts
        .staves
        .iter()
        .enumerate()
        .zip(state.staves.iter_mut())
        .map(|((staff_index, staff), staff_state)| {
            let staff_number = staff_index + 1;
            let syllables = parts
                .verses
                .get(&staff.name)
                .map(|verses| measure_syllables(verses, measure_index))
                .unwrap_or_default();
            let layer = if staff.lyrics_only {
                lyrics_only_layer(measure, &staff.name, &syllables)
            } else {
                part_slice(measure, &staff.name).map_or_else(
                    || "<mRest/>".to_string(),
                    |slice| {
                        let context = LayerContext {
                            staff,
                            staff_number,
                            measure_index,
                            key: &state.key,
                            verses: &syllables,
                        };
                        layer_xml(slice, &context, staff_state, &mut state.slurs)
                    },
                )
            };
            format!("<staff n=\"{staff_number}\"><layer n=\"1\">{layer}</layer></staff>\n")
        })
        .join("");
    MeasureXml {
        preamble,
        staves,
        controls: measure_controls(measure, parts.staves, state),
    }
}

/// System break and key/time changes written before the measure.
fn measure_preamble(measure: &MultiPartMeasure, state: &mut ScoreState) -> String {
    let break_xml = if measure.system_break { "<sb/>\n" } else { "" };
    let key_change = measure.key.as_ref().filter(|key| **key != state.key);
    let time_change = measure
        .time_signature
        .as_ref()
        .filter(|time| **time != state.time);
    let key_attribute = key_change
        .map(|key| format!(" key.sig=\"{}\"", key_signature(key)))
        .unwrap_or_default();
    let time_attribute = time_change
        .map(|time| {
            format!(
                " meter.count=\"{}\" meter.unit=\"{}\"",
                time.numerator, time.denominator
            )
        })
        .unwrap_or_default();
    if let Some(key) = key_change {
        state.key = key.clone();
    }
    if let Some(time) = time_change {
        state.time = time.clone();
    }
    if key_attribute.is_empty() && time_attribute.is_empty() {
        break_xml.to_string()
    } else {
        format!("{break_xml}<scoreDef{key_attribute}{time_attribute}/>\n")
    }
}

/// Rehearsal mark and chord symbols. Condensing hides staves with nothing to
/// play, so they hang on the top staff that has notes in this measure.
fn measure_controls(
    measure: &MultiPartMeasure,
    staves: &[StaffPart],
    state: &ScoreState,
) -> String {
    let top_staff = staves
        .iter()
        .position(|staff| {
            part_slice(measure, &staff.name).is_some_and(|slice| {
                slice
                    .notes
                    .events
                    .iter()
                    .any(|event| matches!(event, NoteEvent::Note(_)))
            })
        })
        .unwrap_or(0)
        + 1;
    let rehearsal = measure
        .label
        .as_ref()
        .map(|label| {
            format!(
                "<reh staff=\"{top_staff}\" tstamp=\"1\">{}</reh>\n",
                escape(label)
            )
        })
        .unwrap_or_default();
    let placement = HarmonyPlacement {
        key: &state.key,
        meter_unit: state.time.denominator,
        staff: top_staff,
    };
    let harmony = measure
        .parts
        .iter()
        .map(|row| row.slice())
        .find(|slice| matches!(slice.kind, PartKind::Chords))
        .map(|slice| harm_xml(slice, &placement))
        .unwrap_or_default();
    format!("{rehearsal}{harmony}")
}

/// The verses of a hidden part, on its rhythm as copied into the lyric parts
/// that target it.
fn lyrics_only_layer(
    measure: &MultiPartMeasure,
    target: &str,
    syllables: &[lyrics::MeasureSyllables],
) -> String {
    measure
        .parts
        .iter()
        .map(|row| row.slice())
        .find_map(|slice| {
            let lyrics = slice.lyrics.as_ref()?;
            (lyrics.target_name == target)
                .then(|| lyrics_only_layer_xml(lyrics, slice.resolution_multiplier, syllables))
        })
        .unwrap_or_else(|| "<mSpace/>".to_string())
}

fn part_slice<'a>(measure: &'a MultiPartMeasure, name: &str) -> Option<&'a PartSlice> {
    measure
        .parts
        .iter()
        .map(|row| row.slice())
        .find(|slice| slice.name.as_deref() == Some(name))
}

fn score_def_xml(staves: &[StaffPart], key: &KeyChange, time: &TimeSignature) -> String {
    let staff_defs = staves
        .iter()
        .enumerate()
        .map(|(index, staff)| {
            if staff.lyrics_only {
                return lyrics_only_staff_def(index + 1, key, time);
            }
            format!(
                "<staffDef n=\"{}\" lines=\"5\" {}><label>{}</label><labelAbbr>{}</labelAbbr></staffDef>",
                index + 1,
                staff.clef.attributes(),
                escape(&staff.label),
                escape(&staff.name)
            )
        })
        .join("\n");
    // `optimize` lets Verovio's `condense: "encoded"` hide empty staves per system.
    format!(
        "<scoreDef optimize=\"true\" key.sig=\"{}\" meter.count=\"{}\" meter.unit=\"{}\">\n<staffGrp symbol=\"bracket\" bar.thru=\"false\">\n{staff_defs}\n</staffGrp>\n</scoreDef>",
        key_signature(key),
        time.numerator,
        time.denominator
    )
}

/// A one-line staff with its line, clef, key and meter hidden, so only the
/// verses on its invisible notes show. Its own `takt` group shrinks its
/// barlines to a short tick, the least Verovio draws.
fn lyrics_only_staff_def(staff_number: usize, key: &KeyChange, time: &TimeSignature) -> String {
    format!(
        "<staffGrp bar.method=\"takt\"><staffDef n=\"{staff_number}\" lines=\"1\" lines.visible=\"false\"><clef shape=\"G\" line=\"1\" visible=\"false\"/><keySig sig=\"{}\" visible=\"false\"/><meterSig count=\"{}\" unit=\"{}\" visible=\"false\"/></staffDef></staffGrp>",
        key_signature(key),
        time.numerator,
        time.denominator
    )
}

fn document_xml(score: &Score, score_def: &str, measures: &[MeasureXml]) -> String {
    let body = measures
        .iter()
        .enumerate()
        .map(|(index, measure)| {
            format!(
                "{}<measure n=\"{}\">\n{}{}</measure>\n",
                measure.preamble,
                index + 1,
                measure.staves,
                measure.controls
            )
        })
        .join("");
    let title = score
        .metadata
        .title
        .as_deref()
        .map(escape)
        .unwrap_or_default();
    let composer = score
        .metadata
        .author
        .as_deref()
        .map(escape)
        .unwrap_or_default();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei" meiversion="5.0">
<meiHead><fileDesc><titleStmt><title>{title}</title><respStmt><persName role="composer">{composer}</persName></respStmt></titleStmt><pubStmt/></fileDesc></meiHead>
<music><body><mdiv><score>
{score_def}
<section>
{body}</section>
</score></mdiv></body></music>
</mei>
"#
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
