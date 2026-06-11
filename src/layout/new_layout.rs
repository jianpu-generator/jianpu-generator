use crate::compiler::types::{Decoration, ElementContent, MeasureBlock, RowId};
use crate::layout::new_types::{Footer, Header, Page, RowLabel, System};
use crate::render_config::RenderConfig;

use super::PAGE_MARGIN;

/// Compute the column width of a MeasureBlock by finding the BarLine element.
fn block_column_width(block: &MeasureBlock) -> u32 {
    block
        .rows
        .first()
        .and_then(|row| {
            row.elements
                .iter()
                .find(|e| e.content == ElementContent::BarLine)
        })
        .map(|e| e.column + 1)
        .unwrap_or(1)
}

/// Infer the row height (in row-units) for a single MeasureRow based on its elements.
fn infer_row_height(row: &crate::compiler::types::MeasureRow) -> u32 {
    let has_lyric = row
        .elements
        .iter()
        .any(|e| matches!(e.content, ElementContent::Lyric(_)));
    if has_lyric {
        return 4;
    }

    // Check if all note/rest-type elements are ChordSymbol (no NoteHead, no Rest)
    let has_note_head = row
        .elements
        .iter()
        .any(|e| matches!(e.content, ElementContent::NoteHead { .. }));
    let has_rest = row
        .elements
        .iter()
        .any(|e| matches!(e.content, ElementContent::Rest { .. }));

    if !has_note_head && !has_rest {
        // Only ChordSymbol note-type elements (or no note elements) → chord row
        let has_chord = row
            .elements
            .iter()
            .any(|e| matches!(e.content, ElementContent::ChordSymbol(_)));
        if has_chord {
            return 2;
        }
    }

    3
}

/// Compute the total row-units for a system based on the first MeasureBlock's rows.
fn system_row_height(block: &MeasureBlock) -> u32 {
    block.rows.iter().map(infer_row_height).sum()
}

/// Extract the RowId set from a block (used to check system compatibility).
fn row_ids(block: &MeasureBlock) -> Vec<&RowId> {
    block.rows.iter().map(|r| &r.id).collect()
}

/// Check whether the first block of a system has any directive decoration (Bpm or TimeSignature).
fn has_directive(block: &MeasureBlock) -> bool {
    block
        .decorations
        .iter()
        .any(|d| matches!(d, Decoration::Bpm(_) | Decoration::TimeSignature { .. }))
}

/// Break `blocks` into `Page`s containing `System`s.
///
/// This is a pure geometry function: it knows nothing about musical content beyond
/// column widths and row heights.
pub fn layout_new(
    blocks: &[MeasureBlock],
    config: &RenderConfig,
    header: &Header,
    page_width_pt: f32,
    page_height_pt: f32,
) -> Vec<Page> {
    // --- Phase 1: break blocks into systems ---
    let mut systems: Vec<System> = Vec::new();
    let mut current_measures: Vec<MeasureBlock> = Vec::new();
    let mut current_columns: u32 = 0;

    for block in blocks {
        let col_width = block_column_width(block);

        // Determine if we need a new system
        let needs_new_system = if let Some(first_block) = current_measures.first() {
            let columns_would_exceed = current_columns + col_width > config.max_columns;
            let row_ids_differ = row_ids(block) != row_ids(first_block);
            columns_would_exceed || row_ids_differ
        } else {
            false
        };

        if needs_new_system {
            // Flush current system
            if let Some(first) = current_measures.first() {
                let row_labels = first
                    .rows
                    .iter()
                    .map(|r| RowLabel {
                        id: r.id.clone(),
                        text: r.label.clone(),
                    })
                    .collect();
                systems.push(System {
                    row_labels,
                    measures: std::mem::take(&mut current_measures),
                });
                current_columns = 0;
            }
        }

        current_columns += col_width;
        current_measures.push(block.clone());
    }

    // Flush any remaining measures into a final system
    if let Some(first) = current_measures.first() {
        let row_labels = first
            .rows
            .iter()
            .map(|r| RowLabel {
                id: r.id.clone(),
                text: r.label.clone(),
            })
            .collect();
        systems.push(System {
            row_labels,
            measures: current_measures,
        });
    }

    // --- Phase 2: break systems into pages ---
    let has_subtitle = header.subtitle.is_some();
    let header_rows: u32 = if has_subtitle { 3 } else { 2 };
    let footer_rows: u32 = 1;
    let row_height_pt = config.row_height as f32;
    let usable_height = page_height_pt - 2.0 * PAGE_MARGIN;
    let max_rows_per_page =
        (usable_height / row_height_pt).floor() as u32 - header_rows - footer_rows;

    let mut pages: Vec<Page> = Vec::new();
    let mut current_page_systems: Vec<System> = Vec::new();
    let mut used_rows: u32 = 0;

    for system in systems {
        // Compute this system's row cost
        let base_height = system.measures.first().map(system_row_height).unwrap_or(0);
        let directive_extra: u32 = system
            .measures
            .first()
            .map(|b| u32::from(has_directive(b)))
            .unwrap_or(0);
        let system_rows = base_height + directive_extra;

        // If this system doesn't fit on the current page, flush the page
        if !current_page_systems.is_empty() && used_rows + system_rows > max_rows_per_page {
            pages.push(Page {
                header: header.to_owned(),
                footer: Footer { page: 0, total: 0 }, // filled in below
                systems: std::mem::take(&mut current_page_systems),
                page_width_pt,
                page_height_pt,
            });
            used_rows = 0;
        }

        used_rows += system_rows;
        current_page_systems.push(system);
    }

    // Always emit at least one page (even if empty)
    pages.push(Page {
        header: header.to_owned(),
        footer: Footer { page: 0, total: 0 },
        systems: current_page_systems,
        page_width_pt,
        page_height_pt,
    });

    // Fill in footer page numbers
    let total = pages.len() as u32;
    for (i, page) in pages.iter_mut().enumerate() {
        page.footer.page = i as u32 + 1;
        page.footer.total = total;
    }

    pages
}
