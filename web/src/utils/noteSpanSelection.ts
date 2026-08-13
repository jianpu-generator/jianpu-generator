import type { Monaco } from '@monaco-editor/react'
import type * as monacoEditor from 'monaco-editor'
import type { NoteSpan } from '../types'
import { byteOffsetToStringIndex } from './byteSpan'

/** One rendered note/rest hit-tested off the SVG during a drag-select, keyed
 * the same way as `Tag::Note`'s `data-part-index`/`data-note-id` attributes. */
export interface NoteCell {
  sourcePartIndex: number
  noteId: number
}

/** One contiguous drag-selected byte range within a single part's single measure. */
export interface NoteSelectionRun {
  sourcePartIndex: number
  measureIndex: number
  startByte: number
  endByte: number
}

function cellKey(sourcePartIndex: number, noteId: number): string {
  return `${sourcePartIndex}:${noteId}`
}

/**
 * Groups a drag-selected set of `(sourcePartIndex, noteId)` cells into
 * contiguous per-`(part, measure)` source byte runs, ready to become Monaco
 * multicursor selections. A rest cell (no `start`/`end`) is skipped rather
 * than breaking an otherwise-contiguous run; a run with no non-rest cells at
 * all simply never appears in the result.
 */
export function groupSelectedNotesIntoContiguousRuns(
  selectedCells: NoteCell[],
  noteSpans: NoteSpan[],
): NoteSelectionRun[] {
  const selectedKeys = new Set(
    selectedCells.map((cell) => cellKey(cell.sourcePartIndex, cell.noteId)),
  )

  const runsByPartMeasure = new Map<string, NoteSelectionRun>()
  for (const span of noteSpans) {
    if (!selectedKeys.has(cellKey(span.sourcePartIndex, span.noteId))) {
      continue
    }
    if (span.start === undefined || span.end === undefined) continue

    const key = `${span.sourcePartIndex}:${span.measureIndex}`
    const existing = runsByPartMeasure.get(key)
    if (existing) {
      existing.startByte = Math.min(existing.startByte, span.start)
      existing.endByte = Math.max(existing.endByte, span.end)
    } else {
      runsByPartMeasure.set(key, {
        sourcePartIndex: span.sourcePartIndex,
        measureIndex: span.measureIndex,
        startByte: span.start,
        endByte: span.end,
      })
    }
  }

  return Array.from(runsByPartMeasure.values()).sort(
    (a, b) =>
      a.sourcePartIndex - b.sourcePartIndex || a.measureIndex - b.measureIndex,
  )
}

/**
 * Converts note-selection runs into Monaco multicursor selections, reusing
 * the byte→position conversion `monacoRenameProvider.ts`'s `toRange()`
 * already proves out (`byteOffsetToStringIndex`), generalized to several
 * disjoint selections instead of one. Only `startByte`/`endByte` are read, so
 * this also serves as the shared implementation behind
 * `EditorHandle.setSelections`'s generic byte-range input.
 */
export function buildMonacoSelections(
  runs: Array<Pick<NoteSelectionRun, 'startByte' | 'endByte'>>,
  source: string,
  monacoApi: Monaco,
  model: monacoEditor.editor.ITextModel,
): monacoEditor.Selection[] {
  return runs.map((run) => {
    const startPos = model.getPositionAt(
      byteOffsetToStringIndex(source, run.startByte),
    )
    const endPos = model.getPositionAt(
      byteOffsetToStringIndex(source, run.endByte),
    )
    return new monacoApi.Selection(
      startPos.lineNumber,
      startPos.column,
      endPos.lineNumber,
      endPos.column,
    )
  })
}
