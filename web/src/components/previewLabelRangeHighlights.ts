import {
  DATA_RANGE_ACTIVE_FLAG,
  DATA_VARIANT,
  groupTagSelector,
  tagFromElement,
} from '../dataAttributes'
import type { LyricSpan, NoteSpan } from '../types'
import {
  type LyricLabelHit,
  lyricCellsForLyricLabels,
  noteCellsForPartLabels,
  type PartLabelHit,
} from './previewLabelSelection'
import type { AnchorPoint } from './previewRangeHighlights'
import type { LyricCell, NoteCell } from './previewSelection'

/** Narrows a part-label group's own `TagOut` (already known to be
 * `'partLabel'`, via `tagFromElement`) into a `PartLabelHit`, or `undefined`
 * if it isn't one — shared by every part-label hit-test/highlight below. */
function partLabelHitFromGroup(group: Element): PartLabelHit | undefined {
  const tag = tagFromElement(group)
  if (tag?.type !== 'partLabel') return undefined
  return {
    sourcePartIndex: tag.source_part_index,
    measureIndexStart: tag.measure_index_start,
    measureIndexEnd: tag.measure_index_end,
  }
}

/** The lyric-label mirror of `partLabelHitFromGroup`. */
function lyricLabelHitFromGroup(group: Element): LyricLabelHit | undefined {
  const tag = tagFromElement(group)
  if (tag?.type !== 'lyricLabel') return undefined
  return {
    sourcePartIndex: tag.source_part_index,
    verse: tag.verse,
    measureIndexStart: tag.measure_index_start,
    measureIndexEnd: tag.measure_index_end,
  }
}

/** Every part-label click target whose rect overlaps the axis-aligned
 * marquee spanned by `anchor`/`current`, restricted to `anchorSystem` — the
 * `measureIndexStart`/`measureIndexEnd` of the label the selection started on.
 *
 * Every part label in a given system shares the same `measureIndexStart`/
 * `measureIndexEnd` (one `PartLabelClickTarget` per part *per system*, see
 * `grid_layout::click_targets::compute_all_part_label_click_targets`), so
 * that pair is a reliable key for "which system". Without this filter, a
 * vertical marquee that travels far enough to reach a different system's label
 * row would splice that system's notes into the selection too — the marquee
 * rect has no innate awareness of the gap between systems, it just
 * intersects against every label in the whole document. The vertical extent
 * still naturally picks up every part row *within the anchor's own system*
 * that the marquee crosses, matching `selectedNoteCellsInMarquee`. */
export function partLabelsInMarquee(
  container: HTMLElement,
  anchor: AnchorPoint,
  current: AnchorPoint,
  anchorSystem: { measureIndexStart: number; measureIndexEnd: number },
): PartLabelHit[] {
  const minX = Math.min(anchor.x, current.x)
  const maxX = Math.max(anchor.x, current.x)
  const minY = Math.min(anchor.y, current.y)
  const maxY = Math.max(anchor.y, current.y)
  const hits: PartLabelHit[] = []
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      `rect[data-variant="${DATA_VARIANT.partLabelClickTarget}"]`,
    ),
  )) {
    const bounds = rect.getBoundingClientRect()
    const intersects =
      bounds.left < maxX &&
      bounds.right > minX &&
      bounds.top < maxY &&
      bounds.bottom > minY
    if (!intersects) continue
    const group = rect.closest(groupTagSelector('partLabel'))
    const hit = group && partLabelHitFromGroup(group)
    if (!hit) continue
    if (
      hit.measureIndexStart !== anchorSystem.measureIndexStart ||
      hit.measureIndexEnd !== anchorSystem.measureIndexEnd
    )
      continue
    hits.push(hit)
  }
  return hits
}

/** Every part-label click target belonging to any system the axis-aligned
 * marquee spanned by `anchor`/`current` touches — the Cmd/Ctrl-gated,
 * cross-system sibling of `partLabelsInMarquee`. Where `partLabelsInMarquee`
 * restricts to one `anchorSystem`, this drops that restriction entirely: it
 * first finds every *system* (a `measureIndexStart`/`measureIndexEnd` pair)
 * with at least one part-label rect touched by the marquee, then returns
 * *every* part in each of those systems, touched or not — so brushing past
 * even one part row of a system pulls in that whole system, matching how
 * 'measure' mode selects whole measures rather than partially-overlapped
 * note ranges. A zero-movement marquee (a plain click, `anchor === current`)
 * touches only the row directly under the pointer, so this still resolves a
 * bare Cmd/Ctrl-click on one label to every part in that one system. */
export function partLabelsInMarqueeAcrossSystems(
  container: HTMLElement,
  anchor: AnchorPoint,
  current: AnchorPoint,
): PartLabelHit[] {
  const minX = Math.min(anchor.x, current.x)
  const maxX = Math.max(anchor.x, current.x)
  const minY = Math.min(anchor.y, current.y)
  const maxY = Math.max(anchor.y, current.y)
  const allHits: PartLabelHit[] = []
  const touchedSystems = new Set<string>()
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      `rect[data-variant="${DATA_VARIANT.partLabelClickTarget}"]`,
    ),
  )) {
    const group = rect.closest(groupTagSelector('partLabel'))
    const hit = group && partLabelHitFromGroup(group)
    if (!hit) continue
    allHits.push(hit)
    const bounds = rect.getBoundingClientRect()
    const intersects =
      bounds.left < maxX &&
      bounds.right > minX &&
      bounds.top < maxY &&
      bounds.bottom > minY
    if (intersects)
      touchedSystems.add(`${hit.measureIndexStart}:${hit.measureIndexEnd}`)
  }
  return allHits.filter((hit) =>
    touchedSystems.has(`${hit.measureIndexStart}:${hit.measureIndexEnd}`),
  )
}

