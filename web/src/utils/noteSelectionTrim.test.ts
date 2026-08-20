import type { NoteTimingOut } from 'jianpu-wasm'
import { describe, expect, it } from 'vitest'
import { computeNoteSelectionTrimWindow } from './noteSelectionTrim'

function timing(
  sourcePartIndex: number,
  noteId: number,
  startS: number,
  endS: number,
): NoteTimingOut {
  return {
    source_part_index: sourcePartIndex,
    note_id: noteId,
    start_s: startS,
    end_s: endS,
  }
}

describe('computeNoteSelectionTrimWindow', () => {
  it('narrows to the min start / max end of the selected notes only', () => {
    const noteTimings = [
      timing(0, 0, 0, 1),
      timing(0, 1, 1, 2),
      timing(0, 2, 2, 3),
      timing(0, 3, 3, 4),
    ]
    const trim = computeNoteSelectionTrimWindow(
      [
        { sourcePartIndex: 0, noteId: 1 },
        { sourcePartIndex: 0, noteId: 2 },
      ],
      noteTimings,
    )
    expect(trim).toEqual({ start: 1, end: 3 })
  })

  it('ignores timings from parts/notes not in the selection', () => {
    const noteTimings = [timing(0, 0, 0, 1), timing(1, 0, 0, 5)]
    const trim = computeNoteSelectionTrimWindow(
      [{ sourcePartIndex: 0, noteId: 0 }],
      noteTimings,
    )
    expect(trim).toEqual({ start: 0, end: 1 })
  })

  it('returns null when no selected cell has a matching timing', () => {
    const noteTimings = [timing(0, 0, 0, 1)]
    const trim = computeNoteSelectionTrimWindow(
      [{ sourcePartIndex: 5, noteId: 9 }],
      noteTimings,
    )
    expect(trim).toBeNull()
  })

  it('returns null for an empty selection', () => {
    const noteTimings = [timing(0, 0, 0, 1)]
    expect(computeNoteSelectionTrimWindow([], noteTimings)).toBeNull()
  })
})
