//! The letter-name view: every note head's jianpu degree spelled as its
//! pitch's letter name (`F♯`), and every chord symbol's root and bass
//! likewise (`F♯m⁷/C♯`), against the key in effect at its measure.
//! Everything else about the layout stays as in the jianpu view. The guitar
//! chord view spells the same letter names after the degrees instead
//! (`3 (F♯)`, `6m (Bm)`). Also home
//! to `label_note_heads`, which dispatches to every view that keeps the
//! jianpu layout — see `NoteLabels`.

use super::types::{
    ColumnElement, CompileResult, ElementContent, LetterNamePlacement, MeasureBlock, MeasureRow,
    NoteHeadLetterName,
};
use crate::ast::grouped::Score;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};
use crate::pitch_description::spelling::SpelledPitch;

/// Which view of the jianpu layout to draw: what a note head reads as — its
/// jianpu degree (`1`–`7`), or its letter name (`C`–`B`, with any
/// sharps/flats the key implies) — and whether chord symbols carry guitar
/// chord diagrams.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NoteLabels {
    #[default]
    Degrees,
    LetterNames,
    /// Degrees followed by their letter names (`1 (C)`, `6m (Am)`), plus a
    /// guitar chord diagram above every chord symbol (see `guitar_chords`).
    GuitarChords,
}

/// `result`, compiled from `score`, labelled as `labels` says: unchanged for
/// degrees, each note head's `letter_name` set for letter names, and for
/// guitar chords each note head's `letter_name` set to follow its degree
/// and each chord symbol's `guitar_voicing` set.
pub fn label_note_heads(result: CompileResult, score: &Score, labels: NoteLabels) -> CompileResult {
    match labels {
        NoteLabels::Degrees => result,
        NoteLabels::LetterNames => {
            spell_letter_names(result, score, LetterNamePlacement::InPlaceOfDegree)
        }
        NoteLabels::GuitarChords => spell_letter_names(
            super::guitar_chords::place_guitar_voicings(result, score),
            score,
            LetterNamePlacement::AfterDegree,
        ),
    }
}

fn spell_letter_names(
    result: CompileResult,
    score: &Score,
    placement: LetterNamePlacement,
) -> CompileResult {
    let tonics = block_tonics(&result.blocks, score);
    CompileResult {
        blocks: result
            .blocks
            .into_iter()
            .zip(tonics)
            .map(|(block, tonic)| spell_block(block, tonic, placement))
            .collect(),
        ..result
    }
}

/// The tonic of the key in effect at each block. A merged rest run's block
/// stands in for `represents_measures` source measures, so the walk advances
/// by that many and keeps the last key change among them.
pub(super) fn block_tonics(blocks: &[MeasureBlock], score: &Score) -> Vec<SpelledPitch> {
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

fn spell_block(
    block: MeasureBlock,
    tonic: SpelledPitch,
    placement: LetterNamePlacement,
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
                        content: spell_content(element.content, tonic, placement),
                        ..element
                    })
                    .collect(),
                ..row
            })
            .collect(),
        ..block
    }
}

fn spell_content(
    content: ElementContent,
    tonic: SpelledPitch,
    placement: LetterNamePlacement,
) -> ElementContent {
    match content {
        ElementContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
            double_dotted,
            letter_name: _,
        } => ElementContent::NoteHead {
            letter_name: Some(NoteHeadLetterName {
                text: tonic.scale_degree(&pitch, &accidental).engraved_name(),
                placement,
            }),
            pitch,
            accidental,
            octave,
            dotted,
            double_dotted,
        },
        ElementContent::ChordSymbol {
            text,
            degrees,
            dotted,
            double_dotted,
            guitar_voicing,
        } => ElementContent::ChordSymbol {
            text: match placement {
                LetterNamePlacement::InPlaceOfDegree => degrees.letter_symbol(tonic),
                LetterNamePlacement::AfterDegree => {
                    format!("{text} ({})", degrees.letter_symbol(tonic))
                }
            },
            degrees,
            dotted,
            double_dotted,
            guitar_voicing,
        },
        other => other,
    }
}
