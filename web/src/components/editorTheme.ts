import type { Monaco } from '@monaco-editor/react'

export const EDITOR_THEME = 'jianpu'
// Matches preview measure highlight (rgba(255, 200, 0, 0.25)); Monaco only accepts hex.
const MEASURE_HIGHLIGHT_COLOR = '#ffc80040'

export function defineJianpuEditorTheme(monacoApi: Monaco) {
  monacoApi.editor.defineTheme(EDITOR_THEME, {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'comment', foreground: '008000', fontStyle: 'italic' },
      { token: 'string', foreground: 'a31515' },
      { token: 'keyword.section', foreground: '0000ff', fontStyle: 'bold' },
      { token: 'tag', foreground: '267f99', fontStyle: 'bold' },
      { token: 'keyword.directive', foreground: '0000ff' },
      {
        token: 'keyword.control',
        foreground: 'af00db',
        fontStyle: 'bold italic',
      },
      { token: 'type', foreground: '267f99' },
      { token: 'variable', foreground: '001080' },
      { token: 'operator', foreground: '795e26' },
    ],
    colors: {
      'editor.lineHighlightBackground': MEASURE_HIGHLIGHT_COLOR,
      'editor.lineHighlightBorder': '#00000000',
    },
  })
}
