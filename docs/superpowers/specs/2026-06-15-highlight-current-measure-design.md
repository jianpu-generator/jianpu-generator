# Highlight Current Measure in SVG Preview

**Date:** 2026-06-15

## Goal

When the caret in the Monaco editor sits inside a measure, highlight that measure in the rendered SVG preview with a semi-transparent background rectangle.

## Visual Design

- Semi-transparent amber rectangle behind the measure's notes and staff rows
- Rectangle spans all part rows in the system for that measure (full vertical stack)
- Rectangle spans from the measure's first column to its last column
- Persistent: stays visible while the caret remains in the measure; updates when the cursor moves to a different measure

## Pipeline Changes

### 1. Grid Layout — `src/grid_layout/`

`layout` gains a `highlighted_measure_index: Option<usize>` parameter. Return type stays `Vec<GridPage>`.

`GridPage` gains an optional `measure_highlight` field:

```rust
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
    pub measure_highlight: Option<MeasureHighlight>,
}

pub struct MeasureHighlight {
    pub row_start: usize,   // first row index on this page (inclusive)
    pub row_end: usize,     // last row index on this page (inclusive)
    pub column_start: u32,  // first column of the measure within the row
    pub column_end: u32,    // last column (exclusive) of the measure
}
```

All fields are grid-space — no absolute coordinates. At most one page carries a non-`None` highlight. `page_index` is unnecessary because the highlight is embedded in the page that owns it.

### 2. Coordinate Resolver — `src/coordinate_resolver/`

`resolve` signature and `AbsolutePage` struct are unchanged. The resolver converts `GridPage.measure_highlight` to absolute coordinates using the same column-width and row-height arithmetic applied to all `GridElement`s, then prepends a new `AbsoluteElement` to the page's element list:

```rust
// New AbsoluteContent variant
AbsoluteContent::MeasureHighlight { width: f32, height: f32 }
```

`x` and `y` come from the `AbsoluteElement` wrapper as usual. Inserting it first in `elements` ensures it renders behind all note content (SVG paints in document order).

### 3. Renderer — `src/renderer/`

`SvgDocument` and `render_new` signature are unchanged. The renderer handles the new `AbsoluteContent::MeasureHighlight` arm in the existing match, producing a new `SvgKind::Rect`:

```rust
// New SvgKind variant
SvgKind::Rect { width: f32, height: f32 }
```

Fill colour and corner radius are constants baked into the renderer for this variant.

### 4. Serializer — `src/serializer/`

Handles the new `SvgKind::Rect` arm and emits:

```svg
<rect x="..." y="..." width="..." height="..." fill="rgba(255,200,0,0.25)" rx="2"/>
```

No structural changes to the serializer — just a new match arm.

### 5. WASM — `crates/jianpu-wasm/`

New export following the `generate_wav_for_measure` pattern:

```rust
pub fn render_with_highlight(
    source: &str,
    highlighted_measure_index: usize,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
) -> JsValue  // RenderResponse { status: "ok", svgs } | { status: "err", diagnostics }
```

Existing `render` is unchanged.

## Frontend Changes

### Worker — `web/src/worker/jianpu.worker.ts`

New request/response pair:

```ts
// Request
{ type: 'renderWithHighlight'; source: string; id: number; highlightedMeasureIndex: number; enabledTracks?: string[]; disabledLyrics?: string[] }

// Response
{ type: 'highlightOk'; id: number; svgs: string[] }
```

### `useJianpuWorker` hook

- New state: `highlightedSvgs: string[]` (starts empty)
- New request ID refs: `highlightRenderRequestIdRef`, `latestHighlightRenderIdRef`
- Effect: when `currentMeasureIndex` changes (and is non-null), fire `renderWithHighlight`; on `highlightOk` response, set `highlightedSvgs`
- When `currentMeasureIndex` becomes `null` (cursor leaves a measure or source changes), reset `highlightedSvgs` to `[]`
- No loading state exposed — highlight re-renders are silent

Hook returns `highlightedSvgs` alongside existing fields.

### `Preview` component and `App`

`Preview` receives `highlightedSvgs` from the hook. When non-empty, renders `highlightedSvgs` instead of `svgs`. Falls back to `svgs` when empty.

## ARCHITECTURE.md Updates Required

- `GridPage` key type: add `measure_highlight: Option<MeasureHighlight>` field
- New key types: `MeasureHighlight`, `AbsoluteContent::MeasureHighlight`, `SvgKind::Rect`
- All layer entry signatures unchanged
