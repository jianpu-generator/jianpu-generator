import type { RefObject } from 'react'
import { useEffect, useRef } from 'react'
import type { NoteSpan } from '../types'
import {
  applyNoteDragHighlights,
  applyPartLabelDragHighlight,
  applyPersistedNoteHighlights,
  applyPersistedPartLabelHighlights,
  type DragPoint,
  NOTE_DRAG_ARM_THRESHOLD_PX,
  partLabelsInMarquee,
} from './previewDragHighlights'
import {
  getMeasureAtPoint,
  type MeasureRange,
  type NoteCell,
  noteCellsForPartLabels,
  noteCellsInMeasureRange,
} from './previewSelection'

// One discriminated ref rather than a separate measure-drag and note-drag
// ref: a single mousedown can only ever arm one mode (see the handler
// below), and two independent refs risk both firing at once.
//
// A mousedown that lands on a note doesn't commit to 'note' mode right
// away — it starts 'pending' instead, and only arms into 'note' once the
// pointer has actually moved past `NOTE_DRAG_ARM_THRESHOLD_PX` (see
// `handleMouseMove`). A plain click on a note (mouseup with no meaningful
// movement) resolves as a shortcut for selecting every note in that note's
// measure instead, using `measureRangeAtAnchor` — clicking a measure (on a
// note or the empty space around it) is just a fast way to select all of
// its notes, there's no separate "measure selected" state anymore.
export type PreviewDragState =
  | { mode: 'measure'; anchor: MeasureRange; current: MeasureRange }
  | { mode: 'note'; anchor: DragPoint; current: DragPoint }
  | {
      mode: 'part-label'
      anchor: DragPoint
      current: DragPoint
      anchorSystem: { measureIndexStart: number; measureIndexEnd: number }
    }
  | {
      mode: 'pending'
      anchor: DragPoint
      noteCellAtAnchor: NoteCell
      measureRangeAtAnchor: MeasureRange | undefined
    }
  | null

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
) {
  // The mousemove/mouseup handlers below live in a `useEffect(() => {...},
  // [])` with an empty dep array (registered once on mount), so they'd
  // otherwise close over the `noteSpans`/`onNoteRangeSelect` from that first
  // render — refs keep them reading the latest value.
  const noteSpansRef = useRef(noteSpans)
  noteSpansRef.current = noteSpans
  const onNoteRangeSelectRef = useRef(onNoteRangeSelect)
  onNoteRangeSelectRef.current = onNoteRangeSelect

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
        return
      }

      if (dragState.mode === 'note') {
        dragState.current = { x: e.clientX, y: e.clientY }
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
        return
      }

      const range = getMeasureAtPoint(e.clientX, e.clientY)
      if (range !== undefined) {
        dragState.current = range
        const min = Math.min(dragState.anchor.start, dragState.current.start)
        const max = Math.max(dragState.anchor.end, dragState.current.end)
        applyPersistedNoteHighlights(
          container,
          noteCellsInMeasureRange(noteSpansRef.current, {
            start: min,
            end: max,
          }),
        )
      }
    }

    const handleMouseUp = (e: MouseEvent) => {
      const dragState = dragStateRef.current
      if (!dragState) return
      const container = previewPagesRef.current

      if (dragState.mode === 'pending') {
        // Never armed into a note-drag — a plain click, which is a shortcut
        // for selecting every note/rest cell in the clicked note's measure.
        const cells =
          dragState.measureRangeAtAnchor !== undefined
            ? noteCellsInMeasureRange(
                noteSpansRef.current,
                dragState.measureRangeAtAnchor,
              )
            : [dragState.noteCellAtAnchor]
        if (container) applyPersistedNoteHighlights(container, cells)
        onNoteRangeSelectRef.current?.(cells)
        dragStateRef.current = null
        return
      }

      if (dragState.mode === 'note') {
        const current = { x: e.clientX, y: e.clientY }
        // Leave the highlight as the drag left it — don't clear it here.
        // `onNoteRangeSelectRef` feeds `selectedNoteCells` back in, and
        // `Preview`'s declarative effect keeps it applied (and re-applies it
        // if a re-render swaps the DOM out from under it).
        const cells = container
          ? applyNoteDragHighlights(container, dragState.anchor, current)
          : []
        onNoteRangeSelectRef.current?.(cells)
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
        if (container) {
          applyPersistedNoteHighlights(container, cells)
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
        onNoteRangeSelectRef.current?.(cells)
        dragStateRef.current = null
        return
      }

      const finalRange =
        getMeasureAtPoint(e.clientX, e.clientY) ?? dragState.current
      const min = Math.min(dragState.anchor.start, finalRange.start)
      const max = Math.max(dragState.anchor.end, finalRange.end)
      const cells = noteCellsInMeasureRange(noteSpansRef.current, {
        start: min,
        end: max,
      })
      if (container) applyPersistedNoteHighlights(container, cells)
      onNoteRangeSelectRef.current?.(cells)
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
