# TODO: unenforced cross-boundary invariants (Rust ↔ TS)

Each item below is a "simple" invariant (finite tags, field names, fixed shapes) that a
type system could enforce but currently doesn't — a typo/rename compiles clean on both
`cargo build` and `tsc` and only breaks at runtime. See the "Cross-boundary invariants
(Rust ↔ TS)" section in `AGENTS.md` for the general principle (tagged unions / make
illegal states unrepresentable) to apply when fixing these.

Clear them one at a time — each is independent and can be its own commit.

---

## [x] 1. GM instrument/percussion `"N: Name"` labels (highest risk)

The leading GM program number is embedded in a free-text label string and re-parsed by
splitting on `:` — the string is the *only* channel carrying the number, on both sides,
in three independent hand-written places.

- Rust parses it back out: `src/part_info.rs:32-43` (`instrument_program_to_label`,
  `instrument.value.split(':').next().and_then(|prefix| prefix.trim().parse::<u8>().ok())`).
  `InstrumentInfo` has no separate numeric field.
- Rust also *builds* the percussion version independently: `src/gm_percussion.rs:21`
  (`format!("{}: {}", entry.key, entry.name)`).
- TS hand-authors all 128 GM instrument entries in this exact shape:
  `web/src/utils/gmInstruments.ts:41+`.
- TS re-implements the same `split(':')` parsing independently, twice:
  `web/src/components/SoundfontSearchModal.tsx:87` and `:202`.

**Failure mode:** a typo'd/duplicated/missing leading number in any of the 128 entries
compiles fine everywhere; the instrument silently resolves to the wrong (or "Unknown")
GM program — wrong sound plays, and/or the wrong label round-trips through `# parts`.

**Direction:** give `InstrumentInfo` (and the percussion equivalent) a real `program: u8`
field instead of deriving it from the label text; expose it to TS as a proper field
instead of a string to re-split.

---

## [x] 2. `TransparentRectRoleOut` → `data-variant` kebab-case strings

Same shape as the `data-tag`/`TagOut` issue below (item 6), but for `data-variant` — and
with more call sites. (This item's own "already-fixed `data-tag` issue" aside was stale:
at the time this item was fixed, `data-tag` was *not* actually centralized anywhere —
`TagOut` was still hand-typed as `data-tag` strings independently in `PreviewSvgRenderer.tsx`
and ~7 other TS files, exactly the shape `AGENTS.md`'s "Cross-boundary invariants" section
names as its running example. See item 6 for the actual fix.)

- Source: `TransparentRectRoleOut` enum, `crates/jianpu-wasm/src/svg_types.rs:20-30`.
- Hand-mapped to kebab-case strings in `web/src/components/PreviewSvgRenderer.tsx:45+`
  (`transparentRectRoleToDataVariant`), plus a standalone literal
  `data-variant="playback-cursor-rect"` at line 308 for `PlaybackCursorRect`.
- Re-typed as raw strings independently in:
  - `web/src/preview.css:220,227,234,241,248`
  - `web/src/index.css:52,57,61-62,66-67`
  - `web/src/components/usePlaybackCursor.ts:50,92,104`
  - `web/src/components/previewRangeHighlights.ts:107,121`
  - `web/src/components/previewLabelRangeHighlights.ts:38,100,154,189,243,276,331`

**Failure mode:** a mismatch anywhere compiles fine; the CSS rule or `querySelector`
silently matches nothing — hover states, drag highlights, playback-cursor lookup quietly
stop working for that element, with zero error anywhere.

**Direction:** one shared TS constant/type for the 9 `data-variant` values (e.g. a
`const DATA_VARIANT = {...} as const` derived list), with the switch and every CSS/TS
consumer referencing it instead of re-typing the literal. (Note: `preview.css`/`index.css`
are plain CSS, so they can't reference a TS constant directly — consider a codegen step,
a shared list with a test that asserts the CSS file contains each value, or moving those
selectors to be generated/injected rather than hand-written.)

---

## [x] 3. `PartMode` wasm string protocol (`"chords"`/`"notes"`/`"percussion"`/`"follow[...]"`)

Two independent hand-written parsers for the same concept, not derived from the serde
tags that already exist for it.

- Rust hand-parses these literal strings: `src/source_edit/mod.rs:12-23` (`PartMode::parse`).
- This is entirely independent of `PartDeclarationModeOut`'s existing serde tags
  (`crates/jianpu-wasm/src/types.rs:74`), consumed as TS's `PartMode`
  (`web/src/types.ts:16`).
- TS reconstructs the raw strings to send across the boundary in
  `web/src/worker/jianpu.worker.ts:114-118` (`modeToWasmString`).

**Failure mode:** typed on the TS input (`mode: PartMode`) but not on the Rust parser
side — if the two string sets drift, `PartMode::parse` silently returns `None` and
`update_part_declaration_source` (`src/source_edit/part_declarations.rs:93-95`) falls
through to `return source.to_owned()`: the "Edit Parts" UI's mode change is silently
dropped with no error shown to the user.

**Direction:** have `PartMode::parse` and `PartDeclarationModeOut` derive from (or be
tested against) the same tag list, or route the wasm call through the existing
`PartDeclarationModeOut` tagged union instead of a hand-built string.

---

