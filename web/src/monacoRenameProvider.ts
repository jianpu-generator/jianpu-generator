import type { Monaco } from '@monaco-editor/react'
import type * as monacoEditor from 'monaco-editor'
import { JIANPU_LANGUAGE_ID } from './monacoJianpuLanguage'
import {
  listRenameSymbols,
  renameSymbolEdits,
  symbolAtByteOffset,
} from './renameSymbol'
import {
  byteOffsetToStringIndex,
  stringIndexToByteOffset,
} from './utils/byteSpan'

let registered = false

function toRange(
  monacoApi: Monaco,
  model: monacoEditor.editor.ITextModel,
  source: string,
  startByte: number,
  endByte: number,
): monacoEditor.Range {
  const startPos = model.getPositionAt(
    byteOffsetToStringIndex(source, startByte),
  )
  const endPos = model.getPositionAt(byteOffsetToStringIndex(source, endByte))
  return new monacoApi.Range(
    startPos.lineNumber,
    startPos.column,
    endPos.lineNumber,
    endPos.column,
  )
}

/** Registers rename support for part/group abbreviations and section labels. */
export function registerJianpuRenameProvider(monacoApi: Monaco) {
  if (registered) return
  registered = true

  monacoApi.languages.registerRenameProvider(JIANPU_LANGUAGE_ID, {
    async resolveRenameLocation(
      model: monacoEditor.editor.ITextModel,
      position: monacoEditor.Position,
    ) {
      const source = model.getValue()
      const byteOffset = stringIndexToByteOffset(
        source,
        model.getOffsetAt(position),
      )
      const symbols = await listRenameSymbols(source)
      const symbol = symbolAtByteOffset(symbols, byteOffset)
      const occurrence = symbol?.occurrences.find(
        (o) => byteOffset >= o.span.start && byteOffset < o.span.end,
      )
      if (!symbol || !occurrence) {
        return {
          range: new monacoApi.Range(
            position.lineNumber,
            position.column,
            position.lineNumber,
            position.column,
          ),
          text: '',
          rejectReason:
            'This position does not name a part/group abbreviation or section label.',
        }
      }
      return {
        range: toRange(
          monacoApi,
          model,
          source,
          occurrence.span.start,
          occurrence.span.end,
        ),
        text: symbol.name,
      }
    },

    async provideRenameEdits(
      model: monacoEditor.editor.ITextModel,
      position: monacoEditor.Position,
      newName: string,
    ) {
      const source = model.getValue()
      const byteOffset = stringIndexToByteOffset(
        source,
        model.getOffsetAt(position),
      )
      const symbols = await listRenameSymbols(source)
      const symbol = symbolAtByteOffset(symbols, byteOffset)
      if (!symbol) return { edits: [] }

      const edits = await renameSymbolEdits(
        source,
        symbol.kind,
        symbol.name,
        newName,
      )
      return {
        edits: edits.map((edit) => ({
          resource: model.uri,
          textEdit: {
            range: toRange(monacoApi, model, source, edit.start, edit.end),
            text: edit.replacement,
          },
          versionId: model.getVersionId(),
        })),
      }
    },
  })
}
