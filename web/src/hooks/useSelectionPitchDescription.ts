import { useEffect, useRef, useState } from 'react'
import type { EditorSelection, PitchDescription } from '../types'

/** One answer from `describeSelection`. `id` is fresh for every committed
 * selection, even one that re-selects the same note, so the drawer can tell
 * a new selection apart from the one it was dismissed on. */
export interface DescribedSelection {
  id: number
  description: PitchDescription
}

/** Keeps the pitch drawer's description in step with the editor's
 * committed selection. Holds the previous answer while a click-and-click
 * gesture waits on its second click, and re-queries once it commits. */
export function useSelectionPitchDescription(
  selectionByteRanges: EditorSelection[] | null,
  selectionPending: boolean,
  describeSelection: (
    ranges: EditorSelection[],
  ) => Promise<PitchDescription | null>,
): DescribedSelection | null {
  const [described, setDescribed] = useState<DescribedSelection | null>(null)
  const nextIdRef = useRef(0)
  // Read through refs so neither a new ranges array holding the same
  // offsets (every Monaco cursor echo) nor `describeSelection`'s per-render
  // identity re-runs the query below.
  const describeSelectionRef = useRef(describeSelection)
  describeSelectionRef.current = describeSelection
  const rangesRef = useRef(selectionByteRanges)
  rangesRef.current = selectionByteRanges
  const rangesKey =
    selectionByteRanges === null ? null : JSON.stringify(selectionByteRanges)

  useEffect(() => {
    if (selectionPending) return
    const ranges = rangesRef.current
    if (rangesKey === null || ranges === null) {
      setDescribed(null)
      return
    }
    let cancelled = false
    void describeSelectionRef.current(ranges).then((description) => {
      if (cancelled) return
      setDescribed(
        description === null ? null : { id: ++nextIdRef.current, description },
      )
    })
    return () => {
      cancelled = true
    }
  }, [rangesKey, selectionPending])

  return described
}
