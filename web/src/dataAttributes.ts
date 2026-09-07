import type { TagOut, TransparentRectRoleOut } from './jianpuWasm'

/**
 * Every `data-variant` attribute value written onto rendered preview SVG
 * elements — the kebab-case wire format for `TransparentRectRoleOut`
 * (`crates/jianpu-wasm/src/svg_types.rs:20-30`), plus `playbackCursorRect`
 * (a distinct `SvgKindOut::PlaybackCursorRect` element, not one of the
 * `TransparentRectRoleOut` variants, but a `data-variant` value in its own
 * right).
 *
 * Single source of truth for every place that used to re-type these strings
 * independently: `PreviewSvgRenderer.tsx`'s `transparentRectRoleToDataVariant`
 * switch and `playbackCursorRect` case, `usePlaybackCursor.ts`,
 * `previewRangeHighlights.ts`, and `previewLabelRangeHighlights.ts`'s
 * `querySelector`/`closest` selectors. `preview.css` and `index.css` are
 * plain CSS and can't import this constant, so `dataAttributes.test.ts`
 * instead asserts every `data-variant` literal they contain is still a
 * value produced here — a rename here that isn't mirrored in the CSS fails
 * that test instead of the CSS rule silently matching nothing.
 */
export const DATA_VARIANT = {
  measureClickTarget: 'measure-click-target-rect',
  barNumberClickTarget: 'bar-number-click-target-rect',
  sectionLabelBackground: 'section-label-bg',
  sectionLabelClickTarget: 'section-label-click-target-rect',
  noteClickTarget: 'note-click-target-rect',
  partLabelClickTarget: 'part-label-click-target-rect',
  lyricClickTarget: 'lyric-click-target-rect',
  lyricLabelClickTarget: 'lyric-label-click-target-rect',
  barLineClickTarget: 'bar-line-click-target-rect',
  playbackCursorRect: 'playback-cursor-rect',
} as const satisfies Record<
  TransparentRectRoleOut | 'playbackCursorRect',
  string
>

/**
 * Every `data-tag` attribute value written onto a rendered preview SVG `<g>`
 * group — the kebab-case wire format for `TagOut['type']`
 * (`crates/jianpu-wasm/src/svg_types.rs`). `satisfies Record<TagOut['type'], string>`
 * means renaming, adding, or removing a `TagOut` variant fails the TS build
 * right here until this map (and everything derived from it below) is
 * updated — closing the exact `data-tag`/`TagOut` gap the "Cross-boundary
 * invariants" section of `AGENTS.md` names.
 *
 * This file is the single source of truth for the tag ↔ attribute-name ↔
 * field mapping in both directions: `groupAttrsForTag` (write) and
 * `tagFromElement` (read) below, plus the typed selector-builders
 * (`groupTagSelector`/`measureGroupSelector`/`measureGroupByEndSelector`/
 * `noteGroupSelector`) that replace every other file's hand-formatted
 * `[data-tag="..."]` selector string.
 */
export const DATA_TAG = {
  measure: 'measure',
  barNumber: 'bar-number',
  sectionLabel: 'section-label',
  note: 'note',
  partLabel: 'part-label',
  lyric: 'lyric',
  lyricLabel: 'lyric-label',
  barLine: 'bar-line',
} as const satisfies Record<TagOut['type'], string>

/**
 * The two boolean `data-*-range-active` flags imperatively toggled on a
 * part-label/lyric-label click-target rect by `previewLabelRangeHighlights.ts`
 * (`setAttribute`/`removeAttribute`, no value — a presence-only flag, unlike
 * `DATA_VARIANT`'s valued attributes) and read back by `index.css`. Kept
 * alongside `DATA_VARIANT`/`DATA_TAG` so `dataAttributes.test.ts` can guard
 * `index.css` against the same kind of drift.
 */
export const DATA_RANGE_ACTIVE_FLAG = {
  partLabel: 'data-part-label-range-active',
  lyricLabel: 'data-lyric-label-range-active',
} as const

/** Parses a `dataset` string into an integer, or `undefined` if the string is
 * missing or not a valid integer — the common per-field step behind
 * `tagFromElement` below. */
function parseDatasetInt(value: string | undefined): number | undefined {
  if (value === undefined) return undefined
  const parsed = Number.parseInt(value, 10)
  return Number.isNaN(parsed) ? undefined : parsed
}

