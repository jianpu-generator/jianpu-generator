# Warning View Zones Design

## Goal

Recoverable parse errors are currently displayed as red error view zones. They should instead appear as amber/yellow warning view zones, visually distinct from blocking (irrecoverable) errors.

Additionally, if a single source line has both errors and warnings, they must appear as separate view zones stacked below that line — errors first, warnings second.

## Approach

Grouping logic moves fully to Rust. TypeScript receives a pre-grouped list of view zone descriptors and renders them without any grouping, sorting, or line-number computation.

## Rust Changes

### `crates/jianpu-wasm/src/types.rs`

**Fix severity of recoverable errors:**

`diagnostic_from_recoverable_error` currently sets `DiagnosticSeverity::Error`. Change it to `DiagnosticSeverity::Warning`.

**Add new output types:**

```rust
pub struct DiagnosticMessageOut {
    pub message: String,
    pub report: Option<String>,  // skip_serializing_if = None
}

pub struct DiagnosticViewZoneOut {
    pub severity: DiagnosticSeverity,
    /// 1-based line number in the source. View zone is inserted after this line.
    pub after_line_number: usize,
    pub messages: Vec<DiagnosticMessageOut>,
}
```

**Add grouping helper:**

```rust
pub(crate) fn group_diagnostics_into_view_zones(
    source: &str,
    diagnostics: &[DiagnosticOut],
) -> Vec<DiagnosticViewZoneOut>
```

Logic:
1. For each diagnostic, compute `after_line_number` by counting `\n` in `source[..span.end.min(source.len())]` and adding 1.
2. Group by `(after_line_number, severity)` — preserve source order within each group.
3. Sort groups: primary key = `after_line_number` ascending, secondary key = errors before warnings.
4. Return one `DiagnosticViewZoneOut` per group.

**Extend `RenderResponse`:**

```rust
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

### `crates/jianpu-wasm/src/lib.rs`

Call `group_diagnostics_into_view_zones(source, &diagnostics)` when constructing both `RenderResponse::Ok` and `RenderResponse::Err`, and include the result as `diagnostic_view_zones`.

## TypeScript Changes

### `web/src/components/Editor.tsx`

**New prop:** Add `diagnosticViewZones?: DiagnosticViewZoneOut[]` alongside the existing `diagnostics` prop. `diagnostics` continues to drive Monaco squiggle markers only.

**`createDiagnosticViewZoneDomNode(severity, messages)`:** Rename from `createErrorViewZoneDomNode`. Accepts severity and message list. Uses class `editor-error-zone` for errors, `editor-warning-zone` for warnings.

**`applyDiagnosticViewZones`:** Replace grouping logic with a simple loop over `diagnosticViewZones`. For each entry, convert `after_line_number` directly to a Monaco `afterLineNumber` (they are the same 1-based value). No range computation, no grouping.

### `web/src/App.tsx` (or wherever `Editor` is used)

Pass `diagnostic_view_zones` from the render response as the `diagnosticViewZones` prop.

## CSS Changes

### `web/src/index.css`

Add:
```css
--warning: #92400e;
--warning-bg: #fffbeb;
```

### `web/src/App.css`

Add `.editor-warning-zone` and `.editor-warning-zone-message` mirroring the error zone styles but using `--warning` and `--warning-bg`.

## Non-Goals

- Warning view zones for non-render responses (PDF, WAV) — not needed, those don't have a live editor.
- Changing Monaco squiggle marker colors — that already works correctly via `MarkerSeverity.Warning`.
