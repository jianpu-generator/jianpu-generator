import type { Monaco } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import { spanPositions } from '../../semanticTokens'

/** Class of the ghost `𝄀N` text injected where measure N starts. */
export const MEASURE_MARKER_CLASS = 'part-stream-measure-marker'

/**
 * One injected-text decoration per entry of `measureStarts` (UTF-8 byte
 * offsets into `text`, as Rust reports them), labelled with the 1-based
 * measure number. Several measures may start at the same offset (lyric
 * measures with no slots); each still gets its own marker. `text` must be
 * LF-only, as the Rust side measures it.
 */
export function measureMarkerDecorations(
  monacoApi: Monaco,
  text: string,
  measureStarts: Uint32Array,
): editor.IModelDeltaDecoration[] {
  const offsets = Array.from(measureStarts)
  const positions = spanPositions(
    text,
    offsets.map((offset) => ({ start: offset, end: offset })),
  )
  return offsets.flatMap((offset, index) => {
    const position = positions.get(offset)
    if (!position) return []
    const lineNumber = position.line + 1
    const column = position.character + 1
    return [
      {
        range: new monacoApi.Range(lineNumber, column, lineNumber, column),
        options: {
          description: 'part-stream-measure-marker',
          // Monaco drops injected text on an empty range unless this is set.
          showIfCollapsed: true,
          before: {
            content: `𝄀${index + 1}`,
            inlineClassName: MEASURE_MARKER_CLASS,
            cursorStops: monacoApi.editor.InjectedTextCursorStops.None,
          },
        },
      },
    ]
  })
}
