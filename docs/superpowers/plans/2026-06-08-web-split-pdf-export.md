# Web Split-Track PDF Export (ZIP) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an **Export parts (ZIP)** button to the web UI that downloads one PDF per part (CLI `--split-tracks` semantics) via a shared Rust library function and WASM export.

**Architecture:** Move split-track helpers into `jianpu-generator`, add `write_split_pdfs_from_source` + `zip_split_pdfs`, refactor CLI PDF split path to call the library, expose `generate_split_pdfs` from WASM, and wire a new worker message through React hook → Preview button.

**Tech Stack:** Rust (`zip` crate), wasm-bindgen, React/TypeScript, existing `pdf`/`svg2pdf` pipeline.

---

## File map

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Optional `zip` dep on `pdf` feature |
| `src/lib.rs` | `SplitPdfEntry`, track helpers, `write_split_pdfs_from_source`, `zip_split_pdfs`, unit tests |
| `src/main.rs` | Refactor `generate_pdf` split path; import helpers from lib |
| `crates/jianpu-wasm/src/lib.rs` | `generate_split_pdfs` WASM export + test |
| `web/src/worker/jianpu.worker.ts` | `generateSplitPdf` worker message |
| `web/src/hooks/useJianpuWorker.ts` | `exportSplitPdf`, `splitPdfExporting`, ZIP download |
| `web/src/components/Preview.tsx` | Second export button |
| `web/src/App.tsx` | Wire new props |

---

### Task 1: Add `zip` dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add optional `zip` dependency on `pdf` feature**

In `Cargo.toml`, update the `pdf` feature and add the dependency:

```toml
pdf = ["dep:svg2pdf", "dep:pdf-writer", "dep:zip"]

# in [dependencies]
zip = { version = "2", default-features = false, features = ["deflate"], optional = true }
```

- [ ] **Step 2: Verify it resolves**

Run: `cargo check --features pdf 2>&1`

Expected: compiles with no errors.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "build: add zip crate as optional pdf dependency"
```

---

### Task 2: Move track helpers to library (TDD)

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add helpers and `split_pdf_filename` to `src/lib.rs`**

Add after `apply_lyrics_filter` (before the `wav` section):

```rust
/// Sanitize a track name for use in filenames (mirrors CLI).
pub fn sanitize_track_name(name: &str) -> String {
    name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
}

/// Collect unique part names from score measures (order of first appearance).
pub fn collect_track_names(score: &Score) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut names = Vec::new();
    for measure in &score.measures {
        for part in &measure.parts {
            if let PartRow::Notes(part_slice) = part {
                if let Some(name) = part_slice.name.as_ref() {
                    if seen.insert(name.clone()) {
                        names.push(name.clone());
                    }
                }
            }
        }
    }
    names
}

/// Build a split-track PDF filename: `{base_name} - {track}.pdf`.
pub fn split_pdf_filename(base_name: &str, track: &str) -> String {
    format!("{} - {}.pdf", base_name, sanitize_track_name(track))
}

/// Track list for split export. Empty `tracks_filter` → all score tracks;
/// falls back to `[parts]` declaration abbreviations when score has no named parts.
pub fn split_track_names(
    source: &str,
    filename: &str,
    score: &Score,
    tracks_filter: &[String],
) -> Result<Vec<String>, JianPuError> {
    let mut names = if tracks_filter.is_empty() {
        collect_track_names(score)
    } else {
        tracks_filter.to_vec()
    };
    if names.is_empty() {
        names = list_parts_from_source(source, filename)?
            .into_iter()
            .map(|part| part.abbreviation)
            .collect();
    }
    Ok(names)
}
```

Add `use ast::grouped::PartRow;` at the top if not already imported via `PartRow` usage (the file already imports `PartRow`).

- [ ] **Step 2: Write failing unit test for `split_track_names` fallback**

In `src/lib.rs` `mod tests`, add:

```rust
#[test]
fn split_track_names_falls_back_to_part_declarations() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Melody = notes lyrics\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "a b c d\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let names = split_track_names(input, "test.jianpu", &score, &[]).unwrap();
    assert_eq!(names, vec!["Melody"]);
}