## [x] 4. `TextStyle` field/kind names mirrored as bare string lists

- Rust matches field-name strings in `src/parser/text_style_parser.rs:33,40,47,54,61,68,75`
  (`"font_size"`, `"horizontal_padding_pt"`, `"vertical_padding_pt"`, `"bold"`,
  `"italic"`, `"underline"`, `"font_family"`) and kind-name strings in
  `src/parser/metadata_parser.rs:161-169` (`"part_legend"`, `"measure_number"`,
  `"section_label"`, `"page_number"`, `"part_label"`, `"note_dash"`, plus
  `title`/`subtitle`/`author`/etc.).
- TS mirrors both lists as untyped `as const` string arrays:
  `web/src/utils/textStyleFields.ts:5-19` (`textStyleKinds`) and `:25-53`
  (`textStyleNumericComponents`, `textStyleBooleanComponents`, `fontFamilyValues`).

**Failure mode:** a name typo'd differently between the two lists compiles cleanly on
both sides; the Rust parser silently treats the corresponding source line's field as
unrecognized (ignored, or a generic diagnostic rather than a specific one), or the TS
editor UI offers a field the Rust parser never recognizes, so edits to it are silent
no-ops on the next round-trip.

**Direction:** expose the Rust-side kind/field name lists to TS (wasm-exported constant
or generated from the same source) instead of a hand-copied array, or add a test that
fails when the two lists diverge.

---

## [ ] 5. Monaco directive-keyword regex vs. Rust directive lexer (low/cosmetic)

- Rust lexes directive keys structurally, e.g. `src/parser/score/timed_parser/timed_lexer.rs:174-175`
  plus scattered handling for `key=`, `time=`, `label=`,
  `merge_duplicate_measures_across_parts=`, `hide_resting_parts=` across the
  timed/interleaved directive parsers.
- TS hardcodes the same keyword set for editor syntax coloring only:
  `web/src/monacoJianpuLanguage.ts:39`
  (`/\b(bpm|key|time|label|merge_duplicate_measures_across_parts|hide_resting_parts)(?=\s*=)/`).

**Failure mode:** cosmetic only — Rust never consults this regex, so a drift just means a
valid/new directive keyword fails to get syntax-highlighted (or an invalid one gets
highlighted as if valid). Lowest priority; fix opportunistically.

**Direction:** lowest priority given no functional impact — consider only if touching
this file anyway; a shared keyword list (wasm-exported or generated) would still be the
correct fix in principle.

---

## [x] 6. `TagOut` → `data-tag` kebab-case strings (the running example in `AGENTS.md`)

- Source: `TagOut` enum, `crates/jianpu-wasm/src/svg_types.rs`.
- Was hand-mapped to kebab-case strings in `web/src/components/PreviewSvgRenderer.tsx`'s
  `groupAttrsForTag`, and re-typed independently as raw `data-tag="..."` string literals
  in `web/src/components/clickableElementId.ts` (its own hand-typed
  `switch (dataset.tag)`), plus hand-formatted `` `[data-tag="..."][data-...="${x}"]` ``
  selector strings and hand-parsed `dataset.partIndex`/`Number(...)` fields duplicated
  across `usePlaybackCursor.ts`, `previewRangeHighlights.ts`,
  `previewLabelRangeHighlights.ts`, `previewLabelSelection.ts`, and `Preview.tsx` — 52
  occurrences total, the largest surface of any item in this file.

**Failure mode:** a typo'd/renamed `TagOut` variant compiles clean everywhere; the
corresponding `querySelector`/`closest`/`dataset` read silently matches nothing or
returns `undefined` — click handling, playback cursor, drag-range selection, and
scroll-to-measure quietly stop working for that element, with zero error anywhere.

**Fix:** `web/src/dataAttributes.ts` (renamed from `dataVariant.ts`, which already held
the equivalent `DATA_VARIANT`/item 2 fix) now also holds `DATA_TAG`
(`satisfies Record<TagOut['type'], string>`), the writer (`groupAttrsForTag`) and reader
(`tagFromElement`) for a group's `data-*` attributes, and typed selector-builders
(`groupTagSelector`/`measureGroupSelector`/`measureGroupByEndSelector`/
`noteGroupSelector`) — every consumer above now calls into these instead of re-deriving
the tag ↔ attribute mapping itself.

**Follow-ups intentionally left out of that fix's scope:**

- `SvgElementOut.variant: Option<String>` on text elements is a **separate**,
  already-raw Rust string, unrelated to `TransparentRectRoleOut`/`DATA_VARIANT` — a
  styling hook, not an identity concern. Not covered by `DATA_VARIANT` or this item;
  possible future item if it ever needs the same treatment.
- `ClickableElementId` (Rust `selection_range_types.rs`, hand-mirrored in
  `clickableElementId.ts`) is a separate parallel enum used for the *input* direction
  (TS → Rust, `resolve_selection_range` args) and uses different field-name casing
  (`sourcePartIndex`) than `TagOut` (`source_part_index`). Unifying the two Rust-side
  enums so there's only one vocabulary crossing the boundary in both directions would be
  a larger, separate Rust-side change. `clickableElementIdFromElement` was updated to
  derive from `tagFromElement` instead of re-parsing `dataset` independently (removing
  the TS-side duplication), but the two-enum split itself remains.