interface GroupTagAttrs {
  dataTag?: string
  dataMeasureIndex?: number
  dataMeasureIndexEnd?: number
  dataSectionLabel?: string
  dataPartIndex?: number
  dataNoteId?: number
  dataVerse?: number
  dataMeasureIndexStart?: number
  dataMeasureIndexNext?: number
  dataMeasureIndexPrev?: number
  cursor: boolean
}

/** The `data-*` attributes (and whether the group gets a pointer cursor) a
 * rendered `<g>` group should carry for its `TagOut` — the writer half of
 * this file's tag ↔ attribute mapping. `PreviewSvgRenderer.tsx` spreads the
 * result directly onto the `<g>` it renders. */
export function groupAttrsForTag(tag: TagOut | undefined): GroupTagAttrs {
  if (!tag) return { cursor: false }
  switch (tag.type) {
    case 'measure':
      return {
        dataTag: DATA_TAG.measure,
        dataMeasureIndex: tag.index,
        dataMeasureIndexEnd: tag.end,
        cursor: true,
      }
    case 'barNumber':
      return {
        dataTag: DATA_TAG.barNumber,
        dataMeasureIndex: tag.index,
        dataMeasureIndexEnd: tag.end,
        cursor: true,
      }
    case 'sectionLabel':
      return {
        dataTag: DATA_TAG.sectionLabel,
        dataSectionLabel: tag.label,
        cursor: true,
      }
    case 'note':
      return {
        dataTag: DATA_TAG.note,
        dataPartIndex: tag.source_part_index,
        dataNoteId: tag.note_id,
        cursor: false,
      }
    case 'partLabel':
      return {
        dataTag: DATA_TAG.partLabel,
        dataPartIndex: tag.source_part_index,
        dataMeasureIndexStart: tag.measure_index_start,
        dataMeasureIndexEnd: tag.measure_index_end,
        cursor: true,
      }
    case 'lyric':
      return {
        dataTag: DATA_TAG.lyric,
        dataPartIndex: tag.source_part_index,
        dataNoteId: tag.note_id,
        dataVerse: tag.verse,
        cursor: false,
      }
    case 'lyricLabel':
      return {
        dataTag: DATA_TAG.lyricLabel,
        dataPartIndex: tag.source_part_index,
        dataVerse: tag.verse,
        dataMeasureIndexStart: tag.measure_index_start,
        dataMeasureIndexEnd: tag.measure_index_end,
        cursor: true,
      }
    case 'barLine':
      return {
        dataTag: DATA_TAG.barLine,
        dataMeasureIndexNext: tag.measure_index_next,
        dataMeasureIndexPrev: tag.measure_index_prev,
        cursor: true,
      }
    default: {
      const exhaustiveCheck: never = tag
      throw new Error(
        `Unhandled Tag variant: ${JSON.stringify(exhaustiveCheck)}`,
      )
    }
  }
}

/** Reverse lookup of `DATA_TAG`, e.g. `'bar-number'` → `'barNumber'` — keeps
 * `tagFromElement`'s switch below keyed off a real `TagOut['type']` literal
 * union (`keyof typeof DATA_TAG`) instead of the bare `string` `dataset.tag`
 * itself carries, so its `default` arm's `never` check actually catches an
 * unhandled `TagOut` variant at compile time rather than only at the
 * `DATA_TAG` declaration above. */
function tagTypeFromDataTagValue(
  value: string | undefined,
): TagOut['type'] | undefined {
  const entry = (Object.entries(DATA_TAG) as [TagOut['type'], string][]).find(
    ([, dataTagValue]) => dataTagValue === value,
  )
  return entry?.[0]
}

/**
 * The `TagOut` a rendered `<g>` group's own `data-*` attributes encode, or
 * `undefined` if `el` isn't such a group (no recognized `data-tag`) or is
 * missing a field that `type` requires — the reader half of this file's tag
 * ↔ attribute mapping, mirroring `groupAttrsForTag` field-for-field. Replaces
 * every hand-rolled `dataset.partIndex`/`Number(...)` parsing that used to be
 * duplicated per consumer (`clickableElementIdFromElement`,
 * `usePlaybackCursor.ts`, `previewRangeHighlights.ts`,
 * `previewLabelRangeHighlights.ts`).
 */
