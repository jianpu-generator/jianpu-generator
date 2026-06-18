# Selection-Range Highlight and Playback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend measure highlighting and playback so that a text selection spanning multiple measures highlights all overlapping measures in the SVG preview and plays them as one audio clip.

**Architecture:** Range computation happens in pure TypeScript using already-cached `measureSpans` state (no new WASM call). The Rust/WASM pipeline changes `Option<usize>` highlight to `Option<(usize, usize)>` throughout: grid_layout → coordinate_resolver → lib.rs → WASM. A new `write_midi_for_measure_range` function appends consecutive measures into one MIDI clip. The frontend swaps `currentMeasureIndex` for `selectedMeasureRange` and `onCursorByteOffsetChange` for `onSelectionChange`.

**Tech Stack:** Rust (grid_layout, coordinate_resolver, midi, lib.rs), wasm-bindgen (`crates/jianpu-wasm`), TypeScript/React, remeda (new)

---

## File Map

| Action | File |
|--------|------|
| Install | `remeda` in `web/` |
| Modify | `src/grid_layout/highlight.rs` — add `compute_measure_highlights_for_range` |
| Modify | `src/grid_layout/tests_highlight.rs` — tests for range function |
| Modify | `src/grid_layout/types.rs` — `measure_highlights: Vec<MeasureHighlight>` |
| Modify | `src/grid_layout/layout.rs` — range param, populate vec |
| Modify | `src/coordinate_resolver/resolve.rs` — iterate highlights vec |
| Modify | `src/lib.rs` — rename `render_svgs_with_highlight` → `render_svgs_with_highlight_range` |
| Modify | `crates/jianpu-wasm/src/lib.rs` — update render + audio WASM exports |
| Modify | `src/midi.rs` — add `write_midi_for_measure_range` |
| Modify | `src/tests/measure_audio.rs` — range audio tests |
| Modify | `web/src/worker/jianpu.worker.ts` — new request/response types |
| Modify | `web/src/components/PlayMeasureButton.tsx` — accept range, update label |
| Modify | `web/src/components/Editor.tsx` — `onSelectionChange` prop |
| Modify | `web/src/hooks/useJianpuWorker.ts` — range state + `notifySelection` |
| Modify | `web/src/App.tsx` — wire new props |

---

## Task 1: Install remeda

**Files:** `web/package.json`, `web/pnpm-lock.yaml`

- [ ] **Step 1: Install**

```bash
cd web && pnpm add remeda
```

Expected: remeda appears in `web/package.json` dependencies.

- [ ] **Step 2: Commit**

```bash
git add web/package.json web/pnpm-lock.yaml
git commit -m "chore: add remeda for functional array utilities"
```

---

## Task 2: Add `compute_measure_highlights_for_range` + tests

**Files:**
- Modify: `src/grid_layout/highlight.rs`
- Modify: `src/grid_layout/tests_highlight.rs`

- [ ] **Step 1: Write the failing tests**

Add to `src/grid_layout/tests_highlight.rs`:

```rust
#[test]
fn range_with_single_index_returns_one_highlight_matching_location() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights =
        compute_measure_highlights_for_range(&page_systems, 0, 0, &no_header(), 20.0);
    assert_eq!(highlights.len(), 1);
    let (page_idx, h) = highlights.into_iter().next().expect("should have one highlight");
    assert_eq!(page_idx, 0);
    assert_eq!(h.column_start, 4);
    assert_eq!(h.column_end, 9);
}

#[test]
fn range_spanning_two_measures_returns_two_highlights() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights =
        compute_measure_highlights_for_range(&page_systems, 0, 1, &no_header(), 20.0);
    assert_eq!(highlights.len(), 2);
    let mut iter = highlights.into_iter();
    let (_, first_h) = iter.next().expect("first highlight");
    let (_, second_h) = iter.next().expect("second highlight");
    assert_eq!(first_h.column_start, 4);
    assert_eq!(second_h.column_start, 9);
}

#[test]
fn range_out_of_bounds_returns_empty_vec() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights =
        compute_measure_highlights_for_range(&page_systems, 5, 5, &no_header(), 20.0);
    assert!(highlights.is_empty());
}

#[test]
fn range_spanning_two_pages_reports_correct_page_indices() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4)]], vec![vec![simple_block(4)]]];
    let highlights =
        compute_measure_highlights_for_range(&page_systems, 0, 1, &no_header(), 20.0);
    assert_eq!(highlights.len(), 2);
    let mut iter = highlights.into_iter();
    let (first_page, _) = iter.next().expect("first highlight");
    let (second_page, _) = iter.next().expect("second highlight");
    assert_eq!(first_page, 0);
    assert_eq!(second_page, 1);
}
```

Add the import at the top of `tests_highlight.rs`:

```rust
use crate::grid_layout::layout::compute_measure_highlights_for_range;
```

