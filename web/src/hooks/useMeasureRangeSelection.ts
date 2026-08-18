import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { LyricCell, NoteCell } from '../components/previewSelection'
import type { EditorHandle, LyricSpan, NoteSpan } from '../types'
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
  handleNoteRangeSelect: (cells: NoteCell[]) => Promise<void>,
  handleLyricRangeSelect: (cells: LyricCell[]) => Promise<void>,
  applyNoteSelectionSilently: (
    cells: NoteCell[],
    runs: Awaited<ReturnType<typeof groupSelectedNotesIntoContiguousRuns>>,
  ) => void,
  applyLyricSelectionSilently: (
    cells: LyricCell[],
    runs: Awaited<ReturnType<typeof groupSelectedLyricsIntoContiguousRuns>>,
  ) => void,
) {
  return useCallback(
    async (noteCells: NoteCell[], lyricCells: LyricCell[]) => {
      if (!editorRef.current) {
        // No mounted editor (Live/shared view) — no Monaco selection for the
        // two pushes to conflict over, so the ordinary independent path is
        // safe (and reuses `useNoteSelection`'s measure-range fallback).
        await Promise.all([
          handleNoteRangeSelect(noteCells),
          handleLyricRangeSelect(lyricCells),
        ])
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
      handleNoteRangeSelect,
      handleLyricRangeSelect,
      applyNoteSelectionSilently,
      applyLyricSelectionSilently,
    ],
  )
}
