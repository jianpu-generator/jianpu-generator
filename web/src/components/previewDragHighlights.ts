import type { NoteSpan } from '../types'
import {
  type LyricCell,
  type NoteCell,
  noteCellsForPartLabels,
  type PartLabelHit,
} from './previewSelection'

export interface DragPoint {
  x: number
  y: number
}

/**
 * Shape driving `cellsInMarquee`/`applyPersistedHighlights`/
 * `applyDragHighlights` — the generic core behind
 * `selectedNoteCellsInMarquee`/`applyNoteDragHighlights`/
 * `applyPersistedNoteHighlights` and their lyric counterparts. Deliberately
 * *not* used for part-label highlighting: that uses a different marking
 * mechanism (`setAttribute`/`removeAttribute` on the rect itself, rather
 * than a dataset boolean on its enclosing group) plus a system-scoping
 * filter (`anchorSystem`) with no note/lyric analog.
 */
interface DragHighlightSpec<Cell> {
  /** CSS selector for each candidate hit-target rect, e.g.
   * `rect[data-variant="note-click-target-rect"]`. */
  rectSelector: string
  /** `data-tag` of the rect's enclosing group, e.g. `'note'`. */
  tag: string
  /** Parses a `Cell` out of that group's `dataset`, or `undefined` if a
   * required field is missing. */
  parseCell: (dataset: DOMStringMap) => Cell | undefined
  /** A string uniquely identifying `cell`, used to test set membership. */
  cellKey: (cell: Cell) => string
  /** `dataset` property (camelCase) toggled on the enclosing group to mark it
   * as drag-selected, e.g. `'noteDragSelected'`. */
  datasetFlag: string
}

/** Every cell whose hit-target rect overlaps the axis-aligned marquee
 * spanned by `anchor`/`current` (real screen geometry, not column math). */
function cellsInMarquee<Cell>(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
  spec: DragHighlightSpec<Cell>,
): Cell[] {
  const minX = Math.min(anchor.x, current.x)
  const maxX = Math.max(anchor.x, current.x)
  const minY = Math.min(anchor.y, current.y)
  const maxY = Math.max(anchor.y, current.y)
  const cells: Cell[] = []
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(spec.rectSelector),
  )) {
    const bounds = rect.getBoundingClientRect()
    const intersects =
      bounds.left < maxX &&
      bounds.right > minX &&
      bounds.top < maxY &&
      bounds.bottom > minY
    if (!intersects) continue
    const group = rect.closest(`[data-tag="${spec.tag}"]`)
    if (!group) continue
    const cell = spec.parseCell((group as HTMLElement).dataset)
    if (cell === undefined) continue
    cells.push(cell)
  }
  return cells
}

/** Marks every hit-target rect's enclosing group whose cell is in `cells`
 * with `spec.datasetFlag`, clearing it from every other one. Re-applies from
 * a fixed cell list rather than a live marquee test — used to keep the
 * selection visible after mouseup and to restore it whenever the SVG DOM is
 * swapped out from under it. Cheap and idempotent, so it's safe to re-run on
 * every relevant render. */
function applyPersistedHighlights<Cell>(
  container: HTMLElement,
  cells: Cell[],
  spec: DragHighlightSpec<Cell>,
): void {
  const selectedKeys = new Set(cells.map(spec.cellKey))
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(spec.rectSelector),
  )) {
    const group = rect.closest(`[data-tag="${spec.tag}"]`) as HTMLElement | null
    if (!group) continue
    const cell = spec.parseCell(group.dataset)
    const selected = cell !== undefined && selectedKeys.has(spec.cellKey(cell))
    if (selected) {
      group.dataset[spec.datasetFlag] = ''
    } else {
      delete group.dataset[spec.datasetFlag]
    }
  }
}

/** Live-marquee-tests then persists the highlight in one step — the
 * mousemove-time counterpart to `applyPersistedHighlights`. */
function applyDragHighlights<Cell>(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
  spec: DragHighlightSpec<Cell>,
): Cell[] {
  const cells = cellsInMarquee(container, anchor, current, spec)
  applyPersistedHighlights(container, cells, spec)
  return cells
}

const noteDragSpec: DragHighlightSpec<NoteCell> = {
  rectSelector: 'rect[data-variant="note-click-target-rect"]',
  tag: 'note',
  parseCell: ({ partIndex, noteId }) => {
    if (partIndex === undefined || noteId === undefined) return undefined
    return {
      sourcePartIndex: Number.parseInt(partIndex, 10),
      noteId: Number.parseInt(noteId, 10),
    }
  },
  cellKey: (c) => `${c.sourcePartIndex}:${c.noteId}`,
  datasetFlag: 'noteDragSelected',
}