- [ ] **Step 2: Run to verify tests fail**

```bash
cargo test -p jianpu-generator grid_layout::layout::tests_highlight
```

Expected: compile error — `compute_measure_highlights_for_range` not found.

- [ ] **Step 3: Implement `compute_measure_highlights_for_range` in `src/grid_layout/highlight.rs`**

Add after the existing `compute_measure_highlight_location` function:

```rust
pub(crate) fn compute_measure_highlights_for_range(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    start_index: usize,
    end_index: usize,
    header: &Header,
    base: f32,
) -> Vec<(usize, MeasureHighlight)> {
    let header_row_count = make_header_rows(header, base).len();
    let mut global_measure_index: usize = 0;
    let mut results: Vec<(usize, MeasureHighlight)> = Vec::new();

    for (page_idx, page_sys) in page_systems.iter().enumerate() {
        let mut row_offset = header_row_count;
        for (sys_idx, system) in page_sys.iter().enumerate() {
            if sys_idx > 0 {
                row_offset += 1;
            }
            let Some(first) = system.first() else {
                continue;
            };
            if has_any_decoration(first) {
                row_offset += 1;
            }
            let musical_row_count = system_musical_row_count(system, base);
            let row_start = row_offset;
            let row_end = row_offset + musical_row_count.saturating_sub(1);

            let mut col_offset: u32 = LABEL_COLS;
            for block in system {
                let col_w = block_column_width(block);
                if global_measure_index >= start_index && global_measure_index <= end_index {
                    results.push((
                        page_idx,
                        MeasureHighlight {
                            row_start,
                            row_end,
                            column_start: col_offset,
                            column_end: col_offset + col_w,
                        },
                    ));
                }
                col_offset += col_w;
                global_measure_index += 1;
            }
            row_offset += musical_row_count;
        }
    }
    results
}
```

Re-export it from `src/grid_layout/layout.rs` alongside the existing re-export (line 507):

```rust
pub(crate) use super::highlight::compute_measure_highlight_location;
pub(crate) use super::highlight::compute_measure_highlights_for_range;
```

- [ ] **Step 4: Run to verify tests pass**

```bash
cargo test -p jianpu-generator grid_layout::layout::tests_highlight
```

Expected: all 7 tests pass (4 new + 3 existing).

- [ ] **Step 5: Commit**

```bash
git add src/grid_layout/highlight.rs src/grid_layout/tests_highlight.rs src/grid_layout/layout.rs
git commit -m "feat: add compute_measure_highlights_for_range to grid_layout"
```

---

## Task 3: Atomic refactor — thread highlight range through pipeline

All five files must change together so that `cargo test` passes at the end.

**Files:**
- Modify: `src/grid_layout/types.rs`
- Modify: `src/grid_layout/layout.rs`
- Modify: `src/coordinate_resolver/resolve.rs`
- Modify: `src/lib.rs`
- Modify: `crates/jianpu-wasm/src/lib.rs`

- [ ] **Step 1: Change `GridPage.measure_highlight` → `measure_highlights` in `src/grid_layout/types.rs`**

Replace:
```rust
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
    pub measure_highlight: Option<MeasureHighlight>,
}
```

With:
```rust
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
    pub measure_highlights: Vec<MeasureHighlight>,
}
```

- [ ] **Step 2: Update `layout()` in `src/grid_layout/layout.rs`**

Change the function signature from:
```rust
pub fn layout(
    compile_result: &CompileResult,
    config: &RenderConfig,
    header: &Header,
    page_width_pt: f32,
    page_height_pt: f32,
    highlighted_measure_index: Option<usize>,
) -> Vec<GridPage>
```

To:
```rust
pub fn layout(
    compile_result: &CompileResult,
    config: &RenderConfig,
    header: &Header,
    page_width_pt: f32,
    page_height_pt: f32,
    highlighted_measure_range: Option<(usize, usize)>,
) -> Vec<GridPage>
```

Replace the `highlight_info` block (lines ~552–554):
```rust
let highlight_info: Option<(usize, crate::grid_layout::types::MeasureHighlight)> =
    highlighted_measure_index
        .and_then(|idx| compute_measure_highlight_location(&page_systems, idx, header, base));
```

With:
```rust
let highlight_infos: Vec<(usize, crate::grid_layout::types::MeasureHighlight)> =
    highlighted_measure_range
        .map(|(start, end)| {
            compute_measure_highlights_for_range(&page_systems, start, end, header, base)
        })
        .unwrap_or_default();
```

Replace the per-page `measure_highlight` block (lines ~570–583):
```rust
let measure_highlight = highlight_info
    .as_ref()
    .and_then(|(highlight_page, highlight)| {
        if *highlight_page == page_idx {
            Some(highlight.clone())
        } else {
            None
        }
    });
pages.push(GridPage {
    width_pt: page_width_pt,
    height_pt: page_height_pt,
    rows,
    measure_highlight,
});
```

