//! One page's slice of every click/highlight target list, split out of
//! `layout.rs` to keep that file under the max line-count lint.

use crate::grid_layout::click_targets::{targets_on_page, HighlightAndClickInfos};
use crate::grid_layout::highlight::measure_highlights_on_page;
use crate::grid_layout::types::{
    BarLineClickTarget, BarNumberClickTarget, LyricClickTarget, LyricLabelClickTarget,
    MeasureClickTarget, MeasureHighlight, PartLabelClickTarget, PlaybackCursorTarget,
};

/// One page's slice of every `HighlightAndClickInfos` list, filtered by
/// `targets_on_page`/`measure_highlights_on_page` — split out of `layout()`
/// to keep it under the max function-length lint.
pub(super) struct PageHighlightsAndTargets {
    pub(super) measure_highlights: Vec<MeasureHighlight>,
    pub(super) error_highlights: Vec<MeasureHighlight>,
    pub(super) measure_click_targets: Vec<MeasureClickTarget>,
    pub(super) playback_cursor_targets: Vec<PlaybackCursorTarget>,
    pub(super) part_label_click_targets: Vec<PartLabelClickTarget>,
    pub(super) lyric_click_targets: Vec<LyricClickTarget>,
    pub(super) lyric_label_click_targets: Vec<LyricLabelClickTarget>,
    pub(super) bar_number_click_targets: Vec<BarNumberClickTarget>,
    pub(super) bar_line_click_targets: Vec<BarLineClickTarget>,
}

impl PageHighlightsAndTargets {
    pub(super) fn for_page(infos: &HighlightAndClickInfos, page_idx: usize) -> Self {
        Self {
            measure_highlights: measure_highlights_on_page(&infos.highlight_infos, page_idx),
            error_highlights: measure_highlights_on_page(&infos.error_highlight_infos, page_idx),
            measure_click_targets: targets_on_page(&infos.all_click_target_infos, page_idx),
            playback_cursor_targets: targets_on_page(
                &infos.all_playback_cursor_target_infos,
                page_idx,
            ),
            part_label_click_targets: targets_on_page(
                &infos.all_part_label_click_target_infos,
                page_idx,
            ),
            lyric_click_targets: targets_on_page(&infos.all_lyric_click_target_infos, page_idx),
            lyric_label_click_targets: targets_on_page(
                &infos.all_lyric_label_click_target_infos,
                page_idx,
            ),
            bar_number_click_targets: targets_on_page(
                &infos.all_bar_number_click_target_infos,
                page_idx,
            ),
            bar_line_click_targets: targets_on_page(
                &infos.all_bar_line_click_target_infos,
                page_idx,
            ),
        }
    }
}
