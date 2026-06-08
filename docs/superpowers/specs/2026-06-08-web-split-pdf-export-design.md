# Design: Web split-track PDF export (ZIP)

**Date:** 2026-06-08

## Summary

Add an **Export parts (ZIP)** button to the web preview pane. It mirrors CLI `jianpu generate pdf --split-tracks`: one PDF per part, lyrics included, packaged as a single ZIP download.

Implementation uses a new shared Rust library function and WASM export (Approach 2). Split-track rendering stays sequential — no `rayon`/`par_iter` (WASM is single-threaded; font reload per PDF is the bigger cost anyway).

## Requirements (confirmed)

| Decision | Choice |
|----------|--------|
| Format | PDF only |
| Delivery | One ZIP file |
| Parts included | All declared parts (ignore editor part toggles) |
| Lyrics | Included per part (ignore editor lyrics toggles; matches CLI `--split-tracks`) |
| Single-part scores | Button always shown; ZIP contains `{base} - {PartAbbrev}.pdf` |
| Parallelism | Out of scope for v1 |

## Current state

- Web preview has **Export PDF** — one combined PDF respecting part/lyrics toggles.
- CLI has `--split-tracks` on `generate pdf` — loops tracks in `main.rs`, writes `{stem} - {track}.pdf` per file.
- WASM exposes `generate_pdf(source, enabledTracks?, disabledLyrics?)` — single PDF only.
- Track-name helpers (`collect_track_names`, `sanitize_track_name`) live in `main.rs`, not the library.

## Goals

1. Web button equivalent to `generate pdf --split-tracks`.
2. Shared Rust logic used by CLI and WASM (no duplicated loop).
3. One ZIP download in the browser.
4. Existing CLI integration tests keep passing.

## Non-goals

- SVG or MIDI split export from the web UI.
- Per-part progress UI.
- Respecting editor part/lyrics toggles for split export.
- Parallel split-track rendering (`rayon`) in v1.
- Refactoring font reuse across multiple `write_pdf` calls.

## Architecture

```
┌─────────────┐     generateSplitPdf      ┌──────────────────┐
│  Preview UI │ ─────────────────────────►│  Web Worker      │
│  (React)    │◄── splitPdf / splitPdfErr │  jianpu.worker   │
└─────────────┘                           └────────┬─────────┘
                                                   │
                                                   ▼
                                          ┌──────────────────┐
                                          │  jianpu-wasm     │
                                          │ generate_split_  │
                                          │ pdfs(source,     │
                                          │ base_name)       │
                                          └────────┬─────────┘
                                                   │
                                                   ▼
                                          ┌──────────────────┐
                                          │ jianpu-generator │
                                          │ write_split_pdfs │
                                          │ _from_source     │
                                          └──────────────────┘
```

CLI `generate pdf --split-tracks` calls the same `write_split_pdfs_from_source` helper instead of inlining the loop.

## Rust core API

### New types and function (`src/lib.rs`, `pdf` feature)

```rust
pub struct SplitPdfEntry {
    pub track_name: String,   // part abbreviation, e.g. "S1"
    pub filename: String,     // e.g. "demo - S1.pdf"
    pub pdf: Vec<u8>,
}

pub fn write_split_pdfs_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
) -> Result<Vec<SplitPdfEntry>, JianPuError>
```

**Algorithm:**

1. `compile(source, filename)` once.
2. Determine track list:
   - Primary: `collect_track_names(&score)` (moved from `main.rs`).
   - Fallback: abbreviations from `[parts]` declarations (via existing parse path) when the score has no named measure parts — ensures single-part split naming still works.
3. For each track:
   - Clone score.
   - `filter_tracks(&mut score, &[track])`.
   - `render_svgs(&score)` → `pdf::write_pdf(&svgs)`.
   - Filename: `{base_name} - {sanitize_track_name(track)}.pdf`.
4. Return all entries (including when there is only one track).

### Helpers moved to library