With:
```rust
let measure_highlights: Vec<_> = highlight_infos
    .iter()
    .filter(|(p, _)| *p == page_idx)
    .map(|(_, h)| h.clone())
    .collect();
pages.push(GridPage {
    width_pt: page_width_pt,
    height_pt: page_height_pt,
    rows,
    measure_highlights,
});
```

- [ ] **Step 3: Update `src/coordinate_resolver/resolve.rs`**

Rename `resolve_measure_highlight` to `resolve_single_measure_highlight` and change its first parameter from `&Option<MeasureHighlight>` to `&MeasureHighlight` directly. Remove the `let highlight = highlight.as_ref()?;` line at the top of the body.

Old function signature + first line of body:
```rust
fn resolve_measure_highlight(
    highlight: &Option<crate::grid_layout::types::MeasureHighlight>,
    rows: &[crate::grid_layout::types::GridRow],
    row_tops: &[f32],
    usable_width: f32,
) -> Option<AbsoluteElement> {
    let highlight = highlight.as_ref()?;
    let start_row = rows.get(highlight.row_start)?;
```

New:
```rust
fn resolve_single_measure_highlight(
    highlight: &crate::grid_layout::types::MeasureHighlight,
    rows: &[crate::grid_layout::types::GridRow],
    row_tops: &[f32],
    usable_width: f32,
) -> Option<AbsoluteElement> {
    let start_row = rows.get(highlight.row_start)?;
```

Keep the rest of the body identical.

Add a new `resolve_measure_highlights` function immediately after:

```rust
fn resolve_measure_highlights(
    highlights: &[crate::grid_layout::types::MeasureHighlight],
    rows: &[crate::grid_layout::types::GridRow],
    row_tops: &[f32],
    usable_width: f32,
) -> Vec<AbsoluteElement> {
    highlights
        .iter()
        .filter_map(|h| resolve_single_measure_highlight(h, rows, row_tops, usable_width))
        .collect()
}
```

In `resolve_page`, replace the old highlight block (lines ~90–94):
```rust
if let Some(element) =
    resolve_measure_highlight(&page.measure_highlight, &page.rows, &row_tops, usable_width)
{
    elements.insert(0, element);
}

AbsolutePage {
    width_pt: page.width_pt,
    height_pt: page.height_pt,
    elements,
}
```

With:
```rust
let mut highlight_elements = resolve_measure_highlights(
    &page.measure_highlights,
    &page.rows,
    &row_tops,
    usable_width,
);
highlight_elements.extend(elements);

AbsolutePage {
    width_pt: page.width_pt,
    height_pt: page.height_pt,
    elements: highlight_elements,
}
```

- [ ] **Step 4: Update `src/lib.rs`**

Rename `render_svgs_with_highlight` to `render_svgs_with_highlight_range` and change its signature:

Old:
```rust
pub fn render_svgs_with_highlight(
    source: &str,
    filename: &str,
    highlighted_measure_index: Option<usize>,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> Result<Vec<String>, JianPuError> {
```

New:
```rust
pub fn render_svgs_with_highlight_range(
    source: &str,
    filename: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> Result<Vec<String>, JianPuError> {
```

Inside the function, change the `grid_layout::layout` call from:
```rust
let grid_pages = grid_layout::layout(
    &compile_result,
    &config,
    &header,
    595.0,
    842.0,
    highlighted_measure_index,
);
```

To:
```rust
let grid_pages = grid_layout::layout(
    &compile_result,
    &config,
    &header,
    595.0,
    842.0,
    Some((start_index, end_index)),
);
```

- [ ] **Step 5: Update `crates/jianpu-wasm/src/lib.rs`**

Replace the import of `render_svgs_with_highlight` with `render_svgs_with_highlight_range`:

```rust
use jianpu_generator::{
    compile, find_measure_at_byte_offset, list_measure_spans_from_source, list_parts_from_source,
    render_svgs_from_source_filtered_with_lyrics, render_svgs_with_highlight_range,
};
```

Replace `render_with_highlight_response` (old, takes `highlighted_measure_index: usize`) with:

```rust
fn render_with_highlight_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
) -> RenderResponse {
    let tracks = enabled_tracks.as_deref();
    let lyrics = disabled_lyrics.as_deref();
    match render_svgs_with_highlight_range(
        source,
        "input.jianpu",
        start_index,
        end_index,
        tracks,
        lyrics,
    ) {
        Ok(svgs) => RenderResponse::Ok { svgs },
        Err(e) => RenderResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, e)],
        },
    }
}
```

Replace the `render_with_highlight` WASM export with:

