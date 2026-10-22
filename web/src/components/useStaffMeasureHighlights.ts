import { type RefObject, useEffect } from 'react'
import type { StaffScore } from '../worker/messages'
import type { MeasureRange } from './previewSelection'
import { applyStaffMeasureHighlights } from './previewStaffMeasures'

interface SelectedMeasures extends MeasureRange {
  highlightRanges?: MeasureRange[]
}

/** The staff view's counterpart of the jianpu measure highlight: paints the
 * editor's selected measures (a `# sequence` chain's exact disjoint ranges
 * when it has them) over Verovio's pages, re-applied whenever those pages
 * are swapped in. */
export function useStaffMeasureHighlights(
  previewPagesRef: RefObject<HTMLDivElement | null>,
  staffScore: StaffScore | null,
  selectedMeasureRange: SelectedMeasures | null,
): void {
  useEffect(() => {
    const container = previewPagesRef.current
    if (!container || !staffScore) return
    const ranges = selectedMeasureRange
      ? (selectedMeasureRange.highlightRanges ?? [selectedMeasureRange])
      : []
    applyStaffMeasureHighlights(container, staffScore.measures, ranges)
  }, [previewPagesRef, staffScore, selectedMeasureRange])
}
