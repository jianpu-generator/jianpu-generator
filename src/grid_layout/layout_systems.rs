use super::{
    block_column_width, chord_part_sub_row_heights, is_chord_only_row, is_lyric_row,
    lyric_row_height, lyric_row_verse, note_part_height_pt,
};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::types::GridElement;
use crate::render_config::RenderConfig;
use std::collections::{HashMap, HashSet};

/// The fixed-width column reserved at the start of every system row for the
/// part label (see [`crate::grid_layout::types::GridRow::column_geometry`]).
/// A single column, not a subdivided region — nothing else places elements
/// at fractional positions within it.
pub(crate) const LABEL_COLS: u32 = 1;

/// First musical column, leaving a dedicated column at `LABEL_COLS` for the
/// leading barline so it gets the same breathing room as inter-measure barlines.
pub(crate) const MUSIC_START_COL: u32 = LABEL_COLS + 1;

/// Total height in points for all musical sub-rows in a system
/// (sum over all non-lyric part rows). `tuplet_part_indices` holds the
/// consolidated part indices (row position within `block.rows`) that have a
/// tuplet bracket in this system, so parts without one don't count the
/// `tuplet_bracket` sub-row's height (see `note_part_height_pt`).
/// Consolidated part indices (row position within `system`'s first block's
/// `rows`) that have a tuplet bracket in this system, per `tuplet_bracket_map`
/// (keyed by `(abs_sys, source_part_index)`, see `resolve_tuplet_spans`).
/// Shared by every pass that needs to know whether a part's `tuplet_bracket`
/// sub-row is reserved in this particular system — layout height math
/// (`system_musical_height_pt`), row expansion (`expand_note_part`), and the
/// highlight/click/playback-cursor row-counting passes (`grid_layout::highlight`,
/// `grid_layout::playback_cursor`) all must agree on this or their row indices
/// drift apart.
pub(crate) fn system_tuplet_part_indices(
    system: &[MeasureBlock],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    abs_sys: usize,
) -> HashSet<usize> {
    let Some(first) = system.first() else {
        return HashSet::new();
    };
    first
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| tuplet_bracket_map.contains_key(&(abs_sys, row.source_part_index)))
        .map(|(idx, _)| idx)
        .collect()
}

pub(crate) fn system_musical_height_pt(
    block: &MeasureBlock,
    base: f32,
    tuplet_part_indices: &HashSet<usize>,
) -> f32 {
    block
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| !is_lyric_row(r))
        .map(|(idx, r)| {
            if is_chord_only_row(r) {
                chord_part_sub_row_heights(base).iter().sum::<f32>()
            } else {
                note_part_height_pt(base, tuplet_part_indices.contains(&idx))
            }
        })
        .sum()
}

/// Total height in points for lyric rows in a system.
pub(crate) fn system_lyric_height_pt(block: &MeasureBlock, base: f32) -> f32 {
    block.rows.iter().filter(|r| super::has_lyrics(r)).count() as f32 * lyric_row_height(base)
}

/// Internal sort key used by [`union_row_order`] to order a system's union of
/// rows: by `[parts]` declaration order first, then by verse (a part's own
/// note/non-lyric row before its verse rows, in verse order). Carries a clone
/// of the first-seen block's own row as `template`, so a later block missing
/// this `RowId` has something to pad from without re-searching `chunk` (see
/// [`pad_chunk_to_union`]).
struct RowSlot {
    source_part_index: usize,
    verse: Option<usize>,
    template: MeasureRow,
}

