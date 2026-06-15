# Measure View Zones Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show a contrasting `[Measure N]` label line before each measure in the Monaco editor using view zones — purely visual, not part of the source text.

**Architecture:** Add a `list_measure_spans` Rust function + WASM export that returns byte spans for all measures. The frontend hook fetches these spans on each source change; the Editor component converts them to line numbers and applies Monaco view zones.

**Tech Stack:** Rust/wasm-bindgen, TypeScript, React, Monaco Editor (`@monaco-editor/react`)

---

## File Map

| File | Change |
|------|--------|
| `src/lib.rs` | Add `pub fn list_measure_spans` |
| `crates/jianpu-wasm/src/types.rs` | Add `ListMeasureSpansResponse` |
| `crates/jianpu-wasm/src/lib.rs` | Add `list_measure_spans_response` helper + WASM export |
| `crates/jianpu-wasm/src/tests.rs` | Add WASM-level tests |
| `web/src/types.ts` | Add `ListMeasureSpansResult` TypeScript type |
| `web/src/worker/jianpu.worker.ts` | Add `listMeasureSpans` request/response + handler |
| `web/src/hooks/useJianpuWorker.ts` | Add `measureSpans` state + worker effect |
| `web/src/components/Editor.tsx` | Add `measureSpans` prop + `applyMeasureViewZones` effect |
| `web/src/App.tsx` | Pass `measureSpans` from hook to `<Editor>` |

---

### Task 1: Rust — `list_measure_spans` in `src/lib.rs`

**Files:**
- Modify: `src/lib.rs` (after `find_measure_at_line_number`, around line 233)

- [ ] **Step 1: Write the failing test**

Add a new test file `tests/measure_spans.rs`:

```rust
use jianpu_generator::list_measure_spans_from_source;

const TWO_MEASURE_SOURCE: &str = concat!(
    "[metadata]\n",
    "title = \"t\"\n",
    "author = \"a\"\n",
    "\n",
    "[parts]\n",
    "Melody = notes\n",
    "\n",
    "[score]\n",
    "(time=4/4 key=C4 bpm=120)\n",
    "1 2 3 4\n",
    "5 6 7 1\n",
);

#[test]
fn returns_one_span_per_measure() {
    let spans = list_measure_spans_from_source(TWO_MEASURE_SOURCE, "test.jianpu").unwrap();
    assert_eq!(spans.len(), 2);
}

#[test]
fn spans_are_ordered_by_source_position() {
    let spans = list_measure_spans_from_source(TWO_MEASURE_SOURCE, "test.jianpu").unwrap();
    assert!(spans[0].start < spans[1].start);
}

#[test]
fn returns_err_on_invalid_source() {
    let result = list_measure_spans_from_source("not valid jianpu", "test.jianpu");
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test --test measure_spans
```

Expected: compile error — `list_measure_spans` not found.

- [ ] **Step 3: Implement the function**

In `src/lib.rs`, after `find_measure_at_line_number` (around line 233), add:

```rust
/// Return the source byte span of every measure in the compiled score.
///
/// Spans are in source order and correspond 1-to-1 with measures.
pub fn list_measure_spans_from_source(
    source: &str,
    filename: &str,
) -> Result<Vec<error::Span>, JianPuError> {
    let score = compile(source, filename)?;
    Ok(score.measures.iter().map(|m| m.source_span.clone()).collect())
}
```

- [ ] **Step 4: Run tests to confirm they pass**

```bash
cargo test --test measure_spans
```

Expected: all 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs tests/measure_spans.rs
git commit -m "feat: add list_measure_spans to return byte spans for all measures"
```

---

### Task 2: WASM — types, export, and tests

**Files:**
- Modify: `crates/jianpu-wasm/src/types.rs`
- Modify: `crates/jianpu-wasm/src/lib.rs`
- Modify: `crates/jianpu-wasm/src/tests.rs`

- [ ] **Step 1: Write the failing WASM test**

In `crates/jianpu-wasm/src/tests.rs`, add at the end:

```rust
#[test]
fn list_measure_spans_returns_one_span_per_measure() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Melody = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
    );
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok { spans } => {
            assert_eq!(spans.len(), 2);
            assert!(spans[0].start < spans[1].start);
        }
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}

#[test]
fn list_measure_spans_returns_err_on_invalid_source() {
    let resp = list_measure_spans_response("not valid jianpu");
    assert!(matches!(resp, ListMeasureSpansResponse::Err));
}
```

- [ ] **Step 2: Run test to confirm it fails**

```bash
cargo test -p jianpu-wasm
```

Expected: compile error — `list_measure_spans_response` and `ListMeasureSpansResponse` not found.

- [ ] **Step 3: Add `ListMeasureSpansResponse` to `crates/jianpu-wasm/src/types.rs`**

After `MeasureAtOffsetResponse` (around line 56), add:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub(crate) enum ListMeasureSpansResponse {
    Ok { spans: Vec<SpanOut> },
    Err,
}
```

- [ ] **Step 4: Add response helper and WASM export to `crates/jianpu-wasm/src/lib.rs`**

