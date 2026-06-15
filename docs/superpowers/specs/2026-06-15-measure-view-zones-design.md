---
title: Measure View Zones in Monaco Editor
date: 2026-06-15
---

## Overview

Add contrasting `[Measure N]` label lines in the Monaco editor before each measure, implemented as Monaco view zones. These are purely visual — they do not exist in the source text and do not affect editing.

## Backend

### New Rust function (`src/lib.rs`)

```rust
pub fn list_measure_spans(source: &str, filename: &str) -> Result<Vec<ByteSpan>, JianPuError>
```

Compiles the source and maps `score.measures[i].source_span` into a `Vec<ByteSpan>`.

### New WASM export (`crates/jianpu-wasm/src/lib.rs`)

```rust
#[wasm_bindgen]
pub fn list_measure_spans(source: &str) -> JsValue
```

Returns:
- `{ status: "ok", spans: [{ start: number, end: number }, ...] }` on success
- `{ status: "err" }` on parse failure (no diagnostics — the render pipeline already surfaces errors)

### New TypeScript type (`web/src/types.ts`)

```ts
type ListMeasureSpansOk = { status: 'ok'; spans: Array<{ start: number; end: number }> }
type ListMeasureSpansErr = { status: 'err' }
export type ListMeasureSpansResult = ListMeasureSpansOk | ListMeasureSpansErr
```

## Worker (`web/src/worker/jianpu.worker.ts`)

New request type:
```ts
{ type: 'listMeasureSpans'; source: string; id: number }
```

New response type:
```ts
{ type: 'measureSpans'; id: number; spans: Array<{ start: number; end: number }> }
```

On `listMeasureSpans` request: call WASM `list_measure_spans`, return spans (empty array on `status: 'err'`).

## Hook (`web/src/hooks/useJianpuWorker.ts`)

- Add `measureSpans: Array<{ start: number; end: number }>` to `JianpuWorkerState` (default `[]`)
- Fire `listMeasureSpans` whenever `source` changes, debounced at `debounceMs` (same pattern as `listParts`)
- On `measureSpans` response: `setMeasureSpans(msg.spans)`

## Editor (`web/src/components/Editor.tsx`)

### New prop

```ts
measureSpans?: Array<{ start: number; end: number }>
```

### View zone management

A `applyMeasureViewZones()` function (called on mount and whenever `measureSpans` changes via `useEffect`):

1. For each span, convert `span.start` byte offset → string index via `byteOffsetToStringIndex`, then `model.getPositionAt(stringIndex)` to get a line number
2. Call `editor.changeViewZones(accessor => { ... })`:
   - Remove all previously registered zone IDs (tracked in a `useRef`)
   - Add one view zone per measure:
     - `afterLineNumber`: line *before* the measure's first line (i.e. `lineNumber - 1`)
     - `heightInLines`: 1
     - `domNode`: a `<div>` with the label (see Styling)

### Styling

Each view zone `domNode`:
- `width: 100%`
- `height: 21px` (matches editor `lineHeight`)
- `background: #dbeafe` (light blue, contrasting)
- `color: #1e40af` (dark blue)
- `font-family: var(--mono)`, `font-size: 14px`, `font-weight: bold`
- `display: flex; align-items: center`
- `padding-left`: set to match the editor content area (to align with code)
- Content: `[Measure 1]`, `[Measure 2]`, etc.

DOM nodes are created imperatively (`document.createElement('div')`) inside `applyMeasureViewZones`, not via React rendering.

## Data flow

```
source change
  → debounced listMeasureSpans worker request
  → WASM list_measure_spans
  → measureSpans response
  → setMeasureSpans in hook
  → measureSpans prop to Editor
  → applyMeasureViewZones effect
  → editor.changeViewZones(...)
```

## Out of scope

- Per-measure SVG preview inside view zones (deferred)
- TypeScript type generation via `tsify` (deferred)
