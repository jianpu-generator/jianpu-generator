import type { NoteTimingOut } from 'jianpu-wasm'
import type { NoteCell } from './noteSpanSelection'

/** Elapsed-seconds window, relative to the start of a generated audio clip. */
export interface TrimWindow {
  start: number
  end: number
}

/**
 * Narrows a measure-range clip's note timings down to the elapsed-seconds
 * window that exactly covers a drag-selected set of notes, so "play
 * selection" (see `useMeasureAudioPlayback.playNoteSelection`) can seek/stop
 * at the real note boundaries instead of playing the full boundary measures
 * the selection touches.
 *
 * Returns `null` when none of `cells` has a matching timing (nothing to
 * trim to — falls back to playing the clip in full) or the resulting window
 * is empty/inverted.
 */
export function computeNoteSelectionTrimWindow(
  cells: NoteCell[],
  noteTimings: NoteTimingOut[],
): TrimWindow | null {
  const keys = new Set(cells.map((c) => `${c.sourcePartIndex}:${c.noteId}`))
  const matched = noteTimings.filter((t) =>
    keys.has(`${t.source_part_index}:${t.note_id}`),
  )
  if (matched.length === 0) return null
  const start = Math.min(...matched.map((t) => t.start_s))
  const end = Math.max(...matched.map((t) => t.end_s))
  if (end <= start) return null
  return { start, end }
}
