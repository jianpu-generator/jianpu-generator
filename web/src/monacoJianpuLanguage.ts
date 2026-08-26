import type { Monaco } from '@monaco-editor/react'

export const JIANPU_LANGUAGE_ID = 'jianpu'

let registered = false

/** Basic Monarch tokenizer for `.jianpu` files — see syntax.md for the grammar. */
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
        [/^\s*#\s*(metadata|parts|sequence|score)\s*$/, 'keyword.section'],
        // [Abbrev] key prefix on parts/score lines.
        [/\[[^\]\n]*\]/, 'tag'],
        // Directive-line keywords, e.g. `bpm=92 key=C4 time=4/4 label="..."`.
        [
          /\b(bpm|key|time|label|merge_duplicate_measures_across_parts|hide_resting_parts)(?=\s*=)/,
          'keyword.directive',
        ],
        // Part-kind keywords in `# parts` declarations.
        [/\bfollow(?=\[)/, 'type'],
        [/\bnotes\+lyrics\b/, 'type'],
        [/\b(chords|notes|percussion)\b/, 'type'],
        // Generic `key = value` metadata field name.
        [/^\s*[a-zA-Z_][a-zA-Z0-9_]*(?=\s*=)/, 'variable'],
        [/#/, 'operator'],
        [/[()]/, '@brackets'],
        [/[_=.\-~',]/, 'operator'],
      ],
    },
  })
}
