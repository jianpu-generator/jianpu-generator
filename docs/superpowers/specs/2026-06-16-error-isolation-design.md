# Error Isolation: Red-Highlight Erroneous Measures

## Goal

Parse/group errors in a single measure should not abort the entire render.
Instead, the measure renders with whatever valid content exists, and a
semi-transparent red rectangle is overlaid on its bounding box in the SVG.
Recoverable errors are collected and returned alongside the SVGs so callers
can still surface them (e.g. in the editor). Irrecoverable errors remain in
the `Err` branch and abort the render as before.

---

## Scope

### POC (this implementation)

Only **lyrics-underflow** (`#syllables < #notes`) is recoverable. All other
errors remain fail-fast.

### Future recoverable errors (deferred — not implemented now)

The table below documents agreed recovery strategies for when those cases are
tackled:

| Error | Recovery strategy |
|---|---|
| Lyrics overflow (`#syllables > #notes`) | Tail-trim syllables to match note count; collect error on measure. |
| Beat overflow (total beats > beats-per-measure) | Drop extra beats beyond the measure boundary; collect error on measure. |
| Beat underflow (total beats < beats-per-measure) | Render partial bar, leave trailing space; collect error on measure. |
| Malformed/unrecognized note token | Skip the bad token, continue parsing remaining tokens; collect error on measure. |
| Ditto (`"`) with no prior measure | Render blank red placeholder (no partial content possible). |
| Parts line count mismatch | Fewer lines than parts → treat missing parts as empty for that measure. More lines than parts → ignore extra lines. Collect error on measure. |
| Unclosed tie/slur spanning into an errored measure | Drop the unclosed arc; render what's inside normally. |

Document-structural errors (missing `[metadata]`/`[parts]`/`[score]` section,
duplicate sections, unrecognised section headers) always abort the entire
render and are never isolated.

---

## Design

### 1. Errors field on `MultiPartMeasure`

Replace the boolean flag idea with a `Vec<JianPuError>` so the errors are
preserved and reportable:

```rust
pub struct MultiPartMeasure {
    // ... existing fields ...
    pub errors: Vec<JianPuError>,
}
```

A measure is considered erroneous when `!errors.is_empty()`. This drives both
the red overlay in the renderer and the error list returned to the caller.

### 2. Recovery in the grouper

When pairing syllables to notes in a `ParsedTimedTrack`:

- If `#syllables < #notes`: pad the syllable list with empty syllables to reach
  the note count. Push a `JianPuError` describing the underflow into
  `measure.errors`. Return `Ok` instead of `Err`.
- All other errors in the grouper remain `Err` (fail-fast) for now.

### 3. Public API — `RenderOutput`

The render functions currently return `Result<Vec<String>, JianPuError>`.
Change the `Ok` payload to a new struct so recoverable errors are accessible:

```rust
pub struct RenderOutput {
    pub svgs: Vec<String>,
    pub errors: Vec<JianPuError>,   // collected from all erroneous measures
}
```

All existing render entry points (`render_svgs_from_source`,
`render_svgs_with_highlight_range`, etc.) change their `Ok(Vec<String>)` to
`Ok(RenderOutput)`. The `Err` branch is unchanged.

### 4. Propagation through the pipeline

`errors: Vec<JianPuError>` on `MultiPartMeasure` is threaded through each
intermediate type, forwarded without logic changes:

```
MultiPartMeasure.errors
  → MeasureBlock.errors        (compiler)
  → GridElement / GridRow      (grid_layout — whichever type owns a measure row)
  → AbsoluteElement.errors     (coordinate_resolver)
  → renderer overlay step
```

At the top-level render function, all per-measure error vecs are flattened into
`RenderOutput.errors` before returning.

### 5. Renderer overlay

After rendering a measure's normal SVG content, if `!errors.is_empty()`, append
a rectangle covering the measure's bounding box:

```svg
<rect x="..." y="..." width="..." height="..."
      fill="rgba(255,0,0,0.15)" />
```

Opacity 0.15 (15%) keeps the note content legible beneath the tint.
The bounding box is derived from the absolute coordinates already computed by
the coordinate resolver.

---

## What is NOT changing

- WAV / MIDI output paths are unaffected; they do not consume `errors`.
- `error_reporter.rs` is unchanged; the caller decides how to display the
  returned `RenderOutput.errors`.
