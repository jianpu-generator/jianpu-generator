import init, { render } from 'jianpu-wasm'
import type { RenderError } from '../types'

export type WorkerRequest = { type: 'render'; source: string; id: number }

export type WorkerResponse =
  | { type: 'ready' }
  | { type: 'ok'; id: number; svgs: string[] }
  | { type: 'err'; id: number; error: RenderError }

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

  try {
    const result = render(msg.source) as { svgs: string[] }
    postMessage({
      type: 'ok',
      id: msg.id,
      svgs: result.svgs,
    } satisfies WorkerResponse)
  } catch (thrown) {
    const error = thrown as RenderError
    postMessage({
      type: 'err',
      id: msg.id,
      error: {
        message: error.message ?? String(thrown),
        span: error.span ?? { start: 0, end: 0 },
        report: error.report,
      },
    } satisfies WorkerResponse)
  }
}
