import type { RefObject } from 'react'
import { useCallback, useMemo, useRef, useState } from 'react'
import type { EditorHandle, MeasureSpan, NoteSpan, PartInfo } from '../types'
import {
  groupSelectedNotesIntoContiguousRuns,
  type NoteCell,
  type NoteSelectionRun,
} from '../utils/noteSpanSelection'

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
  const [lastRuns, setLastRuns] = useState<NoteSelectionRun[]>([])
  // The raw cells behind `lastRuns`, kept around so the SVG preview can
  // re-apply the same highlight after any DOM change (e.g. a re-render
  // triggered by the Monaco selection this drag just pushed) — see
  // `Preview.tsx`'s `selectedNoteCells` prop.
  const [lastSelectedCells, setLastSelectedCells] = useState<NoteCell[]>([])
  // Set right before `handleNoteRangeSelect` pushes a selection into Monaco,
  // so the very next `handleEditorSelectionChange` call — which fires
  // synchronously off that same `setSelections`, echoing the selection back
  // — can no-op instead of re-deriving `lastSelectedCells` from it. That
  // re-derivation drops any rest cells the drag included (a rest has no
  // `start`/`end` byte span, so it never became part of the pushed Monaco
  // selection in the first place — see `groupSelectedNotesIntoContiguousRuns`),
  // which would otherwise silently shrink the preview highlight right after
  // every drag that touched a rest.
  const suppressNextEditorSelectionSyncRef = useRef(false)

  const handleNoteRangeSelect = useCallback(
    (selectedCells: NoteCell[]) => {
      const runs = groupSelectedNotesIntoContiguousRuns(
        selectedCells,
        noteSpans,
      )
      // Live/shared views never mount an Editor, so there's no Monaco
      // selection to round-trip through `handleEditorSelectionChange` and
      // no note-selection playback UI to drive either — fall back to a
      // plain measure-range selection via `notifySelection` directly,
      // matching the pre-note-drag behavior (see `useSectionNavigation`'s
      // `selectSectionRange`), so the selection still lands.
      if (!editorRef.current) {
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
        return
      }
      setLastSelectedCells(selectedCells)
      setLastRuns(runs)
      if (runs.length === 0) return
      suppressNextEditorSelectionSyncRef.current = true
      editorRef.current.setSelections(
        runs.map((run) => ({ start: run.startByte, end: run.endByte })),
      )
    },
    [noteSpans, editorRef, measureSpans, notifySelection],
  )

  /**
   * The reverse of `handleNoteRangeSelect`: keeps the preview's note
   * highlight in sync with whatever is actually selected in Monaco,
   * including a selection made by typing/selecting in the editor directly
   * rather than dragging in the preview — previously the preview highlight
   * only ever reflected a preview-side drag, so it went stale (or just
   * never appeared) the moment the editor's own selection changed some
   * other way.
   *
   * `startByte`/`endByte` are byte offsets into the same source `noteSpans`
   * is keyed against (see `Editor.tsx`'s `onSelectionOffsetChange`) — the
   * caller is expected to only invoke this for the Zipped view, since
   * `noteSpans` has no meaning against the Unzipped view's projected text.
   */
  const handleEditorSelectionChange = useCallback(
    (startByte: number, endByte: number) => {
      if (suppressNextEditorSelectionSyncRef.current) {
        suppressNextEditorSelectionSyncRef.current = false
        return
      }
      const cells: NoteCell[] =
        startByte === endByte
          ? []
          : noteSpans
              .filter(
                (span) =>
                  span.start !== undefined &&
                  span.end !== undefined &&
                  span.start < endByte &&
                  span.end > startByte,
              )
              .map((span) => ({
                sourcePartIndex: span.sourcePartIndex,
                noteId: span.noteId,
              }))
      setLastSelectedCells(cells)
      setLastRuns(groupSelectedNotesIntoContiguousRuns(cells, noteSpans))
    },
    [noteSpans],
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
