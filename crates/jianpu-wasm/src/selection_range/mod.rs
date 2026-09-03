use wasm_bindgen::prelude::*;

mod helpers;
mod lyric_label;
mod lyric_lyric;
mod measure;
mod mixed;
mod note_lyric;
mod note_note;
mod part_label;
mod types;

use crate::types::{LyricSpanOut, NoteSpanOut};
use types::{ClickableElementId, ResolveSelectionRangeResponse};

/// Resolves the click-and-click range between two clickable elements — the
/// ID-based replacement for pixel-marquee range resolution (see
/// `PLAN-clickable-element-id-selection.md`). Pure grouping over the
/// already-fetched `note_spans`/`lyric_spans`, called directly on the main
/// thread like `group_note_selection`. `raw_anchor`/`raw_current` each
/// decode to a `ClickableElementId`. `Err` covers a malformed `JsValue` and
/// any `(anchor, current)` combination not yet ID-resolvable (caller must
/// fall back to the pixel-marquee path) — `Measure ↔ Measure`, `Note ↔ Note`
/// (same part and cross-part), `Note ↔ Lyric` (cross-row, either ordering),
/// `Lyric ↔ Lyric` (every scope — same part+verse, same part cross-verse,
/// and cross-part), `PartLabel ↔ PartLabel` (any system), `LyricLabel ↔
/// LyricLabel` (same verse, any system), and every label-mixed pair
/// (`Note ↔ PartLabel`, `Lyric ↔ LyricLabel`, `Note ↔ LyricLabel`,
/// `Lyric ↔ PartLabel`, `PartLabel ↔ LyricLabel`) are implemented so far —
/// every `(anchor, current)` discriminant pair this plan ever scoped now
/// resolves without `Err`, other than a malformed `JsValue` or a
/// same-system-but-different-verse `LyricLabel ↔ LyricLabel` pair.
#[wasm_bindgen]
pub fn resolve_selection_range(
    raw_note_spans: JsValue,
    raw_lyric_spans: JsValue,
    raw_anchor: JsValue,
    raw_current: JsValue,
) -> ResolveSelectionRangeResponse {
    let note_spans: Vec<NoteSpanOut> =
        serde_wasm_bindgen::from_value(raw_note_spans).unwrap_or_default();
    let lyric_spans: Vec<LyricSpanOut> =
        serde_wasm_bindgen::from_value(raw_lyric_spans).unwrap_or_default();
    let (Ok(anchor), Ok(current)) = (
        serde_wasm_bindgen::from_value::<ClickableElementId>(raw_anchor),
        serde_wasm_bindgen::from_value::<ClickableElementId>(raw_current),
    ) else {
        return ResolveSelectionRangeResponse::Err;
    };
    resolve_selection_range_response(&note_spans, &lyric_spans, &anchor, &current)
}

/// Resolves the click-and-click range between two clickable elements into
/// the note/lyric cells it covers — the ID-based replacement for
/// pixel-marquee range resolution (see
/// `PLAN-clickable-element-id-selection.md`). Pure grouping over
/// already-fetched `note_spans`/`lyric_spans`, exactly like
/// `group_note_selection`'s doc comment describes for why callers must call
/// this on the main thread with cached spans, not re-parse `source`.
///
/// `Measure ↔ Measure`, `Note ↔ Note` (same part and cross-part), `Note ↔
/// Lyric` (cross-row, either ordering), `Lyric ↔ Lyric` (every scope —
/// same-part-and-verse, same-part cross-verse, and cross-part),
/// `PartLabel ↔ PartLabel` (any system), and `LyricLabel ↔ LyricLabel` (same
/// verse, any system) are implemented so far — `Measure ↔ Measure` was Phase
/// 1's simplest, lowest-risk row, chosen to validate the wasm plumbing
/// end-to-end before 'note'/'lyric'/label modes followed; cross-part `Note ↔
/// Note` and `Note ↔ Lyric` are Phase 2's first two rows, added once Phase 1
/// proved the plumbing out (see `PLAN-clickable-element-id-selection.md`).
/// `PartLabel ↔ PartLabel` and `LyricLabel ↔ LyricLabel` both deliberately
/// drop any same-system restriction — range resolution has no concept of
/// "system" to begin with (see each topic module's own doc comments), so a
/// cross-system pair of either kind resolves exactly like a same-system one,
/// no Cmd/Ctrl modifier required (the Cmd/Ctrl-gated `'part-label-system'`
/// mode still exists as a separate, coarser "every part in every system
/// swept" tool — see `previewSelectionResolver.ts`). `Lyric ↔ Lyric`'s
/// cross-verse and cross-part arms (see `lyric_lyric`'s own doc comments)
/// closed out that row's last gap. A `LyricLabel ↔ LyricLabel` pair across
/// different verses is still deliberately left unresolved — falls through to
/// `Err` like every other not-yet-ported combination, so the caller falls
/// back to the existing pixel-marquee path.
///
/// Every label-mixed pair (`Note ↔ PartLabel`, `Lyric ↔ LyricLabel`,
/// `Note ↔ LyricLabel`, `Lyric ↔ PartLabel`, `PartLabel ↔ LyricLabel`) is
/// implemented too, closing out every combination this plan's Phase 1/Phase
/// 2 tables ever scoped — see `PLAN-clickable-element-id-selection.md`'s
/// Status section for the full per-pair design writeup and each topic
/// module's own doc comments below for each rule.
///
/// Implementation is split by topic across sibling modules — `measure`,
/// `note_note`, `note_lyric`, `lyric_lyric`, `part_label`, `lyric_label`,
/// and `mixed` — each owning a disjoint slice of the `(anchor, current)`
/// variant space via its own `pub(crate) fn resolve(..) -> Option<..>`.
/// This function just tries each in turn, falling through topic by topic
/// until one returns `Some`, ending in `ResolveSelectionRangeResponse::Err`
/// for any pair no topic module claims.
pub(crate) fn resolve_selection_range_response(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> ResolveSelectionRangeResponse {
    measure::resolve(note_spans, lyric_spans, anchor, current)
        .or_else(|| note_note::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| note_lyric::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| lyric_lyric::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| part_label::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| lyric_label::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| mixed::resolve(note_spans, lyric_spans, anchor, current))
        .unwrap_or(ResolveSelectionRangeResponse::Err)
}

#[cfg(test)]
mod tests;