const lyricDragSpec: DragHighlightSpec<LyricCell> = {
  rectSelector: 'rect[data-variant="lyric-click-target-rect"]',
  tag: 'lyric',
  parseCell: ({ partIndex, noteId, verse }) => {
    if (partIndex === undefined || noteId === undefined || verse === undefined)
      return undefined
    return {
      sourcePartIndex: Number.parseInt(partIndex, 10),
      noteId: Number.parseInt(noteId, 10),
      verse: Number.parseInt(verse, 10),
    }
  },
  cellKey: (c) => `${c.sourcePartIndex}:${c.noteId}:${c.verse}`,
  datasetFlag: 'lyricDragSelected',
}

/** Screen-pixel distance a mousedown that started on a note must travel
 * before it's treated as a genuine note-drag rather than a plain click on
 * that note (see `dragStateRef`'s 'pending' mode). */
export const NOTE_DRAG_ARM_THRESHOLD_PX = 4

/** Every part-label click target whose rect overlaps the axis-aligned
 * marquee spanned by `anchor`/`current`, restricted to `anchorSystem` — the
 * `measureIndexStart`/`measureIndexEnd` of the label the drag started on.
 *
 * Every part label in a given system shares the same `measureIndexStart`/
 * `measureIndexEnd` (one `PartLabelClickTarget` per part *per system*, see
 * `grid_layout::click_targets::compute_all_part_label_click_targets`), so
 * that pair is a reliable key for "which system". Without this filter, a
 * vertical drag that travels far enough to reach a different system's label
 * row would splice that system's notes into the selection too — the marquee
 * rect has no innate awareness of the gap between systems, it just
 * intersects against every label in the whole document. The vertical extent
 * still naturally picks up every part row *within the anchor's own system*
 * that a drag crosses, matching `selectedNoteCellsInMarquee`. */
export function partLabelsInMarquee(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
  anchorSystem: { measureIndexStart: number; measureIndexEnd: number },
): PartLabelHit[] {
  const minX = Math.min(anchor.x, current.x)
  const maxX = Math.max(anchor.x, current.x)
  const minY = Math.min(anchor.y, current.y)
  const maxY = Math.max(anchor.y, current.y)
  const hits: PartLabelHit[] = []
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      'rect[data-variant="part-label-click-target-rect"]',
    ),
  )) {
    const bounds = rect.getBoundingClientRect()
    const intersects =
      bounds.left < maxX &&
      bounds.right > minX &&
      bounds.top < maxY &&
      bounds.bottom > minY
    if (!intersects) continue
    const group = rect.closest('[data-tag="part-label"]')
    if (!group) continue
    const { partIndex, measureIndexStart, measureIndexEnd } = (
      group as HTMLElement
    ).dataset
    if (
      partIndex === undefined ||
      measureIndexStart === undefined ||
      measureIndexEnd === undefined
    )
      continue
    const start = Number.parseInt(measureIndexStart, 10)
    const end = Number.parseInt(measureIndexEnd, 10)
    if (
      start !== anchorSystem.measureIndexStart ||
      end !== anchorSystem.measureIndexEnd
    )
      continue
    hits.push({
      sourcePartIndex: Number.parseInt(partIndex, 10),
      measureIndexStart: start,
      measureIndexEnd: end,
    })
  }
  return hits
}

/** Marks every part-label click-target rect belonging to `hits` with
 * `data-part-label-drag-active`, clearing it from every other one. Driven
 * from JS state rather than left to pure CSS `:hover` — the label a
 * part-label drag started on must keep showing the hovered fill for the
 * whole gesture, even once the pointer has moved off its rect onto another
 * label's (or off every label entirely), matching how `partLabelsInMarquee`
 * keeps that label part of the selection regardless of where the pointer
 * currently sits. */
export function applyPartLabelDragHighlight(
  container: HTMLElement,
  hits: PartLabelHit[],
): void {
  const activeKeys = new Set(
    hits.map(
      (hit) =>
        `${hit.sourcePartIndex}:${hit.measureIndexStart}:${hit.measureIndexEnd}`,
    ),
  )
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      'rect[data-variant="part-label-click-target-rect"]',
    ),
  )) {
    const group = rect.closest('[data-tag="part-label"]') as HTMLElement | null
    if (!group) continue
    const { partIndex, measureIndexStart, measureIndexEnd } = group.dataset
    const key = `${partIndex}:${measureIndexStart}:${measureIndexEnd}`
    if (activeKeys.has(key)) {
      rect.setAttribute('data-part-label-drag-active', '')
    } else {
      rect.removeAttribute('data-part-label-drag-active')
    }
  }
}

