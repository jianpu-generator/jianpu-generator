import type { RefObject } from 'react'
import { useEffect, useRef } from 'react'
import type { LyricSpan, NoteSpan } from '../types'
import { cancelAnchor } from './previewClickHandler'
import {
  applyLyricDragHighlights,
  applyNoteDragHighlights,
  applyPersistedLyricHighlights,
  applyPersistedNoteHighlights,
} from './previewDragHighlights'
import type { PreviewDragState } from './previewDragState'
import {
  applyLyricLabelDragHighlight,
  applyPartLabelDragHighlight,
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
  getMeasureAtPoint,
  type LyricCell,
  lyricCellsInMeasureRange,
  type NoteCell,
  noteCellsInMeasureRange,
} from './previewSelection'

export type { PreviewDragState } from './previewDragState'

/** Owns the note/measure/part-label click-and-click selection gesture for
 * `Preview`: a first click (handled by `Preview` itself via
 * `handlePreviewClick`, which writes the anchored mode into the returned
 * ref) anchors one of `PreviewDragState`'s modes, and the document-level
 * `mousemove` listener registered here live-updates the hover preview
 * between the anchor and the pointer for mouse users (a no-op for touch,
 * which has no hover) until a second click — also routed through
 * `handlePreviewClick` — resolves and commits it. A document-level
 * `keydown` listener cancels the anchored gesture back to idle on Escape,
 * the click-click model's equivalent of releasing a held button to abort.
 * Split out of `Preview` to keep that component under its line-count cap. */
export function usePreviewClickSelection(
  previewPagesRef: RefObject<HTMLDivElement | null>,
  noteSpans: NoteSpan[],
  onNoteRangeSelect: ((selectedCells: NoteCell[]) => void) | undefined,
  onLyricRangeSelect?: (selectedCells: LyricCell[]) => void,
  lyricSpans: LyricSpan[] = [],
  // Fired instead of `onNoteRangeSelect`/`onLyricRangeSelect` for a
  // measure/bar-line click, which resolves both note cells and lyric cells
  // at once — see `useAppController`'s `handleMeasureRangeSelect` for why
  // those two can't just be called back-to-back (each independently pushes
  // its own Monaco selection, and the second call's push clobbers the
  // first's).
  onMeasureRangeSelect?: (
    noteCells: NoteCell[],
    lyricCells: LyricCell[],
  ) => void,
) {
  // The mousemove/keydown handlers below live in a `useEffect(() => {...},
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

      if (dragState.mode === 'note') {
        dragState.current = { x: e.clientX, y: e.clientY }
        applyNoteDragHighlights(container, dragState.anchor, dragState.current)
        applyLyricDragHighlights(container, dragState.anchor, dragState.current)
        return
      }

      if (dragState.mode === 'lyric') {
        dragState.current = { x: e.clientX, y: e.clientY }
        applyLyricDragHighlights(container, dragState.anchor, dragState.current)
        // Symmetric to 'note' mode above — a lyric hover preview can also
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
        // as every other mode above.
        applyPersistedLyricHighlights(
          container,
          lyricCellsForPartLabels(lyricSpansRef.current, hits),
        )
        return
      }

      if (dragState.mode === 'part-label-system') {
        dragState.current = { x: e.clientX, y: e.clientY }
        const hits = partLabelsInMarqueeAcrossSystems(
          container,
          dragState.anchor,
          dragState.current,
        )
        applyPartLabelDragHighlight(container, hits)
        applyPersistedNoteHighlights(
          container,
          noteCellsForPartLabels(noteSpansRef.current, hits),
        )
        // Swept systems carry their lyric rows too — union those in, same as
        // 'part-label' mode above.
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

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return
      const dragState = dragStateRef.current
      if (!dragState) return
      cancelAnchor(dragStateRef, dragState, {
        dragStateRef,
        previewPagesRef,
        noteSpans: noteSpansRef.current,
        lyricSpans: lyricSpansRef.current,
        onSectionLabelClick: undefined,
        onNoteRangeSelect: onNoteRangeSelectRef.current,
        onLyricRangeSelect: onLyricRangeSelectRef.current,
        onMeasureRangeSelect: onMeasureRangeSelectRef.current,
      })
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [previewPagesRef])

  return dragStateRef
}
