import {
  measureGroupByEndSelector,
  measureGroupSelector,
  tagFromElement,
} from '../dataAttributes'

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
 * `el`'s own `TagOut` (via `tagFromElement`, `../dataAttributes.ts`), so `el`
 * must already be the `[data-tag="..."]` group itself (the caller's
 * `closest()`/`elementFromPoint` walk has already found it) — not just some
 * descendant of it.
 */
export function clickableElementIdFromElement(
  el: Element,
): ClickableElementId | undefined {
  const tag = tagFromElement(el)
  if (!tag) return undefined
  switch (tag.type) {
    case 'note':
      return {
        kind: 'note',
        sourcePartIndex: tag.source_part_index,
        noteId: tag.note_id,
      }
    case 'lyric':
      return {
        kind: 'lyric',
        sourcePartIndex: tag.source_part_index,
        noteId: tag.note_id,
        verse: tag.verse,
      }
    case 'partLabel':
      return {
        kind: 'partLabel',
        sourcePartIndex: tag.source_part_index,
        measureIndexStart: tag.measure_index_start,
        measureIndexEnd: tag.measure_index_end,
      }
    case 'lyricLabel':
      return {
        kind: 'lyricLabel',
        sourcePartIndex: tag.source_part_index,
        verse: tag.verse,
        measureIndexStart: tag.measure_index_start,
        measureIndexEnd: tag.measure_index_end,
      }
    case 'measure':
    case 'barNumber':
      return {
        kind: 'measure',
        measureIndexStart: tag.index,
        measureIndexEnd: tag.end,
      }
    case 'barLine': {
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
      // own `TagOut` only carries a neighboring measure *index*, not that
      // measure's own possibly-wider range.
      const { measure_index_next, measure_index_prev } = tag
      const measureEl =
        measure_index_next !== undefined
          ? document.querySelector<HTMLElement>(
              measureGroupSelector({ index: measure_index_next }),
            )
          : measure_index_prev !== undefined
            ? document.querySelector<HTMLElement>(
                measureGroupByEndSelector({ end: measure_index_prev }),
              )
            : null
      const measureTag = measureEl && tagFromElement(measureEl)
      if (
        !measureTag ||
        (measureTag.type !== 'measure' && measureTag.type !== 'barNumber')
      )
        return undefined
      return {
        kind: 'measure',
        measureIndexStart: measureTag.index,
        measureIndexEnd: measureTag.end,
      }
    }
    case 'sectionLabel':
      return undefined
    default: {
      const exhaustiveCheck: never = tag
      throw new Error(
        `Unhandled Tag variant: ${JSON.stringify(exhaustiveCheck)}`,
      )
    }
  }
}
