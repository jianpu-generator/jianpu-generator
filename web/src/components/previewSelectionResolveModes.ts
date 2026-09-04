import { resolve_selection_range } from '../jianpuWasm'
import type { LyricSpan, NoteSpan } from '../types'
import type { ClickableElementId } from './clickableElementId'
import {
  anyClickableElementIdAtPoint,
  measureClickableElementId,
} from './previewClickableElementIdBuilders'
import {
  applyPersistedLyricHighlights,
  applyPersistedNoteHighlights,
} from './previewDragHighlights'
import type { PreviewDragState } from './previewDragState'
import {
  applyLyricRangeSelection,
  applyNoteRangeSelection,
} from './previewRangeSelection'
import {
  getMeasureAtPoint,
  type LyricCell,
  lyricCellsInMeasureRange,
  type NoteCell,
  noteCellsInMeasureRange,
} from './previewSelection'
import type { ResolvedSelection } from './previewSelectionResolver'

/** Everything a mode's own resolver needs beyond `dragState`/`point`/
 * `currentIdHint` themselves — shared by `resolveMeasureSelection`,
 * `resolveNoteSelection`, and `resolveLyricSelection` below (and their
 * label-mode siblings in `previewSelectionResolveLabelModes.ts`), split out
 * of `resolveSelection`'s own body once its per-mode branches grew too long
 * for one function. */
export interface ResolveModeArgs {
  container: HTMLDivElement | null
  point: { x: number; y: number } | undefined
  currentIdHint: ClickableElementId | undefined
  noteSpans: NoteSpan[]
  lyricSpans: LyricSpan[]
}

type MeasureDragState = Extract<
  NonNullable<PreviewDragState>,
  { mode: 'measure' }
>
type NoteDragState = Extract<NonNullable<PreviewDragState>, { mode: 'note' }>
type LyricDragState = Extract<NonNullable<PreviewDragState>, { mode: 'lyric' }>

export function resolveMeasureSelection(
  dragState: MeasureDragState,
  { container, point, currentIdHint, noteSpans, lyricSpans }: ResolveModeArgs,
): ResolvedSelection {
  const finalRange = point
    ? (getMeasureAtPoint(point.x, point.y) ?? dragState.current)
    : dragState.anchor
  const response = resolve_selection_range(
    noteSpans,
    lyricSpans,
    dragState.anchorId,
    currentIdHint ?? measureClickableElementId(finalRange),
  )
  // `Measure ↔ Measure` is fully ID-based now (see
  // `resolve_selection_range_response` in `selection_range.rs`), so this
  // never actually returns 'err' for 'measure' mode — the pixel-based
  // fallback below is dead code kept only as a safety net until the wasm
  // side is proven out further.
  let noteCells: NoteCell[]
  let lyricCells: LyricCell[]
  if (response.status === 'ok') {
    noteCells = response.note_cells.map((cell) => ({
      sourcePartIndex: cell.sourcePartIndex,
      noteId: cell.noteId,
    }))
    lyricCells = response.lyric_cells.map((cell) => ({
      sourcePartIndex: cell.sourcePartIndex,
      noteId: cell.noteId,
      verse: cell.verse,
    }))
  } else {
    const min = Math.min(dragState.anchor.start, finalRange.start)
    const max = Math.max(dragState.anchor.end, finalRange.end)
    const measureRange = { start: min, end: max }
    noteCells = noteCellsInMeasureRange(noteSpans, measureRange)
    lyricCells = lyricCellsInMeasureRange(lyricSpans, measureRange)
  }
  if (container) {
    applyPersistedNoteHighlights(container, noteCells)
    applyPersistedLyricHighlights(container, lyricCells)
  }
  return { noteCells, lyricCells }
}

