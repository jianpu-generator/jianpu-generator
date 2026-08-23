import type { RefObject } from 'react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import type { EditorHandle, MeasureSpan, SequenceEntry } from '../types'

interface SelectedSequenceRange {
  start: number
  end: number
  entryStartIndex: number
  entryEndIndex: number
}

export function useSequenceNavigation(
  sequenceEntries: SequenceEntry[],
  measureSpans: MeasureSpan[],
  editorRef: RefObject<EditorHandle | null>,
  notifySelection: (
    firstLine: number,
    lastLine: number,
    isEmpty: boolean,
  ) => void,
  /**
   * Owned by the caller (`useAppController`) rather than this hook, and
   * also handed to `useMeasureAudioPlayback` (inside `useJianpuWorker`,
   * constructed before this hook runs since this hook needs
   * `useJianpuWorker`'s own `notifySelection` output) — a ref lets that
   * earlier-constructed consumer read the latest selection at click time
   * without a circular dependency between the two hooks.
   */
  selectedSequenceRangeRef: RefObject<SelectedSequenceRange | null>,
) {
  const [dragStartIndex, setDragStartIndex] = useState<number | null>(null)
  const [dragCurrentIndex, setDragCurrentIndex] = useState<number | null>(null)
  const [selectedIndexRange, setSelectedIndexRange] = useState<{
    start: number
    end: number
  } | null>(null)

  const dragHighlightedIndices = useMemo<Set<number>>(() => {
    if (dragStartIndex === null || dragCurrentIndex === null) return new Set()
    const min = Math.min(dragStartIndex, dragCurrentIndex)
    const max = Math.max(dragStartIndex, dragCurrentIndex)
    const indices = new Set<number>()
    for (let i = min; i <= max; i++) indices.add(i)
    return indices
  }, [dragStartIndex, dragCurrentIndex])

  const activeHighlightedIndices = useMemo(() => {
    if (dragStartIndex !== null) return dragHighlightedIndices
    if (!selectedIndexRange) return new Set<number>()
    const indices = new Set<number>()
    for (let i = selectedIndexRange.start; i <= selectedIndexRange.end; i++) {
      indices.add(i)
    }
    return indices
  }, [dragStartIndex, dragHighlightedIndices, selectedIndexRange])

  const selectedSequenceRange = useMemo(() => {
    if (!selectedIndexRange) return null
    const startEntry = sequenceEntries[selectedIndexRange.start]
    const endEntry = sequenceEntries[selectedIndexRange.end]
    if (!startEntry || !endEntry) return null
    return {
      start: startEntry.start_measure_index,
      end: endEntry.end_measure_index,
      // The selected entries' own 0-based index into `# sequence`, needed
      // to disambiguate a repeated label (e.g. `A, B(-x), B`): every
      // occurrence shares the same written measure range above, so without
      // this the backend can't tell which occurrence was actually clicked.
      entryStartIndex: selectedIndexRange.start,
      entryEndIndex: selectedIndexRange.end,
    }
  }, [selectedIndexRange, sequenceEntries])

  const handleSequenceEntryRangeSelect = useCallback(
    (indexA: number, indexB: number) => {
      const start = Math.min(indexA, indexB)
      const end = Math.max(indexA, indexB)
      setSelectedIndexRange({ start, end })

      // Mirrors `useSectionNavigation`'s `selectSectionRange`: push the
      // range into the Monaco selection and the SVG preview's highlight
      // pipeline. A sequence entry only carries measure indices, not
      // lines, so resolve each end's measure index to a source line via
      // `measureSpans` first.
      const startSpan =
        measureSpans[sequenceEntries[start]?.start_measure_index ?? -1]
      const endSpan =
        measureSpans[sequenceEntries[end]?.end_measure_index ?? -1]
      if (!startSpan || !endSpan) return

      editorRef.current?.setSelectionByLines(
        startSpan.start_line,
        endSpan.end_line,
      )
      editorRef.current?.focus()
      // A sequence jump selects a real (non-empty) range, so — like section
      // jumps — the preview's caret-only measure-background highlight stays
      // off; the toolbar buttons carry their own highlighting instead.
      notifySelection(startSpan.start_line, endSpan.end_line, false)
    },
    [sequenceEntries, measureSpans, editorRef, notifySelection],
  )

  const handleSequenceEntryClick = useCallback(
    (index: number) => handleSequenceEntryRangeSelect(index, index),
    [handleSequenceEntryRangeSelect],
  )

  useEffect(() => {
    const clearDrag = () => {
      setDragStartIndex(null)
      setDragCurrentIndex(null)
    }
    window.addEventListener('mouseup', clearDrag)
    return () => window.removeEventListener('mouseup', clearDrag)
  }, [])

  // Kept in sync every render (not via effect) so `useMeasureAudioPlayback`
  // can read the latest selection at click time through the stable ref —
  // see the parameter doc comment above for why the ref is owned by the
  // caller instead of this hook.
  selectedSequenceRangeRef.current = selectedSequenceRange

  return {
    selectedSequenceRange,
    sequenceJumpToolbarProps: {
      sequenceEntries,
      dragStartIndex,
      setDragStartIndex,
      setDragCurrentIndex,
      activeHighlightedIndices,
      handleSequenceEntryClick,
      handleSequenceEntryRangeSelect,
    },
  }
}
