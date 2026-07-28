import init from 'jianpu-wasm'
import wasmBinaryUrl from '../../crates/jianpu-wasm/pkg/jianpu_wasm_bg.wasm?url'

let modulePromise: Promise<WebAssembly.Module> | null = null

/** Fetches and compiles the wasm binary exactly once, sharing a single in-flight
 * promise across every caller (both main-thread consumers and the render worker,
 * which is handed the resolved module via `postMessage`) so the ~24MB binary is
 * only downloaded once per page load instead of once per JS realm. */
export function ensureWasmModule(): Promise<WebAssembly.Module> {
  if (!modulePromise) {
    modulePromise = WebAssembly.compileStreaming(fetch(wasmBinaryUrl)).catch(
      async () =>
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