#[test]
fn split_pdf_filename_sanitizes_track_name() {
    assert_eq!(
        split_pdf_filename("song", "A1&T"),
        "song - A1&T.pdf"
    );
    assert_eq!(
        split_pdf_filename("song", "bad/name"),
        "song - bad-name.pdf"
    );
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test split_track_names split_pdf_filename --features pdf 2>&1`

Expected: PASS

- [ ] **Step 4: Update `src/main.rs` to use library helpers**

Remove local `sanitize_track_name`, `collect_track_names` from `main.rs`.

Replace calls:
- `sanitize_track_name(track)` → `jg::sanitize_track_name(track)`
- `collect_track_names(score)` → `jg::collect_track_names(score)`
- `track_output_path` body uses `jg::sanitize_track_name(track)` (already does via local fn — update to `jg::`)

Delete the local `fn sanitize_track_name` and `fn collect_track_names` definitions at the bottom of `main.rs`.

- [ ] **Step 5: Verify CLI still compiles**

Run: `cargo build 2>&1`

Expected: compiles with no errors.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/main.rs
git commit -m "refactor: move split-track name helpers into library"
```

---

### Task 3: `write_split_pdfs_from_source` (TDD)

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing tests**

Add to `src/lib.rs` `mod tests` (inside `#[cfg(feature = "pdf")]` module or guard individual tests):

```rust
#[cfg(feature = "pdf")]
mod split_pdf_tests {
    use super::*;

    fn multi_track_input() -> &'static str {
        concat!(
            "[metadata]\n",
            "title = \"test score\"\n",
            "author = \"tester\"\n",
            "\n",
            "[parts]\n",
            "Soprano 1 (S1) = notes lyrics\n",
            "Soprano 2 (S2) = notes lyrics\n",
            "\n",
            "[score]\n",
            "(time=4/4 key=C4 bpm=120)\n",
            "1 2 3 4\n",
            "do re mi fa\n",
            "5 6 7 1\n",
            "sol la ti do\n",
        )
    }

    #[test]
    fn write_split_pdfs_from_source_produces_one_pdf_per_track() {
        let entries =
            write_split_pdfs_from_source(multi_track_input(), "test.jianpu", "test_split", &[])
                .unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].track_name, "S1");
        assert_eq!(entries[0].filename, "test_split - S1.pdf");
        assert_eq!(entries[1].track_name, "S2");
        assert_eq!(entries[1].filename, "test_split - S2.pdf");
        assert_eq!(&entries[0].pdf[0..4], b"%PDF");
        assert_eq!(&entries[1].pdf[0..4], b"%PDF");
    }

    #[test]
    fn write_split_pdfs_from_source_single_part_uses_split_naming() {
        let input = concat!(
            "[metadata]\n",
            "title = \"t\"\n",
            "author = \"a\"\n",
            "\n",
            "[parts]\n",
            "Melody = notes lyrics\n",
            "\n",
            "[score]\n",
            "(time=4/4 key=C4 bpm=120)\n",
            "1 2 3 4\n",
            "a b c d\n",
        );
        let entries =
            write_split_pdfs_from_source(input, "test.jianpu", "song", &[]).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].filename, "song - Melody.pdf");
        assert_eq!(&entries[0].pdf[0..4], b"%PDF");
    }

    #[test]
    fn write_split_pdfs_from_source_invalid_source_errors() {
        let err = write_split_pdfs_from_source("not valid", "test.jianpu", "song", &[])
            .unwrap_err();
        assert!(!err.message.is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test write_split_pdfs_from_source --features pdf 2>&1`

Expected: FAIL — `write_split_pdfs_from_source` not found

- [ ] **Step 3: Implement `SplitPdfEntry` and `write_split_pdfs_from_source`**

Add before `#[cfg(test)]` in `src/lib.rs`:

```rust
/// One PDF produced by split-track export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPdfEntry {
    pub track_name: String,
    pub filename: String,
    pub pdf: Vec<u8>,
}

/// Parse once, render one PDF per track (CLI `--split-tracks` semantics).
///
/// `tracks_filter`: empty → all tracks; non-empty → only listed abbreviations.
/// Lyrics are always included (no lyrics filter).
#[cfg(feature = "pdf")]
pub fn write_split_pdfs_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
    tracks_filter: &[String],
) -> Result<Vec<SplitPdfEntry>, JianPuError> {
    let score = compile(source, filename)?;
    let track_names = split_track_names(source, filename, &score, tracks_filter)?;
    let mut entries = Vec::with_capacity(track_names.len());
    for track in track_names {
        let mut score_clone = score.clone();
        filter_tracks(&mut score_clone, std::slice::from_ref(&track));
        let svgs = render_svgs(&score_clone);
        let pdf = pdf::write_pdf(&svgs)?;
        entries.push(SplitPdfEntry {
            track_name: track.clone(),
            filename: split_pdf_filename(base_name, &track),
            pdf,
        });
    }
    Ok(entries)
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test write_split_pdfs_from_source split_track_names split_pdf_filename --features pdf 2>&1`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add write_split_pdfs_from_source for split-track PDF export"
```

---

### Task 4: `zip_split_pdfs` (TDD)

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test**

Add to the `split_pdf_tests` module:

```rust
#[test]
fn zip_split_pdfs_contains_named_entries() {
    use std::io::Read;
    use zip::ZipArchive;

    let entries = write_split_pdfs_from_source(
        multi_track_input(),
        "test.jianpu",
        "test_split",
        &[],
    )
    .unwrap();
    let zip_bytes = zip_split_pdfs(&entries).unwrap();
    assert_eq!(&zip_bytes[0..2], b"PK");

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    assert_eq!(archive.len(), 2);
    let mut names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec!["test_split - S1.pdf".to_string(), "test_split - S2.pdf".to_string()]
    );

    let mut first = archive.by_name("test_split - S1.pdf").unwrap();
    let mut buf = Vec::new();
    first.read_to_end(&mut buf).unwrap();
    assert_eq!(&buf[0..4], b"%PDF");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test zip_split_pdfs --features pdf 2>&1`

Expected: FAIL — `zip_split_pdfs` not found

- [ ] **Step 3: Implement `zip_split_pdfs`**

Add after `write_split_pdfs_from_source`:

```rust
#[cfg(feature = "pdf")]
pub fn zip_split_pdfs(entries: &[SplitPdfEntry]) -> Result<Vec<u8>, JianPuError> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buffer = Vec::new();
    {
        let mut writer = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for entry in entries {
            writer
                .start_file(&entry.filename, options)
                .map_err(|e| JianPuError::new(error::Span::new(0, 0), format!("zip start_file: {e}")))?;
            writer
                .write_all(&entry.pdf)
                .map_err(|e| JianPuError::new(error::Span::new(0, 0), format!("zip write: {e}")))?;
        }
        writer
            .finish()
            .map_err(|e| JianPuError::new(error::Span::new(0, 0), format!("zip finish: {e}")))?;
    }
    Ok(buffer)
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test zip_split_pdfs write_split_pdfs --features pdf 2>&1`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat: zip split-track PDF entries for browser download"
```

---

### Task 5: Refactor CLI `generate_pdf` split path

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Refactor `generate_pdf` to use library function**

Replace the `try_split_tracks` block inside `generate_pdf` with:

```rust
fn generate_pdf(opts: &GenerateInput) -> Result<(), jg::error::JianPuError> {
    if opts.split_tracks {
        let content = std::fs::read_to_string(&opts.input).map_err(|e| {
            jg::error::JianPuError::new(
                jg::error::Span::new(0, 0),
                format!("could not read {:?}: {e}", opts.input),
            )
        })?;
        let (_, base_name) = split_track_base(&opts.input, opts.output.as_deref());
        let entries = jg::write_split_pdfs_from_source(
            &content,
            opts.input.to_str().unwrap_or("input.jianpu"),
            &base_name,
            &opts.tracks,
        )?;
        if entries.is_empty() {
            eprintln!(
                "warning: --split-tracks given but score has no named tracks; generating single file"
            );
        } else {
            let (base, _) = split_track_base(&opts.input, opts.output.as_deref());
            for entry in &entries {
                let track_path = base.with_file_name(&entry.filename);
                write_file(&track_path, &entry.pdf)?;
                println!("written to {track_path:?}");
            }
            return Ok(());
        }
    }

    let score = parse_and_group(&opts.input)?;
    let mut score = score;
    jg::filter_tracks(&mut score, &opts.tracks);
    let svgs = jg::render_svgs(&score);
    let pdf_bytes = jg::pdf::write_pdf(&svgs)?;
    let output_path =
        output_stem(&opts.input, &opts.tracks, opts.output.as_deref()).with_extension("pdf");
    write_file(&output_path, &pdf_bytes)?;
    println!("written to {output_path:?}");
    Ok(())
}
```

Note: preserve existing fallback when `entries.is_empty()` — fall through to single-file generation (matches prior CLI warning behavior).

- [ ] **Step 2: Run integration tests**

Run: `cargo test --test integration split_tracks 2>&1`

Expected: PASS (`split_tracks_generates_one_pdf_per_track`, `split_tracks_with_output_stem`)

- [ ] **Step 3: Run full test suite**

Run: `cargo test --features pdf 2>&1`

Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "refactor: CLI split-track PDF uses write_split_pdfs_from_source"
```

---

### Task 6: WASM `generate_split_pdfs` export

**Files:**
- Modify: `crates/jianpu-wasm/src/lib.rs`

- [ ] **Step 1: Write failing WASM test**

Add to `crates/jianpu-wasm/src/lib.rs` `mod tests`:

```rust
#[cfg(feature = "pdf")]
#[test]
fn demo_jianpu_generates_split_pdf_zip() {
    use std::io::Read;
    use zip::ZipArchive;

    let source = include_str!("../../../demo.jianpu");
    let resp = generate_split_pdfs_response(source, "demo");
    match resp {
        GenerateSplitPdfsResponse::Ok { zip } => {
            assert!(zip.len() > 4);
            assert_eq!(&zip[0..2], b"PK");
            let cursor = std::io::Cursor::new(zip);
            let mut archive = ZipArchive::new(cursor).unwrap();
            assert!(archive.len() >= 1);
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).unwrap();
                let name = file.name().to_string();
                assert!(
                    name.starts_with("demo - ") && name.ends_with(".pdf"),
                    "unexpected zip entry: {name}"
                );
                let mut buf = [0u8; 4];
                file.read_exact(&mut buf).unwrap();
                assert_eq!(&buf, b"%PDF");
            }
        }
        GenerateSplitPdfsResponse::Err { diagnostics } => {
            panic!(
                "demo.jianpu failed in wasm split pdf path: {}",
                diagnostics[0].message
            );
        }
    }
}
```

Add `zip` as dev-dependency in `crates/jianpu-wasm/Cargo.toml`:

```toml
[dev-dependencies]
zip = { version = "2", default-features = false, features = ["deflate"] }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p jianpu-wasm demo_jianpu_generates_split_pdf_zip --features pdf 2>&1`

Expected: FAIL — symbols not found

- [ ] **Step 3: Implement response type and functions**

Near existing `GeneratePdfResponse`, add:

```rust
#[cfg(feature = "pdf")]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
enum GenerateSplitPdfsResponse {
    Ok { zip: Vec<u8> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[cfg(feature = "pdf")]
fn generate_split_pdfs_response(source: &str, base_name: &str) -> GenerateSplitPdfsResponse {
    use jianpu_generator::{write_split_pdfs_from_source, zip_split_pdfs};

    match write_split_pdfs_from_source(source, "input.jianpu", base_name, &[]) {
        Ok(entries) => match zip_split_pdfs(&entries) {
            Ok(zip) => GenerateSplitPdfsResponse::Ok { zip },
            Err(e) => GenerateSplitPdfsResponse::Err {
                diagnostics: vec![diagnostic_from_error(source, e)],
            },
        },
        Err(e) => GenerateSplitPdfsResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, e)],
        },
    }
}
```

Add JS export mirroring `generate_pdf`:

```rust
#[cfg(feature = "pdf")]
fn generate_split_pdfs_to_js(source: &str, base_name: &str) -> JsValue {
    use js_sys::{Object, Reflect, Uint8Array};

    match generate_split_pdfs_response(source, base_name) {
        GenerateSplitPdfsResponse::Ok { zip } => {
            let obj = Object::new();
            let _ = Reflect::set(&obj, &JsValue::from_str("status"), &JsValue::from_str("ok"));
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("zip"),
                &Uint8Array::from(zip.as_slice()),
            );
            obj.into()
        }
        GenerateSplitPdfsResponse::Err { diagnostics } => {
            to_js_value(&GenerateSplitPdfsResponse::Err { diagnostics })
        }
    }
}

