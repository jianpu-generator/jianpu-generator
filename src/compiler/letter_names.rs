//! The letter-name and harmonica-tab views: every note head's jianpu degree
//! replaced by its pitch's letter name (`F♯`) or its harmonica tab (`-4`),
//! resolved against the key in effect at its measure. The letter-name view
//! also respells chord symbols (`F♯m⁷/C♯`). Everything else about the layout
//! stays as in the jianpu view. Also home to `label_note_heads`, which
//! dispatches to every view that keeps the jianpu layout — see `NoteLabels`.

use super::harmonica_tabs::HarmonicaTabs;
use super::types::{
    ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow, NoteHeadLabel,
};
use crate::ast::grouped::Score;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};
use crate::pitch_description::spelling::SpelledPitch;

/// Which view of the jianpu layout to draw: what a note head reads as — its
/// jianpu degree (`1`–`7`), its letter name (`C`–`B`, with any sharps/flats
/// the key implies), or its tab on a 10-hole diatonic harmonica (see
/// `harmonica_tabs`) — and whether chord symbols carry guitar chord diagrams.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NoteLabels {
    #[default]
    Degrees,
    LetterNames,
    HarmonicaTabs,
    /// Degrees, plus a guitar chord diagram above every chord symbol (see
    /// `guitar_chords`).
    GuitarChords,
}

/// `result`, compiled from `score`, labelled as `labels` says: unchanged for
/// degrees, each note head's `head_label` set for letter names and harmonica
/// tabs, each chord symbol's `guitar_voicing` set for guitar chords.
pub fn label_note_heads(result: CompileResult, score: &Score, labels: NoteLabels) -> CompileResult {
    match labels {
        NoteLabels::Degrees => result,
        NoteLabels::LetterNames => relabel_blocks(result, score, |content, key| {
            spell_content(content, SpelledPitch::key_tonic(key))
        }),
        NoteLabels::HarmonicaTabs => {
            let harmonica = HarmonicaTabs::tuned_to(opening_key(score));
            relabel_blocks(result, score, |content, key| {
                harmonica.tab_content(content, key)
            })
        }
        NoteLabels::GuitarChords => super::guitar_chords::place_guitar_voicings(result, score),
    }
}

/// `result` with every element's content passed through `relabel`, along
/// with the key in effect at its block.
fn relabel_blocks(
    result: CompileResult,
    score: &Score,
    relabel: impl Fn(ElementContent, &KeyChange) -> ElementContent,
) -> CompileResult {
    let keys = block_keys(&result.blocks, score);
    CompileResult {
        blocks: result
            .blocks
            .into_iter()
            .zip(keys)
            .map(|(block, key)| relabel_block(block, |content| relabel(content, &key)))
            .collect(),
        ..result
    }
}

/// The tonic of the key in effect at each block (see `block_keys`).
pub(super) fn block_tonics(blocks: &[MeasureBlock], score: &Score) -> Vec<SpelledPitch> {
    block_keys(blocks, score)
        .iter()
        .map(SpelledPitch::key_tonic)
        .collect()
}

fn c_major() -> KeyChange {
    KeyChange {
        note: Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    }
}

/// The key the score opens in, which the grouper always sets on the first
/// measure.
fn opening_key(score: &Score) -> KeyChange {
    score
        .measures
        .first()
        .and_then(|measure| measure.key.clone())
        .unwrap_or_else(c_major)
}

/// The key in effect at each block. A merged rest run's block stands in for
/// `represents_measures` source measures, so the walk advances by that many
/// and keeps the last key change among them.
fn block_keys(blocks: &[MeasureBlock], score: &Score) -> Vec<KeyChange> {
    blocks
        .iter()
        // Overwritten by the first measure's key, as `opening_key`.
        .scan((0, c_major()), |(next_measure, key), block| {
            if let Some(change) = score
                .measures
                .iter()
                .skip(*next_measure)
                .take(block.represents_measures)
                .filter_map(|measure| measure.key.as_ref())
                .next_back()
            {
                *key = change.clone();
            }
            *next_measure += block.represents_measures;
            Some(key.clone())
        })
        .collect()
}

fn relabel_block(
    block: MeasureBlock,
    relabel: impl Fn(ElementContent) -> ElementContent,
) -> MeasureBlock {
    MeasureBlock {
        rows: block
            .rows
            .into_iter()
            .map(|row| MeasureRow {
                elements: row
                    .elements
                    .into_iter()
                    .map(|element| ColumnElement {
                        content: relabel(element.content),
                        ..element
                    })
                    .collect(),
                ..row
            })
            .collect(),
        ..block
    }
}

fn spell_content(content: ElementContent, tonic: SpelledPitch) -> ElementContent {
    match content {
        ElementContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
            double_dotted,
            head_label: _,
        } => ElementContent::NoteHead {
            head_label: Some(NoteHeadLabel::playable(
                tonic.scale_degree(&pitch, &accidental).engraved_name(),
            )),
            pitch,
            accidental,
            octave,
            dotted,
            double_dotted,
        },
        ElementContent::ChordSymbol {
            text: _,
            degrees,
            dotted,
            double_dotted,
            guitar_voicing,
        } => ElementContent::ChordSymbol {
            text: degrees.letter_symbol(tonic),
            degrees,
            dotted,
            double_dotted,
            guitar_voicing,
        },
        other => other,
    }
}
