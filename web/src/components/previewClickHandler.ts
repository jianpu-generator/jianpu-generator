import type { MouseEvent, RefObject } from 'react'
import type { NoteSpan } from '../types'
import type { PreviewDragState } from './previewDragState'
import {
  getLyricLabelAtPoint,
  getPartLabelAtPoint,
} from './previewLabelSelection'
import {
  getBarLineMeasureAtPoint,
  getBarNumberMeasureAtPoint,
  getLyricAtPoint,
  getMeasureAtPoint,
  getNoteAtPoint,
  getSectionLabelAtPoint,
  nearestNoteCellInMeasureRange,
} from './previewSelection'
import {
  fireCommit,
  type HandlePreviewClickArgs,
  resolveSelection,
} from './previewSelectionResolver'

export type { HandlePreviewClickArgs } from './previewSelectionResolver'

/** Anchors `dragStateRef` to `newState` and immediately self-commits its
 * single-target resolution (matching a plain click's long-standing
 * instant-select behavior — see the single-click e2e specs) — but, unlike
 * the old held-button drag model, leaves `dragStateRef` anchored rather than
 * resetting to idle: a second click can still land elsewhere and widen this
 * into a real range (see `handleCommitClick`). */
function anchorAndCommit(
  dragStateRef: RefObject<PreviewDragState>,
  newState: NonNullable<PreviewDragState>,
  args: HandlePreviewClickArgs,
): void {
  dragStateRef.current = newState
  fireCommit(resolveSelection(newState, undefined, args), args)
}

/** Resets `dragStateRef` to idle and re-applies the highlight `dragState`'s
 * anchoring click already committed — used both when a second click misses
 * every recognizable target and when the gesture is cancelled via Escape
 * (see `usePreviewClickSelection`). No callback fires: the anchoring click's
 * own commit already did, and nothing about that selection has changed. */
export function cancelAnchor(
  dragStateRef: RefObject<PreviewDragState>,
  dragState: NonNullable<PreviewDragState>,
  args: HandlePreviewClickArgs,
): void {
  resolveSelection(dragState, undefined, args)
  dragStateRef.current = null
}

/** Whether `(x, y)` doesn't land on anything this gesture can resolve a
 * selection from — a second click here cancels the anchored gesture rather
 * than committing a nonsensical range (see `PreviewDragState`'s doc comment
 * and this module's `handleCommitClick`). Mirrors the same hit-test chain
 * `handleAnchorClick` uses for a first click, since anything that would
 * anchor a *new* gesture also counts as a recognizable target for
 * *resolving* one already in progress. */
function isEmptySpace(x: number, y: number, noteSpans: NoteSpan[]): boolean {
  if (getSectionLabelAtPoint(x, y) !== undefined) return false
  if (getPartLabelAtPoint(x, y) !== undefined) return false
  if (getLyricLabelAtPoint(x, y) !== undefined) return false
  if (getBarLineMeasureAtPoint(x, y) !== undefined) return false
  if (getBarNumberMeasureAtPoint(x, y) !== undefined) return false
  if (getLyricAtPoint(x, y) !== undefined) return false
  if (getNoteAtPoint(x, y) !== undefined) return false
  const range = getMeasureAtPoint(x, y)
  if (range === undefined) return true
  return nearestNoteCellInMeasureRange(noteSpans, range, x, y) === undefined
}

/** The first click of a click-and-click gesture: figures out what got
 * clicked (a section label, a part/lyric label, a note/chord, a lyric
 * syllable, or plain measure space), anchors `dragStateRef` with the mode
 * that gesture should carry through, and self-commits that single-target
 * selection immediately (see `anchorAndCommit`). `handlePreviewClick`
 * dispatches here when `dragStateRef` is idle. */
