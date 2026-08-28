use crate::compositor::types::AbsoluteElement;
use crate::grid_layout::types::GridPage;

use super::highlights::{
    resolve_bar_number_click_target, resolve_lyric_click_target, resolve_lyric_label_click_target,
    resolve_measure_click_target, resolve_note_click_target, resolve_part_label_click_target,
    resolve_playback_cursor_target,
};

/// Resolves every click/drag hit target on a page — measure, playback
/// cursor, note, part-label, lyric, and lyric-label — appended in that order
/// so later ones stay topmost for `elementFromPoint` hit-testing (e.g. a
/// note click target over its enclosing measure's, and a lyric syllable's
/// own target over the note click target that geometrically covers its
/// row).
pub(super) fn resolve_click_target_elements(
    page: &GridPage,
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
    measure_number_font_size: f32,
) -> Vec<AbsoluteElement> {
    let mut elements: Vec<AbsoluteElement> = page
        .measure_click_targets
        .iter()
        .filter_map(|t| {
            resolve_measure_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
        })
        .collect();

    elements.extend(page.playback_cursor_targets.iter().filter_map(|t| {
        resolve_playback_cursor_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements.extend(page.playback_cursor_targets.iter().filter_map(|t| {
        resolve_note_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements.extend(page.part_label_click_targets.iter().filter_map(|t| {
        resolve_part_label_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements.extend(page.lyric_click_targets.iter().filter_map(|t| {
        resolve_lyric_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements.extend(page.lyric_label_click_targets.iter().filter_map(|t| {
        resolve_lyric_label_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements.extend(page.bar_number_click_targets.iter().filter_map(|t| {
        resolve_bar_number_click_target(
            t,
            &page.rows,
            row_tops,
            usable_width,
            part_label_width_pt,
            measure_number_font_size,
        )
    }));

    elements
}
