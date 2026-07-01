import { loader } from '@monaco-editor/react'
import * as monaco from 'monaco-editor'
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'

declare global {
  interface Window {
    MonacoEnvironment?: {
      getWorker: (_workerId: string, _label: string) => Worker
    }
    monaco?: typeof monaco
  }
}

window.MonacoEnvironment = {
  getWorker(_workerId, _label) {
    return new editorWorker()
  },
}

loader.config({ monaco })
window.monaco = monaco
