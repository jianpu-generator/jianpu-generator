---
title: Architecture Documentation Design
date: 2026-06-13
status: approved
---

## Goal

Create `ARCHITECTURE.md` at the project root to document the rendering pipeline transformation layers and domain jargon. Audience: AI agents (Claude). Update `CLAUDE.md` with a rule to keep `ARCHITECTURE.md` current.

## Document Location

- `ARCHITECTURE.md` — project root
- Updated `CLAUDE.md` — add a rule requiring `ARCHITECTURE.md` to be updated when any layer's types, module structure, or entry point changes

## Structure

### Section 1 — Pipeline Overview

ASCII diagram showing the 7-stage transformation chain with input/output types and a one-line description of each output.

```
source (&str)
  → [parser]              → ParsedDocument
                             Raw event stream: notes, rests, chords, directives,
                             lyrics syllables. No measure grouping yet.
  → [grouper]             → Score
                             Measures grouped; ditto rows resolved; parts organized
                             into MultiPartMeasure slices.
  → [compiler]            → CompileResult
                             Logical grid: each note/rest assigned to a column,
                             underlines computed, slur spans recorded.
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

### Section 2 — Layer Details

Each layer entry contains:
- Module path
- Entry function signature
- Key types defined in that layer

Layers: Parser, Grouper, Compiler, Grid Layout, Coordinate Resolver, Renderer, Serializer.

### Section 3 — Glossary

| Term | Definition |
|------|-----------|
| System | A horizontal row of measures that fit on one line of a page. |
| Measure | One bar of music. The score is a flat sequence of `MultiPartMeasure`s. |
| Part | A single instrument or voice track. Declared in `[parts]`. |
| Part Slice | One part's notes and lyrics for a single measure (`PartSlice`). |
| Ditto | A measure where every input line was `"`. Rendered blank; audio output still uses resolved content. |
| Column | A logical horizontal slot in the compiler's grid. Each beat occupies one or more columns. |
| Quarter-beat | The smallest time unit for duration arithmetic. A quarter note = 4 quarter-beats. |
| Underline | A horizontal line below note heads indicating duration subdivision. `level=0` = half-beat, `level=1` = quarter-beat. |
| Octave Dot | A dot above/below a note head to shift its octave. Count = `octave.abs()`. |
| Note Dash | A visual `-` after a note head for each extra beat of duration. |
| Slur / Tie | A curved arc connecting notes. Both use `TieOrSlur` rendering. |
| Slur Span | The full logical extent of one slur/tie arc, possibly crossing measure or system boundaries (`SlurSpan`). |
| Decoration | Measure-level metadata on a `MeasureBlock`: BPM, time signature, section label, bar number. |
| Row Label | The part name displayed at the left margin of a system row. |
| RowId | A unique string identifier for a compiler row, used to correlate rows across layout stages. |

## CLAUDE.md Rule

Add to `CLAUDE.md`:

> When any of the following changes, update `ARCHITECTURE.md` in the same commit:
> - A layer's entry function signature or module path
> - A key type in any layer (added, removed, or renamed)
> - A new transformation layer added or an existing one removed
> - A new domain term introduced or an existing term redefined
