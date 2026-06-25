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
- Key types: `ParsedDocument`, `ParsedTimedTrack`, `ParsedScore`, `ScoreEvent`, `ParsedNote`, `ParsedRest`, `ParsedChordNote`, `ParsedMetadata`, `JianPuPitch`, `Accidental`, `Syllable`

### Grouper
- Module: `src/grouper/`
- Entry: `grouper::group(doc: ParsedDocument) -> Result<Score, IrrecoverableError>`
- Key types: `Score`, `MultiPartMeasure`, `PartRow` (Timed), `PartSlice`, `Notes`, `NoteEvent`, `GroupedNote`, `GroupedRest`, `GroupedChordNote`, `GroupedMeasure` (intermediate: notes + paired lyrics per measure)

### Compiler
- Module: `src/compiler/`
- Entry: `compiler::compile(score: &Score) -> CompileResult`
- Key types: `CompileResult`, `MeasureBlock`, `MeasureRow`, `ColumnElement`, `ElementContent`, `SlurSpan`, `Decoration`

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
- Key types: `SvgDocument`, `SvgElement`, `SvgKind`

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
| **Slur Span** | The full logical extent of one slur/tie arc, possibly crossing measure or system boundaries (`SlurSpan`). |
| **Decoration** | Measure-level metadata attached to a `MeasureBlock`: BPM, time signature, section label, bar number. |
| **Row Label** | The part name displayed at the left margin of a system row. |
| **RowId** | A unique string identifier for a compiler row, used to correlate rows across layout stages. |