#[cfg(feature = "pdf")]
#[wasm_bindgen]
pub fn generate_split_pdfs(source: &str, base_name: &str) -> JsValue {
    generate_split_pdfs_to_js(source, base_name)
}
```

- [ ] **Step 4: Rebuild WASM package**

Run: `cd web && pnpm run build:wasm 2>&1`

Expected: builds successfully

- [ ] **Step 5: Run WASM tests**

Run: `cargo test -p jianpu-wasm --features pdf 2>&1`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/jianpu-wasm/Cargo.toml crates/jianpu-wasm/src/lib.rs
git commit -m "feat(wasm): expose generate_split_pdfs as ZIP bytes"
```

---

### Task 7: Web worker split-PDF message

**Files:**
- Modify: `web/src/worker/jianpu.worker.ts`
- Modify: `web/src/types.ts` (only if new shared types needed — prefer keeping types in worker file)

- [ ] **Step 1: Extend worker request/response types**

In `web/src/worker/jianpu.worker.ts`, add to imports/types:

```typescript
type GenerateSplitPdfResult =
  | { status: 'ok'; zip: Uint8Array | number[] }
  | { status: 'err'; diagnostics: Diagnostic[] }

const generateSplitPdfs =
  'generate_split_pdfs' in jianpuWasm
    ? (jianpuWasm.generate_split_pdfs as (
        source: string,
        baseName: string,
      ) => GenerateSplitPdfResult)
    : null
```