/** Marks every part-label click-target rect belonging to `hits` with
 * `DATA_RANGE_ACTIVE_FLAG.partLabel`, clearing it from every other one.
 * Driven from JS state rather than left to pure CSS `:hover` — the label a
 * part-label selection started on must keep showing the hovered fill for the
 * whole gesture, even once the pointer has moved off its rect onto another
 * label's (or off every label entirely), matching how `partLabelsInMarquee`
 * keeps that label part of the selection regardless of where the pointer
 * currently sits. */
export function applyPartLabelRangeHighlight(
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
      `rect[data-variant="${DATA_VARIANT.partLabelClickTarget}"]`,
    ),
  )) {
    const group = rect.closest(groupTagSelector('partLabel'))
    const hit = group && partLabelHitFromGroup(group)
    const key =
      hit &&
      `${hit.sourcePartIndex}:${hit.measureIndexStart}:${hit.measureIndexEnd}`
    if (typeof key === 'string' && activeKeys.has(key)) {
      rect.setAttribute(DATA_RANGE_ACTIVE_FLAG.partLabel, '')
    } else {
      rect.removeAttribute(DATA_RANGE_ACTIVE_FLAG.partLabel)
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
  anchor: AnchorPoint,
  current: AnchorPoint,
  anchorSystem: { measureIndexStart: number; measureIndexEnd: number },
): LyricLabelHit[] {
  const minX = Math.min(anchor.x, current.x)
  const maxX = Math.max(anchor.x, current.x)
  const minY = Math.min(anchor.y, current.y)
  const maxY = Math.max(anchor.y, current.y)
  const hits: LyricLabelHit[] = []
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(
      `rect[data-variant="${DATA_VARIANT.lyricLabelClickTarget}"]`,
    ),
  )) {
    const bounds = rect.getBoundingClientRect()
    const intersects =
      bounds.left < maxX &&
      bounds.right > minX &&
      bounds.top < maxY &&
      bounds.bottom > minY
    if (!intersects) continue
    const group = rect.closest(groupTagSelector('lyricLabel'))
    const hit = group && lyricLabelHitFromGroup(group)
    if (!hit) continue
    if (
      hit.measureIndexStart !== anchorSystem.measureIndexStart ||
      hit.measureIndexEnd !== anchorSystem.measureIndexEnd
    )
      continue
    hits.push(hit)
  }
  return hits
}

/** Marks every lyric-label click-target rect belonging to `hits` with
 * `DATA_RANGE_ACTIVE_FLAG.lyricLabel`, clearing it from every other one —
 * the lyric-side mirror of `applyPartLabelRangeHighlight`. */
export function applyLyricLabelRangeHighlight(
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
      `rect[data-variant="${DATA_VARIANT.lyricLabelClickTarget}"]`,
    ),
  )) {
    const group = rect.closest(groupTagSelector('lyricLabel'))
    const hit = group && lyricLabelHitFromGroup(group)
    const key =
      hit &&
      `${hit.sourcePartIndex}:${hit.verse}:${hit.measureIndexStart}:${hit.measureIndexEnd}`
    if (typeof key === 'string' && activeKeys.has(key)) {
      rect.setAttribute(DATA_RANGE_ACTIVE_FLAG.lyricLabel, '')
    } else {
      rect.removeAttribute(DATA_RANGE_ACTIVE_FLAG.lyricLabel)
    }
  }
}

/** Re-applies the lyric-label hover fill from the given selected lyric
 * cells rather than from a live selection — the lyric-side mirror of
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
      `rect[data-variant="${DATA_VARIANT.lyricLabelClickTarget}"]`,
    ),
  )) {
    const group = rect.closest(groupTagSelector('lyricLabel'))
    const hit = group && lyricLabelHitFromGroup(group)
    if (!hit) continue
    const cells = lyricCellsForLyricLabels(lyricSpans, [hit])
    const fullySelected =
      cells.length > 0 &&
      cells.every((cell) =>
        selectedKeys.has(
          `${cell.sourcePartIndex}:${cell.noteId}:${cell.verse}`,
        ),
      )
    if (fullySelected) {
      rect.setAttribute(DATA_RANGE_ACTIVE_FLAG.lyricLabel, '')
    } else {
      rect.removeAttribute(DATA_RANGE_ACTIVE_FLAG.lyricLabel)
    }
  }
}

/** Re-applies the part-label hover fill from the given selected note cells
 * rather than from a live selection — a part label counts as selected once
 * *every* note/rest it covers (its part across its whole system, see
 * `noteCellsForPartLabels`) is present in `selectedNoteCells`, so the fill
 * persists after the second click (and survives an SVG DOM swap, same as
 * `applyPersistedNoteHighlights`) instead of being cleared the instant the
 * selection ends. Mirrors that a part-label selection is just a shortcut for
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
      `rect[data-variant="${DATA_VARIANT.partLabelClickTarget}"]`,
    ),
  )) {
    const group = rect.closest(groupTagSelector('partLabel'))
    const hit = group && partLabelHitFromGroup(group)
    if (!hit) continue
    const cells = noteCellsForPartLabels(noteSpans, [hit])
    const fullySelected =
      cells.length > 0 &&
      cells.every((cell) =>
        selectedKeys.has(`${cell.sourcePartIndex}:${cell.noteId}`),
      )
    if (fullySelected) {
      rect.setAttribute(DATA_RANGE_ACTIVE_FLAG.partLabel, '')
    } else {
      rect.removeAttribute(DATA_RANGE_ACTIVE_FLAG.partLabel)
    }
  }
}
