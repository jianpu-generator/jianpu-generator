import type { Monaco } from '@monaco-editor/react'

export const JIANPU_LANGUAGE_ID = 'jianpu'

let registered = false

/** Basic Monarch tokenizer for `.jianpu` files — see syntax.md for the
 * grammar. It only colors punctuation-level structure; keywords come from
 * the Rust lexers via `registerJianpuSemanticTokensProvider`. */
export function registerJianpuLanguage(monacoApi: Monaco) {
  if (registered) return
  registered = true

  monacoApi.languages.register({ id: JIANPU_LANGUAGE_ID })

  monacoApi.languages.setLanguageConfiguration(JIANPU_LANGUAGE_ID, {
    comments: { lineComment: '//' },
    brackets: [
      ['(', ')'],
      ['[', ']'],
    ],
    autoClosingPairs: [
      { open: '(', close: ')' },
      { open: '[', close: ']' },
      { open: '"', close: '"' },
    ],
  })

  monacoApi.languages.setMonarchTokensProvider(JIANPU_LANGUAGE_ID, {
    defaultToken: '',
    tokenizer: {
      root: [
        // Quoted strings first: a `//` inside a string is not a comment.
        [/"([^"\\]|\\.)*"/, 'string'],
        [/\/\/.*$/, 'comment'],
        // [Abbrev] key prefix on parts/score lines.
        [/\[[^\]\n]*\]/, 'tag'],
        [/#/, 'operator'],
        [/[()]/, '@brackets'],
        [/[_=.\-~',]/, 'operator'],
      ],
    },
  })
}
