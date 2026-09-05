import type { RefObject } from 'react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import type { EditorHandle, MeasureSpan, SectionRange } from '../types'
import { measureRangeInSpan } from './workerHelpers'

export function useSectionNavigation(
  sectionRanges: SectionRange[],
  editorRef: RefObject<EditorHandle | null>,
  /** Resolves a section's line range to its own measure-index range, so a
   * no-mounted-editor jump (below) can hand it to `notifySelection` as an
   * explicit highlight range — see that branch's own comment. */
  measureSpans: MeasureSpan[],
  notifySelection: (
    firstLine: number,
    lastLine: number,
    isEmpty: boolean,
    revealLine?: number,
    measureRanges?: { start: number; end: number }[],
  ) => void,
  /** Drops whatever stale note/lyric/measure highlight a prior no-mounted-
   * editor (Live/shared view) tap or bar-line click left painted — see
   * `useAppSelectionAndNavigation`'s wiring. A section jump replaces that
   * highlight entirely rather than layering on top of it, but (unlike the
   * editor-mounted path, where pushing a real Monaco selection round-trips
   * back through `handleEditorSelectionChange` and naturally re-derives
   * these) there's no Monaco selection here to do that for it. */
  clearNoMountedEditorHighlights: () => void,
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
      if (!editorRef.current) {
        // No mounted editor (Live/shared view): there's no Monaco selection
        // round-trip to re-derive note/lyric/measure highlighting from (see
        // this hook's own `clearNoMountedEditorHighlights` param doc
        // comment), so this jump must clear it itself, or a prior tap/
        // bar-line click's highlight would otherwise keep painting over the
        // section it just jumped to.
        clearNoMountedEditorHighlights()
        setSelectedLineRange({ firstLine, lastLine })
        // With no Monaco selection to echo back a highlight from, this is
        // the *only* paint this jump gets — unlike the editor-mounted
        // branch below, where `setSelectionByLines` round-trips through
        // `handleEditorSelectionChange` to blue-highlight the section's
        // notes/lyrics. Passing an explicit measure range here (mirroring
        // `useSequenceNavigation`'s own `measureRanges` argument) makes
        // `notifySelection` paint the amber whole-measure background for
        // it instead, bypassing the caret-only gate that would otherwise
        // leave a section jump invisible in this view.
        const measureRange = measureRangeInSpan(
          measureSpans,
          firstLine,
          lastLine,
        )
        notifySelection(
          firstLine,
          lastLine,
          false,
          undefined,
          measureRange ? [measureRange] : undefined,
        )
        return
      }
      editorRef.current.setSelectionByLines(firstLine, lastLine)
      editorRef.current.focus()
      setSelectedLineRange({ firstLine, lastLine })
      // A section jump selects a real (non-empty) line range, so the
      // preview's caret-only measure-background highlight stays off here;
      // the Monaco selection echo above blue-highlights the section's
      // notes/lyrics instead, and the section buttons carry their own
      // highlighting too.
      notifySelection(firstLine, lastLine, false)
    },
    [editorRef, measureSpans, notifySelection, clearNoMountedEditorHighlights],
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

  return {
    selectedLineRange,
    setSelectedLineRange,
    handleSectionJump,
    sectionJumpToolbarProps: {
      sectionLabels,
      dragStartLabel,
      setDragStartLabel,
      setDragCurrentLabel,
      activeHighlightedLabels,
      handleSectionJump,
      handleSectionRangeSelect,
    },
  }
}
