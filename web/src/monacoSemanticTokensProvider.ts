import type { Monaco } from '@monaco-editor/react'
import type * as monacoEditor from 'monaco-editor'
import { highlightKindStyles } from './components/editorTheme'
import { type HighlightKind, jianpuWasm } from './jianpuWasm'
import { JIANPU_LANGUAGE_ID } from './monacoJianpuLanguage'
import { encodeSemanticTokens } from './semanticTokens'
import { ensureWasmInit } from './wasmInit'

let registered = false

/** Highlights keywords from the Rust lexers' `highlight-tokens`, so the
 * editor never re-spells the `.jianpu` vocabulary. */
export function registerJianpuSemanticTokensProvider(monacoApi: Monaco) {
  if (registered) return
  registered = true

  const tokenTypes = Object.keys(highlightKindStyles) as HighlightKind[]
  monacoApi.languages.registerDocumentSemanticTokensProvider(
    JIANPU_LANGUAGE_ID,
    {
      getLegend: () => ({ tokenTypes, tokenModifiers: [] }),
      async provideDocumentSemanticTokens(
        model: monacoEditor.editor.ITextModel,
      ) {
        await ensureWasmInit()
        const source = model.getValue(monacoApi.editor.EndOfLinePreference.LF)
        const tokens = jianpuWasm().highlightTokens(source)
        return { data: encodeSemanticTokens(source, tokens, tokenTypes) }
      },
      releaseDocumentSemanticTokens() {},
    },
  )
}