export function tagFromElement(el: Element): TagOut | undefined {
  const dataset = (el as HTMLElement).dataset
  const tagType = tagTypeFromDataTagValue(dataset.tag)
  if (tagType === undefined) return undefined
  switch (tagType) {
    case 'measure':
    case 'barNumber': {
      const index = parseDatasetInt(dataset.measureIndex)
      const end = parseDatasetInt(dataset.measureIndexEnd)
      if (index === undefined || end === undefined) return undefined
      return { type: tagType, index, end }
    }
    case 'sectionLabel': {
      if (dataset.sectionLabel === undefined) return undefined
      return { type: 'sectionLabel', label: dataset.sectionLabel }
    }
    case 'note': {
      const source_part_index = parseDatasetInt(dataset.partIndex)
      const note_id = parseDatasetInt(dataset.noteId)
      if (source_part_index === undefined || note_id === undefined)
        return undefined
      return { type: 'note', source_part_index, note_id }
    }
    case 'partLabel': {
      const source_part_index = parseDatasetInt(dataset.partIndex)
      const measure_index_start = parseDatasetInt(dataset.measureIndexStart)
      const measure_index_end = parseDatasetInt(dataset.measureIndexEnd)
      if (
        source_part_index === undefined ||
        measure_index_start === undefined ||
        measure_index_end === undefined
      )
        return undefined
      return {
        type: 'partLabel',
        source_part_index,
        measure_index_start,
        measure_index_end,
      }
    }
    case 'lyric': {
      const source_part_index = parseDatasetInt(dataset.partIndex)
      const note_id = parseDatasetInt(dataset.noteId)
      const verse = parseDatasetInt(dataset.verse)
      if (
        source_part_index === undefined ||
        note_id === undefined ||
        verse === undefined
      )
        return undefined
      return { type: 'lyric', source_part_index, note_id, verse }
    }
    case 'lyricLabel': {
      const source_part_index = parseDatasetInt(dataset.partIndex)
      const verse = parseDatasetInt(dataset.verse)
      const measure_index_start = parseDatasetInt(dataset.measureIndexStart)
      const measure_index_end = parseDatasetInt(dataset.measureIndexEnd)
      if (
        source_part_index === undefined ||
        verse === undefined ||
        measure_index_start === undefined ||
        measure_index_end === undefined
      )
        return undefined
      return {
        type: 'lyricLabel',
        source_part_index,
        verse,
        measure_index_start,
        measure_index_end,
      }
    }
    case 'barLine': {
      return {
        type: 'barLine',
        measure_index_next: parseDatasetInt(dataset.measureIndexNext),
        measure_index_prev: parseDatasetInt(dataset.measureIndexPrev),
      }
    }
    default: {
      const exhaustiveCheck: never = tagType
      throw new Error(
        `Unhandled data-tag value: ${JSON.stringify(exhaustiveCheck)}`,
      )
    }
  }
}

/** A bare `[data-tag="..."]` selector for `type` — the typed replacement for
 * every consumer that used to hand-format e.g. `'[data-tag="note"]'`
 * itself. */
export function groupTagSelector(type: TagOut['type']): string {
  return `[data-tag="${DATA_TAG[type]}"]`
}

/** Selects the `[data-tag="measure"]` group whose own `data-measure-index`
 * is `index` — the typed replacement for e.g.
 * `` `[data-tag="measure"][data-measure-index="${index}"]` ``
 * (`Preview.tsx`'s scroll-to-selection lookup, and
 * `clickableElementIdFromElement`'s bar-line-to-next-measure lookup). */
export function measureGroupSelector({ index }: { index: number }): string {
  return `${groupTagSelector('measure')}[data-measure-index="${index}"]`
}

/** Selects the `[data-tag="measure"]` group whose own `data-measure-index-end`
 * is `end` — the typed replacement for
 * `` `[data-tag="measure"][data-measure-index-end="${end}"]` ``
 * (`clickableElementIdFromElement`'s bar-line-to-previous-measure lookup, for
 * a merged multi-measure-rest block whose trailing index is what a bar
 * line's own `measure_index_prev` names). */
export function measureGroupByEndSelector({ end }: { end: number }): string {
  return `${groupTagSelector('measure')}[data-measure-index-end="${end}"]`
}

/** Selects the `[data-tag="note"]` group for a specific note/rest — the
 * typed replacement for
 * `` `[data-tag="note"][data-part-index="${partIndex}"][data-note-id="${noteId}"]` ``
 * (`usePlaybackCursor.ts`'s per-note highlight/scroll lookups). */
export function noteGroupSelector({
  partIndex,
  noteId,
}: {
  partIndex: number
  noteId: number
}): string {
  return `${groupTagSelector('note')}[data-part-index="${partIndex}"][data-note-id="${noteId}"]`
}