function handleAnchorClick(
  e: MouseEvent<HTMLDivElement>,
  args: HandlePreviewClickArgs,
): void {
  const { dragStateRef, noteSpans, onSectionLabelClick } = args
  const sectionLabel = getSectionLabelAtPoint(e.clientX, e.clientY)
  if (sectionLabel !== undefined) {
    onSectionLabelClick?.(sectionLabel)
    e.preventDefault()
    return
  }
  const partLabel = getPartLabelAtPoint(e.clientX, e.clientY)
  if (partLabel !== undefined) {
    const point = { x: e.clientX, y: e.clientY }
    // Cmd/Ctrl-click on a part label elevates the selection from "this one
    // part's system" to "every part in every system touched" — see
    // `PreviewDragState`'s 'part-label-system' doc comment. Checked ahead of
    // the plain part-label anchor below so it takes priority.
    if (e.metaKey || e.ctrlKey) {
      anchorAndCommit(
        dragStateRef,
        { mode: 'part-label-system', anchor: point, current: point },
        args,
      )
      e.preventDefault()
      return
    }
    anchorAndCommit(
      dragStateRef,
      {
        mode: 'part-label',
        anchor: point,
        current: point,
        anchorSystem: {
          measureIndexStart: partLabel.measureIndexStart,
          measureIndexEnd: partLabel.measureIndexEnd,
        },
      },
      args,
    )
    e.preventDefault()
    return
  }
  // The lyric-side mirror of the part-label check above — a verse row's own
  // label (e.g. "M:v1"), scoped to that one verse instead of a whole part.
  const lyricLabel = getLyricLabelAtPoint(e.clientX, e.clientY)
  if (lyricLabel !== undefined) {
    const point = { x: e.clientX, y: e.clientY }
    anchorAndCommit(
      dragStateRef,
      {
        mode: 'lyric-label',
        anchor: point,
        current: point,
        anchorSystem: {
          measureIndexStart: lyricLabel.measureIndexStart,
          measureIndexEnd: lyricLabel.measureIndexEnd,
        },
      },
      args,
    )
    e.preventDefault()
    return
  }
  // Grabbing a bar line's own divider always anchors a measure-range
  // selection, no Cmd/Ctrl required: the divider is a dedicated click
  // handle (see `renderBarLineDragHandle`), so landing on it is an
  // unambiguous request to select measures, unlike a plain click on a
  // note/lyric/gutter pixel (ambiguous enough to need the modifier gate
  // below).
  const barLineRange = getBarLineMeasureAtPoint(e.clientX, e.clientY)
  if (barLineRange !== undefined) {
    anchorAndCommit(
      dragStateRef,
      { mode: 'measure', anchor: barLineRange, current: barLineRange },
      args,
    )
    e.preventDefault()
    return
  }
  // Grabbing a measure's own bar number (drawn in the directive row above)
  // always anchors a measure-range selection too, no Cmd/Ctrl required —
  // same rationale as the bar-line-handle check above: landing on the bar
  // number itself is an unambiguous request to select that measure, unlike
  // a click on a note/lyric/gutter pixel.
  const barNumberRange = getBarNumberMeasureAtPoint(e.clientX, e.clientY)
  if (barNumberRange !== undefined) {
    anchorAndCommit(
      dragStateRef,
      { mode: 'measure', anchor: barNumberRange, current: barNumberRange },
      args,
    )
    e.preventDefault()
    return
  }
  // Cmd/Ctrl-click always selects the whole measure under the pointer,
  // regardless of what structurally sits under it (note, chord, lyric,
  // bar-line, or empty gutter) — checked ahead of the lyric/note checks
  // below so it takes priority over them. Off a bar line, this is the only
  // way to reach 'measure' mode; a plain click elsewhere resolves to
  // note/chord/syllable granularity instead (see `PreviewDragState`'s doc
  // comment).
  if (e.metaKey || e.ctrlKey) {
    const range = getMeasureAtPoint(e.clientX, e.clientY)
    if (range !== undefined) {
      anchorAndCommit(
        dragStateRef,
        { mode: 'measure', anchor: range, current: range },
        args,
      )
      e.preventDefault()
      return
    }
  }
  // Checked before the note click-target below: a lyric syllable's own
  // click target paints on top of (and never overlaps outside of) the
  // note's wider click-target rect, so a hit here means the click landed on
  // the syllable's own rect — see `Tag::Lyric`'s doc comment and
  // `resolve_click_target_elements`'s append order.
  const lyricCell = getLyricAtPoint(e.clientX, e.clientY)
  if (lyricCell !== undefined) {
    const point = { x: e.clientX, y: e.clientY }
    anchorAndCommit(
      dragStateRef,
      {
        mode: 'lyric',
        anchor: point,
        current: point,
        lyricCellAtAnchor: lyricCell,
      },
      args,
    )
    e.preventDefault()
    return
  }
  const noteCell = getNoteAtPoint(e.clientX, e.clientY)
  if (noteCell !== undefined) {
    const point = { x: e.clientX, y: e.clientY }
    anchorAndCommit(
      dragStateRef,
      {
        mode: 'note',
        anchor: point,
        current: point,
        noteCellAtAnchor: noteCell,
      },
      args,
    )
    e.preventDefault()
    return
  }
  // Missed every note/lyric click target (e.g. a bar-line or the gutter
  // around notes) — rather than no-op or fall back to whole-measure
  // selection (now Cmd/Ctrl-gated above), resolve to the nearest note/chord
  // cell in whatever measure was clicked, via the same 'note' mode a direct
  // note hit anchors. Its real screen-coordinate anchor still lets a second
  // click from here resolve into 'note' mode's marquee normally.
  const range = getMeasureAtPoint(e.clientX, e.clientY)
  if (range === undefined) return
  const nearestCell = nearestNoteCellInMeasureRange(
    noteSpans,
    range,
    e.clientX,
    e.clientY,
  )
  if (nearestCell === undefined) return
  const point = { x: e.clientX, y: e.clientY }
  anchorAndCommit(
    dragStateRef,
    {
      mode: 'note',
      anchor: point,
      current: point,
      noteCellAtAnchor: nearestCell,
    },
    args,
  )
  e.preventDefault()
}