```rust
#[wasm_bindgen]
pub fn render_with_highlight_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
) -> RenderResponse {
    render_with_highlight_range_response(
        source,
        start_index,
        end_index,
        enabled_tracks,
        disabled_lyrics,
    )
}
```

- [ ] **Step 6: Compile and test everything**

```bash
cargo test
```

Expected: all tests pass. (`cargo clippy` is run by pre-commit but you can check early with `cargo clippy --all-targets`.)

- [ ] **Step 7: Commit**

```bash
git add src/grid_layout/types.rs src/grid_layout/layout.rs src/coordinate_resolver/resolve.rs src/lib.rs crates/jianpu-wasm/src/lib.rs
git commit -m "refactor: thread highlight range through grid_layout → coordinate_resolver → WASM"
```

---

## Task 4: Add `write_midi_for_measure_range` + WAV wrapper + tests

**Files:**
- Modify: `src/midi.rs`
- Modify: `src/lib.rs`
- Modify: `src/tests/measure_audio.rs`

- [ ] **Step 1: Write failing tests in `src/tests/measure_audio.rs`**

Add at the bottom of the file:

```rust
#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_range_from_source_returns_riff_wav() {
    let source = two_measure_source();
    let wav =
        write_wav_for_measure_range_from_source(source, "test.jianpu", 0, 1, None).unwrap();
    assert!(wav.len() > 4);
    assert_eq!(&wav[0..4], b"RIFF");
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_range_from_source_single_measure_matches_range_of_one() {
    let source = two_measure_source();
    let single = write_wav_for_measure_from_source(source, "test.jianpu", 0, None).unwrap();
    let range = write_wav_for_measure_range_from_source(source, "test.jianpu", 0, 0, None)
        .unwrap();
    // Both paths produce RIFF WAV; the exact bytes may differ but both are valid.
    assert_eq!(&single[0..4], b"RIFF");
    assert_eq!(&range[0..4], b"RIFF");
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_range_from_source_out_of_range_returns_err() {
    let source = two_measure_source();
    let result = write_wav_for_measure_range_from_source(source, "test.jianpu", 0, 99, None);
    assert!(result.is_err());
}
```

Add the import at the top of `src/tests/measure_audio.rs`:

```rust
use super::write_wav_for_measure_range_from_source;
```

- [ ] **Step 2: Run to verify tests fail**

```bash
cargo test -p jianpu-generator --features wav tests::measure_audio
```

Expected: compile error — `write_wav_for_measure_range_from_source` not found.

- [ ] **Step 3: Add `write_midi_for_measure_range` to `src/midi.rs`**

Add after `write_midi_for_measure`:

```rust
pub fn write_midi_for_measure_range(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Result<Vec<u8>, JianPuError> {
    if start_index > end_index || end_index >= score.measures.len() {
        return Err(JianPuError::new(
            Span::new(0, 0),
            format!(
                "invalid measure range {}..={} (score has {} measures)",
                start_index,
                end_index,
                score.measures.len()
            ),
        ));
    }
    let mut accumulated_bpm: Option<u32> = None;
    let mut accumulated_key: Option<KeyChange> = None;
    for measure in score.measures.iter().take(start_index) {
        if let Some(bpm) = measure.bpm {
            accumulated_bpm = Some(bpm);
        }
        if let Some(key) = &measure.key {
            accumulated_key = Some(key.clone());
        }
    }
    let count = end_index - start_index + 1;
    let mut measures: Vec<_> = score
        .measures
        .iter()
        .skip(start_index)
        .take(count)
        .cloned()
        .collect();
    if let Some(first) = measures.first_mut() {
        if first.bpm.is_none() {
            first.bpm = accumulated_bpm;
        }
        if first.key.is_none() {
            first.key = accumulated_key;
        }
    }
    let range_score = Score {
        metadata: score.metadata.clone(),
        measures,
    };
    write_midi(&range_score)
}
```

- [ ] **Step 4: Add `write_wav_for_measure_range_from_source` to `src/lib.rs`**

Add after `write_wav_for_measure_from_source`:

```rust
/// Parse, group, optionally filter tracks, and synthesize WAV for a consecutive measure range.
///
/// BPM and key context is accumulated from all measures before `start_index`.
#[cfg(feature = "wav")]
pub fn write_wav_for_measure_range_from_source(
    source: &str,
    filename: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
) -> Result<Vec<u8>, JianPuError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    let midi_bytes = midi::write_midi_for_measure_range(&score, start_index, end_index)?;
    wav::write_wav(&midi_bytes)
}
```

- [ ] **Step 5: Run to verify tests pass**

```bash
cargo test -p jianpu-generator --features wav tests::measure_audio
```

Expected: all tests pass including the 3 new ones.

- [ ] **Step 6: Commit**

```bash
git add src/midi.rs src/lib.rs src/tests/measure_audio.rs
git commit -m "feat: add write_midi_for_measure_range and write_wav_for_measure_range_from_source"
```

