import { format_unzipped_text } from 'jianpu-wasm'
import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { EditorHandle } from '../types'

interface UseUnzippedTextFormatParams {
  source: string
  unzippedText: string
  editorRef: RefObject<EditorHandle | null>
  setUnzippedText: (text: string) => void
}

/** Formats the whole-document Unzipped-view text, breaking each measure onto
 * its own line, reading the live editor text if available. */
export function useUnzippedTextFormat({
  source,
  unzippedText,
  editorRef,
  setUnzippedText,
}: UseUnzippedTextFormatParams) {
  return useCallback(() => {
    const currentUnzippedText =
      editorRef.current?.getEditor()?.getModel()?.getValue() ?? unzippedText
    const result = format_unzipped_text(source, currentUnzippedText)
    if (result.status === 'ok') {
      setUnzippedText(result.text)
    }
  }, [source, unzippedText, editorRef, setUnzippedText])
}
