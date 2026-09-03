/**
 * TS mirror of `ClickableElementId` (`crates/jianpu-wasm/src/selection_range_types.rs`)
 * — a tagged union over every rendered element `resolve_selection_range` can
 * resolve a selection range against, keyed exactly the way each element's
 * own `data-*` attributes already carry its identity (see
 * `groupAttrsForTag` in `PreviewSvgRenderer.tsx`). Hand-written rather than
 * generated, per that Rust type's own doc comment.
 */
export type ClickableElementId =
  | { kind: 'note'; sourcePartIndex: number; noteId: number }
  | { kind: 'lyric'; sourcePartIndex: number; noteId: number; verse: number }
  | { kind: 'measure'; measureIndexStart: number; measureIndexEnd: number }
  | {
      kind: 'partLabel'
      sourcePartIndex: number
      measureIndexStart: number
      measureIndexEnd: number
    }
  | {
      kind: 'lyricLabel'
      sourcePartIndex: number
      verse: number
      measureIndexStart: number
      measureIndexEnd: number
    }

/** Parses a `dataset` string into an integer, or `undefined` if the string is
 * missing or not a valid integer — the common per-field step behind every
 * `data-*` id below. */
export function parseDatasetInt(value: string | undefined): number | undefined {
  if (value === undefined) return undefined
  const parsed = Number.parseInt(value, 10)
  return Number.isNaN(parsed) ? undefined : parsed
}

/** Reads a `{ measureIndexStart, measureIndexEnd }` pair off a
 * `[data-tag="measure"]`/`[data-tag="bar-number"]` group's own `dataset` —
 * shared by `clickableElementIdFromElement`'s `'measure'`/`'bar-number'`/
 * `'bar-line'` cases below. A merged multi-measure-rest bar carries a
 * `data-measure-index-end` wider than its own `data-measure-index`, so a
 * click/hover anywhere on it resolves to every source measure it
 * represents; a plain single measure has no `data-measure-index-end` at
 * all, so `measureIndex` doubles as its own end. */
function measureIdFromDataset(
  dataset: DOMStringMap,
): { measureIndexStart: number; measureIndexEnd: number } | undefined {
  const start = parseDatasetInt(dataset.measureIndex)
  if (start === undefined) return undefined
  const end = parseDatasetInt(dataset.measureIndexEnd) ?? start
  return { measureIndexStart: start, measureIndexEnd: end }
}

/**
 * The `ClickableElementId` for an already-resolved DOM element — the
 * delegated-event (`mouseover`/`mouseout` `closest()`) counterpart of the
 * point-based `getNoteAtPoint`/`getLyricAtPoint`/`getMeasureAtPoint`/
 * `getPartLabelAtPoint`/`getLyricLabelAtPoint` in `previewSelection.ts`/
 * `previewLabelSelection.ts`, which funnel their own "element → fields"
 * parsing through this same function rather than duplicating it. Switches
 * on `el.dataset.tag`, so `el` must already be the `[data-tag="..."]` group
 * itself (the caller's `closest()`/`elementFromPoint` walk has already
 * found it) — not just some descendant of it.
 */
export function clickableElementIdFromElement(
  el: Element,
): ClickableElementId | undefined {
  const dataset = (el as HTMLElement).dataset
  switch (dataset.tag) {
    case 'note': {
      const sourcePartIndex = parseDatasetInt(dataset.partIndex)
      const noteId = parseDatasetInt(dataset.noteId)
      if (sourcePartIndex === undefined || noteId === undefined)
        return undefined
      return { kind: 'note', sourcePartIndex, noteId }
    }
    case 'lyric': {
      const sourcePartIndex = parseDatasetInt(dataset.partIndex)
      const noteId = parseDatasetInt(dataset.noteId)
      const verse = parseDatasetInt(dataset.verse)
      if (
        sourcePartIndex === undefined ||
        noteId === undefined ||
        verse === undefined
      )
        return undefined
      return { kind: 'lyric', sourcePartIndex, noteId, verse }
    }
    case 'part-label': {
      const sourcePartIndex = parseDatasetInt(dataset.partIndex)
      const measureIndexStart = parseDatasetInt(dataset.measureIndexStart)
      const measureIndexEnd = parseDatasetInt(dataset.measureIndexEnd)
      if (
        sourcePartIndex === undefined ||
        measureIndexStart === undefined ||
        measureIndexEnd === undefined
      )
        return undefined
      return {
        kind: 'partLabel',
        sourcePartIndex,
        measureIndexStart,
        measureIndexEnd,
      }
    }
    case 'lyric-label': {
      const sourcePartIndex = parseDatasetInt(dataset.partIndex)
      const verse = parseDatasetInt(dataset.verse)
      const measureIndexStart = parseDatasetInt(dataset.measureIndexStart)
      const measureIndexEnd = parseDatasetInt(dataset.measureIndexEnd)
      if (
        sourcePartIndex === undefined ||
        verse === undefined ||
        measureIndexStart === undefined ||
        measureIndexEnd === undefined
      )
        return undefined
      return {
        kind: 'lyricLabel',
        sourcePartIndex,
        verse,
        measureIndexStart,
        measureIndexEnd,
      }
    }
    case 'measure':
    case 'bar-number': {
      const measureId = measureIdFromDataset(dataset)
      if (measureId === undefined) return undefined
      return { kind: 'measure', ...measureId }
    }
    case 'bar-line': {
      // A bar line visually introduces the measure *after* it, so `next`
      // wins when present; a system's *last* bar line (its closing line,
      // with no following measure on the same row) has no `next` and falls
      // back to `prev` instead. `next`/`prev` each name a single
      // source-measure index, not a range, so this looks up the actual
      // `[data-tag="measure"]` element that index belongs to (matching on
      // `data-measure-index` for `next`, since that's a block's own leading
      // index, or `data-measure-index-end` for `prev`, its trailing index)
      // to recover the merged-multi-measure-rest range, if any, that
      // measure is actually part of. Ported unchanged from
      // `getBarLineMeasureAtPoint` (`previewSelection.ts`) — the one
      // `document.querySelector` this function needs, since a bar line's
      // own `dataset` only carries a neighboring measure *index*, not that
      // measure's own possibly-wider range.
      const next = parseDatasetInt(dataset.measureIndexNext)
      const prev = parseDatasetInt(dataset.measureIndexPrev)
      const measure =
        next !== undefined
          ? document.querySelector<HTMLElement>(
              `[data-tag="measure"][data-measure-index="${next}"]`,
            )
          : prev !== undefined
            ? document.querySelector<HTMLElement>(
                `[data-tag="measure"][data-measure-index-end="${prev}"]`,
              )
            : null
      const measureId = measure && measureIdFromDataset(measure.dataset)
      if (!measureId) return undefined
      return { kind: 'measure', ...measureId }
    }
    default:
      return undefined
  }
}
