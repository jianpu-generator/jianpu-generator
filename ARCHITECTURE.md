# Architecture

## Pipeline Overview

```
source (&str)
  → [parser]              → ParsedDocument
                             Raw event stream: notes, rests, chords, directives,
                             lyrics syllables per measure. No note measure grouping yet.
  → [grouper]             → Score
                             Notes grouped into measures; lyrics paired to each
                             measure's lyric slots (tie-aware);
                             parts organized into MultiPartMeasure slices.
  → [compiler]            → CompileResult
                             Logical grid: each note/rest assigned to a column,
                             underlines computed, slur spans recorded.
  → [consolidator]        → CompileResult
                             Mixed notes+lyrics rows split; duplicate rows with
                             identical content suppressed per measure.
  → [grid_layout]         → Vec<GridPage>
                             Grid elements placed into rows with heights and column
                             counts; rows wrapped across pages; slur arcs resolved
                             to same-system or cross-system variants.
  → [coordinate_resolver] → Vec<AbsolutePage>
                             Every element assigned absolute x/y coordinates in
                             points; grid geometry collapsed to flat element list.
  → [renderer]            → Vec<SvgDocument>
                             SVG primitives (Text, Line, Circle, Path) produced
                             for each absolute element.
  → [serializer]          → Vec<String>
                             SVG strings, one per page, ready to write to disk.
```

## Layer Details

### Parser
- Module: `src/parser/`
- Entry: `parser::parse(source: &str, filename: &str) -> Result<ParsedDocument, IrrecoverableError>`
- Key types: `ParsedDocument`, `ParsedTimedTrack`, `ParsedScore`, `ScoreEvent`, `ParsedNote` (carries `Accidental` for melody notes; tie intent via `tie_to_next_span: Option<Span>` with `tie_to_next()` accessor), `ParsedRest`, `ParsedChordNote` (also carries `Accidental` and `tie_to_next_span`), `ParsedMetadata`, `JianPuPitch`, `Accidental` (`Sharp`/`Flat`/`Natural`; applies to both melody notes and chord notes), `Syllable`, `Soundfont` (vocal/piano/string; selects MIDI channel+program), `PartDecl` (carries `soundfont`, `volume`, `octave_offset`)

### Grouper
- Module: `src/grouper/`
- Entry: `grouper::group(doc: ParsedDocument) -> Result<Score, IrrecoverableError>`
- Key types: `Score`, `MultiPartMeasure`, `PartRow` (Timed), `PartSlice` (carries `soundfont`, `volume`, `octave_offset`), `Notes`, `NoteEvent`, `GroupedNote`, `GroupedRest`, `GroupedChordNote` (`GroupedNote`/`GroupedChordNote` use `tie_to_next_span` + `tie_to_next()` accessor), `GroupedMeasure` (intermediate: notes + paired lyrics per measure)

### Compiler
- Module: `src/compiler/`
- Entry: `compiler::compile(score: &Score) -> CompileResult`
- Key types: `CompileResult`, `MeasureBlock`, `MeasureRow`, `ColumnElement`, `ElementContent`, `SlurSpan`, `ArcKind`, `Decoration`

### Consolidator
- Module: `src/consolidator/`
- Entry: `consolidator::consolidate(result: CompileResult) -> CompileResult`
- Splits mixed `notes lyrics` rows into separate notes and lyrics rows, then removes duplicate rows within each measure when their `elements` are identical (labels and ids are not compared). `slur_spans` are passed through unchanged.

### Grid Layout
- Module: `src/grid_layout/`
- Entry: `grid_layout::layout(result: &CompileResult, config: &RenderConfig, header: &Header, width_pt: f32, height_pt: f32) -> Vec<GridPage>`
- Key types: `GridPage`, `GridRow`, `GridElement`, `GridContent`, `HAlign`, `VAlign`

### Coordinate Resolver
- Module: `src/coordinate_resolver/`
- Entry: `coordinate_resolver::resolve(pages: &[GridPage], note_number_width: f32) -> Vec<AbsolutePage>`
- Key types: `AbsolutePage`, `AbsoluteElement`, `AbsoluteContent`, `PostArcGridContent`
- `PostArcGridContent`: `GridContent` minus the three arc variants (`TieOrSlur`, `TieOrSlurTail`, `TieOrSlurHead`); arc variants are resolved before `grid_to_absolute` and must not appear in the coordinate-resolver layer.

### Renderer
- Module: `src/renderer/`
- Entry: `renderer::new_renderer::render_new(pages: &[AbsolutePage], config: &RenderConfig) -> Vec<SvgDocument>`
- Key types: `SvgDocument`, `SvgElement`, `SvgKind`, `SvgVariant`, `TransparentRectRole`
- `SvgElement.variant` is `Option<SvgVariant>`: `None` for group wrappers and highlight rects; `Some(...)` for musical/export drawable primitives
- `SvgKind::TransparentRect` carries a `role: TransparentRectRole` for CSS hover targets (`data-variant` in serializer/preview); roles are `MeasureClickTarget` and `SectionLabelBackground`

### Serializer
- Module: `src/serializer/`
- Entry: `serializer::serialize(docs: &[SvgDocument]) -> Vec<String>`

