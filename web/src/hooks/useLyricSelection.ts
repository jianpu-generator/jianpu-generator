import { group_lyric_selection } from 'jianpu-wasm'
import type { RefObject } from 'react'
import { useCallback, useRef, useState } from 'react'
import type { LyricCell } from '../components/previewSelection'
import type { EditorHandle, LyricSpan } from '../types'
import { ensureWasmInit } from '../wasmInit'

/** One contiguous drag-selected byte range within a single verse line of a
 * single part's single measure, as grouped by the wasm export
 * `group_lyric_selection` (`lyric_spans::group_selected_lyrics_into_contiguous_runs`
 * in Rust). */
export interface LyricSelectionRun {
  sourcePartIndex: number
  measureIndex: number
  startByte: number
  endByte: number
}

/** Calls the wasm `group_lyric_selection` export directly on the main
 * thread (bypassing the debounced render worker) — this is pure grouping
 * over an already-fetched flat `lyric_spans` array, so it doesn't need to
 * re-parse `source` and stays responsive on every selection-change tick. */
async function groupSelectedLyricsIntoContiguousRuns(
  selectedCells: LyricCell[],
  lyricSpans: LyricSpan[],
): Promise<LyricSelectionRun[]> {
  await ensureWasmInit()
  const response = group_lyric_selection(lyricSpans, selectedCells)
  return response.status === 'ok'
    ? response.runs.map((r) => ({
        sourcePartIndex: r.sourcePartIndex,
        measureIndex: r.measureIndex,
        startByte: r.startByte,
        endByte: r.endByte,
      }))
    : []
}

/**
 * Turns a click/drag-select over lyric syllables (a set of `(source_part_index,
 * note_id, verse)` cells hit-tested off the SVG, see `Preview.tsx`) into a
 * Monaco multicursor selection over the source text — one disjoint range per
 * `(part, verse, measure)` the drag touched.
 *
 * Deliberately independent of `useNoteSelection`: a lyric selection never
 * drives note highlighting and vice versa, so this hook keeps its own state
 * and its own Monaco-push logic rather than threading lyric cells through
 * `useNoteSelection`'s `selectedNoteCells`/`runs` machinery.
 */
export function useLyricSelection(
  lyricSpans: LyricSpan[],
  editorRef: RefObject<EditorHandle | null>,
) {
  const [lastSelectedCells, setLastSelectedCells] = useState<LyricCell[]>([])
  // Mirrors `useNoteSelection`'s `suppressNextEditorSelectionSyncRef`: set
  // right before `handleLyricRangeSelect` pushes a selection into Monaco, so
  // the echoed-back `handleEditorSelectionChange` call doesn't redundantly
  // re-derive `lastSelectedCells` from a selection that already matches it.
  const suppressNextEditorSelectionSyncRef = useRef(false)

  const handleLyricRangeSelect = useCallback(
    async (selectedCells: LyricCell[]) => {
      const runs = await groupSelectedLyricsIntoContiguousRuns(
        selectedCells,
        lyricSpans,
      )
      setLastSelectedCells(selectedCells)
      if (!editorRef.current || runs.length === 0) return
      suppressNextEditorSelectionSyncRef.current = true
      editorRef.current.setSelections(
        runs.map((run) => ({ start: run.startByte, end: run.endByte })),
      )
    },
    [lyricSpans, editorRef],
  )

  const handleEditorSelectionChange = useCallback(
    async (startByte: number, endByte: number) => {
      if (suppressNextEditorSelectionSyncRef.current) {
        suppressNextEditorSelectionSyncRef.current = false
        return
      }
      const cells: LyricCell[] =
        startByte === endByte
          ? []
          : lyricSpans
              .filter((span) => span.start < endByte && span.end > startByte)
              .map((span) => ({
                sourcePartIndex: span.sourcePartIndex,
                noteId: span.noteId,
                verse: span.verse,
              }))
      setLastSelectedCells(cells)
    },
    [lyricSpans],
  )

  return {
    handleLyricRangeSelect,
    handleEditorSelectionChange,
    selectedLyricCells: lastSelectedCells,
  }
}
