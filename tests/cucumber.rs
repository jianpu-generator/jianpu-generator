//! Cucumber harness for the positional-lyrics syntax proposal (bare,
//! unprefixed lyric lines attaching to the nearest preceding part, or
//! standing alone as an adurational block when they open a measure — see
//! `tests/features/positional_lyrics.feature`).
//!
//! This syntax is not implemented yet. Every scenario in that feature file
//! is expected to FAIL until the parser/desugar work lands — do not "fix"
//! these tests by loosening their expectations; fix the implementation.
//!
//! Clippy's `allow-*-in-tests` (clippy.toml) only recognizes `#[test]`-
//! attributed functions as test code; cucumber's `#[given]`/`#[when]`/
//! `#[then]` step functions don't qualify even though this whole file only
//! ever runs under `cargo test`. Mirrors `tests/main/integration.rs`'s
//! `#![allow(clippy::disallowed_macros)]` for the same reason.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_macros,
    clippy::needless_pass_by_value
)]

use cucumber::gherkin::Step;
use cucumber::{given, then, when, World as _};
use jianpu_generator::compile;

#[derive(Debug, Clone, Default)]
struct PartSnapshot {
    name: String,
    note_event_count: usize,
    /// One entry per verse; each verse is its syllables' text, in order.
    lyric_verses: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
struct MeasureSnapshot {
    parts: Vec<PartSnapshot>,
    diagnostics: Vec<String>,
}

#[derive(Debug, Default, cucumber::World)]
struct JianpuWorld {
    source: String,
    measures: Vec<MeasureSnapshot>,
}

fn find_part<'a>(measure: &'a MeasureSnapshot, name: &str) -> &'a PartSnapshot {
    measure
        .parts
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| {
            let present: Vec<&String> = measure.parts.iter().map(|p| &p.name).collect();
            panic!("no part named {name:?} in this measure; parts present: {present:?}")
        })
}

fn find_verse(part: &PartSnapshot, verse: usize) -> &Vec<String> {
    part.lyric_verses.get(verse - 1).unwrap_or_else(|| {
        panic!(
            "part {:?} has only {} lyric verse(s), no verse {verse}",
            part.name,
            part.lyric_verses.len()
        )
    })
}

#[given(expr = "the score source:")]
fn given_score_source(world: &mut JianpuWorld, step: &Step) {
    world.source = step.docstring().cloned().unwrap_or_default();
}

#[when(expr = "it is compiled")]
fn when_compiled(world: &mut JianpuWorld) {
    let score = compile(&world.source, "test.jianpu", &[])
        .unwrap_or_else(|err| panic!("compile returned an irrecoverable error: {}", err.message()));
    world.measures = score
        .measures
        .iter()
        .map(|measure| MeasureSnapshot {
            diagnostics: measure.diagnostics.iter().map(|d| d.message()).collect(),
            parts: measure
                .parts
                .iter()
                .map(|part| {
                    let slice = part.slice();
                    PartSnapshot {
                        name: slice.name.clone().unwrap_or_default(),
                        note_event_count: slice.notes.events.len(),
                        lyric_verses: slice
                            .lyrics
                            .iter()
                            .map(|verse| verse.syllables.iter().map(|s| s.text.clone()).collect())
                            .collect(),
                    }
                })
                .collect(),
        })
        .collect();
}

#[then(expr = "part {string} measure {int} has {int} note event(s)")]
fn then_note_event_count(world: &mut JianpuWorld, name: String, measure: usize, count: usize) {
    let m = &world.measures[measure - 1];
    let part = find_part(m, &name);
    assert_eq!(
        part.note_event_count, count,
        "expected {count} note event(s) on part {name:?} in measure {measure}, got {}",
        part.note_event_count
    );
}

#[then(expr = "part {string} measure {int} has {int} lyric verse(s)")]
fn then_lyric_verse_count(world: &mut JianpuWorld, name: String, measure: usize, count: usize) {
    let m = &world.measures[measure - 1];
    let part = find_part(m, &name);
    assert_eq!(
        part.lyric_verses.len(),
        count,
        "expected {count} lyric verse(s) on part {name:?} in measure {measure}, got {}",
        part.lyric_verses.len()
    );
}

#[then(expr = "part {string} measure {int} verse {int} has syllables {string}")]
fn then_verse_syllables(
    world: &mut JianpuWorld,
    name: String,
    measure: usize,
    verse: usize,
    syllables: String,
) {
    let m = &world.measures[measure - 1];
    let part = find_part(m, &name);
    let actual = find_verse(part, verse);
    let expected: Vec<String> = if syllables.is_empty() {
        Vec::new()
    } else {
        syllables.split(", ").map(str::to_string).collect()
    };
    assert_eq!(
        *actual, expected,
        "verse {verse} syllables for part {name:?} in measure {measure}"
    );
}

#[then(expr = "part {string} measure {int} verse {int} has {int} syllables")]
fn then_verse_syllable_count(
    world: &mut JianpuWorld,
    name: String,
    measure: usize,
    verse: usize,
    count: usize,
) {
    let m = &world.measures[measure - 1];
    let part = find_part(m, &name);
    let actual = find_verse(part, verse);
    assert_eq!(
        actual.len(),
        count,
        "verse {verse} syllable count for part {name:?} in measure {measure}"
    );
}

#[then(expr = "measure {int} has no diagnostics")]
fn then_no_diagnostics(world: &mut JianpuWorld, measure: usize) {
    let m = &world.measures[measure - 1];
    assert!(
        m.diagnostics.is_empty(),
        "expected no diagnostics on measure {measure}, got: {:?}",
        m.diagnostics
    );
}

#[tokio::main]
async fn main() {
    JianpuWorld::run("tests/features/positional_lyrics.feature").await;
}
