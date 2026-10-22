//! The letter-name view: every note head's jianpu degree spelled as its
//! pitch's letter name (`F♯`), and every chord symbol's root and bass
//! likewise (`F♯m⁷/C♯`), against the key in effect at its measure.
//! Everything else about the layout stays as in the jianpu view.

use super::types::{ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow};
use crate::ast::grouped::Score;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};
use crate::pitch_description::spelling::SpelledPitch;

/// What a note head reads as: its jianpu degree (`1`–`7`), or its letter
/// name (`C`–`B`, with any sharps/flats the key implies).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NoteLabels {
    #[default]
    Degrees,
    LetterNames,
}

/// `result`, compiled from `score`, with its note heads labelled as
/// `labels` says: unchanged for degrees, each note head's `letter_name` set
/// for letter names.
pub fn label_note_heads(result: CompileResult, score: &Score, labels: NoteLabels) -> CompileResult {
    match labels {
        NoteLabels::Degrees => result,
        NoteLabels::LetterNames => spell_letter_names(result, score),
    }
}

fn spell_letter_names(result: CompileResult, score: &Score) -> CompileResult {
    let tonics = block_tonics(&result.blocks, score);
    CompileResult {
        blocks: result
            .blocks
            .into_iter()
            .zip(tonics)
            .map(|(block, tonic)| spell_block(block, tonic))
            .collect(),
        ..result
    }
}

/// The tonic of the key in effect at each block. A merged rest run's block
/// stands in for `represents_measures` source measures, so the walk advances
/// by that many and keeps the last key change among them.
fn block_tonics(blocks: &[MeasureBlock], score: &Score) -> Vec<SpelledPitch> {
    // Overwritten by the first measure, whose `key` the grouper always sets.
    let c_major = SpelledPitch::key_tonic(&KeyChange {
        note: Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    });
    blocks
        .iter()
        .scan((0, c_major), |(next_measure, tonic), block| {
            if let Some(key) = score
                .measures
                .iter()
                .skip(*next_measure)
                .take(block.represents_measures)
                .filter_map(|measure| measure.key.as_ref())
                .next_back()
            {
                *tonic = SpelledPitch::key_tonic(key);
            }
            *next_measure += block.represents_measures;
            Some(*tonic)
        })
        .collect()
}

fn spell_block(block: MeasureBlock, tonic: SpelledPitch) -> MeasureBlock {
    MeasureBlock {
        rows: block
            .rows
            .into_iter()
            .map(|row| MeasureRow {
                elements: row
                    .elements
                    .into_iter()
                    .map(|element| ColumnElement {
                        content: spell_content(element.content, tonic),
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
            letter_name: _,
        } => ElementContent::NoteHead {
            letter_name: Some(tonic.scale_degree(&pitch, &accidental).engraved_name()),
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
        } => ElementContent::ChordSymbol {
            text: degrees.letter_symbol(tonic),
            degrees,
            dotted,
            double_dotted,
        },
        other => other,
    }
}