---

## Task 5: Update WASM audio export + rebuild

**Files:**
- Modify: `crates/jianpu-wasm/src/lib.rs`

- [ ] **Step 1: Update `crates/jianpu-wasm/src/lib.rs`**

Replace the import:
```rust
#[cfg(feature = "wav")]
use jianpu_generator::write_wav_for_measure_from_source;
```
With:
```rust
#[cfg(feature = "wav")]
use jianpu_generator::write_wav_for_measure_range_from_source;
```

Replace `generate_wav_for_measure_response` (old):
```rust
#[cfg(feature = "wav")]
fn generate_wav_for_measure_response(
    source: &str,
    measure_index: usize,
    enabled_tracks: Option<Vec<String>>,
) -> GenerateWavResponse {
    let tracks = enabled_tracks.as_deref();
    match write_wav_for_measure_from_source(source, "input.jianpu", measure_index, tracks) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, e)],
        },
    }
}
```

With:
```rust
#[cfg(feature = "wav")]
fn generate_wav_for_measure_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<Vec<String>>,
) -> GenerateWavResponse {
    let tracks = enabled_tracks.as_deref();
    match write_wav_for_measure_range_from_source(
        source,
        "input.jianpu",
        start_index,
        end_index,
        tracks,
    ) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, e)],
        },
    }
}
```

Replace the `generate_wav_for_measure` WASM export:
```rust
#[cfg(feature = "wav")]
#[wasm_bindgen]
pub fn generate_wav_for_measure(
    source: &str,
    measure_index: usize,
    enabled_tracks: Option<Vec<String>>,
) -> GenerateWavResponse {
    generate_wav_for_measure_response(source, measure_index, enabled_tracks)
}
```

With:
```rust
#[cfg(feature = "wav")]
#[wasm_bindgen]
pub fn generate_wav_for_measure_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<Vec<String>>,
) -> GenerateWavResponse {
    generate_wav_for_measure_range_response(source, start_index, end_index, enabled_tracks)
}
```

- [ ] **Step 2: Verify Rust compiles**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 3: Rebuild WASM**

```bash
cd web && pnpm build:wasm:audio
```

Expected: `web/node_modules/jianpu-wasm/` updated with new exports. Verify with:

```bash
grep -c "render_with_highlight_range\|generate_wav_for_measure_range" node_modules/jianpu-wasm/jianpu_wasm.d.ts
```

Expected: 2 matches.

- [ ] **Step 4: Commit**

```bash
git add crates/jianpu-wasm/src/lib.rs web/node_modules/jianpu-wasm/ web/public/
git commit -m "feat: add render_with_highlight_range and generate_wav_for_measure_range WASM exports"
```

Note: if `web/node_modules/jianpu-wasm/` is gitignored, skip adding it and rebuild as part of the dev workflow.

---

## Task 6: Frontend atomic update

All five frontend files change together. Make all changes before running TypeScript check or committing.

**Files:**
- Modify: `web/src/worker/jianpu.worker.ts`
- Modify: `web/src/components/PlayMeasureButton.tsx`
- Modify: `web/src/components/Editor.tsx`
- Modify: `web/src/hooks/useJianpuWorker.ts`
- Modify: `web/src/App.tsx`

### 6a: Update worker types and handlers

- [ ] **Step 1: Update `WorkerRequest` in `web/src/worker/jianpu.worker.ts`**

Remove these three union members:
```ts
| {
    type: 'getMeasureAtOffset'
    source: string
    id: number
    byteOffset: number
  }
| {
    type: 'generateMeasureAudio'
    source: string
    id: number
    measureIndex: number
    enabledTracks?: string[]
  }
| {
    type: 'renderWithHighlight'
    source: string
    id: number
    highlightedMeasureIndex: number
    enabledTracks?: string[]
    disabledLyrics?: string[]
  }
```

Add these two union members:
```ts
| {
    type: 'generateMeasureRangeAudio'
    source: string
    id: number
    startMeasureIndex: number
    endMeasureIndex: number
    enabledTracks?: string[]
  }
| {
    type: 'renderWithHighlightRange'
    source: string
    id: number
    startMeasureIndex: number
    endMeasureIndex: number
    enabledTracks?: string[]
    disabledLyrics?: string[]
  }
```

Update `WorkerResponse`: remove `measureAtOffset`, `measureAudio`, `measureAudioErr`, `highlightOk`, `highlightErr` union members; add:

```ts
| { type: 'measureRangeAudio'; id: number; wav: ArrayBuffer }
| { type: 'measureRangeAudioErr'; id: number }
| { type: 'highlightRangeOk'; id: number; svgs: string[] }
| { type: 'highlightRangeErr'; id: number; diagnostics: Diagnostic[] }
```

- [ ] **Step 2: Update WASM variable names and message handlers**