Add the import at the top alongside the existing imports:

```rust
use jianpu_generator::list_measure_spans_from_source;
```

Add the response helper and export after `get_measure_at_offset_response` (around line 88):

```rust
fn list_measure_spans_response(source: &str) -> ListMeasureSpansResponse {
    match list_measure_spans_from_source(source, "input.jianpu") {
        Ok(spans) => ListMeasureSpansResponse::Ok {
            spans: spans
                .into_iter()
                .map(|s| SpanOut { start: s.start, end: s.end })
                .collect(),
        },
        Err(_) => ListMeasureSpansResponse::Err,
    }
}

/// Return the byte span of every measure in the source.
///
/// - `{ "status": "ok", "spans": [{ "start": N, "end": N }, ...] }` on success
/// - `{ "status": "err" }` on parse failure
#[wasm_bindgen]
pub fn list_measure_spans(source: &str) -> JsValue {
    to_js_value(&list_measure_spans_response(source))
}
```

Also update the existing `use types::{ ... }` import at the top of `crates/jianpu-wasm/src/lib.rs` to include `ListMeasureSpansResponse`:

```rust
use types::{
    diagnostic_from_error, to_js_value, ListMeasureSpansResponse, ListPartsResponse,
    MeasureAtOffsetResponse, PartOut, RenderResponse,
};
```

- [ ] **Step 5: Run tests to confirm they pass**

```bash
cargo test -p jianpu-wasm
```

Expected: all tests pass including the two new ones.

- [ ] **Step 6: Commit**

```bash
git add crates/jianpu-wasm/src/types.rs crates/jianpu-wasm/src/lib.rs crates/jianpu-wasm/src/tests.rs
git commit -m "feat: expose list_measure_spans as WASM export"
```

---

### Task 3: TypeScript — types and worker

**Files:**
- Modify: `web/src/types.ts`
- Modify: `web/src/worker/jianpu.worker.ts`

- [ ] **Step 1: Add TypeScript type to `web/src/types.ts`**

After `MeasureAtOffsetResult` (around line 52), add:

```ts
type ListMeasureSpansOk = { status: 'ok'; spans: Array<{ start: number; end: number }> }
type ListMeasureSpansErr = { status: 'err' }
export type ListMeasureSpansResult = ListMeasureSpansOk | ListMeasureSpansErr
```

- [ ] **Step 2: Add request/response types to `web/src/worker/jianpu.worker.ts`**

Import `ListMeasureSpansResult` in the import block at the top:

```ts
import type {
  Diagnostic,
  GeneratePdfResult,
  GenerateWavResult,
  ListMeasureSpansResult,
  ListPartsResult,
  MeasureAtOffsetResult,
  PartInfo,
  RenderResult,
} from '../types'
```

Add to `WorkerRequest` union (after `renderWithHighlight`):

```ts
| { type: 'listMeasureSpans'; source: string; id: number }
```

Add to `WorkerResponse` union (after `highlightErr`):

```ts
| { type: 'measureSpans'; id: number; spans: Array<{ start: number; end: number }> }
```

Add the WASM import alongside existing imports:

```ts
import init, * as jianpuWasm from 'jianpu-wasm'
import { get_measure_index_at_offset, list_measure_spans, list_parts, render } from 'jianpu-wasm'
```

Add the handler inside `self.onmessage` before the `if (msg.type !== 'render') return` guard:

```ts
if (msg.type === 'listMeasureSpans') {
  const result = list_measure_spans(msg.source) as ListMeasureSpansResult
  postMessage({
    type: 'measureSpans',
    id: msg.id,
    spans: result.status === 'ok' ? result.spans : [],
  } satisfies WorkerResponse)
  return
}
```

- [ ] **Step 3: Verify TypeScript compiles**

```bash
cd web && npx tsc --noEmit
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add web/src/types.ts web/src/worker/jianpu.worker.ts
git commit -m "feat: add listMeasureSpans worker request/response"
```

---

### Task 4: Hook — `measureSpans` state in `useJianpuWorker`

**Files:**
- Modify: `web/src/hooks/useJianpuWorker.ts`

- [ ] **Step 1: Add state to `JianpuWorkerState` interface**

In the `JianpuWorkerState` interface (around line 30), add:

```ts
measureSpans: Array<{ start: number; end: number }>
```

- [ ] **Step 2: Add `useState` and handle worker response**

After `const [highlightedSvgs, setHighlightedSvgs] = useState<string[]>([])` (around line 116), add:

```ts
const [measureSpans, setMeasureSpans] = useState<Array<{ start: number; end: number }>>([])
```

Add request ID refs after `latestHighlightRenderIdRef` (around line 118):

```ts
const measureSpansRequestIdRef = useRef(0)
const latestMeasureSpansIdRef = useRef(0)
```

In the `worker.onmessage` handler, add after the `highlightErr` block (before the `err` block):

```ts
if (msg.type === 'measureSpans') {
  if (msg.id !== latestMeasureSpansIdRef.current) return
  setMeasureSpans(msg.spans)
  return
}
```

- [ ] **Step 3: Add the worker effect that fires on source change**

