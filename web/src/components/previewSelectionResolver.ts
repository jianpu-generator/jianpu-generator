import type { RefObject } from 'react'
import type { LyricSpan, NoteSpan } from '../types'
import {
  applyPersistedLyricHighlights,
  applyPersistedNoteHighlights,
} from './previewDragHighlights'
import type { PreviewDragState } from './previewDragState'
import {
  applyPersistedLyricLabelHighlights,
  applyPersistedPartLabelHighlights,
  lyricLabelsInMarquee,
  partLabelsInMarquee,
  partLabelsInMarqueeAcrossSystems,
} from './previewLabelDragHighlights'
import {
  lyricCellsForLyricLabels,
  lyricCellsForPartLabels,
  noteCellsForPartLabels,
} from './previewLabelSelection'
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

/** Shared by `previewClickHandler.ts`'s idle/anchored dispatch and
 * `usePreviewClickSelection`'s Escape handler — everything a click-and-click
 * gesture needs to resolve, highlight, and commit a selection. */
export interface HandlePreviewClickArgs {
  dragStateRef: RefObject<PreviewDragState>
  /** Armed by a gesture's anchoring (first) click — see
   * `previewClickHandler.ts`'s `anchorAndCommit` — and consumed once by
   * `Preview.tsx`'s scroll-to-selection effect, so that self-commit's own
   * debounced Monaco-selection round-trip doesn't auto-scroll the preview
   * out from under a user still moving toward a second click (e.g. onto
   * another page) to widen the gesture into a range. Deliberately a
   * one-shot flag rather than a persistent "is a gesture anchored" check:
   * a click-and-click gesture's anchor stays live indefinitely (until a
   * second click or Escape), so gating on that directly would silently
   * suppress every *later*, unrelated reveal too (e.g. a keyboard
   * navigation) for as long as an old anchor from a single, never-followed-up
   * click happens to still be sitting there. */
  suppressNextRevealRef: RefObject<boolean>
  previewPagesRef: RefObject<HTMLDivElement | null>
  noteSpans: NoteSpan[]
  lyricSpans: LyricSpan[]
  onSectionLabelClick: ((label: string) => void) | undefined
  onNoteRangeSelect: ((selectedCells: NoteCell[]) => void) | undefined
  onLyricRangeSelect: ((selectedCells: LyricCell[]) => void) | undefined
  onMeasureRangeSelect:
    | ((noteCells: NoteCell[], lyricCells: LyricCell[]) => void)
    | undefined
}

export interface ResolvedSelection {
  noteCells: NoteCell[]
  lyricCells: LyricCell[]
}

/**
 * The per-mode marquee/range resolution shared by every point
 * `previewClickHandler.ts` needs it: the first click's immediate
 * self-commit, a second click's final commit, and reverting a cancelled
 * second click back to what the first click already committed. `point` is
 * the screen point (or, for 'measure' mode, resolved into a measure range
 * internally) the gesture should resolve against as its "current" side of
 * the anchor→current marquee — pass `undefined` to resolve against the
 * anchor itself (i.e. zero movement), which is exactly what the first
 * click's own self-commit and a cancelled second click's revert both need.
 * Applies the resulting highlight to the DOM as a side effect and returns
 * the cells it resolved, but never fires a callback or touches
 * `dragStateRef` — callers own that.
 */