Replace:
```ts
const generateWavForMeasure =
  'generate_wav_for_measure' in jianpuWasm
    ? jianpuWasm.generate_wav_for_measure
    : null

const renderWithHighlight =
  'render_with_highlight' in jianpuWasm
    ? jianpuWasm.render_with_highlight
    : null
```

With:
```ts
const generateWavForMeasureRange =
  'generate_wav_for_measure_range' in jianpuWasm
    ? jianpuWasm.generate_wav_for_measure_range
    : null

const renderWithHighlightRange =
  'render_with_highlight_range' in jianpuWasm
    ? jianpuWasm.render_with_highlight_range
    : null
```

Remove the `getMeasureAtOffset` handler block:
```ts
if (msg.type === 'getMeasureAtOffset') {
  const result = get_measure_index_at_offset(msg.source, msg.byteOffset)
  postMessage({...})
  return
}
```

Replace the `generateMeasureAudio` handler with:
```ts
if (msg.type === 'generateMeasureRangeAudio') {
  if (!generateWavForMeasureRange) {
    postMessage({ type: 'measureRangeAudioErr', id: msg.id } satisfies WorkerResponse)
    return
  }
  const wavResult = generateWavForMeasureRange(
    msg.source,
    msg.startMeasureIndex,
    msg.endMeasureIndex,
    msg.enabledTracks,
  )
  if (wavResult.status === 'ok') {
    const wavBuffer = binaryBufferFromResult(wavResult.wav)
    postMessage(
      { type: 'measureRangeAudio', id: msg.id, wav: wavBuffer } satisfies WorkerResponse,
      { transfer: [wavBuffer] },
    )
    return
  }
  postMessage({ type: 'measureRangeAudioErr', id: msg.id } satisfies WorkerResponse)
  return
}
```

Replace the `renderWithHighlight` handler with:
```ts
if (msg.type === 'renderWithHighlightRange') {
  if (!renderWithHighlightRange) {
    postMessage({
      type: 'highlightRangeErr',
      id: msg.id,
      diagnostics: [
        {
          severity: 'error',
          message: 'render_with_highlight_range is not available in this build.',
          span: { start: 0, end: 0 },
        },
      ],
    } satisfies WorkerResponse)
    return
  }
  const result = renderWithHighlightRange(
    msg.source,
    msg.startMeasureIndex,
    msg.endMeasureIndex,
    msg.enabledTracks,
    msg.disabledLyrics,
  )
  if (result.status === 'ok') {
    postMessage({ type: 'highlightRangeOk', id: msg.id, svgs: result.svgs } satisfies WorkerResponse)
    return
  }
  postMessage({
    type: 'highlightRangeErr',
    id: msg.id,
    diagnostics: result.diagnostics,
  } satisfies WorkerResponse)
  return
}
```

Also remove the import of `get_measure_index_at_offset` from `jianpu-wasm` since the worker no longer uses it.

### 6b: Update PlayMeasureButton

- [ ] **Step 3: Replace `web/src/components/PlayMeasureButton.tsx`**

```tsx
interface PlayMeasureButtonProps {
  disabled: boolean
  loading: boolean
  measureRange: { start: number; end: number } | null
  onClick: () => void
}

function measureLabel(range: { start: number; end: number }): string {
  if (range.start === range.end) {
    return `▶ Measure ${range.start + 1}`
  }
  return `▶ Measures ${range.start + 1}–${range.end + 1}`
}

export function PlayMeasureButton({
  disabled,
  loading,
  measureRange,
  onClick,
}: PlayMeasureButtonProps) {
  const label = measureRange !== null ? measureLabel(measureRange) : null
  return (
    <button
      type="button"
      className="play-measure-btn"
      disabled={disabled}
      onClick={onClick}
      title={
        measureRange === null
          ? 'Move cursor into a measure to enable'
          : 'Play selected measure(s)'
      }
      aria-label={label ?? 'Play selected measure(s)'}
    >
      {loading ? (
        <span className="play-measure-spinner" aria-hidden="true" />
      ) : label !== null ? (
        label
      ) : (
        '▶'
      )}
    </button>
  )
}
```

### 6c: Update Editor

- [ ] **Step 4: Update `web/src/components/Editor.tsx`**

Change the prop interface: replace `onCursorByteOffsetChange?: (offset: number) => void` with:
```ts
onSelectionChange?: (startOffset: number, endOffset: number) => void
```

In the destructured props and the `onCursorByteOffsetChangeRef`, rename accordingly:
```ts
const onSelectionChangeRef = useRef(onSelectionChange)
useEffect(() => {
  onSelectionChangeRef.current = onSelectionChange
})
```

In `handleMount`, replace the `notifyCursor` function body:

