import type { LyricSpan, NoteSpan } from '../types'
import type { DragPoint } from './previewDragHighlights'
import {
  type LyricLabelHit,
  lyricCellsForLyricLabels,
  noteCellsForPartLabels,
  type PartLabelHit,
} from './previewLabelSelection'
import type { LyricCell, NoteCell } from './previewSelection'

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

/** Every lyric-label click target whose rect overlaps the axis-aligned
 * marquee spanned by `anchor`/`current`, restricted to `anchorSystem` — the
 * lyric-side mirror of `partLabelsInMarquee`. Every lyric label in a given
 * system shares the same `measureIndexStart`/`measureIndexEnd` (one
 * `LyricLabelClickTarget` per verse row *per system*, see
 * `grid_layout::click_targets::compute_all_lyric_label_click_targets`), so
 * that pair is a reliable key for "which system", same as `partLabelsInMarquee`. */
export function lyricLabelsInMarquee(
  container: HTMLElement,
  anchor: DragPoint,
  current: DragPoint,
  anchorSystem: { measureIndexStart: number; measureIndexEnd: number },
): LyricLabelHit[] {
  const minX = Math.min(anchor.x, current.x)
  const maxX = Math.max(anchor.x, current.x)
  const minY = Math.min(anchor.y, current.y)
  const maxY = Math.max(anchor.y, current.y)
  const hits: LyricLabelHit[] = []
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      'rect[data-variant="lyric-label-click-target-rect"]',
    ),
  )) {
    const bounds = rect.getBoundingClientRect()
    const intersects =
      bounds.left < maxX &&
      bounds.right > minX &&
      bounds.top < maxY &&
      bounds.bottom > minY
    if (!intersects) continue
    const group = rect.closest('[data-tag="lyric-label"]')
    if (!group) continue
    const { partIndex, verse, measureIndexStart, measureIndexEnd } = (
      group as HTMLElement
    ).dataset
    if (
      partIndex === undefined ||
      verse === undefined ||
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
      verse: Number.parseInt(verse, 10),
      measureIndexStart: start,
      measureIndexEnd: end,
    })
  }
  return hits
}

/** Marks every lyric-label click-target rect belonging to `hits` with
 * `data-lyric-label-drag-active`, clearing it from every other one — the
 * lyric-side mirror of `applyPartLabelDragHighlight`. */
export function applyLyricLabelDragHighlight(
  container: HTMLElement,
  hits: LyricLabelHit[],
): void {
  const activeKeys = new Set(
    hits.map(
      (hit) =>
        `${hit.sourcePartIndex}:${hit.verse}:${hit.measureIndexStart}:${hit.measureIndexEnd}`,
    ),
  )
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      'rect[data-variant="lyric-label-click-target-rect"]',
    ),
  )) {
    const group = rect.closest('[data-tag="lyric-label"]') as HTMLElement | null
    if (!group) continue
    const { partIndex, verse, measureIndexStart, measureIndexEnd } =
      group.dataset
    const key = `${partIndex}:${verse}:${measureIndexStart}:${measureIndexEnd}`
    if (activeKeys.has(key)) {
      rect.setAttribute('data-lyric-label-drag-active', '')
    } else {
      rect.removeAttribute('data-lyric-label-drag-active')
    }
  }
}

/** Re-applies the lyric-label hover fill from the given selected lyric
 * cells rather than from a live drag — the lyric-side mirror of
 * `applyPersistedPartLabelHighlights`. A lyric label counts as selected
 * once *every* syllable it covers (its verse across its whole system, see
 * `lyricCellsForLyricLabels`) is present in `selectedLyricCells`. */
export function applyPersistedLyricLabelHighlights(
  container: HTMLElement,
  lyricSpans: LyricSpan[],
  selectedLyricCells: LyricCell[],
): void {
  const selectedKeys = new Set(
    selectedLyricCells.map(
      (c) => `${c.sourcePartIndex}:${c.noteId}:${c.verse}`,
    ),
  )
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      'rect[data-variant="lyric-label-click-target-rect"]',
    ),
  )) {
    const group = rect.closest('[data-tag="lyric-label"]') as HTMLElement | null
    if (!group) continue
    const { partIndex, verse, measureIndexStart, measureIndexEnd } =
      group.dataset
    if (
      partIndex === undefined ||
      verse === undefined ||
      measureIndexStart === undefined ||
      measureIndexEnd === undefined
    )
      continue
    const hit: LyricLabelHit = {
      sourcePartIndex: Number.parseInt(partIndex, 10),
      verse: Number.parseInt(verse, 10),
      measureIndexStart: Number.parseInt(measureIndexStart, 10),
      measureIndexEnd: Number.parseInt(measureIndexEnd, 10),
    }
    const cells = lyricCellsForLyricLabels(lyricSpans, [hit])
    const fullySelected =
      cells.length > 0 &&
      cells.every((cell) =>
        selectedKeys.has(
          `${cell.sourcePartIndex}:${cell.noteId}:${cell.verse}`,
        ),
      )
    if (fullySelected) {
      rect.setAttribute('data-lyric-label-drag-active', '')
    } else {
      rect.removeAttribute('data-lyric-label-drag-active')
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