export function resolveSelection(
  dragState: NonNullable<PreviewDragState>,
  point: { x: number; y: number } | undefined,
  { previewPagesRef, noteSpans, lyricSpans }: HandlePreviewClickArgs,
): ResolvedSelection {
  const container = previewPagesRef.current

  if (dragState.mode === 'measure') {
    const finalRange = point
      ? (getMeasureAtPoint(point.x, point.y) ?? dragState.current)
      : dragState.anchor
    const min = Math.min(dragState.anchor.start, finalRange.start)
    const max = Math.max(dragState.anchor.end, finalRange.end)
    const measureRange = { start: min, end: max }
    const noteCells = noteCellsInMeasureRange(noteSpans, measureRange)
    const lyricCells = lyricCellsInMeasureRange(lyricSpans, measureRange)
    if (container) {
      applyPersistedNoteHighlights(container, noteCells)
      applyPersistedLyricHighlights(container, lyricCells)
    }
    return { noteCells, lyricCells }
  }

  const current = point ?? dragState.anchor

  if (dragState.mode === 'note') {
    let { noteCells, lyricCells } = container
      ? applyNoteRangeSelection(
          container,
          noteSpans,
          dragState.noteCellAtAnchor,
          dragState.anchor,
          current,
        )
      : { noteCells: [], lyricCells: [] }
    if (noteCells.length === 0) {
      // The resolved point missed every note's click target (e.g. it landed
      // back on the anchor note, or on a bar-line/gutter pixel) — fall back
      // to the cell the anchoring click resolved, so the gesture still
      // collapses to a single selection instead of an empty one.
      noteCells = [dragState.noteCellAtAnchor]
      lyricCells = []
      if (container) {
        applyPersistedNoteHighlights(container, noteCells)
        applyPersistedLyricHighlights(container, [])
      }
    }
    return { noteCells, lyricCells }
  }

  if (dragState.mode === 'lyric') {
    const { noteCells, lyricCells } = container
      ? applyLyricRangeSelection(
          container,
          lyricSpans,
          dragState.lyricCellAtAnchor,
          dragState.anchor,
          current,
        )
      : { noteCells: [], lyricCells: [] }
    return { noteCells, lyricCells }
  }

  if (dragState.mode === 'part-label') {
    const hits = container
      ? partLabelsInMarquee(
          container,
          dragState.anchor,
          current,
          dragState.anchorSystem,
        )
      : []
    const noteCells = noteCellsForPartLabels(noteSpans, hits)
    // Only union in the lyric row underneath when the resolved point actually
    // swept past a different label — a plain click that resolves back to
    // the same single label it anchored on selects just that part's notes,
    // not its lyrics too (see the "does not also select the lyric row"
    // regression test in `part-label-click-selects-notes.feature`).
    const lyricCells =
      hits.length > 1 ? lyricCellsForPartLabels(lyricSpans, hits) : []
    if (container) {
      applyPersistedNoteHighlights(container, noteCells)
      applyPersistedLyricHighlights(container, lyricCells)
      // Replaces the transient hover-driven fill with the persisted one
      // immediately, rather than waiting for `selectedNoteCells` to
      // round-trip back down as a prop — otherwise every swept label's fill
      // would flash off for a frame between commit and that round-trip
      // landing.
      applyPersistedPartLabelHighlights(container, noteSpans, noteCells)
    }
    return { noteCells, lyricCells }
  }

  if (dragState.mode === 'part-label-system') {
    const hits = container
      ? partLabelsInMarqueeAcrossSystems(container, dragState.anchor, current)
      : []
    const noteCells = noteCellsForPartLabels(noteSpans, hits)
    const lyricCells = lyricCellsForPartLabels(lyricSpans, hits)
    if (container) {
      applyPersistedNoteHighlights(container, noteCells)
      applyPersistedLyricHighlights(container, lyricCells)
      // Mirrors 'part-label' mode's own resolution above — see its comment.
      applyPersistedPartLabelHighlights(container, noteSpans, noteCells)
    }
    return { noteCells, lyricCells }
  }

  // 'lyric-label' mode.
  const hits = container
    ? lyricLabelsInMarquee(
        container,
        dragState.anchor,
        current,
        dragState.anchorSystem,
      )
    : []
  const lyricCells = lyricCellsForLyricLabels(lyricSpans, hits)
  if (container) {
    applyPersistedLyricHighlights(container, lyricCells)
    // Mirrors 'part-label' mode's own resolution above — see its comment.
    applyPersistedLyricLabelHighlights(container, lyricSpans, lyricCells)
  }
  return { noteCells: [], lyricCells }
}

/** Fires whichever of `onNoteRangeSelect`/`onLyricRangeSelect`/
 * `onMeasureRangeSelect` is wired up for a resolved selection — shared by
 * the first click's immediate self-commit and a second click's final
 * commit. */
export function fireCommit(
  { noteCells, lyricCells }: ResolvedSelection,
  args: HandlePreviewClickArgs,
): void {
  if (args.onMeasureRangeSelect) {
    args.onMeasureRangeSelect(noteCells, lyricCells)
  } else {
    args.onNoteRangeSelect?.(noteCells)
    args.onLyricRangeSelect?.(lyricCells)
  }
}