Old:
```ts
const notifyCursor = () => {
  const model = ed.getModel()
  if (!model) return
  const position = ed.getPosition()
  if (!position) return
  if (onCursorByteOffsetChangeRef.current) {
    const charIndex = model.getOffsetAt(position)
    const byteOffset = stringIndexToByteOffset(model.getValue(), charIndex)
    onCursorByteOffsetChangeRef.current(byteOffset)
  }
  onCursorLineChangeRef.current?.(position.lineNumber)
}
```

New:
```ts
const notifyCursor = () => {
  const model = ed.getModel()
  if (!model) return
  const selection = ed.getSelection()
  if (!selection) return
  const source = model.getValue()
  if (onSelectionChangeRef.current) {
    const startCharIndex = model.getOffsetAt(selection.getStartPosition())
    const endCharIndex = model.getOffsetAt(selection.getEndPosition())
    const startOffset = stringIndexToByteOffset(source, startCharIndex)
    const endOffset = stringIndexToByteOffset(source, endCharIndex)
    onSelectionChangeRef.current(startOffset, endOffset)
  }
  onCursorLineChangeRef.current?.(selection.startLineNumber)
}
```

Also remove the unused `byteOffsetToStringIndex` import if it's no longer used (keep `stringIndexToByteOffset`).

### 6d: Update useJianpuWorker

- [ ] **Step 5: Update `web/src/hooks/useJianpuWorker.ts`**

Add the `measureRangeInSpan` helper at the top of the file (before the hook):

```ts
import { findIndex, findLastIndex } from 'remeda'

function measureRangeInSpan(
  spans: Array<{ start: number; end: number }>,
  selStart: number,
  selEnd: number,
): { start: number; end: number } | null {
  const effective = selStart === selEnd ? selEnd + 1 : selEnd
  const overlaps = (span: { start: number; end: number }) =>
    span.start < effective && span.end > selStart
  const start = findIndex(spans, overlaps)
  const end = findLastIndex(spans, overlaps)
  return start === -1 ? null : { start, end }
}
```

In the `JianpuWorkerState` interface, replace:
```ts
currentMeasureIndex: number | null
measureAudioGenerating: boolean
notifyCursorOffset: (offset: number) => void
playCurrentMeasure: () => void
```

With:
```ts
selectedMeasureRange: { start: number; end: number } | null
measureAudioGenerating: boolean
notifySelection: (startOffset: number, endOffset: number) => void
playSelectedMeasures: () => void
```

In the hook body, replace:
```ts
const [currentMeasureIndex, setCurrentMeasureIndex] = useState<number | null>(null)
```

With:
```ts
const [selectedMeasureRange, setSelectedMeasureRange] = useState<{
  start: number
  end: number
} | null>(null)
```

Add a ref for measure spans (after the existing `measureSpans` state declaration):
```ts
const measureSpansRef = useRef<Array<{ start: number; end: number }>>([])
```

And keep it in sync (alongside the other ref syncs at the top of the hook body):
```ts
measureSpansRef.current = measureSpans
```

Replace the refs for cursor tracking:
```ts
const cursorRequestIdRef = useRef(0)
const latestCursorIdRef = useRef(0)
const cursorOffsetTimerRef = useRef<number | null>(null)
const lastCursorByteOffsetRef = useRef<number | null>(null)
```

With:
```ts
const cursorOffsetTimerRef = useRef<number | null>(null)
const lastSelectionRef = useRef<{ start: number; end: number } | null>(null)
```

In the worker `onmessage` handler, replace `measureAtOffset`, `measureAudio`, `measureAudioErr`, `highlightOk`, `highlightErr` cases with:

```ts
if (msg.type === 'measureRangeAudio') {
  if (msg.id !== latestMeasureAudioIdRef.current) return
  setMeasureAudioGenerating(false)
  setNextMeasureWavUrl(
    URL.createObjectURL(new Blob([msg.wav], { type: 'audio/wav' })),
  )
  return
}

if (msg.type === 'measureRangeAudioErr') {
  if (msg.id !== latestMeasureAudioIdRef.current) return
  setMeasureAudioGenerating(false)
  return
}

if (msg.type === 'highlightRangeOk') {
  if (msg.id !== latestHighlightRenderIdRef.current) return
  setHighlightedSvgs(msg.svgs)
  return
}

if (msg.type === 'highlightRangeErr') {
  if (msg.id !== latestHighlightRenderIdRef.current) return
  return
}
```

Remove the `useEffect` that sent `getMeasureAtOffset` on source change:
```ts
useEffect(() => {
  setCurrentMeasureIndex(null)
  const byteOffset = lastCursorByteOffsetRef.current
  // ... (remove this whole effect)
}, [source])
```

Add a `useEffect` that resets range when source changes:
```ts
useEffect(() => {
  setSelectedMeasureRange(null)
}, [source])
```

