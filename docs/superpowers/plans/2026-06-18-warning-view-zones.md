# Warning View Zones Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render recoverable (warning-severity) diagnostics as amber view zones in the Monaco editor, separate from red error view zones, with all grouping logic on the Rust/WASM side.

**Architecture:** Rust computes `DiagnosticViewZoneOut` groups (one per line×severity pair) and includes them in `RenderResponse`. The worker passes them through to the hook; the Editor receives them as a prop and renders each directly as a Monaco view zone, with CSS class driven by severity.

**Tech Stack:** Rust, wasm-pack, tsify/serde, TypeScript, React, Monaco Editor, CSS custom properties.

## Global Constraints

- All new Rust structs must derive `Debug, Clone, Tsify, Serialize, PartialEq, Eq` and `#[tsify(into_wasm_abi)]`
- WASM rebuild command: `cd web && pnpm run build:wasm` (run from repo root as `cd /path/to/jianpu-generator/web && pnpm run build:wasm`)
- Rust tests: `cargo test -p jianpu-wasm` (from repo root)
- Never use tuples in new data structures — use named structs

---

### Task 1: Add `DiagnosticViewZoneOut` types and grouping helper (Rust)

**Files:**
- Modify: `crates/jianpu-wasm/src/types.rs`
- Modify: `crates/jianpu-wasm/src/tests.rs`

**Interfaces:**
- Produces:
  - `pub struct DiagnosticMessageOut { pub message: String, pub report: Option<String> }`
  - `pub struct DiagnosticViewZoneOut { pub severity: DiagnosticSeverity, pub after_line_number: usize, pub messages: Vec<DiagnosticMessageOut> }`
  - `pub(crate) fn group_diagnostics_into_view_zones(source: &str, diagnostics: &[DiagnosticOut]) -> Vec<DiagnosticViewZoneOut>`

- [ ] **Step 1: Add `DiagnosticMessageOut` and `DiagnosticViewZoneOut` structs to `types.rs`**

  In `crates/jianpu-wasm/src/types.rs`, after the `DiagnosticOut` struct definition, add:

  ```rust
  #[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
  #[tsify(into_wasm_abi)]
  pub struct DiagnosticMessageOut {
      pub message: String,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub report: Option<String>,
  }

  #[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
  #[tsify(into_wasm_abi)]
  pub struct DiagnosticViewZoneOut {
      pub severity: DiagnosticSeverity,
      /// 1-based line number; view zone is inserted after this line.
      pub after_line_number: usize,
      pub messages: Vec<DiagnosticMessageOut>,
  }
  ```

- [ ] **Step 2: Add `group_diagnostics_into_view_zones` helper to `types.rs`**

  At the bottom of `crates/jianpu-wasm/src/types.rs`, add:

  ```rust
  fn byte_offset_to_line_number(source: &str, byte_offset: usize) -> usize {
      source[..byte_offset.min(source.len())]
          .bytes()
          .filter(|&b| b == b'\n')
          .count()
          + 1
  }

  pub(crate) fn group_diagnostics_into_view_zones(
      source: &str,
      diagnostics: &[DiagnosticOut],
  ) -> Vec<DiagnosticViewZoneOut> {
      use std::collections::BTreeMap;

      let mut groups: BTreeMap<(usize, u8), (DiagnosticSeverity, Vec<DiagnosticMessageOut>)> =
          BTreeMap::new();

      for d in diagnostics {
          let line = byte_offset_to_line_number(source, d.span.end);
          let severity_order = match d.severity {
              DiagnosticSeverity::Error => 0,
              DiagnosticSeverity::Warning => 1,
          };
          let entry = groups
              .entry((line, severity_order))
              .or_insert_with(|| (d.severity.clone(), Vec::new()));
          entry.1.push(DiagnosticMessageOut {
              message: d.message.clone(),
              report: d.report.clone(),
          });
      }

      groups
          .into_iter()
          .map(|((line, _), (severity, messages))| DiagnosticViewZoneOut {
              severity,
              after_line_number: line,
              messages,
          })
          .collect()
  }
  ```