/** The second click of a click-and-click gesture: resolves the range between
 * `dragState`'s anchor and this click and commits it, returning
 * `dragStateRef` to idle — unless this click misses every recognizable
 * target, in which case the gesture is cancelled instead, leaving the first
 * click's own self-commit untouched (see `isEmptySpace`/`cancelAnchor`).
 * `handlePreviewClick` dispatches here when `dragStateRef` is already
 * anchored. */
function handleCommitClick(
  e: MouseEvent<HTMLDivElement>,
  dragState: NonNullable<PreviewDragState>,
  args: HandlePreviewClickArgs,
): void {
  const { dragStateRef, noteSpans } = args
  if (isEmptySpace(e.clientX, e.clientY, noteSpans)) {
    cancelAnchor(dragStateRef, dragState, args)
    e.preventDefault()
    return
  }
  const point = { x: e.clientX, y: e.clientY }
  fireCommit(resolveSelection(dragState, point, args), args)
  dragStateRef.current = null
  e.preventDefault()
}

/**
 * The `mousedown` dispatch for `Preview`'s SVG surface, driving the click-
 * and-click range-selection gesture: idle → a first click anchors a mode and
 * self-commits its single-target resolution (`handleAnchorClick`); anchored
 * → a second click resolves and commits the range between the anchor and
 * this click, or cancels the gesture (leaving the first click's own commit
 * in place) if it misses every recognizable target (`handleCommitClick`).
 * `usePreviewClickSelection`'s `mousemove` listener live-updates the
 * highlight between the two clicks for mouse users; a touch tap synthesizes
 * `mousedown`/`mouseup` with no intervening movement, so the same two
 * dispatches cover both input types.
 *
 * Wired to `mousedown` rather than the browser's synthesized `click` event
 * deliberately: on this codebase's target platforms, a Cmd/Ctrl-held primary
 * click doesn't reliably fire `click` at all (it's the OS-level secondary-
 * click gesture), which would silently break every Cmd/Ctrl-gated mode below
 * (`'measure'` off a bare click, `'part-label-system'`) — `mousedown` has no
 * such gap. Split out of `Preview` to keep that component under its
 * line-count cap; the per-mode marquee/range resolution itself lives in
 * `previewSelectionResolver.ts` for the same reason.
 */
export function handlePreviewClick(
  e: MouseEvent<HTMLDivElement>,
  args: HandlePreviewClickArgs,
): void {
  const dragState = args.dragStateRef.current
  if (dragState === null) {
    handleAnchorClick(e, args)
    return
  }
  handleCommitClick(e, dragState, args)
}