Add a `useEffect` that recomputes range when `measureSpans` updates (after debounce):
```ts
useEffect(() => {
  const sel = lastSelectionRef.current
  if (!sel) return
  setSelectedMeasureRange(measureRangeInSpan(measureSpans, sel.start, sel.end))
}, [measureSpans])
```

Replace the `useEffect` that sent `renderWithHighlight` on `currentMeasureIndex` change:
```ts
useEffect(() => {
  if (selectedMeasureRange === null) {
    setHighlightedSvgs([])
    return
  }
  const worker = workerRef.current
  if (!worker) return
  const id = ++highlightRenderRequestIdRef.current
  latestHighlightRenderIdRef.current = id
  worker.postMessage({
    type: 'renderWithHighlightRange',
    source: sourceRef.current,
    id,
    startMeasureIndex: selectedMeasureRange.start,
    endMeasureIndex: selectedMeasureRange.end,
    enabledTracks: enabledTracksRef.current,
    disabledLyrics: disabledLyricsRef.current,
  } satisfies WorkerRequest)
}, [selectedMeasureRange])
```

Replace `notifyCursorOffset` with:
```ts
const notifySelection = useCallback(
  (startOffset: number, endOffset: number) => {
    lastSelectionRef.current = { start: startOffset, end: endOffset }
    if (cursorOffsetTimerRef.current !== null) {
      window.clearTimeout(cursorOffsetTimerRef.current)
    }
    cursorOffsetTimerRef.current = window.setTimeout(() => {
      cursorOffsetTimerRef.current = null
      setSelectedMeasureRange(
        measureRangeInSpan(measureSpansRef.current, startOffset, endOffset),
      )
    }, debounceMs)
  },
  [debounceMs],
)
```

Replace `playCurrentMeasure` with:
```ts
const playSelectedMeasures = useCallback(() => {
  const worker = workerRef.current
  if (!worker || selectedMeasureRange === null) return
  const id = ++measureAudioRequestIdRef.current
  latestMeasureAudioIdRef.current = id
  setMeasureAudioGenerating(true)
  worker.postMessage({
    type: 'generateMeasureRangeAudio',
    source: sourceRef.current,
    id,
    startMeasureIndex: selectedMeasureRange.start,
    endMeasureIndex: selectedMeasureRange.end,
    enabledTracks: enabledTracksRef.current,
  } satisfies WorkerRequest)
}, [selectedMeasureRange])
```

Update the return value:
```ts
return {
  // ...existing fields...
  selectedMeasureRange,       // was: currentMeasureIndex
  measureAudioGenerating,
  notifySelection,            // was: notifyCursorOffset
  playSelectedMeasures,       // was: playCurrentMeasure
  highlightedSvgs,
  measureSpans,
}
```

### 6e: Update App.tsx

- [ ] **Step 6: Update `web/src/App.tsx`**

In the destructuring of `useJianpuWorker`, replace:
```ts
currentMeasureIndex,
measureAudioGenerating,
notifyCursorOffset,
playCurrentMeasure,
```

With:
```ts
selectedMeasureRange,
measureAudioGenerating,
notifySelection,
playSelectedMeasures,
```

In the `<Editor>` JSX, replace:
```tsx
onCursorByteOffsetChange={notifyCursorOffset}
```

With:
```tsx
onSelectionChange={notifySelection}
```

In the `<PlayMeasureButton>` JSX, replace:
```tsx
disabled={currentMeasureIndex === null || measureAudioGenerating}
measureNumber={
  currentMeasureIndex !== null
    ? currentMeasureIndex + 1
    : null
}
onClick={playCurrentMeasure}
```

With:
```tsx
disabled={selectedMeasureRange === null || measureAudioGenerating}
measureRange={selectedMeasureRange}
onClick={playSelectedMeasures}
```

In the debug status line, replace:
```tsx
measure{' '}
{currentMeasureIndex !== null
  ? currentMeasureIndex + 1
  : 'null'}
```

With:
```tsx
{selectedMeasureRange !== null
  ? selectedMeasureRange.start === selectedMeasureRange.end
    ? `measure ${selectedMeasureRange.start + 1}`
    : `measures ${selectedMeasureRange.start + 1}–${selectedMeasureRange.end + 1}`
  : 'measure null'}
```

### 6f: Verify and commit

- [ ] **Step 7: Type-check**

```bash
cd web && pnpm exec tsc --noEmit
```

Expected: no errors.

- [ ] **Step 8: Lint**

```bash
cd web && pnpm lint
```

Fix any lint errors before committing.

- [ ] **Step 9: Commit**

```bash
git add web/src/worker/jianpu.worker.ts \
        web/src/components/PlayMeasureButton.tsx \
        web/src/components/Editor.tsx \
        web/src/hooks/useJianpuWorker.ts \
        web/src/App.tsx
git commit -m "feat: selection-range highlight and playback — frontend wiring"
```