- [ ] **Step 3: Write failing tests for `group_diagnostics_into_view_zones` in `tests.rs`**

  Add a new test module at the bottom of `crates/jianpu-wasm/src/tests.rs`:

  ```rust
  mod group_diagnostics_tests {
      use super::types::{
          DiagnosticOut, DiagnosticSeverity, DiagnosticViewZoneOut, SpanOut,
          group_diagnostics_into_view_zones,
      };

      fn make_diagnostic(severity: DiagnosticSeverity, message: &str, span_end: usize) -> DiagnosticOut {
          DiagnosticOut {
              severity,
              message: message.to_string(),
              span: SpanOut { start: 0, end: span_end },
              report: None,
          }
      }

      #[test]
      fn single_error_produces_one_error_zone() {
          // "line1\nline2\n" — byte offset 10 is on line 2
          let source = "line1\nline2\n";
          let diagnostics = vec![make_diagnostic(DiagnosticSeverity::Error, "oops", 10)];
          let zones = group_diagnostics_into_view_zones(source, &diagnostics);
          assert_eq!(zones.len(), 1);
          assert_eq!(zones[0].severity, DiagnosticSeverity::Error);
          assert_eq!(zones[0].after_line_number, 2);
          assert_eq!(zones[0].messages.len(), 1);
          assert_eq!(zones[0].messages[0].message, "oops");
      }

      #[test]
      fn single_warning_produces_one_warning_zone() {
          let source = "line1\n";
          let diagnostics = vec![make_diagnostic(DiagnosticSeverity::Warning, "note", 4)];
          let zones = group_diagnostics_into_view_zones(source, &diagnostics);
          assert_eq!(zones.len(), 1);
          assert_eq!(zones[0].severity, DiagnosticSeverity::Warning);
          assert_eq!(zones[0].after_line_number, 1);
      }

      #[test]
      fn two_errors_same_line_merge_into_one_zone() {
          let source = "line1\nline2\n";
          let diagnostics = vec![
              make_diagnostic(DiagnosticSeverity::Error, "first", 8),
              make_diagnostic(DiagnosticSeverity::Error, "second", 10),
          ];
          let zones = group_diagnostics_into_view_zones(source, &diagnostics);
          assert_eq!(zones.len(), 1);
          assert_eq!(zones[0].messages.len(), 2);
          assert_eq!(zones[0].messages[0].message, "first");
          assert_eq!(zones[0].messages[1].message, "second");
      }

      #[test]
      fn error_and_warning_on_same_line_produce_two_zones_error_first() {
          let source = "line1\nline2\n";
          let diagnostics = vec![
              make_diagnostic(DiagnosticSeverity::Warning, "warn", 8),
              make_diagnostic(DiagnosticSeverity::Error, "err", 10),
          ];
          let zones = group_diagnostics_into_view_zones(source, &diagnostics);
          assert_eq!(zones.len(), 2);
          assert_eq!(zones[0].severity, DiagnosticSeverity::Error);
          assert_eq!(zones[1].severity, DiagnosticSeverity::Warning);
          assert_eq!(zones[0].after_line_number, 2);
          assert_eq!(zones[1].after_line_number, 2);
      }

      #[test]
      fn zones_sorted_by_line_number_ascending() {
          let source = "a\nb\nc\n";
          let diagnostics = vec![
              make_diagnostic(DiagnosticSeverity::Error, "line3", 5),
              make_diagnostic(DiagnosticSeverity::Error, "line1", 1),
          ];
          let zones = group_diagnostics_into_view_zones(source, &diagnostics);
          assert_eq!(zones.len(), 2);
          assert!(zones[0].after_line_number < zones[1].after_line_number);
      }

      #[test]
      fn empty_diagnostics_returns_empty_zones() {
          let zones = group_diagnostics_into_view_zones("source", &[]);
          assert!(zones.is_empty());
      }
  }
  ```

- [ ] **Step 4: Run tests — expect them to fail (function not yet public)**

  ```bash
  cargo test -p jianpu-wasm group_diagnostics_tests
  ```

  Expected: compile error — `group_diagnostics_into_view_zones` is `pub(crate)`, not accessible from test module with `use super::types::group_diagnostics_into_view_zones`. Fix by adjusting the use path. In tests.rs the module is inside `crates/jianpu-wasm/src/tests.rs` which is `mod tests` inside `lib.rs`. So use `use crate::types::{..., group_diagnostics_into_view_zones}`.

  Update the use line in the test module:
  ```rust
  use crate::types::{
      DiagnosticOut, DiagnosticSeverity, DiagnosticViewZoneOut, SpanOut,
      group_diagnostics_into_view_zones,
  };
  ```

