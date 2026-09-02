import type { LyricCell, NoteCell } from './previewSelection'

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
