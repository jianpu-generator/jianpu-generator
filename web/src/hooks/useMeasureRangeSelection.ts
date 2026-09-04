import type { RefObject } from 'react'
import { useCallback } from 'react'
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
    async (noteCells: NoteCell[], lyricCells: LyricCell[]) => {
      if (!editorRef.current) {
        // No mounted editor (Live/shared view) — deliberately doesn't route
        // through `handleNoteRangeSelect`/`handleLyricRangeSelect` here (each
        // would populate `selectedCells`, flipping `noteSelectionActive` on
        // and hijacking the play-measure button's "Measures N–M" label into
        // "Selection" — see `PlayMeasureButton`'s doc comment): the note/
        // lyric cells themselves already got their own precise blue/lyric
        // highlight painted directly on the SVG (`resolveMeasureSelection`),
        // so all that's left is this mode's own whole-measure indicator —
        // reporting the range as a caret-only `notifySelection` to paint the
        // amber measure-background highlight, the pre-note-drag behavior
        // this mode has always had (see `useSectionNavigation`'s
        // `selectSectionRange`).
        const noteRuns = await groupSelectedNotesIntoContiguousRuns(
          noteCells,
          noteSpans,
        )
        if (noteRuns.length > 0) {
          const measureIndices = noteRuns.map((run) => run.measureIndex)
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
