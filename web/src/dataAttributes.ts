import type {
  SvgKind,
  SvgVariant,
  Tag,
  TransparentRectRole,
} from './jianpuWasm'

/**
 * Every `data-variant` attribute value written onto rendered preview SVG
 * elements: a text element's `SvgVariant`, a transparent rect's
 * `TransparentRectRole`, or the `playback-cursor-rect` element kind — each
 * written verbatim as the jco-generated string, so there is no TS-side
 * mapping to keep in step with the WIT enums.
 */
export type DataVariant =
  | SvgVariant
  | TransparentRectRole
  | Extract<SvgKind['tag'], 'playback-cursor-rect'>

/** Selects the `<rect>` elements whose `data-variant` is `variant`. */
export function rectVariantSelector(variant: DataVariant): string {
  return `rect[data-variant="${variant}"]`
}

/**
 * Every `data-tag` attribute value written onto a rendered preview SVG `<g>`
 * group, keyed by `Tag['tag']`
 * (`crates/jianpu-wasm/wit/world.wit`). `satisfies Record<Tag['tag'], string>`
 * means renaming, adding, or removing a `Tag` variant fails the TS build
 * right here until this map (and everything derived from it below) is
 * updated — closing the exact `data-tag`/`Tag` gap the "Cross-boundary
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
  'bar-number': 'bar-number',
  'section-label': 'section-label',
  note: 'note',
  'part-label': 'part-label',
  lyric: 'lyric',
  'lyric-label': 'lyric-label',
  'bar-line': 'bar-line',
} as const satisfies Record<Tag['tag'], string>

/**
 * The two boolean `data-*-range-active` flags imperatively toggled on a
 * part-label/lyric-label click-target rect by `previewLabelRangeHighlights.ts`
 * (`setAttribute`/`removeAttribute`, no value — a presence-only flag) and
 * read back by `injectPreviewInteractionStyles.ts`.
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
 * rendered `<g>` group should carry for its `Tag` — the writer half of
 * this file's tag ↔ attribute mapping. `PreviewSvgRenderer.tsx` spreads the
 * result directly onto the `<g>` it renders. */
export function groupAttrsForTag(tag: Tag | undefined): GroupTagAttrs {
  if (!tag) return { cursor: false }
  const dataTag = DATA_TAG[tag.tag]
  switch (tag.tag) {
    case 'measure':
    case 'bar-number':
      return {
        dataTag,
        dataMeasureIndex: tag.val.index,
        dataMeasureIndexEnd: tag.val.end,
        cursor: true,
      }
    case 'section-label':
      return { dataTag, dataSectionLabel: tag.val.label, cursor: true }
    case 'note':
      return {
        dataTag,
        dataPartIndex: tag.val.sourcePartIndex,
        dataNoteId: tag.val.noteId,
        cursor: false,
      }
    case 'part-label':
      return {
        dataTag,
        dataPartIndex: tag.val.sourcePartIndex,
        dataMeasureIndexStart: tag.val.measureIndexStart,
        dataMeasureIndexEnd: tag.val.measureIndexEnd,
        cursor: true,
      }
    case 'lyric':
      return {
        dataTag,
        dataPartIndex: tag.val.sourcePartIndex,
        dataNoteId: tag.val.noteId,
        dataVerse: tag.val.verse,
        cursor: false,
      }
    case 'lyric-label':
      return {
        dataTag,
        dataPartIndex: tag.val.sourcePartIndex,
        dataVerse: tag.val.verse,
        dataMeasureIndexStart: tag.val.measureIndexStart,
        dataMeasureIndexEnd: tag.val.measureIndexEnd,
        cursor: true,
      }
    case 'bar-line':
      return {
        dataTag,
        dataMeasureIndexNext: tag.val.measureIndexNext,
        dataMeasureIndexPrev: tag.val.measureIndexPrev,
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

/** Reverse lookup of `DATA_TAG` — keeps
 * `tagFromElement`'s switch below keyed off a real `Tag['tag']` literal
 * union (`keyof typeof DATA_TAG`) instead of the bare `string` `dataset.tag`
 * itself carries, so its `default` arm's `never` check actually catches an
 * unhandled `Tag` variant at compile time rather than only at the
 * `DATA_TAG` declaration above. */
function tagTypeFromDataTagValue(
  value: string | undefined,
): Tag['tag'] | undefined {
  const entry = (Object.entries(DATA_TAG) as [Tag['tag'], string][]).find(
    ([, dataTagValue]) => dataTagValue === value,
  )
  return entry?.[0]
}

/**
 * The `Tag` a rendered `<g>` group's own `data-*` attributes encode, or
 * `undefined` if `el` isn't such a group (no recognized `data-tag`) or is
 * missing a field that `type` requires — the reader half of this file's tag
 * ↔ attribute mapping, mirroring `groupAttrsForTag` field-for-field. Replaces
 * every hand-rolled `dataset.partIndex`/`Number(...)` parsing that used to be
 * duplicated per consumer (`clickableElementIdFromElement`,
 * `usePlaybackCursor.ts`, `previewRangeHighlights.ts`,
 * `previewLabelRangeHighlights.ts`).
 */
export function tagFromElement(el: Element): Tag | undefined {
  const dataset = (el as HTMLElement).dataset
  const tagType = tagTypeFromDataTagValue(dataset.tag)
  if (tagType === undefined) return undefined
  switch (tagType) {
    case 'measure':
    case 'bar-number': {
      const index = parseDatasetInt(dataset.measureIndex)
      const end = parseDatasetInt(dataset.measureIndexEnd)
      if (index === undefined || end === undefined) return undefined
      return { tag: tagType, val: { index, end } }
    }
    case 'section-label': {
      if (dataset.sectionLabel === undefined) return undefined
      return { tag: 'section-label', val: { label: dataset.sectionLabel } }
    }
    case 'note': {
      const sourcePartIndex = parseDatasetInt(dataset.partIndex)
      const noteId = parseDatasetInt(dataset.noteId)
      if (sourcePartIndex === undefined || noteId === undefined)
        return undefined
      return { tag: 'note', val: { sourcePartIndex, noteId } }
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
        tag: 'part-label',
        val: { sourcePartIndex, measureIndexStart, measureIndexEnd },
      }
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
      return { tag: 'lyric', val: { sourcePartIndex, noteId, verse } }
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
        tag: 'lyric-label',
        val: { sourcePartIndex, verse, measureIndexStart, measureIndexEnd },
      }
    }
    case 'bar-line': {
      return {
        tag: 'bar-line',
        val: {
          measureIndexNext: parseDatasetInt(dataset.measureIndexNext),
          measureIndexPrev: parseDatasetInt(dataset.measureIndexPrev),
        },
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
export function groupTagSelector(type: Tag['tag']): string {
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
 * line's own `measureIndexPrev` names). */
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
