import type { NoteSpan } from '../types'
import {
  type NoteCell,
  noteCellsForPartLabels,
  type PartLabelHit,
} from './previewSelection'

export interface DragPoint {
  x: number
  y: number
}

/** Screen-pixel distance a mousedown that started on a note must travel
 * before it's treated as a genuine note-drag rather than a plain click on
 * that note (see `dragStateRef`'s 'pending' mode). */
export const NOTE_DRAG_ARM_THRESHOLD_PX = 4

/** Every part-label click target whose rect overlaps the axis-aligned
 * marquee spanned by `anchor`/`current` — the vertical extent naturally picks
 * up every part row a vertical drag crosses, matching `selectedNoteCellsInMarquee`. */
export function partLabelsInMarquee(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
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
    hits.push({
      sourcePartIndex: Number.parseInt(partIndex, 10),
      measureIndexStart: Number.parseInt(measureIndexStart, 10),
      measureIndexEnd: Number.parseInt(measureIndexEnd, 10),
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
  const minX = Math.min(anchor.x, current.x)
  const maxX = Math.max(anchor.x, current.x)
  const minY = Math.min(anchor.y, current.y)
  const maxY = Math.max(anchor.y, current.y)
  const cells: NoteCell[] = []
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      'rect[data-variant="note-click-target-rect"]',
    ),
  )) {
    const bounds = rect.getBoundingClientRect()
    const intersects =
      bounds.left < maxX &&
      bounds.right > minX &&
      bounds.top < maxY &&
      bounds.bottom > minY
    if (!intersects) continue
    const group = rect.closest('[data-tag="note"]')
    if (!group) continue
    const { partIndex, noteId } = (group as HTMLElement).dataset
    if (partIndex === undefined || noteId === undefined) continue
    cells.push({
      sourcePartIndex: Number.parseInt(partIndex, 10),
      noteId: Number.parseInt(noteId, 10),
    })
  }
  return cells
}

export function applyNoteDragHighlights(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
): NoteCell[] {
  const cells = selectedNoteCellsInMarquee(container, anchor, current)
  const selectedKeys = new Set(
    cells.map((c) => `${c.sourcePartIndex}:${c.noteId}`),
  )
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      'rect[data-variant="note-click-target-rect"]',
    ),
  )) {
    const group = rect.closest('[data-tag="note"]') as HTMLElement | null
    if (!group) continue
    const key = `${group.dataset.partIndex}:${group.dataset.noteId}`
    if (selectedKeys.has(key)) {
      group.dataset.noteDragSelected = ''
    } else {
      delete group.dataset.noteDragSelected
    }
  }
  return cells
}

/** Re-applies the note-drag highlight from a fixed cell list rather than a
 * live marquee test — used to keep the selection visible after mouseup (and
 * to restore it whenever the SVG DOM is swapped out from under it, e.g. by a
 * highlighted re-render triggered by the Monaco selection the drag pushed).
 * Cheap and idempotent, so it's safe to re-run on every relevant render. */
export function applyPersistedNoteHighlights(
  container: HTMLElement,
  cells: NoteCell[],
): void {
  const selectedKeys = new Set(
    cells.map((c) => `${c.sourcePartIndex}:${c.noteId}`),
  )
  // Scope to the click-target rect's enclosing group, same as
  // `applyNoteDragHighlights` — a note also has a sibling `Tag::Note` group
  // for its (pointer-events: none) playback-cursor rect, and marking that
  // one too would be harmless (the CSS rule only paints the click-target
  // rect) but redundant.
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      'rect[data-variant="note-click-target-rect"]',
    ),
  )) {
    const group = rect.closest('[data-tag="note"]') as HTMLElement | null
    if (!group) continue
    const key = `${group.dataset.partIndex}:${group.dataset.noteId}`
    if (selectedKeys.has(key)) {
      group.dataset.noteDragSelected = ''
    } else {
      delete group.dataset.noteDragSelected
    }
  }
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
