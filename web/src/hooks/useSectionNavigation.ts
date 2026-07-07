import type { RefObject } from 'react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import type { EditorHandle, MeasureSpan, SectionRange } from '../types'

export function useSectionNavigation(
  sectionRanges: SectionRange[],
  measureSpans: MeasureSpan[],
  editorRef: RefObject<EditorHandle | null>,
  notifySelection: (firstLine: number, lastLine: number) => void,
) {
  const [dragStartLabel, setDragStartLabel] = useState<string | null>(null)
  const [dragCurrentLabel, setDragCurrentLabel] = useState<string | null>(null)
  const [selectedLineRange, setSelectedLineRange] = useState<{
    firstLine: number
    lastLine: number
  } | null>(null)

  const sectionLabels = useMemo(
    () =>
      sectionRanges
        .filter((r) => r.labels.length === 1)
        .flatMap((r) => r.labels),
    [sectionRanges],
  )

  const dragHighlightedLabels = useMemo<Set<string>>(() => {
    if (dragStartLabel === null || dragCurrentLabel === null) return new Set()
    const a = sectionLabels.indexOf(dragStartLabel)
    const b = sectionLabels.indexOf(dragCurrentLabel)
    if (a === -1 || b === -1) return new Set()
    return new Set(sectionLabels.slice(Math.min(a, b), Math.max(a, b) + 1))
  }, [dragStartLabel, dragCurrentLabel, sectionLabels])

  const activeHighlightedLabels = useMemo(() => {
    if (dragStartLabel !== null) return dragHighlightedLabels
    if (!selectedLineRange) return new Set<string>()
    const match = sectionRanges.find(
      (r) =>
        r.first_line === selectedLineRange.firstLine &&
        r.last_line === selectedLineRange.lastLine,
    )
    return new Set(match?.labels ?? [])
  }, [dragStartLabel, dragHighlightedLabels, selectedLineRange, sectionRanges])

  const selectSectionRange = useCallback(
    (firstLine: number, lastLine: number) => {
      editorRef.current?.setSelectionByLines(firstLine, lastLine)
      editorRef.current?.focus()
      setSelectedLineRange({ firstLine, lastLine })
      notifySelection(firstLine, lastLine)
    },
    [editorRef, notifySelection],
  )

  const handleSectionRangeSelect = useCallback(
    (labelA: string, labelB: string) => {
      const range =
        sectionRanges.find(
          (r) => r.labels[0] === labelA && r.labels.at(-1) === labelB,
        ) ??
        sectionRanges.find(
          (r) => r.labels[0] === labelB && r.labels.at(-1) === labelA,
        )
      if (!range) return
      selectSectionRange(range.first_line, range.last_line)
    },
    [sectionRanges, selectSectionRange],
  )

  const handleSectionJump = useCallback(
    (label: string) => handleSectionRangeSelect(label, label),
    [handleSectionRangeSelect],
  )

  useEffect(() => {
    const clearDrag = () => {
      setDragStartLabel(null)
      setDragCurrentLabel(null)
    }
    window.addEventListener('mouseup', clearDrag)
    return () => window.removeEventListener('mouseup', clearDrag)
  }, [])

  const handleMeasureRangeSelect = useCallback(
    (start: number, end: number) => {
      const s = measureSpans[start]
      const e = measureSpans[end]
      if (!s || !e) return
      editorRef.current?.setSelectionByLines(s.start_line, e.end_line)
    },
    [measureSpans, editorRef],
  )

  return {
    selectedLineRange,
    setSelectedLineRange,
    sectionLabels,
    dragStartLabel,
    setDragStartLabel,
    dragCurrentLabel,
    setDragCurrentLabel,
    activeHighlightedLabels,
    handleSectionRangeSelect,
    handleSectionJump,
    handleMeasureRangeSelect,
  }
}
