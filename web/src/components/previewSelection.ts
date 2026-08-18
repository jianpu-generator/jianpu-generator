import type { LyricSpan, NoteSpan } from '../types'

/** One rendered note/rest, keyed the same way as `Tag::Note`'s
 * `data-part-index`/`data-note-id` SVG attributes. */
export interface NoteCell {
  sourcePartIndex: number
  noteId: number
}

/** Shape driving `getCellAtPoint`: which `data-tag` group to look up, and how
 * to parse a `Cell` out of that group's `dataset`. */
interface CellAtPointSpec<Cell> {
  tag: string
  parseCell: (dataset: DOMStringMap) => Cell | undefined
}

/**
 * Generic hit-test behind `getNoteAtPoint`/`getLyricAtPoint`/
 * `getPartLabelAtPoint`: reads the element under `(x, y)`, walks up to its
 * nearest `[data-tag="{spec.tag}"]` ancestor group, and parses a `Cell` out
 * of that group's `dataset` via `spec.parseCell`.
 */
function getCellAtPoint<Cell>(
  x: number,
  y: number,
  spec: CellAtPointSpec<Cell>,
): Cell | undefined {
  const el = document.elementFromPoint(x, y)
  if (!el) return undefined
  const group = el.closest(`[data-tag="${spec.tag}"]`)
  if (!group) return undefined
  return spec.parseCell((group as HTMLElement).dataset)
}

/** Parses a `dataset` string into an integer, or `undefined` if the string is
 * missing or not a valid integer — the common per-field step behind every
 * `parseCell` below. */
function parseDatasetInt(value: string | undefined): number | undefined {
  if (value === undefined) return undefined
  const parsed = Number.parseInt(value, 10)
  return Number.isNaN(parsed) ? undefined : parsed
}

export function getSectionLabelAtPoint(
  x: number,
  y: number,
): string | undefined {
  const el = document.elementFromPoint(x, y)
  if (!el) return undefined
  const group = el.closest('[data-tag="section-label"]')
  if (!group) return undefined
  return (group as HTMLElement).dataset.sectionLabel
}

export interface MeasureRange {
  start: number
  end: number
}

/** One rendered part-label click target, keyed the same way as
 * `Tag::PartLabel`'s `data-part-index`/`data-measure-index-start`/
 * `data-measure-index-end` SVG attributes — see `getPartLabelAtPoint`. */
export interface PartLabelHit {
  sourcePartIndex: number
  measureIndexStart: number
  measureIndexEnd: number
}

/** The part-label click target under the given point, if any — reads the
 * invisible `PartLabelClickTarget` rect's enclosing `Tag::PartLabel` group
 * (see `renderer::new_renderer::render_part_label_click_target`). */
export function getPartLabelAtPoint(
  x: number,
  y: number,
): PartLabelHit | undefined {
  return getCellAtPoint(x, y, {
    tag: 'part-label',
    parseCell: ({ partIndex, measureIndexStart, measureIndexEnd }) => {
      const sourcePartIndex = parseDatasetInt(partIndex)
      const start = parseDatasetInt(measureIndexStart)
      const end = parseDatasetInt(measureIndexEnd)
      if (
        sourcePartIndex === undefined ||
        start === undefined ||
        end === undefined
      )
        return undefined
      return {
        sourcePartIndex,
        measureIndexStart: start,
        measureIndexEnd: end,
      }
    },
  })
}

/** Every note/rest cell belonging to the given part-label hits — each hit
 * selects its own part's notes across its own `measureIndexStart..=measureIndexEnd`
 * (the whole system the label sits in), mirroring `noteCellsInMeasureRange`. */
export function noteCellsForPartLabels(
  noteSpans: NoteSpan[],
  hits: PartLabelHit[],
): NoteCell[] {
  return hits.flatMap((hit) =>
    noteSpans
      .filter(
        (span) =>
          span.sourcePartIndex === hit.sourcePartIndex &&
          span.measureIndex >= hit.measureIndexStart &&
          span.measureIndex <= hit.measureIndexEnd,
      )
      .map((span) => ({
        sourcePartIndex: span.sourcePartIndex,
        noteId: span.noteId,
      })),
  )
}

/** Every lyric syllable cell belonging to the given part-label hits —
 * the lyric-side mirror of `noteCellsForPartLabels`, so a part-label drag
 * selects the verse lyrics under its swept part rows alongside their notes. */
export function lyricCellsForPartLabels(
  lyricSpans: LyricSpan[],
  hits: PartLabelHit[],
): LyricCell[] {
  return hits.flatMap((hit) =>
    lyricSpans
      .filter(
        (span) =>
          span.sourcePartIndex === hit.sourcePartIndex &&
          span.measureIndex >= hit.measureIndexStart &&
          span.measureIndex <= hit.measureIndexEnd,
      )
      .map((span) => ({
        sourcePartIndex: span.sourcePartIndex,
        noteId: span.noteId,
        verse: span.verse,
      })),
  )
}