/**
 * Every note/rest cell whose click-target rect overlaps the axis-aligned
 * marquee spanned by `anchor`/`current` (real screen geometry, not column
 * math — naturally handles both a horizontal drag within one part's row and
 * a vertical drag extending the selection to more part rows, since it just
 * tests each note's actual rendered bounding box).
 */
export function selectedNoteCellsInMarquee(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
): NoteCell[] {
  return cellsInMarquee(container, anchor, current, noteDragSpec)
}

export function applyNoteDragHighlights(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
): NoteCell[] {
  return applyDragHighlights(container, anchor, current, noteDragSpec)
}

/** Re-applies the note-drag highlight from a fixed cell list rather than a
 * live marquee test — used to keep the selection visible after mouseup (and
 * to restore it whenever the SVG DOM is swapped out from under it, e.g. by a
 * highlighted re-render triggered by the Monaco selection the drag pushed).
 * Cheap and idempotent, so it's safe to re-run on every relevant render.
 * Scope is the click-target rect's enclosing group — a note also has a
 * sibling `Tag::Note` group for its (pointer-events: none) playback-cursor
 * rect, and marking that one too would be harmless (the CSS rule only paints
 * the click-target rect) but redundant. */
export function applyPersistedNoteHighlights(
  container: HTMLElement,
  cells: NoteCell[],
): void {
  applyPersistedHighlights(container, cells, noteDragSpec)
}

/**
 * Every lyric syllable cell whose click-target rect overlaps the axis-aligned
 * marquee spanned by `anchor`/`current` — mirrors `selectedNoteCellsInMarquee`
 * exactly, but scoped to `[data-tag="lyric"]` groups/`lyric-click-target-rect`
 * rects so it's independent of any note selection happening at the same time.
 */
export function selectedLyricCellsInMarquee(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
): LyricCell[] {
  return cellsInMarquee(container, anchor, current, lyricDragSpec)
}

export function applyLyricDragHighlights(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
): LyricCell[] {
  return applyDragHighlights(container, anchor, current, lyricDragSpec)
}

/** Re-applies the lyric-drag highlight from a fixed cell list rather than a
 * live marquee test — mirrors `applyPersistedNoteHighlights`, used to keep
 * the selection visible after mouseup and to restore it whenever the SVG DOM
 * is swapped out from under it. */
export function applyPersistedLyricHighlights(
  container: HTMLElement,
  cells: LyricCell[],
): void {
  applyPersistedHighlights(container, cells, lyricDragSpec)
}

/** Re-applies the part-label hover fill from the given selected note cells
 * rather than from a live drag — a part label counts as selected once
 * *every* note/rest it covers (its part across its whole system, see
 * `noteCellsForPartLabels`) is present in `selectedNoteCells`, so the fill
 * persists after mouseup (and survives an SVG DOM swap, same as
 * `applyPersistedNoteHighlights`) instead of being cleared the instant the
 * drag ends. Mirrors that a part-label drag is just a shortcut for
 * selecting its notes — once selected, the label reflects the selection the
 * same way it would if made some other way (e.g. typing in the editor). */
export function applyPersistedPartLabelHighlights(
  container: HTMLElement,
  noteSpans: NoteSpan[],
  selectedNoteCells: NoteCell[],
): void {
  const selectedKeys = new Set(
    selectedNoteCells.map((c) => `${c.sourcePartIndex}:${c.noteId}`),
  )
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      'rect[data-variant="part-label-click-target-rect"]',
    ),
  )) {
    const group = rect.closest('[data-tag="part-label"]') as HTMLElement | null
    if (!group) continue
    const { partIndex, measureIndexStart, measureIndexEnd } = group.dataset
    if (
      partIndex === undefined ||
      measureIndexStart === undefined ||
      measureIndexEnd === undefined
    )
      continue
    const hit: PartLabelHit = {
      sourcePartIndex: Number.parseInt(partIndex, 10),
      measureIndexStart: Number.parseInt(measureIndexStart, 10),
      measureIndexEnd: Number.parseInt(measureIndexEnd, 10),
    }
    const cells = noteCellsForPartLabels(noteSpans, [hit])
    const fullySelected =
      cells.length > 0 &&
      cells.every((cell) =>
        selectedKeys.has(`${cell.sourcePartIndex}:${cell.noteId}`),
      )
    if (fullySelected) {
      rect.setAttribute('data-part-label-drag-active', '')
    } else {
      rect.removeAttribute('data-part-label-drag-active')
    }
  }
}