Extend `WorkerRequest`:

```typescript
  | {
      type: 'generateSplitPdf'
      source: string
      id: number
      baseName: string
    }
```

Extend `WorkerResponse`:

```typescript
  | { type: 'splitPdf'; id: number; zip: ArrayBuffer }
  | { type: 'splitPdfErr'; id: number; diagnostics: Diagnostic[] }
```

Update `ready` postMessage to include `splitPdfAvailable: generateSplitPdfs !== null` (optional — can reuse `pdfAvailable` since same feature gate).

- [ ] **Step 2: Handle `generateSplitPdf` in worker**

Add handler before the render branch:

```typescript
  if (msg.type === 'generateSplitPdf') {
    if (!generateSplitPdfs) {
      postMessage({
        type: 'splitPdfErr',
        id: msg.id,
        diagnostics: [
          {
            severity: 'error',
            message: 'Split PDF export is not available in this build.',
            span: { start: 0, end: 0 },
          },
        ],
      } satisfies WorkerResponse)
      return
    }

    const result = generateSplitPdfs(msg.source, msg.baseName)
    if (result.status === 'ok') {
      const zipBuffer = binaryBufferFromResult(result.zip)
      postMessage(
        {
          type: 'splitPdf',
          id: msg.id,
          zip: zipBuffer,
        } satisfies WorkerResponse,
        { transfer: [zipBuffer] },
      )
      return
    }

    postMessage({
      type: 'splitPdfErr',
      id: msg.id,
      diagnostics: result.diagnostics,
    } satisfies WorkerResponse)
    return
  }
```

