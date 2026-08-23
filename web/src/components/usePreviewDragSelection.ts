import type { RefObject } from 'react'
import { useEffect, useRef } from 'react'
import type { LyricSpan, NoteSpan } from '../types'
import {
  applyLyricDragHighlights,
  applyNoteDragHighlights,
  applyPersistedLyricHighlights,
  applyPersistedNoteHighlights,
  NOTE_DRAG_ARM_THRESHOLD_PX,
} from './previewDragHighlights'
import type { PreviewDragState } from './previewDragState'
import {
  applyLyricLabelDragHighlight,
  applyPartLabelDragHighlight,
  applyPersistedLyricLabelHighlights,
  applyPersistedPartLabelHighlights,
  lyricLabelsInMarquee,
  partLabelsInMarquee,
} from './previewLabelDragHighlights'
import {
  lyricCellsForLyricLabels,
  lyricCellsForPartLabels,
  noteCellsForPartLabels,
} from './previewLabelSelection'
import {
  getMeasureAtPoint,
  type LyricCell,
  lyricCellsInMeasureRange,
  type NoteCell,
  noteCellsInMeasureRange,
} from './previewSelection'

export type { PreviewDragState } from './previewDragState'

/** Owns the note/measure/part-label drag-select gesture for `Preview`: a
 * `mousedown` on the SVG (handled by `Preview` itself, which writes the
 * initial mode into the returned ref) arms one of the four modes above, and
 * the document-level `mousemove`/`mouseup` handlers registered here carry it
 * through to completion, calling `onNoteRangeSelect` with the resolved note
 * cells on mouseup. Split out of `Preview` to keep that component under its
 * line-count cap. */
