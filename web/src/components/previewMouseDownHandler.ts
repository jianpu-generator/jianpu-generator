import type { MouseEvent, RefObject } from 'react'
import type { LyricSpan, NoteSpan } from '../types'
import {
  applyPersistedLyricHighlights,
  applyPersistedNoteHighlights,
} from './previewDragHighlights'
import type { PreviewDragState } from './previewDragState'
import {
  applyLyricLabelDragHighlight,
  applyPartLabelDragHighlight,
  partLabelsInMarqueeAcrossSystems,
} from './previewLabelDragHighlights'
import {
  getLyricLabelAtPoint,
  getPartLabelAtPoint,
  lyricCellsForLyricLabels,
  noteCellsForPartLabels,
} from './previewLabelSelection'
import {
  getBarLineMeasureAtPoint,
  getBarNumberMeasureAtPoint,
  getLyricAtPoint,
  getMeasureAtPoint,
  getNoteAtPoint,
  getSectionLabelAtPoint,
  lyricCellsInMeasureRange,
  nearestNoteCellInMeasureRange,
  noteCellsInMeasureRange,
} from './previewSelection'

interface HandlePreviewMouseDownArgs {
  dragStateRef: RefObject<PreviewDragState>
  previewPagesRef: RefObject<HTMLDivElement | null>
  noteSpans: NoteSpan[]
  lyricSpans: LyricSpan[]
  onSectionLabelClick: ((label: string) => void) | undefined
}

/**
 * The `mousedown` dispatch for `Preview`'s SVG surface: figures out what got
 * clicked (a section label, a part/lyric label, a note/chord, a lyric
 * syllable, or plain measure space) and arms `dragStateRef` with the mode
 * that gesture should carry through — `usePreviewDragSelection`'s
 * `mousemove`/`mouseup` handlers take it from there. Split out of `Preview`
 * to keep that component under its line-count cap.
 */
