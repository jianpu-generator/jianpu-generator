// The single import surface for the wit-bindgen/jco wasm component: every
// type is re-exported straight from the jco-generated `.d.ts` (derived from
// `crates/jianpu-wasm/wit/world.wit`), and every call goes through
// `jianpuWasm()`, the component's generated `Root`. There is deliberately
// no reshaping layer in between — a field or case renamed on the Rust/WIT
// side surfaces as a type error at every consumer.
import type { Root } from '../../crates/jianpu-wasm/pkg-component/jianpu_wasm.js'

export type * from '../../crates/jianpu-wasm/pkg-component/jianpu_wasm.js'

let wasmRoot: Root | null = null

/** Set by `wasmInit.ts`/`jianpu.worker.ts` once the component is
 * instantiated on their respective thread. */
export function setWasmRoot(root: Root): void {
  wasmRoot = root
}

export function jianpuWasm(): Root {
  if (!wasmRoot) {
    throw new Error(
      'jianpuWasm: called before the wasm component finished initializing',
    )
  }
  return wasmRoot
}