export function usePreviewDragSelection(
  previewPagesRef: RefObject<HTMLDivElement | null>,
  noteSpans: NoteSpan[],
  onNoteRangeSelect: ((selectedCells: NoteCell[]) => void) | undefined,
  onLyricRangeSelect?: (selectedCells: LyricCell[]) => void,
  lyricSpans: LyricSpan[] = [],
  // Fired instead of `onNoteRangeSelect`/`onLyricRangeSelect` for a
  // measure/bar-line click or drag, which resolves both note cells and lyric
  // cells at once — see `useAppController`'s `handleMeasureRangeSelect` for
  // why those two can't just be called back-to-back (each independently
  // pushes its own Monaco selection, and the second call's push clobbers the
  // first's).
  onMeasureRangeSelect?: (
    noteCells: NoteCell[],
    lyricCells: LyricCell[],
  ) => void,
) {
  // The mousemove/mouseup handlers below live in a `useEffect(() => {...},
  // [])` with an empty dep array (registered once on mount), so they'd
  // otherwise close over the `noteSpans`/`onNoteRangeSelect` from that first
  // render — refs keep them reading the latest value.
  const noteSpansRef = useRef(noteSpans)
  noteSpansRef.current = noteSpans
  const lyricSpansRef = useRef(lyricSpans)
  lyricSpansRef.current = lyricSpans
  const onNoteRangeSelectRef = useRef(onNoteRangeSelect)
  onNoteRangeSelectRef.current = onNoteRangeSelect
  const onLyricRangeSelectRef = useRef(onLyricRangeSelect)
  onLyricRangeSelectRef.current = onLyricRangeSelect
  const onMeasureRangeSelectRef = useRef(onMeasureRangeSelect)
  onMeasureRangeSelectRef.current = onMeasureRangeSelect

  const dragStateRef = useRef<PreviewDragState>(null)

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      const dragState = dragStateRef.current
      if (!dragState) return
      const container = previewPagesRef.current
      if (!container) return

      if (dragState.mode === 'pending') {
        const dx = e.clientX - dragState.anchor.x
        const dy = e.clientY - dragState.anchor.y
        if (Math.hypot(dx, dy) < NOTE_DRAG_ARM_THRESHOLD_PX) return
        // Real movement past the threshold — this is a note-drag, not a
        // plain click. The eager per-measure highlight (if any) gets
        // overwritten below since `applyNoteDragHighlights` re-marks every
        // note-click-target rect from scratch on each call.
        const current = { x: e.clientX, y: e.clientY }
        dragStateRef.current = {
          mode: 'note',
          anchor: dragState.anchor,
          current,
        }
        applyNoteDragHighlights(container, dragState.anchor, current)
        // A note-drag's marquee can visually sweep over the lyric row
        // underneath it too — union in whatever lyric cells it also
        // overlaps, mirroring 'measure' mode below (which unions
        // `noteCellsInMeasureRange`/`lyricCellsInMeasureRange` together).
        applyLyricDragHighlights(container, dragState.anchor, current)
        return
      }

      if (dragState.mode === 'note') {
        dragState.current = { x: e.clientX, y: e.clientY }
        applyNoteDragHighlights(container, dragState.anchor, dragState.current)
        applyLyricDragHighlights(container, dragState.anchor, dragState.current)
        return
      }

      if (dragState.mode === 'lyric') {
        dragState.current = { x: e.clientX, y: e.clientY }
        applyLyricDragHighlights(container, dragState.anchor, dragState.current)
        // Symmetric to 'note' mode above — a lyric-drag's marquee can also
        // sweep over the notes above it, so union those in too.
        applyNoteDragHighlights(container, dragState.anchor, dragState.current)
        return
      }

      if (dragState.mode === 'part-label') {
        dragState.current = { x: e.clientX, y: e.clientY }
        const hits = partLabelsInMarquee(
          container,
          dragState.anchor,
          dragState.current,
          dragState.anchorSystem,
        )
        applyPartLabelDragHighlight(container, hits)
        applyPersistedNoteHighlights(
          container,
          noteCellsForPartLabels(noteSpansRef.current, hits),
        )
        // Swept part rows carry their lyric rows too — union those in, same
        // as every other drag mode above.
        applyPersistedLyricHighlights(
          container,
          lyricCellsForPartLabels(lyricSpansRef.current, hits),
        )
        return
      }

      if (dragState.mode === 'lyric-label') {
        dragState.current = { x: e.clientX, y: e.clientY }
        const hits = lyricLabelsInMarquee(
          container,
          dragState.anchor,
          dragState.current,
          dragState.anchorSystem,
        )
        applyLyricLabelDragHighlight(container, hits)
        applyPersistedLyricHighlights(
          container,
          lyricCellsForLyricLabels(lyricSpansRef.current, hits),
        )
        return
      }

      const range = getMeasureAtPoint(e.clientX, e.clientY)
      if (range !== undefined) {
        dragState.current = range
        const min = Math.min(dragState.anchor.start, dragState.current.start)
        const max = Math.max(dragState.anchor.end, dragState.current.end)
        const measureRange = { start: min, end: max }
        applyPersistedNoteHighlights(
          container,
          noteCellsInMeasureRange(noteSpansRef.current, measureRange),
        )
        applyPersistedLyricHighlights(
          container,
          lyricCellsInMeasureRange(lyricSpansRef.current, measureRange),
        )
      }
    }

    const handleMouseUp = (e: MouseEvent) => {
      const dragState = dragStateRef.current
      if (!dragState) return
      const container = previewPagesRef.current

      if (dragState.mode === 'pending') {
        // Never armed into a note-drag — a plain click, which resolves to
        // just the one note/chord cell it landed on (or, for the
        // nearest-note fallback, the nearest one in that measure — see
        // `Preview.tsx`'s `onMouseDown`).
        const cells = [dragState.noteCellAtAnchor]
        if (container) {
          applyPersistedNoteHighlights(container, cells)
          applyPersistedLyricHighlights(container, [])
        }
        onMeasureRangeSelectRef.current?.(cells, [])
        dragStateRef.current = null
        return
      }

      if (dragState.mode === 'note') {
        const current = { x: e.clientX, y: e.clientY }
        // Leave the highlight as the drag left it — don't clear it here.
        // `onMeasureRangeSelectRef` (or the `onNoteRangeSelectRef` fallback)
        // feeds `selectedNoteCells`/`selectedLyricCells` back in, and
        // `Preview`'s declarative effect keeps it applied (and re-applies it
        // if a re-render swaps the DOM out from under it).
        const cells = container
          ? applyNoteDragHighlights(container, dragState.anchor, current)
          : []
        // The marquee can also cover lyric syllables underneath — union them
        // in, same as `handleMouseMove` above and mirroring 'measure' mode's
        // combined note+lyric resolution.
        const lyricCells = container
          ? applyLyricDragHighlights(container, dragState.anchor, current)
          : []
        if (onMeasureRangeSelectRef.current) {
          onMeasureRangeSelectRef.current(cells, lyricCells)
        } else {
          // No combined callback wired up — fall back to the two
          // independent ones (safe here since there's no editor-mounted
          // Monaco push to clobber; see `onMeasureRangeSelect`'s doc comment
          // for why a mounted editor needs the combined path instead).
          onNoteRangeSelectRef.current?.(cells)
          onLyricRangeSelectRef.current?.(lyricCells)
        }
        dragStateRef.current = null
        return
      }

      if (dragState.mode === 'lyric') {
        const current = { x: e.clientX, y: e.clientY }
        // Leave the highlight as the drag left it, same as 'note' mode —
        // the combined callback feeds `selectedNoteCells`/`selectedLyricCells`
        // back in, and `Preview`'s declarative effect keeps it applied.
        const cells = container
          ? applyLyricDragHighlights(container, dragState.anchor, current)
          : []
        // Symmetric to 'note' mode above — union in whatever notes the
        // marquee also covers.
        const noteCells = container
          ? applyNoteDragHighlights(container, dragState.anchor, current)
          : []
        if (onMeasureRangeSelectRef.current) {
          onMeasureRangeSelectRef.current(noteCells, cells)
        } else {
          onLyricRangeSelectRef.current?.(cells)
          onNoteRangeSelectRef.current?.(noteCells)
        }
        dragStateRef.current = null
        return
      }

      if (dragState.mode === 'part-label') {
        const current = { x: e.clientX, y: e.clientY }
        const hits = container
          ? partLabelsInMarquee(
              container,
              dragState.anchor,
              current,
              dragState.anchorSystem,
            )
          : []
        const cells = noteCellsForPartLabels(noteSpansRef.current, hits)
        const lyricCells = lyricCellsForPartLabels(lyricSpansRef.current, hits)
        if (container) {
          applyPersistedNoteHighlights(container, cells)
          applyPersistedLyricHighlights(container, lyricCells)
          // Replaces the transient marquee-driven fill with the persisted
          // one immediately, rather than waiting for `selectedNoteCells` to
          // round-trip back down as a prop — otherwise every dragged-over
          // label's fill would flash off for a frame between mouseup and
          // that round-trip landing.
          applyPersistedPartLabelHighlights(
            container,
            noteSpansRef.current,
            cells,
          )
        }
        if (onMeasureRangeSelectRef.current) {
          onMeasureRangeSelectRef.current(cells, lyricCells)
        } else {
          onNoteRangeSelectRef.current?.(cells)
          onLyricRangeSelectRef.current?.(lyricCells)
        }
        dragStateRef.current = null
        return
      }

      if (dragState.mode === 'lyric-label') {
        const current = { x: e.clientX, y: e.clientY }
        const hits = container
          ? lyricLabelsInMarquee(
              container,
              dragState.anchor,
              current,
              dragState.anchorSystem,
            )
          : []
        const lyricCells = lyricCellsForLyricLabels(lyricSpansRef.current, hits)
        if (container) {
          applyPersistedLyricHighlights(container, lyricCells)
          // Replaces the transient marquee-driven fill with the persisted
          // one immediately, mirroring 'part-label' mode's own mouseup —
          // see its comment for why.
          applyPersistedLyricLabelHighlights(
            container,
            lyricSpansRef.current,
            lyricCells,
          )
        }
        if (onMeasureRangeSelectRef.current) {
          onMeasureRangeSelectRef.current([], lyricCells)
        } else {
          onLyricRangeSelectRef.current?.(lyricCells)
        }
        dragStateRef.current = null
        return
      }

      const finalRange =
        getMeasureAtPoint(e.clientX, e.clientY) ?? dragState.current
      const min = Math.min(dragState.anchor.start, finalRange.start)
      const max = Math.max(dragState.anchor.end, finalRange.end)
      const measureRange = { start: min, end: max }
      const cells = noteCellsInMeasureRange(noteSpansRef.current, measureRange)
      const lyricCells = lyricCellsInMeasureRange(
        lyricSpansRef.current,
        measureRange,
      )
      if (container) {
        applyPersistedNoteHighlights(container, cells)
        applyPersistedLyricHighlights(container, lyricCells)
      }
      onMeasureRangeSelectRef.current?.(cells, lyricCells)
      dragStateRef.current = null
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [previewPagesRef])

  return dragStateRef
}
