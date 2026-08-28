use super::{
    block_column_width, chord_part_sub_row_heights, is_chord_only_row, is_lyric_row,
    lyric_row_height, lyric_row_verse, note_part_height_pt, LyricSizing,
};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::types::GridElement;
use crate::render_config::RenderConfig;
use itertools::Itertools;
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

/// Total height in points for lyric rows in a system. Sizes every row for
/// the CJK lyric font size (always >= the Latin one, see
/// `RenderConfig::lyric_cjk_font_size`) rather than scanning each row's
/// actual syllables for CJK text — a deliberately conservative estimate, so
/// a page never ends up packed tighter than what `expand_lyric_part` (which
/// does look at each row's real syllables) goes on to render.
pub(crate) fn system_lyric_height_pt(
    block: &MeasureBlock,
    base: f32,
    lyric_sizing: LyricSizing,
) -> f32 {
    block.rows.iter().filter(|r| super::has_lyrics(r)).count() as f32
        * lyric_row_height(
            base,
            lyric_sizing.font_sizes.cjk,
            lyric_sizing.click_target_padding_pt,
        )
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
                    implicit_fill: true,
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
        absorbed_rows: Vec::new(),
    }
}

/// The displayed label for `row`, combining its own (part-level) identity
/// with whichever of its `absorbed_rows` are still genuinely, permanently
/// merged into it — i.e. every `RowId` this system needs, per `union_ids`,
/// gets its own separate row *somewhere*, so an absorbed row whose id is in
/// `union_ids` only matched `row`'s content by coincidence in this one
/// measure (see [`pad_chunk_to_union`]) and must not affect `row`'s label
/// here.
fn resolve_label(row: &MeasureRow, union_ids: &HashSet<RowId>) -> String {
    std::iter::once(row.label.as_str())
        .chain(
            row.absorbed_rows
                .iter()
                .filter(|absorbed| !union_ids.contains(&absorbed.id))
                .map(|absorbed| absorbed.label.as_str()),
        )
        .join(" ")
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
/// `label` is then (re)computed by [`resolve_label`], now that `union`'s
/// full `RowId` membership — unknown to `consolidator`, which only ever sees
/// one measure at a time — is available.
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
                    row.label = resolve_label(&row, &union_ids);
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

/// True when `row`'s own content is nothing but rest — a plain `Rest` or a
/// collapsed `MultiMeasureRest` (bar lines don't count either way). Used to
/// decide whether a lone row's part label is worth showing at all (see
/// [`clear_label_if_lone_resting_row`]).
fn row_is_entirely_rest(row: &MeasureRow) -> bool {
    row.elements.iter().all(|el| {
        matches!(
            el.content,
            ElementContent::Rest { .. }
                | ElementContent::MultiMeasureRest { .. }
                | ElementContent::BarLine
        )
    })
}

/// A part-abbreviation label only earns its keep by distinguishing one row
/// from another sharing the same system. When a system boils down to a
/// single row (`union_row_order` found only one distinct `RowId` across the
/// whole chunk) and that row is entirely rest — whether a single resting
/// measure or a run collapsed into one `MultiMeasureRest` — there's nothing
/// else in the system for the label to distinguish it from, so it's cleared
/// on every block's copy of that row. A lone row that actually plays
/// something keeps its label, and a system with more than one row is left
/// untouched even if every row in it happens to be resting.
fn clear_label_if_lone_resting_row(padded: &mut [MeasureBlock]) {
    let Some(first_block) = padded.first() else {
        return;
    };
    let [only_row] = first_block.rows.as_slice() else {
        return;
    };
    if is_lyric_row(only_row) {
        return;
    }
    if !padded
        .iter()
        .all(|block| block.rows.first().is_some_and(row_is_entirely_rest))
    {
        return;
    }
    for block in padded {
        if let Some(row) = block.rows.first_mut() {
            row.label.clear();
        }
    }
}

/// Break `blocks` into systems. Each system is a `Vec<MeasureBlock>`, packed
/// purely by count (chunks of up to `config.max_measures_per_system`
/// measures) — differing parts or lyric verse counts across measures never
/// force an early break. Each system's rows are the union of every row
/// across its measures (see [`union_row_order`]); measures missing a row
/// their system has get it padded in (see [`pad_chunk_to_union`]).
///
/// A block with `system_break` set (from a `break` directive on its measure)
/// always starts a new system, even if the current one has room left under
/// `max_measures_per_system` — a no-op if the block would already be first
/// in its system.
pub(crate) fn pack_into_systems(
    blocks: &[MeasureBlock],
    config: &RenderConfig,
) -> Vec<Vec<MeasureBlock>> {
    fn finish_chunk(chunk: &[MeasureBlock]) -> Vec<MeasureBlock> {
        let union = union_row_order(chunk);
        let mut padded = pad_chunk_to_union(chunk, &union);
        clear_label_if_lone_resting_row(&mut padded);
        padded
    }

    let mut systems: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut current: Vec<MeasureBlock> = Vec::new();

    for block in blocks {
        let needs_new = if let Some(first) = current.first() {
            current.len() as u32 >= config.max_measures_per_system
                || block.merge_duplicate_measures_across_parts
                    != first.merge_duplicate_measures_across_parts
                || block.system_break
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
    lyric_sizing: LyricSizing,
) -> f32 {
    system_musical_height_pt(first, base, tuplet_part_indices)
        + system_lyric_height_pt(first, base, lyric_sizing)
}

pub(crate) fn system_has_any_decoration(system: &[MeasureBlock]) -> bool {
    system.iter().any(|block| !block.decorations.is_empty())
}