/**
 * The measure range under the given point. A merged multi-measure rest bar
 * carries a `start`/`end` wider than a single measure, so clicking anywhere
 * on it resolves to every source measure it represents.
 *
 * Deliberately does *not* use `elementFromPoint`/`elementsFromPoint`: a
 * measure's click-target rect touches its neighbors' flush (no gap — see
 * `resolve_measure_click_target`/`measure_column_bounds`), so a point
 * exactly on that shared edge is, in exact geometry, on the boundary of
 * *two* rects at once. Which of those two `elementsFromPoint` lists first is
 * an artifact of paint/z-order and sub-pixel rounding, not something this
 * code controls — so it isn't a reliable tie-break, and picking whichever
 * rect happens to win could resolve a click to the wrong (previous or next)
 * measure. Scanning every measure rect directly and keeping the one with the
 * *largest* left edge at or before `x` sidesteps that: since measures
 * fully tile each row with no gaps, that's always the unique correct answer
 * regardless of which rect's right edge also happens to reach past `x` from
 * sub-pixel rounding.
 *
 * `x`/`y` (from `MouseEvent.clientX`/`clientY`) are always whole CSS pixels,
 * but a measure boundary can land at a fractional one (measure column widths
 * come from proportional text-layout math, not integer grid snapping) — so
 * the boundary rounds `Math.round(rect.left)` for the comparison too, rather
 * than the raw fractional value: without it, a click on the fractional
 * pixel just below a measure's true left edge (e.g. `x=972` against
 * `rect.left=972.15`) would fail `x >= rect.left` and silently fall back to
 * the previous measure, even though that's the nearest whole pixel to where
 * the boundary actually renders.
 */
function measureRangeFromElement(el: HTMLElement): MeasureRange | undefined {
  const { measureIndex, measureIndexEnd } = el.dataset
  if (measureIndex === undefined) return undefined
  const start = Number.parseInt(measureIndex, 10)
  const end = Number.parseInt(measureIndexEnd ?? measureIndex, 10)
  if (Number.isNaN(start) || Number.isNaN(end)) return undefined
  return { start, end }
}

/**
 * Resolves a click/drag point that's actually over a bar line's invisible
 * `.bar-line-drag-handle` (see `PreviewSvgRenderer.tsx`'s
 * `renderBarLineDragHandle`) to a measure range. A bar line visually
 * introduces the measure *after* it, so that's what it always selects —
 * except a system's *last* bar line (its closing line, with no following
 * measure on the same row), which has nothing after it to introduce and so
 * falls back to the measure *before* it instead.
 *
 * Deliberately re-derives the true boundary from the handle element's own
 * `getBoundingClientRect()` rather than trusting `x` directly: the handle is
 * `BAR_LINE_HIT_WIDTH` pixels wide precisely so a real mouse doesn't have to
 * land on the exact boundary pixel, but that padding means `x` can fall a
 * couple of pixels to either side of the true boundary — enough to flip
 * which measure `getMeasureAtPoint`'s plain geometry scan below would
 * otherwise resolve to.
 *
 * Returns `undefined` when `x`/`y` isn't over a bar-line handle at all, so
 * callers can fall through to the generic point-based lookup.
 */
function getBarLineMeasureAtPoint(
  x: number,
  y: number,
): MeasureRange | undefined {
  const hit = document.elementFromPoint(x, y)
  const handle = hit?.closest('.bar-line-drag-handle')
  if (!handle) return undefined
  const lineRect = handle.getBoundingClientRect()
  const boundaryX = Math.round((lineRect.left + lineRect.right) / 2)
  const centerY = (lineRect.top + lineRect.bottom) / 2

  let next: { rect: DOMRect; el: HTMLElement } | undefined
  let prev: { rect: DOMRect; el: HTMLElement } | undefined
  for (const el of document.querySelectorAll<HTMLElement>(
    '[data-tag="measure"]',
  )) {
    const rect = el.getBoundingClientRect()
    if (centerY < rect.top || centerY >= rect.bottom) continue
    const left = Math.round(rect.left)
    if (left >= boundaryX) {
      if (!next || left < next.rect.left) next = { rect, el }
    } else if (!prev || rect.left > prev.rect.left) {
      prev = { rect, el }
    }
  }
  const chosen = next ?? prev
  return chosen && measureRangeFromElement(chosen.el)
}