- [ ] **Step 3: Typecheck**

Run: `cd web && pnpm exec tsc -b 2>&1`

Expected: no errors (WASM pkg must be rebuilt first from Task 6)

- [ ] **Step 4: Commit**

```bash
git add web/src/worker/jianpu.worker.ts
git commit -m "feat(web): handle generateSplitPdf in jianpu worker"
```

---

### Task 8: Hook — `exportSplitPdf` and download

**Files:**
- Modify: `web/src/hooks/useJianpuWorker.ts`

- [ ] **Step 1: Add helpers and state**

Add after `pdfFilenameFromActiveFile`:

```typescript
function zipFilenameFromActiveFile(activeFile: string): string {
  if (activeFile.endsWith('.jianpu')) {
    return activeFile.replace(/\.jianpu$/, '.zip')
  }
  return `${activeFile}.zip`
}

function baseNameFromActiveFile(activeFile: string): string {
  if (activeFile.endsWith('.jianpu')) {
    return activeFile.replace(/\.jianpu$/, '')
  }
  return activeFile
}

function downloadZip(bytes: ArrayBuffer, filename: string) {
  const url = URL.createObjectURL(
    new Blob([bytes], { type: 'application/zip' }),
  )
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
}
```

Extend `JianpuWorkerState`:

```typescript
  splitPdfExporting: boolean
  exportSplitPdf: () => void
```

Add refs/state: `splitPdfRequestIdRef`, `latestSplitPdfIdRef`, `splitPdfExporting`.

- [ ] **Step 2: Handle worker responses**

In `worker.onmessage`, add before pdf handler:

```typescript
      if (msg.type === 'splitPdf') {
        if (msg.id !== latestSplitPdfIdRef.current) return
        setSplitPdfExporting(false)
        downloadZip(msg.zip, zipFilenameFromActiveFile(activeFileRef.current))
        return
      }

      if (msg.type === 'splitPdfErr') {
        if (msg.id !== latestSplitPdfIdRef.current) return
        setSplitPdfExporting(false)
        setDiagnostics(msg.diagnostics)
        return
      }
```

- [ ] **Step 3: Add `exportSplitPdf` callback**

