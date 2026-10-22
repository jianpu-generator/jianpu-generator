import { type RefObject, useEffect } from 'react'
import type { StaffScore } from '../worker/messages'
import type { PreviewAnchorState } from './previewAnchorState'
import type { MeasureRange } from './previewSelection'
import { applyStaffMeasureHighlights } from './previewStaffMeasures'

export interface SelectedMeasures extends MeasureRange {
  highlightRanges?: MeasureRange[]
}

/** The staff view's counterpart of the jianpu measure highlight: paints the
 * editor's selected measures (a `# sequence` chain's exact disjoint ranges
 * when it has them) over Verovio's pages, re-applied whenever those pages
 * are swapped in. Skipped while a click-and-click gesture is anchored: its
 * hover preview owns the highlight until it commits or cancels (see
 * `hoverStaffMeasure`/`cancelAnchor`), and the anchoring click's own
 * debounced editor selection would otherwise paint over it. */
export function useStaffMeasureHighlights(
  previewPagesRef: RefObject<HTMLDivElement | null>,
  staffScore: StaffScore | null,
  selectedMeasureRange: SelectedMeasures | null,
  anchorStateRef: RefObject<PreviewAnchorState>,
): void {
  useEffect(() => {
    const container = previewPagesRef.current
    if (!container || !staffScore || anchorStateRef.current) return
    const ranges = selectedMeasureRange
      ? (selectedMeasureRange.highlightRanges ?? [selectedMeasureRange])
      : []
    applyStaffMeasureHighlights(container, staffScore.measures, ranges)
  }, [previewPagesRef, staffScore, selectedMeasureRange, anchorStateRef])
}
