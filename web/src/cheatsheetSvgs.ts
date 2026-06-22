import { cheatsheetSections } from './cheatsheetData'
import type { WorkerRequest, WorkerResponse } from './worker/jianpu.worker'

const svgMap = new Map<number, string>()

const snippetWorker = new Worker(
  new URL('./worker/jianpu.worker.ts', import.meta.url),
  { type: 'module' },
)

snippetWorker.onmessage = (event: MessageEvent<WorkerResponse>) => {
  const msg = event.data
  if (msg.type === 'snippetOk') {
    svgMap.set(msg.id, msg.svg)
  }
}

let flatIndex = 0
for (const section of cheatsheetSections) {
  for (const example of section.examples) {
    const id = flatIndex++
    let request: WorkerRequest
    if (example.kind === 'note') {
      request = { type: 'renderNoteTokenSnippet', id, syntax: example.syntax }
    } else if (example.kind === 'chord') {
      request = { type: 'renderChordTokenSnippet', id, syntax: example.syntax }
    } else if (example.kind === 'line') {
      request = {
        type: 'renderNotesLineSnippet',
        id,
        notesLine: example.notes_line,
      }
    } else {
      request = { type: 'renderPartsScoreSnippet', id, source: example.source }
    }
    snippetWorker.postMessage(request)
  }
}

export function getSnippetSvg(id: number): string | undefined {
  return svgMap.get(id)
}
