import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { SequenceEntry } from '../types'

export function useSequenceNavigation(sequenceEntries: SequenceEntry[]) {
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
    }
  }, [selectedIndexRange, sequenceEntries])

  const handleSequenceEntryRangeSelect = useCallback(
    (indexA: number, indexB: number) => {
      setSelectedIndexRange({
        start: Math.min(indexA, indexB),
        end: Math.max(indexA, indexB),
      })
    },
    [],
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

  /**
   * Kept in sync every render (not via effect) so `useMeasureAudioPlayback`,
   * which is constructed before this hook runs, can read the latest
   * selection at click time through a stable ref.
   */
  const selectedSequenceRangeRef = useRef<{
    start: number
    end: number
  } | null>(null)
  selectedSequenceRangeRef.current = selectedSequenceRange

  return {
    selectedSequenceRange,
    selectedSequenceRangeRef,
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