```typescript
  const exportSplitPdf = useCallback(() => {
    const worker = workerRef.current
    if (!worker || pdfExporting || splitPdfExporting) return

    const id = ++splitPdfRequestIdRef.current
    latestSplitPdfIdRef.current = id
    setSplitPdfExporting(true)

    const payload: WorkerRequest = {
      type: 'generateSplitPdf',
      source: sourceRef.current,
      id,
      baseName: baseNameFromActiveFile(activeFileRef.current),
    }
    worker.postMessage(payload)
  }, [pdfExporting, splitPdfExporting])
```

Update `exportPdf` guard: `if (!worker || pdfExporting || splitPdfExporting) return`

Return new fields from hook.

- [ ] **Step 4: Typecheck**

Run: `cd web && pnpm exec tsc -b 2>&1`

Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add web/src/hooks/useJianpuWorker.ts
git commit -m "feat(web): add exportSplitPdf hook with ZIP download"
```

---

### Task 9: Preview button and App wiring

**Files:**
- Modify: `web/src/components/Preview.tsx`
- Modify: `web/src/App.tsx`

- [ ] **Step 1: Extend Preview props and UI**

In `Preview.tsx`:

```typescript
interface PreviewProps {
  svgs: string[]
  rendering: boolean
  wavUrl?: string | null
  audioAvailable?: boolean
  pdfAvailable?: boolean
  pdfExporting?: boolean
  onExportPdf?: () => void
  splitPdfExporting?: boolean
  onExportSplitPdf?: () => void
  partsCount?: number
  emptyMessage?: string
}
```

Add button after Export PDF:

```tsx
          {pdfAvailable ? (
            <button
              type="button"
              className="preview-export-btn"
              disabled={
                !canExportSplitPdf
              }
              onClick={onExportSplitPdf}
            >
              {splitPdfExporting ? 'Exporting parts…' : 'Export parts (ZIP)'}
            </button>
          ) : null}
```

Define:

```typescript
  const canExportSplitPdf =
    pdfAvailable &&
    (partsCount ?? 0) > 0 &&
    !rendering &&
    !pdfExporting &&
    !splitPdfExporting
```

- [ ] **Step 2: Wire App.tsx**

Destructure from hook: `splitPdfExporting`, `exportSplitPdf`.

Pass to Preview:

```tsx
            splitPdfExporting={splitPdfExporting}
            onExportSplitPdf={exportSplitPdf}
            partsCount={parts.length}
```

- [ ] **Step 3: Lint and typecheck**

Run: `cd web && pnpm run lint && pnpm exec tsc -b 2>&1`

Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add web/src/components/Preview.tsx web/src/App.tsx
git commit -m "feat(web): add Export parts (ZIP) button to preview pane"
```

---

### Task 10: End-to-end verification

**Files:** (none — verification only)

- [ ] **Step 1: Run Rust tests**

Run: `cargo test --features pdf 2>&1`

Expected: all tests pass

- [ ] **Step 2: Run integration tests**

Run: `cargo test --test integration 2>&1`

Expected: all tests pass

- [ ] **Step 3: Rebuild WASM and web**

Run: `cd web && pnpm run build 2>&1`

Expected: successful build

- [ ] **Step 4: Manual smoke test**

Run: `cd web && pnpm dev`

1. Open the app in a browser.
2. Load a multi-part `.jianpu` file (e.g. demo).
3. Click **Export parts (ZIP)**.
4. Confirm download is `{filename}.zip`.
5. Unzip — verify `{filename} - {PartAbbrev}.pdf` for each part, each valid PDF.
6. Confirm **Export PDF** and **Export parts (ZIP)** are disabled while either export runs.

- [ ] **Step 5: Final commit if any fixups needed**

Only if smoke test required small fixes.

---

## Spec coverage checklist

| Spec requirement | Task |
|------------------|------|
| Shared `write_split_pdfs_from_source` | Task 3 |
| `zip_split_pdfs` + `zip` dep | Tasks 1, 4 |
| CLI refactor | Task 5 |
| WASM `generate_split_pdfs` | Task 6 |
| Worker message | Task 7 |
| Hook + ZIP download | Task 8 |
| Preview button + App wiring | Task 9 |
| All parts, lyrics included, ignore toggles | Task 3/6 (no filter params on web path) |
| Single-part split naming | Task 3 test |
| Mutual export exclusion | Task 8 |
| Integration regression | Task 5, 10 |
