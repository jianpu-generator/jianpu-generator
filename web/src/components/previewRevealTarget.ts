import { tagSelector } from '../dataAttributes'

interface RevealSelection {
  revealMeasureIndex: number
  highlightRanges?: { start: number; end: number }[]
}

/** The element the preview scrolls into view for a measure selection.
 * A `# sequence` chain selection's `highlightRanges` can paint several
 * disjoint highlight rects (e.g. "Intro" and "C"), and the first one in DOM
 * order is wherever the chain's earliest entry sits in the document — not
 * necessarily `revealMeasureIndex`. So only the caret-only highlight (a
 * single contiguous range) is targeted by its rect; a chain selection
 * always scrolls to its reveal measure. */
export function findRevealTarget(
  container: Element,
  selection: RevealSelection,
): Element | null {
  const highlightRect =
    selection.highlightRanges === undefined
      ? container.querySelector('[data-testid="measure-highlight"]')
      : null
  return (
    highlightRect ??
    container.querySelector(
      tagSelector('measure', { index: selection.revealMeasureIndex }),
    )
  )
}
