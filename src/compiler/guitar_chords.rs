//! The guitar chord view: every chord symbol that chords-db has a voicing
//! for gets a chord diagram, chosen per part so the progression (in written
//! order) needs the least hand movement — see `least_movement_path`.
//! Everything else about the layout stays as in the jianpu view.

use std::collections::HashMap;

use itertools::Itertools;

use super::chord_degrees::ChordDegrees;
use super::guitar_voicing_path::least_movement_path;
use super::letter_names::block_tonics;
use super::types::{ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow};
use crate::ast::grouped::Score;
use crate::pitch_description::spelling::SpelledPitch;
use crate::pitch_description::{guitar_voicings, GuitarVoicing};

/// Where one element sits in a `CompileResult`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ElementPosition {
    block: usize,
    row: usize,
    element: usize,
}

/// A chord symbol and every voicing chords-db has for it.
struct ChordCandidates {
    position: ElementPosition,
    source_part_index: usize,
    voicings: Vec<&'static GuitarVoicing>,
}

/// `result`, compiled from `score`, with each chord symbol's
/// `guitar_voicing` set to its voicing on its part's least-movement path.
/// A chord chords-db has no voicing for keeps `None` and is skipped by the
/// path, as if it weren't there.
pub(super) fn place_guitar_voicings(result: CompileResult, score: &Score) -> CompileResult {
    let tonics = block_tonics(&result.blocks, score);
    let mut chosen: HashMap<ElementPosition, GuitarVoicing> =
        chord_candidates(&result.blocks, &tonics)
            .into_iter()
            .into_group_map_by(|candidates| candidates.source_part_index)
            .into_values()
            .flat_map(|part_chords| {
                let path = least_movement_path(
                    &part_chords
                        .iter()
                        .map(|candidates| candidates.voicings.clone())
                        .collect_vec(),
                );
                part_chords
                    .into_iter()
                    .zip(path)
                    .filter_map(|(candidates, index)| {
                        Some((
                            candidates.position,
                            (*candidates.voicings.get(index)?).clone(),
                        ))
                    })
                    .collect_vec()
            })
            .collect();
    CompileResult {
        blocks: result
            .blocks
            .into_iter()
            .enumerate()
            .map(|(block_index, block)| MeasureBlock {
                rows: block
                    .rows
                    .into_iter()
                    .enumerate()
                    .map(|(row_index, row)| MeasureRow {
                        elements: row
                            .elements
                            .into_iter()
                            .enumerate()
                            .map(|(element_index, element)| {
                                let position = ElementPosition {
                                    block: block_index,
                                    row: row_index,
                                    element: element_index,
                                };
                                match chosen.remove(&position) {
                                    Some(voicing) => with_voicing(element, voicing),
                                    None => element,
                                }
                            })
                            .collect(),
                        ..row
                    })
                    .collect(),
                ..block
            })
            .collect(),
        ..result
    }
}

/// Every chord symbol with at least one chords-db voicing, in written order.
fn chord_candidates(blocks: &[MeasureBlock], tonics: &[SpelledPitch]) -> Vec<ChordCandidates> {
    blocks
        .iter()
        .zip(tonics)
        .enumerate()
        .flat_map(|(block_index, (block, &tonic))| {
            block
                .rows
                .iter()
                .enumerate()
                .flat_map(move |(row_index, row)| {
                    row.elements
                        .iter()
                        .enumerate()
                        .filter_map(move |(element_index, element)| match &element.content {
                            ElementContent::ChordSymbol { degrees, .. } => Some(ChordCandidates {
                                position: ElementPosition {
                                    block: block_index,
                                    row: row_index,
                                    element: element_index,
                                },
                                source_part_index: row.source_part_index,
                                voicings: chord_voicings(degrees, tonic),
                            }),
                            _ => None,
                        })
                })
        })
        .filter(|candidates| !candidates.voicings.is_empty())
        .collect()
}

fn chord_voicings(degrees: &ChordDegrees, tonic: SpelledPitch) -> Vec<&'static GuitarVoicing> {
    guitar_voicings(
        &degrees.triad,
        degrees.extension.as_ref(),
        tonic.scale_degree(&degrees.degree, &degrees.accidental),
        degrees
            .bass
            .as_ref()
            .map(|bass| tonic.scale_degree(&bass.degree, &bass.accidental)),
    )
}

fn with_voicing(element: ColumnElement, voicing: GuitarVoicing) -> ColumnElement {
    match element.content {
        ElementContent::ChordSymbol {
            text,
            degrees,
            dotted,
            double_dotted,
            guitar_voicing: _,
        } => ColumnElement {
            content: ElementContent::ChordSymbol {
                text,
                degrees,
                dotted,
                double_dotted,
                guitar_voicing: Some(voicing),
            },
            ..element
        },
        _ => element,
    }
}

#[cfg(test)]
#[path = "tests_guitar_chords.rs"]
mod tests;