- `collect_track_names(score: &Score) -> Vec<String>` — from `main.rs`.
- `sanitize_track_name(name: &str) -> String` — from `main.rs`.

### ZIP assembly (`src/lib.rs` or `src/pdf.rs`)

New function (pdf feature):

```rust
pub fn zip_split_pdfs(entries: &[SplitPdfEntry]) -> Result<Vec<u8>, JianPuError>
```

Uses the `zip` crate (new optional dependency on `pdf` feature). Each ZIP entry name is `entry.filename` (basename only, no directories).

### CLI refactor

`generate_pdf` with `--split-tracks` calls `write_split_pdfs_from_source`, then writes each entry to disk (unchanged output paths). Removes duplicated clone/filter/render loop from `main.rs`.

## WASM API

### Export (`crates/jianpu-wasm`, `pdf` feature)

```rust
#[wasm_bindgen]
pub fn generate_split_pdfs(source: &str, base_name: &str) -> JsValue
```

**Success:**

```json
{ "status": "ok", "zip": "<Uint8Array>" }
```

**Error:**

```json
{ "status": "err", "diagnostics": [ ... ] }
```

Implementation: call `write_split_pdfs_from_source`, then `zip_split_pdfs`, return bytes via `Uint8Array` (same pattern as `generate_pdf`).

No `enabledTracks` or `disabledLyrics` parameters.

## Web integration

### Worker (`web/src/worker/jianpu.worker.ts`)

| Request | `{ type: 'generateSplitPdf', source, id, baseName }` |
| Response OK | `{ type: 'splitPdf', id, zip: ArrayBuffer }` (transferable) |
| Response err | `{ type: 'splitPdfErr', id, diagnostics }` |

### Hook (`web/src/hooks/useJianpuWorker.ts`)

- State: `splitPdfExporting: boolean`.
- Callback: `exportSplitPdf()`.
- `baseName`: active file stem — `demo.jianpu` → `"demo"` (reuse logic from `pdfFilenameFromActiveFile`, strip extension only).
- Download: `{baseName}.zip` via blob URL.
- `pdfExporting` and `splitPdfExporting` are mutually exclusive.

### Preview UI (`web/src/components/Preview.tsx`)

- New button: **Export parts (ZIP)** beside **Export PDF**.
- Visible when `pdfAvailable`.
- Enabled when: not rendering, not exporting (either kind), `parts.length > 0`.
- Busy label: **Exporting parts…**.

### App (`web/src/App.tsx`)

Pass new hook values into `Preview`. No changes to part toggle behavior.

## Error handling & edge cases

| Case | Behavior |
|------|----------|
| Parse/group error | Show diagnostics in ErrorPanel; no download |
| `parts.length === 0` | Button disabled |
| Single part | ZIP with one file: `{baseName} - {abbrev}.pdf` |
| Export during render | Both export buttons disabled |
| Concurrent exports | Block second export while one is in flight |
| WASM without `pdf` feature | Both PDF buttons hidden |
| Large scores | **Exporting parts…** for full duration; no per-part progress |

## Testing

### Rust (`src/lib.rs`)

- Multi-part input → N `SplitPdfEntry` values with correct filenames and valid PDF headers (`%PDF`).
- Single-part input → one entry with `{base} - {track}.pdf` naming.
- Invalid source → `Err`.

### WASM (`crates/jianpu-wasm`)

- `demo.jianpu` → OK, ZIP magic bytes (`PK`), expected entry names inside.

### CLI regression

- `split_tracks_generates_one_pdf_per_track` and `split_tracks_with_output_stem` still pass after refactor.

### Web

- Manual smoke test only for v1.

## Dependencies

```toml
# Cargo.toml [features]
pdf = [..., "dep:zip"]

[dependencies]
zip = { version = "2", default-features = false, features = ["deflate"], optional = true }
```

## Performance note

Split-track PDF generation is intentionally sequential. Parallel iteration would not help the WASM path; shared font setup across parts is a possible future CLI optimization, not in scope here.
