import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { PreviewDragState } from '../components/previewDragState'
import type { LyricCell, NoteCell } from '../components/previewSelection'
import type { EditorHandle, LyricSpan, MeasureSpan, NoteSpan } from '../types'
import {
  groupSelectedLyricsIntoContiguousRuns,
  lyricRunByteRange,
} from './useLyricSelection'
import {
  groupSelectedNotesIntoContiguousRuns,
  noteRunByteRange,
} from './useNoteSelection'

/**
 * Turns a measure/bar-line click or drag (see `Preview.tsx`'s
 * `onMeasureRangeSelect`) into a single combined Monaco multicursor
 * selection covering both the note cells and the lyric cells under it.
 *
 * Deliberately *not* just calling `handleNoteRangeSelect` and
 * `handleLyricRangeSelect` back-to-back: each independently pushes its own
 * Monaco selection when an editor is mounted, so the second call's
 * `setSelections` clobbers the first's, and the resulting
 * `onDidChangeCursorSelection` echo would then re-derive (and typically
 * empty out) whichever core's `selectedCells` wasn't the last one pushed.
 * This groups both sides itself and pushes one combined selection instead,
 * silently committing each core's `selectedCells`/`runs` beforehand (via
 * `applyNoteSelectionSilently`/`applyLyricSelectionSilently`, see
 * `useByteRangeSelectionCore`'s `applySelectionSilently`) so neither depends
 * on that editor echo.
 */
export function useMeasureRangeSelection(
  editorRef: RefObject<EditorHandle | null>,
  noteSpans: NoteSpan[],
  lyricSpans: LyricSpan[],
  applyNoteSelectionSilently: (
    cells: NoteCell[],
    runs: Awaited<ReturnType<typeof groupSelectedNotesIntoContiguousRuns>>,
  ) => void,
  applyLyricSelectionSilently: (
    cells: LyricCell[],
    runs: Awaited<ReturnType<typeof groupSelectedLyricsIntoContiguousRuns>>,
  ) => void,
  measureSpans: MeasureSpan[],
  notifySelection: (
    startLine: number,
    endLine: number,
    isEmpty: boolean,
  ) => void,
) {
  // biome-ignore lint/correctness/useExhaustiveDependencies: editorRef is a ref object with a stable identity across renders (standard React convention); listing editorRef.current/.setSelections would stale-capture the ref's value at callback-creation time instead of reading it live on each call.
  return useCallback(
    async (
      noteCells: NoteCell[],
      lyricCells: LyricCell[],
      mode: NonNullable<PreviewDragState>['mode'],
    ) => {
      if (!editorRef.current) {
        // No mounted editor (Live/shared view) — deliberately doesn't route
        // through `handleNoteRangeSelect`/`handleLyricRangeSelect` here (each
        // would populate `selectedCells`, flipping `noteSelectionActive` on
        // and hijacking the play-measure button's "Measures N–M" label into
        // "Selection" — see `PlayMeasureButton`'s doc comment): the note/
        // lyric cells themselves already got their own precise blue/lyric
        // highlight painted directly on the SVG (`resolveMeasureSelection`),
        // so the amber whole-measure indicator below is only warranted for a
        // gesture that actually needs one:
        //
        // - a 'measure'/bar-line/label-anchored gesture (`mode` anything but
        //   'note'/'lyric') has no other visual feedback for what it
        //   selected, so it always gets the amber overlay — the pre-note-drag
        //   behavior this mode has always had (see `useSectionNavigation`'s
        //   `selectSectionRange`).
        // - a 'note'/'lyric'-anchored gesture already painted its own
        //   precise blue/lyric highlight directly on the SVG, so a
        //   single-measure tap doesn't need the amber overlay too — it only
        //   earns it once the drag actually spans more than one measure,
        //   which the blue/lyric highlight alone doesn't make obvious (see
        //   the mobile bug report this comment accompanies, and the sibling
        //   e2e coverage for a plain single-note tap vs. a cross-measure
        //   note drag in a no-mounted-editor view).
        const [noteRuns, lyricRuns] = await Promise.all([
          groupSelectedNotesIntoContiguousRuns(noteCells, noteSpans),
          groupSelectedLyricsIntoContiguousRuns(lyricCells, lyricSpans),
        ])
        const measureIndices = [
          ...noteRuns.map((run) => run.measureIndex),
          ...lyricRuns.map((run) => run.measureIndex),
        ]
        if (measureIndices.length === 0) return
        const isWholeMeasureGesture = mode !== 'note' && mode !== 'lyric'
        const spansMultipleMeasures =
          Math.min(...measureIndices) !== Math.max(...measureIndices)
        if (isWholeMeasureGesture || spansMultipleMeasures) {
          const startSpan = measureSpans[Math.min(...measureIndices)]
          const endSpan = measureSpans[Math.max(...measureIndices)]
          if (startSpan && endSpan) {
            notifySelection(startSpan.start_line, endSpan.end_line, true)
          }
        }
        return
      }
      const [noteRuns, lyricRuns] = await Promise.all([
        groupSelectedNotesIntoContiguousRuns(noteCells, noteSpans),
        groupSelectedLyricsIntoContiguousRuns(lyricCells, lyricSpans),
      ])
      applyNoteSelectionSilently(noteCells, noteRuns)
      applyLyricSelectionSilently(lyricCells, lyricRuns)
      const ranges = [
        ...noteRuns.map(noteRunByteRange),
        ...lyricRuns.map(lyricRunByteRange),
      ]
      if (ranges.length === 0) return
      editorRef.current.setSelections(ranges)
    },
    [
      noteSpans,
      lyricSpans,
      measureSpans,
      notifySelection,
      applyNoteSelectionSilently,
      applyLyricSelectionSilently,
    ],
  )
}