- [ ] **Step 5: Run tests — expect them to pass**

  ```bash
  cargo test -p jianpu-wasm group_diagnostics_tests
  ```

  Expected: all 6 tests pass.

- [ ] **Step 6: Commit**

  ```bash
  git add crates/jianpu-wasm/src/types.rs crates/jianpu-wasm/src/tests.rs
  git commit -m "feat: add DiagnosticViewZoneOut types and grouping helper"
  ```

---

### Task 2: Fix recoverable error severity and wire `diagnostic_view_zones` into `RenderResponse`

**Files:**
- Modify: `crates/jianpu-wasm/src/types.rs`
- Modify: `crates/jianpu-wasm/src/lib.rs`
- Modify: `crates/jianpu-wasm/src/tests.rs`

**Interfaces:**
- Consumes: `group_diagnostics_into_view_zones` from Task 1
- Produces: `RenderResponse::Ok { svgs, diagnostics, diagnostic_view_zones }` and `RenderResponse::Err { diagnostics, diagnostic_view_zones }`

- [ ] **Step 1: Fix `diagnostic_from_recoverable_error` to use `DiagnosticSeverity::Warning`**

  In `crates/jianpu-wasm/src/types.rs`, change the `diagnostic_from_recoverable_error` function:

  ```rust
  pub(crate) fn diagnostic_from_recoverable_error(
      source: &str,
      e: RecoverableError,
  ) -> DiagnosticOut {
      let report = error_reporter::render_recoverable_with_source(source, &e);
      DiagnosticOut {
          severity: DiagnosticSeverity::Warning,
          message: e.message,
          span: SpanOut {
              start: e.span.start,
              end: e.span.end,
          },
          report: Some(report),
      }
  }
  ```

- [ ] **Step 2: Add `diagnostic_view_zones` to `RenderResponse` in `types.rs`**

  Replace the `RenderResponse` enum:

  ```rust
  #[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
  #[serde(tag = "status", rename_all = "camelCase")]
  #[tsify(into_wasm_abi)]
  pub enum RenderResponse {
      Ok {
          svgs: Vec<String>,
          diagnostics: Vec<DiagnosticOut>,
          diagnostic_view_zones: Vec<DiagnosticViewZoneOut>,
      },
      Err {
          diagnostics: Vec<DiagnosticOut>,
          diagnostic_view_zones: Vec<DiagnosticViewZoneOut>,
      },
  }
  ```

