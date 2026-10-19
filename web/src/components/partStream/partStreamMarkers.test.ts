import type { Monaco } from '@monaco-editor/react'
import { describe, expect, it } from 'vitest'
import { measureMarkerDecorations } from './partStreamMarkers'

class FakeRange {
  readonly startLineNumber: number
  readonly startColumn: number
  readonly endLineNumber: number
  readonly endColumn: number
  constructor(
    startLineNumber: number,
    startColumn: number,
    endLineNumber: number,
    endColumn: number,
  ) {
    this.startLineNumber = startLineNumber
    this.startColumn = startColumn
    this.endLineNumber = endLineNumber
    this.endColumn = endColumn
  }
}

const fakeMonaco = {
  Range: FakeRange,
  editor: { InjectedTextCursorStops: { None: 3 } },
} as unknown as Monaco

function markerPositions(text: string, measureStarts: number[]) {
  return measureMarkerDecorations(
    fakeMonaco,
    text,
    Uint32Array.from(measureStarts),
  ).map((decoration) => ({
    line: decoration.range.startLineNumber,
    column: decoration.range.startColumn,
    content: decoration.options.before?.content,
  }))
}

describe('measureMarkerDecorations', () => {
  it('converts UTF-8 byte offsets of CJK text to UTF-16 columns', () => {
    // "你 好 " is 8 bytes but 4 UTF-16 code units.
    expect(markerPositions('你 好 世 界', [0, 8])).toEqual([
      { line: 1, column: 1, content: '𝄀1' },
      { line: 1, column: 5, content: '𝄀2' },
    ])
  })

  it('places markers on later lines and keeps one per measure at a shared offset', () => {
    expect(markerPositions('a b\n_', [0, 4, 4])).toEqual([
      { line: 1, column: 1, content: '𝄀1' },
      { line: 2, column: 1, content: '𝄀2' },
      { line: 2, column: 1, content: '𝄀3' },
    ])
  })
})
