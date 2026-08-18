import { group_note_selection } from 'jianpu-wasm'
import type { RefObject } from 'react'
import { useCallback, useMemo } from 'react'
import type { EditorHandle, MeasureSpan, NoteSpan, PartInfo } from '../types'
import type { NoteCell, NoteSelectionRun } from '../utils/noteSpanSelection'
import { ensureWasmInit } from '../wasmInit'
import { useByteRangeSelectionCore } from './useByteRangeSelectionCore'

/** Calls the wasm `group_note_selection` export directly on the main
 * thread (bypassing the debounced render worker) — this is pure grouping
 * over an already-fetched flat `note_spans` array, so it doesn't need to
 * re-parse `source` and stays responsive on every selection-change tick. */
async function groupSelectedNotesIntoContiguousRuns(
  selectedCells: NoteCell[],
  noteSpans: NoteSpan[],
): Promise<NoteSelectionRun[]> {
  await ensureWasmInit()
  const response = group_note_selection(noteSpans, selectedCells)
  return response.status === 'ok'
    ? response.runs.map((r) => ({
        sourcePartIndex: r.sourcePartIndex,
        measureIndex: r.measureIndex,
        startByte: r.startByte,
        endByte: r.endByte,
      }))
    : []
}

function cellFromNoteSpan(span: NoteSpan): NoteCell {
  return { sourcePartIndex: span.sourcePartIndex, noteId: span.noteId }
}

function noteRunByteRange(run: NoteSelectionRun) {
  return { start: run.startByte, end: run.endByte }
}

export interface SelectedNoteRangePlaybackInfo {
  minMeasureIndex: number
  maxMeasureIndex: number
  selectedPartNames: string[]
}

/**
 * Turns a MuseScore-style note drag-select (a set of `(source_part_index,
 * note_id)` cells hit-tested off the SVG, see `Preview.tsx`) into a Monaco
 * multicursor selection over the source text — one disjoint range per
 * `(part, measure)` the drag touched — and derives the info a "play
 * selection" action needs (see `useMeasureAudioPlayback.playNoteSelection`).
 */
export function useNoteSelection(
  noteSpans: NoteSpan[],
  parts: PartInfo[],
  editorRef: RefObject<EditorHandle | null>,
  measureSpans: MeasureSpan[],
  notifySelection: (
    startLine: number,
    endLine: number,
    isEmpty: boolean,
  ) => void,
) {
  // Live/shared views never mount an Editor, so there's no Monaco
  // selection to round-trip through `handleEditorSelectionChange` and
  // no note-selection playback UI to drive either — fall back to a
  // plain measure-range selection via `notifySelection` directly,
  // matching the pre-note-drag behavior (see `useSectionNavigation`'s
  // `selectSectionRange`), so the selection still lands.
  const onNoMountedEditor = useCallback(
    (runs: NoteSelectionRun[]) => {
      if (runs.length === 0) return
      const measureIndices = runs.map((run) => run.measureIndex)
      const startSpan = measureSpans[Math.min(...measureIndices)]
      const endSpan = measureSpans[Math.max(...measureIndices)]
      if (!startSpan || !endSpan) return
      // No mounted editor here (Live/shared view) to show a Monaco
      // selection, so the amber measure-background highlight is this
      // fallback's only visual feedback for the drag — keep it on by
      // reporting the range as caret-only, unlike the editor-mounted path
      // below where the Monaco selection itself is the feedback.
      notifySelection(startSpan.start_line, endSpan.end_line, true)
    },
    [measureSpans, notifySelection],
  )

  const {
    selectedCells: lastSelectedCells,
    runs: lastRuns,
    handleRangeSelect: handleNoteRangeSelect,
    handleEditorSelectionChange,
  } = useByteRangeSelectionCore<NoteCell, NoteSpan, NoteSelectionRun>(
    noteSpans,
    editorRef,
    groupSelectedNotesIntoContiguousRuns,
    cellFromNoteSpan,
    noteRunByteRange,
    onNoMountedEditor,
  )

  const selectedNoteRangePlaybackInfo =
    useMemo<SelectedNoteRangePlaybackInfo | null>(() => {
      if (lastRuns.length === 0) return null
      const measureIndices = lastRuns.map((run) => run.measureIndex)
      const partIndices = new Set(lastRuns.map((run) => run.sourcePartIndex))
      // `sourcePartIndex` is the compiled `measure.parts` index, which is
      // index-aligned 1:1 with `parts` (the `PartInfo[]` from `list_parts`):
      // both ultimately derive from the same `ParsedDocument.declarations`
      // order (see `src/parser/mod.rs` — `declarations` feeds both
      // `list_parts_from_source` and `interleaved_parser::parse`, whose
      // per-part accumulators are `vec![...; declarations.len()]`, so no
      // reordering or gaps can occur between the two arrays).
      const selectedPartNames = Array.from(partIndices)
        .map((partIndex) => parts[partIndex]?.abbreviation)
        .filter((abbreviation): abbreviation is string => abbreviation != null)
      return {
        minMeasureIndex: Math.min(...measureIndices),
        maxMeasureIndex: Math.max(...measureIndices),
        selectedPartNames,
      }
    }, [lastRuns, parts])

  return {
    handleNoteRangeSelect,
    handleEditorSelectionChange,
    selectedNoteRangePlaybackInfo,
    selectedNoteCells: lastSelectedCells,
  }
}
