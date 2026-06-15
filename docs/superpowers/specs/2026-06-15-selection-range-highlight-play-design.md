# Selection-Range Highlight and Playback

**Date:** 2026-06-15

## Goal

Extend the current caret-based measure highlight and playback so that when the user selects a range of text in the editor, all measures that overlap the selection are highlighted in the SVG preview and can be played together as a single audio clip.

## Behaviour

- **Caret only (no selection):** identical to today — the single measure under the caret is highlighted and playable.
- **Selection spanning multiple measures:** every measure that overlaps the selection (greedy — partial overlap counts) is highlighted and playable as a consecutive audio clip.
- **Selection entirely outside measures** (e.g. inside `[metadata]`): no highlight, play button disabled.

The play button label reflects the range:
- Single measure → "Measure N"
- Multiple measures → "Measures N–M"

## Architecture

### 1. Selection tracking — `web/src/components/Editor.tsx`

Replace `onCursorByteOffsetChange?(offset: number)` with `onSelectionChange?(startOffset: number, endOffset: number)`.

Inside `notifyCursor`, use `ed.getSelection()` (returns a Monaco `Selection` with start/end positions) instead of `ed.getPosition()`. Convert both `getStartPosition()` and `getEndPosition()` to byte offsets and fire the new callback. For a caret (no selection), Monaco returns a selection where start == end, so single-measure behaviour is preserved automatically.

### 2. Measure range computation — `web/src/hooks/useJianpuWorker.ts`

Replace `notifyCursorOffset(offset)` with `notifySelection(startOffset, endOffset)`.

Add `measureSpansRef` kept in sync with `measureSpans` state, so the callback can read the latest spans without a stale closure. The callback debounces and then computes the measure range in pure TypeScript using `remeda`:

```ts
import { findIndex, findLastIndex } from 'remeda'

function measureRangeInSpan(
  spans: Array<{ start: number; end: number }>,
  selStart: number,
  selEnd: number,
): { start: number; end: number } | null {
  // For a caret (selStart == selEnd), treat as a half-open point [selStart, selStart+1)
  // so "start < effective && end > selStart" correctly matches the containing measure.
  const effective = selStart === selEnd ? selEnd + 1 : selEnd
  const overlaps = (span: { start: number; end: number }) =>
    span.start < effective && span.end > selStart
  const start = findIndex(spans, overlaps)
  const end = findLastIndex(spans, overlaps)
  return start === -1 ? null : { start, end }
}
```

No new WASM call is needed — `measureSpans` is already maintained in the hook from the existing `listMeasureSpans` worker request.

Replace `currentMeasureIndex: number | null` with `selectedMeasureRange: { start: number; end: number } | null`.

### 3. Rust highlight pipeline

**`src/grid_layout/types.rs`**

Change `GridPage.measure_highlight: Option<MeasureHighlight>` → `measure_highlights: Vec<MeasureHighlight>`. A page with no highlights carries an empty vec.

**`src/grid_layout/layout.rs`**

Change `highlighted_measure_index: Option<usize>` parameter → `highlighted_measure_range: Option<(usize, usize)>`. For each measure the layout places, if its global index falls within `[start, end]` inclusive, push a `MeasureHighlight` onto the current page's vec.

**`src/coordinate_resolver/resolve.rs`**

Iterate over `GridPage.measure_highlights` (vec instead of single option), prepend one `AbsoluteContent::MeasureHighlight` per entry using the same coordinate math as today.

**`src/renderer/` and `src/serializer/`**

No structural changes — these already handle `AbsoluteContent::MeasureHighlight` → `SvgKind::Rect`. Multiple measures on the same page emit multiple `<rect>` elements.

**`crates/jianpu-wasm/src/lib.rs`**

Replace `render_with_highlight(source, highlighted_measure_index, ...)` with `render_with_highlight_range(source, start_index, end_index, ...)`. The old export is removed.

### 4. Rust audio pipeline

**`src/midi.rs`**

Add `write_midi_for_measure_range(score, start_index, end_index)`:
- Validates `end_index < score.measures.len()`
- Accumulates BPM/key context from all measures before `start_index`
- Clones measures `start_index..=end_index`, patching only the first with the accumulated context
- Builds a `Score` with those measures and calls `write_midi`

**`src/lib.rs`**

Add `write_wav_for_measure_range_from_source(source, filename, start_index, end_index, enabled_tracks)` following the same pattern as `write_wav_for_measure_from_source`.

**`crates/jianpu-wasm/src/lib.rs`**

Replace `generate_wav_for_measure(source, measure_index, enabled_tracks)` with `generate_wav_for_measure_range(source, start_index, end_index, enabled_tracks)`. Single-measure playback is `start == end`.

### 5. Worker messages — `web/src/worker/jianpu.worker.ts`

Replace `renderWithHighlight` request (`highlightedMeasureIndex: number`) with `renderWithHighlightRange` (`startMeasureIndex: number; endMeasureIndex: number`).

Replace `generateMeasureAudio` request (`measureIndex: number`) with `generateMeasureRangeAudio` (`startMeasureIndex: number; endMeasureIndex: number`).

Update worker message handlers to call the new WASM exports.

### 6. App wiring — `web/src/App.tsx`

- Wire `onSelectionChange={notifySelection}` on `<Editor>`
- Pass `selectedMeasureRange` to `<PlayMeasureButton>` instead of `currentMeasureIndex`
- Update the debug status line: `measure {n+1}` → `measure {start+1}` when single, `measures {start+1}–{end+1}` when range

### 7. Play button — `web/src/components/PlayMeasureButton.tsx`

Accept `measureRange: { start: number; end: number } | null` instead of `measureNumber: number | null`.

Label logic:
- `null` → disabled, no label
- `start === end` → "Measure N"
- `start !== end` → "Measures N–M"

## File Map

| Action | File |
|--------|------|
| Modify | `web/src/components/Editor.tsx` — replace `onCursorByteOffsetChange` with `onSelectionChange` |
| Modify | `web/src/components/PlayMeasureButton.tsx` — accept range, update label |
| Modify | `web/src/hooks/useJianpuWorker.ts` — range state, `notifySelection`, `measureSpansRef` |
| Modify | `web/src/App.tsx` — wire new callback and props |
| Modify | `web/src/worker/jianpu.worker.ts` — new request/response types, new WASM calls |
| Install | `remeda` in `web/` |
| Modify | `src/grid_layout/types.rs` — `measure_highlights: Vec<MeasureHighlight>` |
| Modify | `src/grid_layout/layout.rs` — range param, populate vec |
| Modify | `src/coordinate_resolver/resolve.rs` — iterate highlights vec |
| Modify | `src/midi.rs` — add `write_midi_for_measure_range` |
| Modify | `src/lib.rs` — add `write_wav_for_measure_range_from_source` |
| Modify | `crates/jianpu-wasm/src/lib.rs` — replace WASM exports with range versions |
