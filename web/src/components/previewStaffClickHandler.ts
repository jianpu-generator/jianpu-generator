import type { MouseEvent } from 'react'
import type { StaffMeasure } from '../jianpuWasm'
import {
  anchorAndCommit,
  cancelAnchor,
  commitAnchoredGesture,
} from './previewGestureSteps'
import {
  type HandlePreviewClickArgs,
  measureClickableElementId,
} from './previewSelectionResolver'
import { staffMeasureRangeAtPoint } from './previewStaffMeasures'

/**
 * The staff view's `mousedown` dispatch (`handlePreviewClick` hands off here
 * when `staffMeasures` is set): the same click-and-click gesture, but only
 * at measure granularity — Verovio's SVG has none of the jianpu preview's
 * note/lyric/label click targets. A first click on a measure anchors 'measure' mode, a second click on a measure commits
 * the range between them, and a click outside every measure cancels an
 * anchored gesture. Both commits go through the editor's selection exactly
 * like the jianpu view's, which is what drives the staff highlight
 * (`useStaffMeasureHighlights`) and measure playback.
 */
export function handleStaffPreviewClick(
  e: MouseEvent<HTMLDivElement>,
  args: HandlePreviewClickArgs,
  measures: StaffMeasure[],
): void {
  const { anchorStateRef, previewPagesRef } = args
  const container = previewPagesRef.current
  const range =
    container &&
    staffMeasureRangeAtPoint(container, measures, e.clientX, e.clientY)
  const anchorState = anchorStateRef.current
  if (!range) {
    if (anchorState !== null) cancelAnchor(anchorStateRef, anchorState, args)
    return
  }
  e.preventDefault()
  const id = measureClickableElementId(range)
  if (anchorState?.mode === 'measure') {
    commitAnchoredGesture(anchorState, { x: e.clientX, y: e.clientY }, id, args)
    return
  }
  anchorAndCommit(
    anchorStateRef,
    { mode: 'measure', anchor: range, current: range, anchorId: id },
    args,
  )
}