export function handlePreviewMouseDown(
  e: MouseEvent<HTMLDivElement>,
  {
    dragStateRef,
    previewPagesRef,
    noteSpans,
    lyricSpans,
    onSectionLabelClick,
  }: HandlePreviewMouseDownArgs,
): void {
  const sectionLabel = getSectionLabelAtPoint(e.clientX, e.clientY)
  if (sectionLabel !== undefined) {
    onSectionLabelClick?.(sectionLabel)
    e.preventDefault()
    return
  }
  const partLabel = getPartLabelAtPoint(e.clientX, e.clientY)
  if (partLabel !== undefined) {
    const point = { x: e.clientX, y: e.clientY }
    const container = previewPagesRef.current
    // Cmd/Ctrl-click(-drag) on a part label elevates the selection from
    // "this one part's system" to "every part in every system touched" —
    // see `PreviewDragState`'s 'part-label-system' doc comment. Checked
    // ahead of the plain part-label arm below so it takes priority.
    if (e.metaKey || e.ctrlKey) {
      dragStateRef.current = {
        mode: 'part-label-system',
        anchor: point,
        current: point,
      }
      if (container) {
        const hits = partLabelsInMarqueeAcrossSystems(container, point, point)
        applyPartLabelDragHighlight(container, hits)
        applyPersistedNoteHighlights(
          container,
          noteCellsForPartLabels(noteSpans, hits),
        )
      }
      e.preventDefault()
      return
    }
    dragStateRef.current = {
      mode: 'part-label',
      anchor: point,
      current: point,
      anchorSystem: {
        measureIndexStart: partLabel.measureIndexStart,
        measureIndexEnd: partLabel.measureIndexEnd,
      },
    }
    if (container) {
      applyPartLabelDragHighlight(container, [partLabel])
      applyPersistedNoteHighlights(
        container,
        noteCellsForPartLabels(noteSpans, [partLabel]),
      )
    }
    e.preventDefault()
    return
  }
  // The lyric-side mirror of the part-label check above — a verse row's own
  // label (e.g. "M:v1"), scoped to that one verse instead of a whole part.
  const lyricLabel = getLyricLabelAtPoint(e.clientX, e.clientY)
  if (lyricLabel !== undefined) {
    const point = { x: e.clientX, y: e.clientY }
    dragStateRef.current = {
      mode: 'lyric-label',
      anchor: point,
      current: point,
      anchorSystem: {
        measureIndexStart: lyricLabel.measureIndexStart,
        measureIndexEnd: lyricLabel.measureIndexEnd,
      },
    }
    const container = previewPagesRef.current
    if (container) {
      applyLyricLabelDragHighlight(container, [lyricLabel])
      applyPersistedLyricHighlights(
        container,
        lyricCellsForLyricLabels(lyricSpans, [lyricLabel]),
      )
    }
    e.preventDefault()
    return
  }
  // Grabbing a bar line's own divider always starts a measure-range drag, no
  // Cmd/Ctrl required: the divider is a dedicated drag handle (see
  // `renderBarLineDragHandle`), so landing on it is an unambiguous request
  // to select measures, unlike a plain click on a note/lyric/gutter pixel
  // (ambiguous enough to need the modifier gate below).
  const barLineRange = getBarLineMeasureAtPoint(e.clientX, e.clientY)
  if (barLineRange !== undefined) {
    dragStateRef.current = {
      mode: 'measure',
      anchor: barLineRange,
      current: barLineRange,
    }
    const container = previewPagesRef.current
    if (container) {
      applyPersistedNoteHighlights(
        container,
        noteCellsInMeasureRange(noteSpans, barLineRange),
      )
      applyPersistedLyricHighlights(
        container,
        lyricCellsInMeasureRange(lyricSpans, barLineRange),
      )
    }
    e.preventDefault()
    return
  }
  // Grabbing a measure's own bar number (drawn in the directive row above)
  // always starts a measure-range drag too, no Cmd/Ctrl required — same
  // rationale as the bar-line-handle check above: landing on the bar number
  // itself is an unambiguous request to select that measure, unlike a click
  // on a note/lyric/gutter pixel.
  const barNumberRange = getBarNumberMeasureAtPoint(e.clientX, e.clientY)
  if (barNumberRange !== undefined) {
    dragStateRef.current = {
      mode: 'measure',
      anchor: barNumberRange,
      current: barNumberRange,
    }
    const container = previewPagesRef.current
    if (container) {
      applyPersistedNoteHighlights(
        container,
        noteCellsInMeasureRange(noteSpans, barNumberRange),
      )
      applyPersistedLyricHighlights(
        container,
        lyricCellsInMeasureRange(lyricSpans, barNumberRange),
      )
    }
    e.preventDefault()
    return
  }
  // Cmd/Ctrl-click(-drag) always selects the whole measure under the
  // pointer, regardless of what structurally sits under it (note, chord,
  // lyric, bar-line, or empty gutter) — checked ahead of the lyric/note
  // checks below so it takes priority over them. Off a bar line, this is the
  // only way to reach 'measure' mode; a plain click/drag elsewhere resolves
  // to note/chord/syllable granularity instead (see `PreviewDragState`'s doc
  // comment).
  if (e.metaKey || e.ctrlKey) {
    const range = getMeasureAtPoint(e.clientX, e.clientY)
    if (range !== undefined) {
      dragStateRef.current = {
        mode: 'measure',
        anchor: range,
        current: range,
      }
      const container = previewPagesRef.current
      if (container) {
        applyPersistedNoteHighlights(
          container,
          noteCellsInMeasureRange(noteSpans, range),
        )
        applyPersistedLyricHighlights(
          container,
          lyricCellsInMeasureRange(lyricSpans, range),
        )
      }
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
    dragStateRef.current = {
      mode: 'lyric',
      anchor: point,
      current: point,
    }
    const container = previewPagesRef.current
    if (container) {
      applyPersistedLyricHighlights(container, [lyricCell])
    }
    e.preventDefault()
    return
  }
  const noteCell = getNoteAtPoint(e.clientX, e.clientY)
  if (noteCell !== undefined) {
    const point = { x: e.clientX, y: e.clientY }
    dragStateRef.current = {
      mode: 'pending',
      anchor: point,
      noteCellAtAnchor: noteCell,
    }
    // Eagerly show just this cell's highlight, matching a plain click's
    // instant-highlight-on-mousedown — overwritten if this turns into a
    // real note-drag (see `usePreviewDragSelection`).
    const container = previewPagesRef.current
    if (container) {
      applyPersistedNoteHighlights(container, [noteCell])
      applyPersistedLyricHighlights(container, [])
    }
    e.preventDefault()
    return
  }
  // Missed every note/lyric click target (e.g. a bar-line or the gutter
  // around notes) — rather than no-op or fall back to whole-measure
  // selection (now Cmd/Ctrl-gated above), resolve to the nearest note/chord
  // cell in whatever measure was clicked, via the same 'pending' mode a
  // direct note hit arms. Its real screen-coordinate anchor still lets a
  // real drag from here promote into 'note' mode's marquee normally.
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
  dragStateRef.current = {
    mode: 'pending',
    anchor: point,
    noteCellAtAnchor: nearestCell,
  }
  const container = previewPagesRef.current
  if (container) {
    applyPersistedNoteHighlights(container, [nearestCell])
    applyPersistedLyricHighlights(container, [])
  }
  e.preventDefault()
}