After the `useEffect` that handles `currentMeasureIndex` (around line 411), add:

```ts
useEffect(() => {
  const worker = workerRef.current
  if (!worker) return

  const id = ++measureSpansRequestIdRef.current
  latestMeasureSpansIdRef.current = id

  const timer = window.setTimeout(() => {
    worker.postMessage({
      type: 'listMeasureSpans',
      source,
      id,
    } satisfies WorkerRequest)
  }, debounceMs)

  return () => window.clearTimeout(timer)
}, [source, debounceMs])
```

- [ ] **Step 4: Add `measureSpans` to the return value**

In the `return { ... }` at the bottom of `useJianpuWorker` (around line 484), add:

```ts
measureSpans,
```

- [ ] **Step 5: Verify TypeScript compiles**

```bash
cd web && npx tsc --noEmit
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add web/src/hooks/useJianpuWorker.ts
git commit -m "feat: expose measureSpans from useJianpuWorker hook"
```

---

### Task 5: Editor — view zones

**Files:**
- Modify: `web/src/components/Editor.tsx`

- [ ] **Step 1: Add the `measureSpans` prop**

In the `EditorProps` interface (around line 17), add:

```ts
measureSpans?: Array<{ start: number; end: number }>
```

In the destructured parameters of `Editor` (around line 51), add:

```ts
measureSpans = [],
```

- [ ] **Step 2: Add a ref to track active view zone IDs**

After `const monacoRef = useRef<Monaco | null>(null)` (around line 63), add:

```ts
const viewZoneIdsRef = useRef<string[]>([])
```

- [ ] **Step 3: Add `applyMeasureViewZones` callback**

After `applyDiagnostics` (around line 100), add:

```ts
const applyMeasureViewZones = useCallback(() => {
  const ed = editorRef.current
  const model = ed?.getModel()
  if (!ed || !model) return

  ed.changeViewZones((accessor) => {
    for (const id of viewZoneIdsRef.current) {
      accessor.removeZone(id)
    }
    viewZoneIdsRef.current = []

    const source = model.getValue()

    measureSpans.forEach((span, index) => {
      const stringIndex = byteOffsetToStringIndex(source, span.start)
      const position = model.getPositionAt(stringIndex)
      const lineNumber = position.lineNumber

      const domNode = document.createElement('div')
      domNode.style.cssText = [
        'width: 100%',
        'height: 21px',
        'background: #dbeafe',
        'color: #1e40af',
        'font-family: var(--mono)',
        'font-size: 14px',
        'font-weight: bold',
        'display: flex',
        'align-items: center',
        'padding-left: 8px',
        'box-sizing: border-box',
      ].join(';')
      domNode.textContent = `[Measure ${index + 1}]`

      const id = accessor.addZone({
        afterLineNumber: lineNumber - 1,
        heightInLines: 1,
        domNode,
      })
      viewZoneIdsRef.current.push(id)
    })
  })
}, [measureSpans])
```

- [ ] **Step 4: Call it in `handleMount` and on `measureSpans` change**

In `handleMount` (around line 157), after `applyDiagnostics()`, add:

```ts
applyMeasureViewZones()
```

After the `useEffect` for `applyDiagnostics` (around line 178), add:

```ts
useEffect(() => {
  applyMeasureViewZones()
}, [applyMeasureViewZones])
```

- [ ] **Step 5: Verify TypeScript compiles**

```bash
cd web && npx tsc --noEmit
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add web/src/components/Editor.tsx
git commit -m "feat: render measure view zones in Monaco editor"
```

---

### Task 6: Wire up in `App.tsx`

**Files:**
- Modify: `web/src/App.tsx`

- [ ] **Step 1: Destructure `measureSpans` from the hook**

In the destructured result of `useJianpuWorker` (around line 46), add:

```ts
measureSpans,
```

- [ ] **Step 2: Pass `measureSpans` to `<Editor>`**

In the `<Editor>` JSX (around line 209), add the prop:

```tsx
<Editor
  ref={editorRef}
  value={source}
  onChange={handleSourceChange}
  readOnly={readOnly}
  diagnostics={diagnostics}
  measureSpans={measureSpans}
  onCursorByteOffsetChange={notifyCursorOffset}
  onCursorLineChange={setCurrentLine}
  toolbar={...}
/>
```

- [ ] **Step 3: Verify TypeScript compiles**

```bash
cd web && npx tsc --noEmit
```

Expected: no errors.

- [ ] **Step 4: Build and rebuild the WASM package**

```bash
cd web && npm run build:wasm
```

Expected: succeeds, `crates/jianpu-wasm/pkg/jianpu_wasm.js` updated with `list_measure_spans` export.

- [ ] **Step 5: Start dev server and verify visually**

```bash
cd web && npm run dev
```

Open the app, load any `.jianpu` file with multiple measures. You should see blue `[Measure 1]`, `[Measure 2]`, etc. label lines appearing before each measure in the editor. They should scroll with the text and not be editable.

- [ ] **Step 6: Commit**

```bash
git add web/src/App.tsx
git commit -m "feat: wire measureSpans into Editor for view zone labels"
```
