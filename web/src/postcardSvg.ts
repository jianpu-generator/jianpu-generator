import { POSTCARD_SOURCE } from './postcardSource'
import type { WorkerRequest, WorkerResponse } from './worker/jianpu.worker'

const POSTCARD_ID = 0

let svg: string | undefined

const listeners = new Set<() => void>()

export function onPostcardUpdate(fn: () => void): () => void {
  listeners.add(fn)
  return () => listeners.delete(fn)
}

export function getPostcardSvg(): string | undefined {
  return svg
}

const worker = new Worker(
  new URL('./worker/jianpu.worker.ts', import.meta.url),
  { type: 'module' },
)

worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
  const msg = event.data
  if (msg.type === 'snippetOk' && msg.id === POSTCARD_ID) {
    svg = msg.svg
    for (const fn of listeners) fn()
  }
}

worker.postMessage({
  type: 'renderPartsScoreSnippet',
  id: POSTCARD_ID,
  source: POSTCARD_SOURCE,
  showDecorations: false,
} satisfies WorkerRequest)