## Glossary

| Term | Definition |
|------|-----------|
| **System** | A horizontal row of measures that fit on one line of a page. The grid layout wraps measures into systems based on column count and page width. |
| **Measure** | One bar of music. The score is a flat sequence of `MultiPartMeasure`s. |
| **Part** | A single instrument or voice track (e.g. soprano, bass). Declared in `[parts]`. |
| **Part Slice** | One part's notes and lyrics for a single measure (`PartSlice`). |
| **Ditto** | A measure where every input line was `"`, meaning it repeats the previous measure. Rendered as blank; audio output still uses the resolved content. |
| **Column** | A logical horizontal slot in the compiler's grid. Each beat occupies one or more columns. |
| **Quarter-beat** | The smallest time unit used for duration arithmetic. A standard quarter note = 4 quarter-beats. |
| **Underline** | A horizontal line drawn below note heads to indicate duration subdivision. `level=0` = half-beat, `level=1` = quarter-beat. |
| **Octave Dot** | A dot drawn above or below a note head to shift its octave. Count = `octave.abs()`. |
| **Note Dash** | A visual `-` drawn after a note head for each extra beat of duration. |
| **Lyrics line** | One plain-text line per measure per `notes lyrics` part, tokenised into syllables and stored per measure (not as a global pool). |
| **Arc Span** | The full logical extent of one slur or tie arc, possibly crossing measure or system boundaries (`SlurSpan`). Carries `ArcKind` to distinguish ties from slurs. |
| **ArcKind** | Discriminant on `SlurSpan`: `Slur` (from `(…)` groups) or `Tie` (from `~`). Both render as arcs; the kind is available for future visual distinction. |
| **Decoration** | Measure-level metadata attached to a `MeasureBlock`: BPM, time signature, section label, bar number. |
| **Row Label** | The part name displayed at the left margin of a system row. |
| **RowId** | A unique string identifier for a compiler row, used to correlate rows across layout stages. |

## Web integration

The React app (`web/`) runs the compiler in a dedicated worker (`web/src/worker/jianpu.worker.ts`) backed by the `jianpu-wasm` crate. The main thread sends source text and asset bytes; the worker calls WASM exports and posts structured results back.

### Storage abstraction

- Module: `web/src/storage/`
- Key types: `StorageBackend` (`web/src/storage/types.ts`) — the interface both the browser-`localStorage` backend and a future GitHub backend implement; `FileStoreState` (`web/src/fileStore.ts`) remains the canonical in-memory shape used by every backend. `StorageBackend.load`/`createFile`/`duplicateFile`/`renameFile`/`deleteFile`/`restoreFile`/`saveContent` are async (to accommodate network-backed backends); `updateActiveContent` is sync with no persistence side effect. `saveContent` is the explicit call that persists the active file's content — debounce ownership lives in the hook layer, not the backend.
- `web/src/storage/localBackend.ts` — the only `StorageBackend` implementation so far. Thin async adapter over `fileStore.ts`'s pure, synchronous functions (`createFile`, `duplicateFile`, `renameFile`, `deleteFile`, `restoreFile`, `updateActiveContent`). Its `saveContent` is a no-op since `useLocalStorage` already persists on every state change.
- Entry: `useStorageBackend()` in `web/src/hooks/useStorageBackend.ts` — holds the `FileStoreState` in React state (via `useLocalStorage`, seeded synchronously through `localBackend`'s local-only read helpers) and exposes `store`/`setStore`/`backend`/`saveStatus`. Callers `await backend.xxxFile(store)` for structural operations, then `setStore` the result. Only `localBackend` is reachable currently — no `switchBackend()` yet, since there is nothing else to switch to.

### Source editing

- Module: `src/source_edit/`
- Entry: `source_edit::update_part_declaration(source, abbreviation, new_mode, new_soundfont, new_volume, new_octave_offset) -> Option<String>`
- Rewrites a single `# parts` declaration line in place (mode, optional quoted soundfont, optional volume `%`, optional octave offset). Used by the Edit Parts modal instead of any TypeScript parser.

### Part declarations (source-level)

- Entry: `list_part_declarations_from_source(source, filename, instruments) -> Result<Vec<SourcePartDeclaration>, IrrecoverableError>` in `src/lib.rs`
- Backed by `parts_parser::collect_source_raw_declarations()` — returns **raw** fields from each declaration line (before `follow[X]` inheritance). The Edit Parts modal displays these values.

### WASM exports (`crates/jianpu-wasm`)

| Export | Purpose |
|--------|---------|
| `list_parts(source, raw_instruments)` | Part summaries for preview toggles **and** `declarations: PartDeclarationOut[]` for the Edit Parts modal |
| `list_part_declarations(source, raw_instruments)` | Declarations only (re-list after a write) |
| `update_part_declaration(source, abbreviation, new_mode, new_soundfont, new_volume, new_octave_offset)` | Returns updated source; empty strings for soundfont/volume/octave mean “omit / default” |

Worker messages: `listParts` → `{ parts, declarations }`; `updatePartDeclaration` → `{ source, declarations }` (hook updates `partDeclarations` immediately, without waiting for the debounced re-render).