export function resolveNoteSelection(
  dragState: NoteDragState,
  { container, point, currentIdHint, noteSpans, lyricSpans }: ResolveModeArgs,
): ResolvedSelection {
  const current = point ?? dragState.anchor
  const anchorCell: NoteCell = {
    sourcePartIndex: dragState.anchorId.sourcePartIndex,
    noteId: dragState.anchorId.noteId,
  }
  // Resolves every `Note ↔ Note` combination (same-part and cross-part)
  // and, when `current` lands on a lyric syllable, part label, or lyric
  // label instead, every one of `Note ↔ Lyric` (cross-row, either
  // ordering), `Note ↔ PartLabel`, and `Note ↔ LyricLabel` (see
  // `resolve_selection_range_response` in `selection_range.rs`) without
  // touching pixels at all, so it's immune to the scroll-invalidated-anchor
  // bug class. This covers every pair wasm can actually be asked to
  // resolve — it does NOT cover `current` missing every recognized click
  // target of any type (a bar-line/gutter or empty-space point), which
  // isn't any resolvable pair at all and has no ID to hand wasm. That case
  // keeps the pixel marquee (`applyNoteRangeSelection`'s fallback branch)
  // exactly as before this mode's wasm migration; collapsing straight to
  // the anchor here (as an earlier revision of this branch did) silently
  // regressed the cross-row case instead of falling back to it (see
  // `note-lyric-cross-drag-select.feature`).
  const currentId =
    currentIdHint ?? anyClickableElementIdAtPoint(current.x, current.y)
  if (currentId === undefined) {
    let { noteCells, lyricCells } = container
      ? applyNoteRangeSelection(
          container,
          noteSpans,
          anchorCell,
          dragState.anchor,
          current,
        )
      : { noteCells: [], lyricCells: [] }
    if (noteCells.length === 0) {
      noteCells = [anchorCell]
      lyricCells = []
      if (container) {
        applyPersistedNoteHighlights(container, noteCells)
        applyPersistedLyricHighlights(container, [])
      }
    }
    return { noteCells, lyricCells }
  }

  const response = resolve_selection_range(
    noteSpans,
    lyricSpans,
    dragState.anchorId,
    currentId,
  )
  if (response.status !== 'ok') {
    // Should be unreachable — every type `current` can resolve to now has
    // an arm in `resolve_selection_range_response` paired with `Note`.
    // Logged rather than thrown so a real click-and-click gesture never
    // hard-fails on it; the empty-selection fallback below still collapses
    // to the anchor.
    console.error(
      'resolve_selection_range returned Err for a Note-anchored pair',
      dragState.anchorId,
      currentId,
    )
  }

  let noteCells: NoteCell[] =
    response.status === 'ok'
      ? response.note_cells.map((cell) => ({
          sourcePartIndex: cell.sourcePartIndex,
          noteId: cell.noteId,
        }))
      : []
  let lyricCells: LyricCell[] =
    response.status === 'ok'
      ? response.lyric_cells.map((cell) => ({
          sourcePartIndex: cell.sourcePartIndex,
          noteId: cell.noteId,
          verse: cell.verse,
        }))
      : []
  if (container) {
    applyPersistedNoteHighlights(container, noteCells)
    applyPersistedLyricHighlights(container, lyricCells)
  }
  if (noteCells.length === 0) {
    // The resolved point missed every note's click target (e.g. it landed
    // back on the anchor note, or on a bar-line/gutter pixel) — fall back
    // to the cell the anchoring click resolved, so the gesture still
    // collapses to a single selection instead of an empty one.
    noteCells = [anchorCell]
    lyricCells = []
    if (container) {
      applyPersistedNoteHighlights(container, noteCells)
      applyPersistedLyricHighlights(container, [])
    }
  }
  return { noteCells, lyricCells }
}

export function resolveLyricSelection(
  dragState: LyricDragState,
  { container, point, currentIdHint, noteSpans, lyricSpans }: ResolveModeArgs,
): ResolvedSelection {
  const current = point ?? dragState.anchor
  const anchorCell: LyricCell = {
    sourcePartIndex: dragState.anchorId.sourcePartIndex,
    noteId: dragState.anchorId.noteId,
    verse: dragState.anchorId.verse,
  }
  // Try wasm's ID-based range resolution first — resolves every
  // `Lyric ↔ Lyric` scope (same part-and-verse, same-part cross-verse, and
  // cross-part) and, when `current` lands on a note, part label, or lyric
  // label instead, every one of `Lyric ↔ Note` (cross-row),
  // `Lyric ↔ PartLabel`, and `Lyric ↔ LyricLabel` (see
  // `resolve_selection_range_response` in `selection_range.rs`) without
  // touching pixels at all. `Err` now only covers `current` missing every
  // recognized click target of any type — falls back to
  // `applyLyricRangeSelection`'s existing marquee path unchanged (the
  // same-mode mirror of 'note' mode's cross-row fallback above — see its
  // comment for why collapsing
  // straight through instead would silently regress the cross-row case).
  const currentId =
    currentIdHint ?? anyClickableElementIdAtPoint(current.x, current.y)
  const response = currentId
    ? resolve_selection_range(
        noteSpans,
        lyricSpans,
        dragState.anchorId,
        currentId,
      )
    : undefined

  if (response?.status === 'ok') {
    const noteCells = response.note_cells.map((cell) => ({
      sourcePartIndex: cell.sourcePartIndex,
      noteId: cell.noteId,
    }))
    const lyricCells = response.lyric_cells.map((cell) => ({
      sourcePartIndex: cell.sourcePartIndex,
      noteId: cell.noteId,
      verse: cell.verse,
    }))
    if (container) {
      applyPersistedNoteHighlights(container, noteCells)
      applyPersistedLyricHighlights(container, lyricCells)
    }
    return { noteCells, lyricCells }
  }

  const { noteCells, lyricCells } = container
    ? applyLyricRangeSelection(
        container,
        lyricSpans,
        anchorCell,
        dragState.anchor,
        current,
      )
    : { noteCells: [], lyricCells: [] }
  return { noteCells, lyricCells }
}
