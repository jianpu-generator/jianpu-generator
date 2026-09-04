// Singleton wiring for the wit-bindgen/jco component instance — set by
// `wasmInit.ts`/`jianpu.worker.ts` once it's instantiated on their
// respective thread. Split out of `jianpuWasm.ts` (which re-exports
// `setWasmRoot`) purely to avoid a circular import between it and the
// `jianpuWasmApi*.ts` files that both need `root()`.
import type { Root } from '../../crates/jianpu-wasm/pkg-component/jianpu_wasm.js'

let wasmRoot: Root | null = null

export function setWasmRoot(root: Root): void {
  wasmRoot = root
}

export function root(): Root {
  if (!wasmRoot) {
    throw new Error(
      'jianpuWasm: called before the wasm component finished initializing',
    )
  }
  return wasmRoot
}