/// Every distinct [`RowId`] across `chunk`'s blocks, as its first-seen row,
/// in `[parts]` declaration order (then verse order within a part). First-seen
/// block wins when the same `RowId` appears in more than one block — they're
/// expected to be equivalent for ordering/template purposes.
fn union_row_order(chunk: &[MeasureBlock]) -> Vec<MeasureRow> {
    let mut slots: HashMap<RowId, RowSlot> = HashMap::new();

    for block in chunk {
        for row in &block.rows {
            slots.entry(row.id.clone()).or_insert_with(|| RowSlot {
                source_part_index: row.source_part_index,
                verse: lyric_row_verse(row),
                template: row.clone(),
            });
        }
    }

    let mut ordered: Vec<RowSlot> = slots.into_values().collect();
    ordered.sort_by_key(|slot| {
        (
            slot.source_part_index,
            slot.verse.map(|v| v + 1).unwrap_or(0),
        )
    });
    ordered.into_iter().map(|slot| slot.template).collect()
}

/// Builds a synthetic row standing in for `template_row`'s `RowId` in a block
/// that's missing it because that part is genuinely silent here: a
/// full-measure rest (or blank verse, for a lyric row), sized to `block`'s own
/// column width. Every element's `ColumnElement::note_id` is `None` so the
/// padded cell produces no playback-cursor target or note click target (see
/// `group_elements_by_note_id` in `playback_cursor.rs`). A padded lyric row's
/// `Lyric` content carries empty `text`, which `compute_all_lyric_click_targets`
/// in `click_targets_lyric.rs` checks for and skips — `ColumnElement::note_id`
/// is always `None` for `Lyric` content, real or padded, so it can't be used
/// as the "is this padding" signal there.
///
/// Only called once [`pad_chunk_to_union`] has ruled out the other reason a
/// `RowId` can be missing from a block — that `consolidator::consolidate_rows`
/// merged its (identical) content into another row of this same block (see
/// `MeasureRow::absorbed_rows`) — since that case re-renders the part's own
/// original row instead of padding at all.
fn make_padding_row(template_row: &MeasureRow, block: &MeasureBlock) -> MeasureRow {
    let width = block_column_width(block);
    let bar_line_column = width.saturating_sub(1);

    let elements = if is_lyric_row(template_row) {
        vec![
            ColumnElement {
                column: 0,
                content: ElementContent::Lyric {
                    text: String::new(),
                    verse: lyric_row_verse(template_row).unwrap_or(0),
                    note_id: 0,
                },
                note_id: None,
            },
            ColumnElement {
                column: bar_line_column,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ]
    } else {
        vec![
            ColumnElement {
                column: 0,
                content: ElementContent::Rest {
                    dotted: false,
                    double_dotted: false,
                },
                note_id: None,
            },
            ColumnElement {
                column: bar_line_column,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ]
    };

    MeasureRow {
        id: template_row.id.clone(),
        label: template_row.label.clone(),
        elements,
        source_part_index: template_row.source_part_index,
        group_provenance: None,
        absorbed_rows: Vec::new(),
    }
}

/// The displayed label for `row`, folding its own (part-level) identity
/// together with whichever of its `absorbed_rows` are still genuinely,
/// permanently merged into it — i.e. every `RowId` this system needs, per
/// `union_ids`, gets its own separate row *somewhere*, so an absorbed row
/// whose id is in `union_ids` only matched `row`'s content by coincidence in
/// this one measure (see [`pad_chunk_to_union`]) and must not affect `row`'s
/// label here. Mirrors the group-abbreviation rule `consolidator` used to
/// apply eagerly per-measure: the running label becomes the shared group
/// abbreviation while every folded-in row traces to the same `[GroupAbbrev]`
/// broadcast, falling back to a space-joined concatenation (and clearing
/// provenance, so a later fold in the same pass can't mistake a
/// partially-diverged cluster for a fully in-group one) the moment one
/// doesn't.
fn resolve_label(row: &MeasureRow, union_ids: &HashSet<RowId>) -> (String, Option<String>) {
    row.absorbed_rows
        .iter()
        .filter(|absorbed| !union_ids.contains(&absorbed.id))
        .fold(
            (row.label.clone(), row.group_provenance.clone()),
            |(label, provenance), absorbed| match (&provenance, &absorbed.group_provenance) {
                (Some(p), Some(q)) if p == q => (p.clone(), Some(p.clone())),
                _ => (format!("{label} {}", absorbed.label), None),
            },
        )
}

/// Rebuilds every block in `chunk` so its `rows` match `union` exactly, in
/// order. For each `union` template's `RowId`, in priority order: a block's
/// own matching row wins if present; otherwise, if some other row in this
/// block absorbed it (recorded in that row's `MeasureRow::absorbed_rows` by
/// `consolidator::consolidate_rows`), that original row is re-rendered on its
/// own — it's genuinely this part's own real content (real `note_id`s, real
/// click/playback targets), just also drawn merged into another row elsewhere
/// in this same system; only when neither holds is the `RowId` genuinely
/// absent here, padded via [`make_padding_row`]. Every resolved row's
/// `label`/`group_provenance` is then (re)computed by [`resolve_label`], now
/// that `union`'s full `RowId` membership — unknown to `consolidator`, which
/// only ever sees one measure at a time — is available.
fn pad_chunk_to_union(chunk: &[MeasureBlock], union: &[MeasureRow]) -> Vec<MeasureBlock> {
    let union_ids: HashSet<RowId> = union.iter().map(|row| row.id.clone()).collect();
    chunk
        .iter()
        .map(|block| {
            let rows = union
                .iter()
                .map(|template| {
                    let mut row = block
                        .rows
                        .iter()
                        .find(|r| r.id == template.id)
                        .cloned()
                        .or_else(|| {
                            block.rows.iter().find_map(|r| {
                                r.absorbed_rows
                                    .iter()
                                    .find(|absorbed| absorbed.id == template.id)
                                    .cloned()
                            })
                        })
                        .unwrap_or_else(|| make_padding_row(template, block));
                    let (label, group_provenance) = resolve_label(&row, &union_ids);
                    row.label = label;
                    row.group_provenance = group_provenance;
                    row
                })
                .collect();

            MeasureBlock {
                rows,
                ..block.clone()
            }
        })
        .collect()
}

/// Break `blocks` into systems. Each system is a `Vec<MeasureBlock>`, packed
/// purely by count (chunks of up to `config.max_measures_per_system`
/// measures) — differing parts or lyric verse counts across measures never
/// force an early break. Each system's rows are the union of every row
/// across its measures (see [`union_row_order`]); measures missing a row
/// their system has get it padded in (see [`pad_chunk_to_union`]).
pub(crate) fn pack_into_systems(
    blocks: &[MeasureBlock],
    config: &RenderConfig,
) -> Vec<Vec<MeasureBlock>> {
    fn finish_chunk(chunk: &[MeasureBlock]) -> Vec<MeasureBlock> {
        let union = union_row_order(chunk);
        pad_chunk_to_union(chunk, &union)
    }

    let mut systems: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut current: Vec<MeasureBlock> = Vec::new();

    for block in blocks {
        let needs_new = if let Some(first) = current.first() {
            current.len() as u32 >= config.max_measures_per_system
                || block.merge_duplicate_measures_across_parts
                    != first.merge_duplicate_measures_across_parts
        } else {
            false
        };

        if needs_new && !current.is_empty() {
            systems.push(finish_chunk(&std::mem::take(&mut current)));
        }

        current.push(block.clone());
    }

    if !current.is_empty() {
        systems.push(finish_chunk(&current));
    }

    systems
}

pub(crate) fn compute_bar_height(
    first: &MeasureBlock,
    base: f32,
    tuplet_part_indices: &HashSet<usize>,
) -> f32 {
    system_musical_height_pt(first, base, tuplet_part_indices) + system_lyric_height_pt(first, base)
}

pub(crate) fn system_has_any_decoration(system: &[MeasureBlock]) -> bool {
    system.iter().any(|block| !block.decorations.is_empty())
}
