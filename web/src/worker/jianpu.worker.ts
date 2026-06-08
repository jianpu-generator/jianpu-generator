import init, { render } from 'jianpu-wasm'
import type { Diagnostic, RenderResult } from '../types'

export type WorkerRequest = { type: 'render'; source: string; id: number }

export type WorkerResponse =
  | { type: 'ready' }
  | { type: 'ok'; id: number; svgs: string[] }
  | { type: 'err'; id: number; diagnostics: Diagnostic[] }

let initialized = false

async function ensureInit() {
  if (!initialized) {
    await init()
    initialized = true
    postMessage({ type: 'ready' } satisfies WorkerResponse)
  }
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const msg = event.data
  if (msg.type !== 'render') return

  await ensureInit()

  const result = render(msg.source) as RenderResult
  if (result.status === 'ok') {
    postMessage({
      type: 'ok',
      id: msg.id,
      svgs: result.svgs,
    } satisfies WorkerResponse)
    return
  }

  postMessage({
    type: 'err',
    id: msg.id,
    diagnostics: result.diagnostics,
  } satisfies WorkerResponse)
}
