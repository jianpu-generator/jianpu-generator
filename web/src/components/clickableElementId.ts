import {
  measureGroupByEndSelector,
  measureGroupSelector,
  tagFromElement,
} from '../dataAttributes'
import {
  jianpuWasm,
  type LyricSpan,
  type NoteSpan,
  type ResolveSelectionRangeResponse,
  type ClickableElementId as WasmClickableElementId,
} from '../jianpuWasm'

/**
 * TS mirror of `ClickableElementId` (`crates/jianpu-wasm/src/selection_range_types.rs`)
 * — a tagged union over every rendered element `resolve_selection_range` can
 * resolve a selection range against, keyed exactly the way each element's
 * own `data-*` attributes already carry its identity (see
 * `groupAttrsForTag` in `../dataAttributes.ts`). Hand-written rather than
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

/**
 * The `ClickableElementId` for an already-resolved DOM element — the
 * delegated-event (`mouseover`/`mouseout` `closest()`) counterpart of the
 * point-based `getNoteAtPoint`/`getLyricAtPoint`/`getMeasureAtPoint`/
 * `getPartLabelAtPoint`/`getLyricLabelAtPoint` in `previewSelection.ts`/
 * `previewLabelSelection.ts`, which funnel their own "element → fields"
 * parsing through this same function rather than duplicating it. Reads off
 * `el`'s own `Tag` (via `tagFromElement`, `../dataAttributes.ts`), so `el`
 * must already be the `[data-tag="..."]` group itself (the caller's
 * `closest()`/`elementFromPoint` walk has already found it) — not just some
 * descendant of it.
 */
export function clickableElementIdFromElement(
  el: Element,
): ClickableElementId | undefined {
  const tag = tagFromElement(el)
  if (!tag) return undefined
  switch (tag.tag) {
    case 'note':
      return {
        kind: 'note',
        sourcePartIndex: tag.val.sourcePartIndex,
        noteId: tag.val.noteId,
      }
    case 'lyric':
      return {
        kind: 'lyric',
        sourcePartIndex: tag.val.sourcePartIndex,
        noteId: tag.val.noteId,
        verse: tag.val.verse,
      }
    case 'part-label':
      return {
        kind: 'partLabel',
        sourcePartIndex: tag.val.sourcePartIndex,
        measureIndexStart: tag.val.measureIndexStart,
        measureIndexEnd: tag.val.measureIndexEnd,
      }
    case 'lyric-label':
      return {
        kind: 'lyricLabel',
        sourcePartIndex: tag.val.sourcePartIndex,
        verse: tag.val.verse,
        measureIndexStart: tag.val.measureIndexStart,
        measureIndexEnd: tag.val.measureIndexEnd,
      }
    case 'measure':
    case 'bar-number':
      return {
        kind: 'measure',
        measureIndexStart: tag.val.index,
        measureIndexEnd: tag.val.end,
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
      // own `Tag` only carries a neighboring measure *index*, not that
      // measure's own possibly-wider range.
      const { measureIndexNext, measureIndexPrev } = tag.val
      const measureEl =
        measureIndexNext !== undefined
          ? document.querySelector<HTMLElement>(
              measureGroupSelector({ index: measureIndexNext }),
            )
          : measureIndexPrev !== undefined
            ? document.querySelector<HTMLElement>(
                measureGroupByEndSelector({ end: measureIndexPrev }),
              )
            : null
      const measureTag = measureEl && tagFromElement(measureEl)
      if (
        !measureTag ||
        (measureTag.tag !== 'measure' && measureTag.tag !== 'bar-number')
      )
        return undefined
      return {
        kind: 'measure',
        measureIndexStart: measureTag.val.index,
        measureIndexEnd: measureTag.val.end,
      }
    }
    case 'section-label':
      return undefined
    default: {
      const exhaustiveCheck: never = tag
      throw new Error(
        `Unhandled Tag variant: ${JSON.stringify(exhaustiveCheck)}`,
      )
    }
  }
}

/** The wasm component's own `clickable-element-id` shape for `id`, as
 * `resolveSelectionRange` takes it. */
export function clickableElementIdToWasm(
  id: ClickableElementId,
): WasmClickableElementId {
  switch (id.kind) {
    case 'note':
      return {
        tag: 'note',
        val: { sourcePartIndex: id.sourcePartIndex, noteId: id.noteId },
      }
    case 'lyric':
      return {
        tag: 'lyric',
        val: {
          sourcePartIndex: id.sourcePartIndex,
          noteId: id.noteId,
          verse: id.verse,
        },
      }
    case 'measure':
      return {
        tag: 'measure',
        val: {
          measureIndexStart: id.measureIndexStart,
          measureIndexEnd: id.measureIndexEnd,
        },
      }
    case 'partLabel':
      return {
        tag: 'part-label',
        val: {
          sourcePartIndex: id.sourcePartIndex,
          measureIndexStart: id.measureIndexStart,
          measureIndexEnd: id.measureIndexEnd,
        },
      }
    case 'lyricLabel':
      return {
        tag: 'lyric-label',
        val: {
          sourcePartIndex: id.sourcePartIndex,
          verse: id.verse,
          measureIndexStart: id.measureIndexStart,
          measureIndexEnd: id.measureIndexEnd,
        },
      }
  }
}

/** `resolveSelectionRange` over two `ClickableElementId`s. */
export function resolveSelectionRange(
  noteSpans: NoteSpan[],
  lyricSpans: LyricSpan[],
  anchor: ClickableElementId,
  current: ClickableElementId,
): ResolveSelectionRangeResponse {
  return jianpuWasm().resolveSelectionRange(
    noteSpans,
    lyricSpans,
    clickableElementIdToWasm(anchor),
    clickableElementIdToWasm(current),
  )
}
