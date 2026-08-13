import { describe, expect, it } from 'vitest'
import type { NoteSpan } from '../types'
import {
  groupSelectedNotesIntoContiguousRuns,
  type NoteCell,
} from './noteSpanSelection'

function span(
  sourcePartIndex: number,
  noteId: number,
  measureIndex: number,
  start: number | undefined,
  end: number | undefined,
): NoteSpan {
  return {
    sourcePartIndex,
    noteId,
    measureIndex,
    start,
    end,
  }
}

describe('groupSelectedNotesIntoContiguousRuns', () => {
  it('groups a single contiguous run within one measure', () => {
    const noteSpans: NoteSpan[] = [
      span(0, 0, 0, 10, 11),
      span(0, 1, 0, 12, 13),
      span(0, 2, 0, 14, 15),
    ]
    const selected: NoteCell[] = [
      { sourcePartIndex: 0, noteId: 0 },
      { sourcePartIndex: 0, noteId: 1 },
      { sourcePartIndex: 0, noteId: 2 },
    ]

    const runs = groupSelectedNotesIntoContiguousRuns(selected, noteSpans)

    expect(runs).toEqual([
      { sourcePartIndex: 0, measureIndex: 0, startByte: 10, endByte: 15 },
    ])
  })

  it('splits a selection spanning 2 measures into 2 runs', () => {
    const noteSpans: NoteSpan[] = [span(0, 0, 0, 10, 11), span(0, 1, 1, 20, 21)]
    const selected: NoteCell[] = [
      { sourcePartIndex: 0, noteId: 0 },
      { sourcePartIndex: 0, noteId: 1 },
    ]

    const runs = groupSelectedNotesIntoContiguousRuns(selected, noteSpans)

    expect(runs).toEqual([
      { sourcePartIndex: 0, measureIndex: 0, startByte: 10, endByte: 11 },
      { sourcePartIndex: 0, measureIndex: 1, startByte: 20, endByte: 21 },
    ])
  })

  it('splits a selection spanning 2 parts into disjoint per-part runs', () => {
    const noteSpans: NoteSpan[] = [span(0, 0, 0, 10, 11), span(1, 0, 0, 30, 31)]
    const selected: NoteCell[] = [
      { sourcePartIndex: 0, noteId: 0 },
      { sourcePartIndex: 1, noteId: 0 },
    ]

    const runs = groupSelectedNotesIntoContiguousRuns(selected, noteSpans)

    expect(runs).toEqual([
      { sourcePartIndex: 0, measureIndex: 0, startByte: 10, endByte: 11 },
      { sourcePartIndex: 1, measureIndex: 0, startByte: 30, endByte: 31 },
    ])
  })

  it('a rest inside a run does not break contiguity', () => {
    const noteSpans: NoteSpan[] = [
      span(0, 0, 0, 10, 11),
      span(0, 1, 0, undefined, undefined), // rest
      span(0, 2, 0, 16, 17),
    ]
    const selected: NoteCell[] = [
      { sourcePartIndex: 0, noteId: 0 },
      { sourcePartIndex: 0, noteId: 1 },
      { sourcePartIndex: 0, noteId: 2 },
    ]

    const runs = groupSelectedNotesIntoContiguousRuns(selected, noteSpans)

    expect(runs).toEqual([
      { sourcePartIndex: 0, measureIndex: 0, startByte: 10, endByte: 17 },
    ])
  })

  it('an all-rest run yields nothing', () => {
    const noteSpans: NoteSpan[] = [
      span(0, 0, 0, undefined, undefined),
      span(0, 1, 0, undefined, undefined),
    ]
    const selected: NoteCell[] = [
      { sourcePartIndex: 0, noteId: 0 },
      { sourcePartIndex: 0, noteId: 1 },
    ]

    const runs = groupSelectedNotesIntoContiguousRuns(selected, noteSpans)

    expect(runs).toEqual([])
  })
})