- [ ] **Step 3: Wire grouping into `render_response` and `render_with_highlight_range_response` in `lib.rs`**

  Add `group_diagnostics_into_view_zones` to the imports at the top of `lib.rs`:

  ```rust
  use types::{
      diagnostic_from_error, diagnostic_from_recoverable_error, group_diagnostics_into_view_zones,
      ListMeasureSpansResponse, ListPartsResponse, ListScoreLineHintsResponse,
      MeasureAtOffsetResponse, PartOut, RenderResponse, ScoreLineHintOut,
  };
  ```

  Replace the body of `render_response`:

  ```rust
  fn render_response(
      source: &str,
      enabled_tracks: Option<Vec<String>>,
      disabled_lyrics: Option<Vec<String>>,
  ) -> RenderResponse {
      let tracks = enabled_tracks.as_deref();
      let lyrics = disabled_lyrics.as_deref();
      match render_svgs_from_source_filtered_with_lyrics(source, "input.jianpu", tracks, lyrics) {
          Ok(output) => {
              let diagnostics: Vec<_> = output
                  .errors
                  .into_iter()
                  .map(|e| diagnostic_from_recoverable_error(source, e))
                  .collect();
              let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
              RenderResponse::Ok {
                  svgs: output.svgs,
                  diagnostics,
                  diagnostic_view_zones,
              }
          }
          Err(e) => {
              let diagnostics = vec![diagnostic_from_error(source, e)];
              let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
              RenderResponse::Err {
                  diagnostics,
                  diagnostic_view_zones,
              }
          }
      }
  }
  ```

  Apply the same pattern to `render_with_highlight_range_response`:

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
          Ok(output) => {
              let diagnostics: Vec<_> = output
                  .errors
                  .into_iter()
                  .map(|e| diagnostic_from_recoverable_error(source, e))
                  .collect();
              let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
              RenderResponse::Ok {
                  svgs: output.svgs,
                  diagnostics,
                  diagnostic_view_zones,
              }
          }
          Err(e) => {
              let diagnostics = vec![diagnostic_from_error(source, e)];
              let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
              RenderResponse::Err {
                  diagnostics,
                  diagnostic_view_zones,
              }
          }
      }
  }
  ```

- [ ] **Step 4: Add test that recoverable error produces a warning-severity view zone**

  In `crates/jianpu-wasm/src/tests.rs`, add after the existing `err_response_has_structured_diagnostic` test:

  ```rust
  #[test]
  fn recoverable_error_produces_warning_severity_view_zone() {
      // lyrics underflow is a recoverable error
      let input = concat!(
          "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
          "[parts]\nMelody = notes lyrics\n\n",
          "[score]\n(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b\n",
      );
      let resp = render_response(input, None, None);
      match resp {
          RenderResponse::Ok { diagnostics, diagnostic_view_zones, .. } => {
              assert_eq!(diagnostics.len(), 1);
              assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
              assert_eq!(diagnostic_view_zones.len(), 1);
              assert_eq!(diagnostic_view_zones[0].severity, DiagnosticSeverity::Warning);
              assert_eq!(diagnostic_view_zones[0].messages.len(), 1);
          }
          RenderResponse::Err { .. } => panic!("expected ok"),
      }
  }
  ```

  Also update the import at the top of `tests.rs` to include `DiagnosticViewZoneOut` if needed (check if already imported via `use super::*`).

- [ ] **Step 5: Update existing test that destructures `RenderResponse::Err`**

  The `err_response_has_structured_diagnostic` test currently matches:
  ```rust
  RenderResponse::Err { diagnostics } => { ... }
  ```

  Since we added `diagnostic_view_zones`, update it to:
  ```rust
  RenderResponse::Err { diagnostics, .. } => { ... }
  ```

- [ ] **Step 6: Run all WASM tests**

  ```bash
  cargo test -p jianpu-wasm
  ```

  Expected: all tests pass, including the new warning-severity test.

- [ ] **Step 7: Rebuild WASM package**

  ```bash
  cd web && pnpm run build:wasm
  ```

  Expected: exits 0. The updated `crates/jianpu-wasm/pkg/jianpu_wasm.d.ts` now exports `DiagnosticMessageOut`, `DiagnosticViewZoneOut`, and the updated `RenderResponse` type.

- [ ] **Step 8: Commit**

  ```bash
  git add crates/jianpu-wasm/src/types.rs crates/jianpu-wasm/src/lib.rs crates/jianpu-wasm/src/tests.rs crates/jianpu-wasm/pkg/
  git commit -m "feat: fix recoverable error severity to warning and add diagnostic_view_zones to RenderResponse"
  ```

---

### Task 3: TypeScript + CSS — consume `diagnosticViewZones` and render warning view zones

**Files:**
- Modify: `web/src/types.ts`
- Modify: `web/src/worker/jianpu.worker.ts`
- Modify: `web/src/hooks/useJianpuWorker.ts`
- Modify: `web/src/components/Editor.tsx`
- Modify: `web/src/App.tsx`
- Modify: `web/src/index.css`
- Modify: `web/src/App.css`

**Interfaces:**
- Consumes: `DiagnosticViewZoneOut`, `DiagnosticMessageOut` from updated WASM package (Task 2)
- Produces: `Editor` prop `diagnosticViewZones: DiagnosticViewZone[]`; hook return `diagnosticViewZones: DiagnosticViewZone[]`

- [ ] **Step 1: Export `DiagnosticViewZone` and `DiagnosticMessage` from `types.ts`**

  In `web/src/types.ts`, update the re-export block:

  ```typescript
  export type {
    DiagnosticMessageOut as DiagnosticMessage,
    DiagnosticOut as Diagnostic,
    DiagnosticViewZoneOut as DiagnosticViewZone,
    GeneratePdfResponse as GeneratePdfResult,
    GenerateSplitPdfsResponse as GenerateSplitPdfResult,
    GenerateWavResponse as GenerateWavResult,
    ListMeasureSpansResponse as ListMeasureSpansResult,
    ListPartsResponse as ListPartsResult,
    ListScoreLineHintsResponse as ListScoreLineHintsResult,
    MeasureAtOffsetResponse as MeasureAtOffsetResult,
    MeasureSpanOut as MeasureSpan,
    PartOut as PartInfo,
    RenderResponse as RenderResult,
    ScoreLineHintOut as ScoreLineHint,
    SpanOut as ByteSpan,
  } from 'jianpu-wasm'
  ```

- [ ] **Step 2: Update `WorkerResponse` type in `jianpu.worker.ts` to carry `diagnosticViewZones`**

  In `web/src/worker/jianpu.worker.ts`, update the import to include `DiagnosticViewZone` and `DiagnosticMessage`:

  ```typescript
  import type { Diagnostic, DiagnosticViewZone, MeasureSpan, PartInfo, ScoreLineHint } from '../types'
  ```

  (The existing import is from `'../types'` — check actual line and add `DiagnosticViewZone` to it.)

  Update the `WorkerResponse` union type — change the `'ok'` and `'err'` variants:

  ```typescript
  | { type: 'ok'; id: number; svgs: string[]; diagnostics: Diagnostic[]; diagnosticViewZones: DiagnosticViewZone[] }
  | { type: 'err'; id: number; diagnostics: Diagnostic[]; diagnosticViewZones: DiagnosticViewZone[] }
  ```

- [ ] **Step 3: Pass `diagnostic_view_zones` from WASM result in the worker's render handler**

  In `web/src/worker/jianpu.worker.ts`, find the `if (msg.type !== 'render') return` section at the bottom and update:

  ```typescript
  const result = render(msg.source, msg.enabledTracks, msg.disabledLyrics)
  if (result.status === 'ok') {
    postMessage({
      type: 'ok',
      id: msg.id,
      svgs: result.svgs,
      diagnostics: result.diagnostics,
      diagnosticViewZones: result.diagnostic_view_zones,
    } satisfies WorkerResponse)
    return
  }

  postMessage({
    type: 'err',
    id: msg.id,
    diagnostics: result.diagnostics,
    diagnosticViewZones: result.diagnostic_view_zones,
  } satisfies WorkerResponse)
  ```

- [ ] **Step 4: Add `diagnosticViewZones` to `JianpuWorkerState` and `useJianpuWorker` hook**

  In `web/src/hooks/useJianpuWorker.ts`, add `DiagnosticViewZone` to imports from `'../types'`.

  Add to `JianpuWorkerState` interface:
  ```typescript
  diagnosticViewZones: DiagnosticViewZone[]
  ```

  Add state:
  ```typescript
  const [diagnosticViewZones, setDiagnosticViewZones] = useState<DiagnosticViewZone[]>([])
  ```

  In the `'ok'` message handler (where `setDiagnostics(msg.diagnostics)` is called), add:
  ```typescript
  setDiagnosticViewZones(msg.diagnosticViewZones)
  ```

  In the `'err'` message handler, add:
  ```typescript
  setDiagnosticViewZones(msg.diagnosticViewZones)
  ```

  Add `diagnosticViewZones` to the return value of the hook.

- [ ] **Step 5: Update `Editor.tsx` — new prop, simplified view zone rendering**

  In `web/src/components/Editor.tsx`:

  Add import:
  ```typescript
  import type { Diagnostic, DiagnosticMessage, DiagnosticViewZone, EditorHandle, MeasureSpan, ScoreLineHint } from '../types'
  ```

  Add `diagnosticViewZones` to `EditorProps`:
  ```typescript
  export interface EditorProps {
    value: string
    onChange: (value: string) => void
    readOnly?: boolean
    diagnostics?: Diagnostic[]
    diagnosticViewZones?: DiagnosticViewZone[]
    measureSpans?: MeasureSpan[]
    scoreLineHints?: ScoreLineHint[]
    toolbar?: ReactNode
    onSelectionChange?: (startOffset: number, endOffset: number) => void
    onCursorLineChange?: (line: number) => void
    onPlayMeasure?: () => void
  }
  ```

  Replace `createErrorViewZoneDomNode` with:

  ```typescript
  function createDiagnosticViewZoneDomNode(
    severity: 'error' | 'warning',
    messages: DiagnosticMessage[],
  ): HTMLElement {
    const zoneClass = severity === 'warning' ? 'editor-warning-zone' : 'editor-error-zone'
    const messageClass = severity === 'warning' ? 'editor-warning-zone-message' : 'editor-error-zone-message'

    const domNode = document.createElement('div')
    domNode.className = zoneClass

    for (const [index, msg] of messages.entries()) {
      if (index > 0) {
        domNode.appendChild(document.createElement('hr'))
      }
      const messageEl = document.createElement('div')
      messageEl.className = messageClass
      messageEl.textContent = msg.message
      domNode.appendChild(messageEl)

      if (msg.report) {
        const report = document.createElement('pre')
        report.className = 'editor-error-zone-report'
        report.textContent = msg.report
        domNode.appendChild(report)
      }
    }

    return domNode
  }
  ```

  Add `diagnosticViewZones = []` to the destructured props (alongside `diagnostics = []`).

  Replace `applyErrorViewZones` with `applyDiagnosticViewZones`:

  ```typescript
  const applyDiagnosticViewZones = useCallback(() => {
    const ed = editorRef.current
    const model = ed?.getModel()
    if (!ed || !model) return

    ed.changeViewZones((accessor) => {
      for (const id of errorViewZoneIdsRef.current) {
        accessor.removeZone(id)
      }
      errorViewZoneIdsRef.current = []

      for (const zone of diagnosticViewZones) {
        const domNode = createDiagnosticViewZoneDomNode(zone.severity, zone.messages)
        const heightInPx = errorViewZoneHeightInPx(domNode, ed.getLayoutInfo().contentWidth)
        const id = accessor.addZone({
          afterLineNumber: zone.after_line_number,
          heightInPx,
          domNode,
        })
        errorViewZoneIdsRef.current.push(id)
      }
    })
  }, [diagnosticViewZones])
  ```

  Update `handleMount` and the `useEffect` to call `applyDiagnosticViewZones` instead of `applyErrorViewZones`.

  Remove `groupDiagnosticsByLine` — no longer needed.

  Remove the `monacoApi` parameter from `applyDiagnosticViewZones` (not needed since no range computation).

- [ ] **Step 6: Pass `diagnosticViewZones` from hook to Editor in `App.tsx`**

  In `web/src/App.tsx`, destructure `diagnosticViewZones` from `useJianpuWorker`:
  ```typescript
  const {
    ...
    diagnostics,
    diagnosticViewZones,
    ...
  } = useJianpuWorker(...)
  ```

  Add to the `<Editor>` JSX:
  ```tsx
  <Editor
    ...
    diagnostics={diagnostics}
    diagnosticViewZones={diagnosticViewZones}
    ...
  />
  ```

- [ ] **Step 7: Add warning CSS variables to `index.css`**

  In `web/src/index.css`, after the `--error-bg` line, add:
  ```css
  --warning: #92400e;
  --warning-bg: #fffbeb;
  ```

- [ ] **Step 8: Add warning zone CSS classes to `App.css`**

  In `web/src/App.css`, after the `.editor-error-zone-report` block, add:

  ```css
  .editor-warning-zone {
    box-sizing: border-box;
    width: 100%;
    padding: 0.35rem 1rem 0.5rem 3.5rem;
    background: var(--warning-bg);
    border-top: 1px solid color-mix(in srgb, var(--warning) 20%, transparent);
    font-size: 0.8rem;
  }

  .editor-warning-zone hr {
    border: none;
    border-top: 1px solid color-mix(in srgb, var(--warning) 15%, transparent);
    margin: 0.35rem 0;
  }

  .editor-warning-zone-message {
    color: var(--warning);
    font-weight: 500;
    line-height: 1.4;
  }
  ```

- [ ] **Step 9: Type-check**

  ```bash
  cd web && npx tsc --noEmit
  ```

  Expected: no errors.

- [ ] **Step 10: Commit**

  ```bash
  git add web/src/types.ts web/src/worker/jianpu.worker.ts web/src/hooks/useJianpuWorker.ts web/src/components/Editor.tsx web/src/App.tsx web/src/index.css web/src/App.css
  git commit -m "feat: render warning view zones in amber, error view zones in red"
  ```