export function getMeasureAtPoint(
  x: number,
  y: number,
): MeasureRange | undefined {
  const barLineRange = getBarLineMeasureAtPoint(x, y)
  if (barLineRange) return barLineRange

  let best: { rect: DOMRect; el: HTMLElement } | undefined
  for (const el of document.querySelectorAll<HTMLElement>(
    '[data-tag="measure"]',
  )) {
    const rect = el.getBoundingClientRect()
    if (y < rect.top || y >= rect.bottom) continue
    if (x < Math.round(rect.left)) continue
    if (!best || rect.left > best.rect.left) {
      best = { rect, el }
    }
  }
  if (!best) return undefined
  return measureRangeFromElement(best.el)
}

/**
 * Every note/rest cell belonging to the given measure range, resolved from
 * `noteSpans`' `(source_part_index, note_id) → measure_index` mapping
 * (the same source-of-truth `groupSelectedNotesIntoContiguousRuns` groups
 * by) rather than a pixel-geometry intersection test. A geometric approach
 * (unioning the range's measure rects' bounding boxes and marquee-testing
 * note rects against it) previously seemed safe since a measure's
 * click-target rect and its boundary notes' rects are built off identical
 * column math and so should only ever touch, never overlap — but two
 * different SVG elements reporting bit-identical `getBoundingClientRect`
 * values for logically-identical coordinates isn't guaranteed (sub-pixel
 * rounding down independent transform chains), so a boundary note could
 * intermittently be pulled into the wrong neighboring measure's selection.
 * Resolving by index instead sidesteps that class of bug entirely.
 */
export function noteCellsInMeasureRange(
  noteSpans: NoteSpan[],
  range: MeasureRange,
): NoteCell[] {
  return noteSpans
    .filter(
      (span) =>
        span.measureIndex >= range.start && span.measureIndex <= range.end,
    )
    .map((span) => ({
      sourcePartIndex: span.sourcePartIndex,
      noteId: span.noteId,
    }))
}

/** The note/rest under the given point, if any — reads the invisible
 * `NoteClickTarget` rect's enclosing `Tag::Note` group (see
 * `renderer::new_renderer::render_note_click_target`), which sits on top of
 * the `pointer-events: none` playback cursor rect for the same note. */
export function getNoteAtPoint(x: number, y: number): NoteCell | undefined {
  return getCellAtPoint(x, y, {
    tag: 'note',
    parseCell: ({ partIndex, noteId }) => {
      const sourcePartIndex = parseDatasetInt(partIndex)
      const id = parseDatasetInt(noteId)
      if (sourcePartIndex === undefined || id === undefined) return undefined
      return { sourcePartIndex, noteId: id }
    },
  })
}

/** One rendered lyric syllable, keyed the same way as `Tag::Lyric`'s
 * `data-part-index`/`data-note-id`/`data-verse` SVG attributes. Structurally
 * identical to `NoteCell` but kept as its own type — a lyric cell and a note
 * cell sharing the same underlying note number are not interchangeable,
 * they're just keyed by the same note for convenience (see
 * `lyric_spans::LyricCell`). */
export interface LyricCell {
  sourcePartIndex: number
  noteId: number
  verse: number
}

/** The lyric syllable under the given point, if any — reads the invisible
 * `LyricClickTarget` rect's enclosing `Tag::Lyric` group (see
 * `renderer::new_renderer::render_lyric_click_target`), which paints on top
 * of the wider `NoteClickTarget` rect that geometrically covers the same
 * lyric row, so a click that lands on the syllable's own rect always
 * resolves here rather than to `getNoteAtPoint`. */
export function getLyricAtPoint(x: number, y: number): LyricCell | undefined {
  return getCellAtPoint(x, y, {
    tag: 'lyric',
    parseCell: ({ partIndex, noteId, verse }) => {
      const sourcePartIndex = parseDatasetInt(partIndex)
      const id = parseDatasetInt(noteId)
      const verseIndex = parseDatasetInt(verse)
      if (
        sourcePartIndex === undefined ||
        id === undefined ||
        verseIndex === undefined
      )
        return undefined
      return { sourcePartIndex, noteId: id, verse: verseIndex }
    },
  })
}

/** Every lyric syllable cell belonging to the given measure range, resolved
 * from `lyricSpans`' `(source_part_index, note_id, verse) → measure_index`
 * mapping — the lyric-side mirror of `noteCellsInMeasureRange`, so a measure
 * click/drag can select the verse lyrics under it alongside its notes. */
export function lyricCellsInMeasureRange(
  lyricSpans: LyricSpan[],
  range: MeasureRange,
): LyricCell[] {
  return lyricSpans
    .filter(
      (span) =>
        span.measureIndex >= range.start && span.measureIndex <= range.end,
    )
    .map((span) => ({
      sourcePartIndex: span.sourcePartIndex,
      noteId: span.noteId,
      verse: span.verse,
    }))
}
