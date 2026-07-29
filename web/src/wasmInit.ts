import init from 'jianpu-wasm'
import wasmBinaryUrl from '../../crates/jianpu-wasm/pkg/jianpu_wasm_bg.wasm?url'

type WasmProgressListener = (loadedBytes: number, totalBytes: number) => void

const progressListeners = new Set<WasmProgressListener>()

/** Lets `useWasmLoader` observe download progress of the shared wasm fetch
 * below without owning the fetch itself, since `ensureWasmModule` may already
 * be in flight (or memoized) by the time the hook mounts. */
export function subscribeWasmProgress(
  listener: WasmProgressListener,
): () => void {
  progressListeners.add(listener)
  return () => progressListeners.delete(listener)
}

function reportProgress(loadedBytes: number, totalBytes: number): void {
  for (const listener of progressListeners) listener(loadedBytes, totalBytes)
}

async function fetchWithProgress(url: string): Promise<Response> {
  const response = await fetch(url)
  if (!response.body) return response

  const totalBytes = Number(response.headers.get('content-length') ?? 0)
  let loadedBytes = 0
  const progressStream = new TransformStream<Uint8Array, Uint8Array>({
    transform(chunk, controller) {
      loadedBytes += chunk.byteLength
      reportProgress(loadedBytes, totalBytes)
      controller.enqueue(chunk)
    },
  })

  return new Response(response.body.pipeThrough(progressStream), {
    headers: response.headers,
  })
}

let modulePromise: Promise<WebAssembly.Module> | null = null

/** Fetches and compiles the wasm binary exactly once, sharing a single in-flight
 * promise across every caller (both main-thread consumers and the render worker,
 * which is handed the resolved module via `postMessage`) so the ~24MB binary is
 * only downloaded once per page load instead of once per JS realm. */
export function ensureWasmModule(): Promise<WebAssembly.Module> {
  if (!modulePromise) {
    modulePromise = fetchWithProgress(wasmBinaryUrl)
      .then((response) => WebAssembly.compileStreaming(response))
      .catch(async () =>
        WebAssembly.compile(await (await fetch(wasmBinaryUrl)).arrayBuffer()),
      )
  }
  return modulePromise
}

let wasmReady: Promise<void> | null = null

/** Initializes the `jianpu-wasm` module on the main thread, sharing a single
 * in-flight promise across every caller so concurrent first uses (e.g. share-link
 * parsing racing a rename-symbol lookup on startup) don't each trigger their own
 * fetch of the wasm binary. */
export function ensureWasmInit(): Promise<void> {
  if (!wasmReady) {
    wasmReady = ensureWasmModule()
      .then((module) => init({ module_or_path: module }))
      .then(() => undefined)
  }
  return wasmReady
}
