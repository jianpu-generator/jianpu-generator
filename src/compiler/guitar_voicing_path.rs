//! Picks one guitar voicing per chord so the whole progression needs the
//! least hand movement: a shortest path through a layered graph where each
//! chord (in written order) is a layer, each of its voicings a node, and
//! each edge the movement between two consecutive voicings. A Viterbi-style
//! dynamic program finds the global optimum in O(n · k²). Each node also
//! costs a little for how far down chords-db's list it is, so a progression
//! of open chords stays open rather than moving up the neck to save a few
//! string changes.

use itertools::Itertools;

use crate::pitch_description::GuitarVoicing;

/// Cost of one string changing fret, against the hand position's shift
/// measured in hundredths of a fret: moving the hand one fret along the neck
/// costs as much as four strings changing fret.
const CHANGED_STRING_COST: u32 = 25;

/// Cost of picking a voicing one place further down chords-db's list (most
/// common first) than its chord's first: half a fret of hand movement.
const LESS_COMMON_VOICING_COST: u32 = 50;

/// For each layer, the index of its voicing on the least-movement path.
/// Every layer must be non-empty, ordered most common first. Ties go to the
/// voicing earliest in its layer, so the result is stable.
pub(crate) fn least_movement_path(layers: &[Vec<&GuitarVoicing>]) -> Vec<usize> {
    let Some(first) = layers.first() else {
        return vec![];
    };
    // `costs[j]`: the cheapest path ending on the current layer's voicing `j`;
    // `back_pointers[i][j]`: that path's voicing on layer `i`.
    let (costs, back_pointers) = layers.iter().tuple_windows().fold(
        (
            (0..first.len()).map(commonness_cost).collect_vec(),
            Vec::<Vec<usize>>::new(),
        ),
        |(costs, mut back_pointers), (previous, layer)| {
            let (next_costs, pointers): (Vec<u32>, Vec<usize>) = layer
                .iter()
                .enumerate()
                .map(|(index, voicing)| {
                    let (from, cost) = previous
                        .iter()
                        .zip(&costs)
                        .map(|(from, cost)| cost + movement_cost(from, voicing))
                        .enumerate()
                        .min_by_key(|&(_, cost)| cost)
                        .unwrap_or_default();
                    (cost + commonness_cost(index), from)
                })
                .unzip();
            back_pointers.push(pointers);
            (next_costs, back_pointers)
        },
    );
    let last = costs.iter().position_min().unwrap_or_default();
    let mut path = back_pointers
        .iter()
        .rev()
        .scan(last, |current, pointers| {
            *current = pointers.get(*current).copied().unwrap_or_default();
            Some(*current)
        })
        .collect_vec();
    path.reverse();
    path.push(last);
    path
}

fn commonness_cost(index_in_layer: usize) -> u32 {
    LESS_COMMON_VOICING_COST * index_in_layer as u32
}

/// How far the hand moves from voicing `from` to voicing `to`: the shift
/// of its position along the neck, plus a little for every string whose
/// fret changes. The same voicing costs 0.
pub(crate) fn movement_cost(from: &GuitarVoicing, to: &GuitarVoicing) -> u32 {
    let changed_strings = from
        .absolute_frets()
        .iter()
        .zip(to.absolute_frets())
        .filter(|(from, to)| **from != *to)
        .count() as u32;
    hand_position(from).abs_diff(hand_position(to)) + CHANGED_STRING_COST * changed_strings
}

/// The hand's position along the neck in hundredths of a fret: the mean of
/// the fretted strings' absolute frets, or 0 when every string is open or
/// muted.
fn hand_position(voicing: &GuitarVoicing) -> u32 {
    let fretted = voicing
        .absolute_frets()
        .into_iter()
        .flatten()
        .filter(|&fret| fret > 0)
        .map(u32::from)
        .collect_vec();
    match fretted.len() as u32 {
        0 => 0,
        count => fretted.iter().sum::<u32>() * 100 / count,
    }
}

#[cfg(test)]
#[path = "tests_guitar_voicing_path.rs"]
mod tests;
