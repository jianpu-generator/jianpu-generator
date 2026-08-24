import type { RefObject } from 'react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import type { EditorHandle, MeasureSpan, SequenceEntry } from '../types'

interface SelectedSequenceRange {
  start: number
  end: number
  entryStartIndex: number
  entryEndIndex: number
}

interface LineRange {
  startLine: number
  endLine: number
}

/**
 * Resolves a single `# sequence` entry (by its 0-based index into
 * `sequenceEntries`) to its own source line range. A repeated label (e.g.
 * the closing `Intro` in `Intro, A, B, C, Intro`) resolves to the same
 * written measures — and therefore the same lines — as every other
 * occurrence, since `# sequence` only replays already-written measures
 * rather than duplicating them in the source.
 */
function resolveEntryLineRange(
  sequenceEntries: SequenceEntry[],
  measureSpans: MeasureSpan[],
  index: number,
): LineRange | null {
  const entry = sequenceEntries[index]
  const startSpan = measureSpans[entry?.start_measure_index ?? -1]
  const endSpan = measureSpans[entry?.end_measure_index ?? -1]
  if (!startSpan || !endSpan) return null
  return { startLine: startSpan.start_line, endLine: endSpan.end_line }
}

/**
 * Resolves a chain-order range of `# sequence` entries (indices into
 * `sequenceEntries`, already sorted so `startIndex <= endIndex`) to one
 * source line range per entry.
 *
 * A chain can reference sections out of document order (e.g. `c, a` when
 * the file declares sections `a, b, c`), so the entries aren't necessarily
 * contiguous, or even ascending, in the source. Returning one range per
 * entry — rather than collapsing the whole selection into a single
 * min/max span — lets the caller build a genuinely disjoint Monaco
 * selection that covers exactly the selected entries and nothing that
 * merely sits between them in the document (e.g. "b", when the chain is
 * "c, a"). See the regression test for the concrete case.
 *
 * An entry with no resolvable measure span (out of bounds, or referencing a
 * measure index `measureSpans` doesn't have) is skipped rather than failing
 * the whole selection.
 */
export function computeSequenceSelectionLineRanges(
  sequenceEntries: SequenceEntry[],
  measureSpans: MeasureSpan[],
  startIndex: number,
  endIndex: number,
): LineRange[] {
  const ranges: LineRange[] = []
  for (let i = startIndex; i <= endIndex; i++) {
    const range = resolveEntryLineRange(sequenceEntries, measureSpans, i)
    if (range) ranges.push(range)
  }
  return ranges
}

/**
 * The smallest single line range spanning every given range — used where a
 * single contiguous range is unavoidable (the `selectedMeasureRange`
 * envelope that drives the selection badge, "play selection", the
 * keyboard-shortcut gate, and the preview's scroll-to-selection target),
 * as opposed to `computeSequenceSelectionLineRanges`'s disjoint per-entry
 * ranges, which drive the actual Monaco selection.
 */
export function envelopeOfLineRanges(ranges: LineRange[]): LineRange | null {
  if (ranges.length === 0) return null
  return {
    startLine: Math.min(...ranges.map((r) => r.startLine)),
    endLine: Math.max(...ranges.map((r) => r.endLine)),
  }
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

      // A sequence entry only carries measure indices, not lines, so
      // resolve each selected entry to its own source line range via
      // `measureSpans` first — one range per entry, since the chain can
      // reference sections out of document order (e.g. `c, a`), so a single
      // min/max span would also select whatever sits between them (`b`).
      const lineRanges = computeSequenceSelectionLineRanges(
        sequenceEntries,
        measureSpans,
        start,
        end,
      )
      if (lineRanges.length === 0) return
      const envelope = envelopeOfLineRanges(lineRanges)
      if (!envelope) return

      // Two disjoint entries can sit far apart in a large score (e.g. a
      // repeated `Intro` resolves to the same written lines as its first
      // occurrence, way earlier in the source than a later entry it's
      // selected alongside) — too far apart to both fit on screen at once.
      // Reveal wherever the drag actually ended (`indexB`, not the smaller
      // of the two chain indices `start`/`end` above) so the editor scrolls
      // to the entry the user is currently pointing at, not always
      // whichever one happens to sit first in the chain.
      const revealRange =
        resolveEntryLineRange(sequenceEntries, measureSpans, indexB) ??
        lineRanges[0]

      // The actual Monaco selection: one disjoint range per entry.
      editorRef.current?.setSelectionsByLines(lineRanges, revealRange.startLine)
      // Mirrors `useSectionNavigation`'s `selectSectionRange` for everything
      // else that only understands a single contiguous range (the selection
      // badge, "play selection", the keyboard-shortcut gate, and the
      // preview's scroll-to-selection target) — see `envelopeOfLineRanges`'s
      // doc comment. A sequence jump selects a real (non-empty) range, so —
      // like section jumps — the preview's caret-only measure-background
      // highlight stays off; the toolbar buttons carry their own
      // highlighting instead.
      notifySelection(envelope.startLine, envelope.endLine, false)
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
