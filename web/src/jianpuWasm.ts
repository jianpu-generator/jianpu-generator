// Drop-in replacement for the old wasm-bindgen `'jianpu-wasm'` package
// (PLAN-wit-bindgen-migration.md Phase 6 cutover), backed by the
// wit-bindgen/cargo-component/jco build instead. Exposes the exact same
// function names/signatures and output shapes (snake_case where the old
// tsify-generated types used snake_case, `{ status: 'ok' | 'err', ... }`
// flat response envelopes, a recursive `SvgElementOut` tree, etc.) as the
// old `pkg/jianpu_wasm.d.ts` did, so every consumer elsewhere in `web/`
// keeps working unchanged — only this file's sibling `jianpuWasm*.ts` files
// and `wasmInit.ts`/`jianpu.worker.ts` (which supply the singleton `Root`
// instance via `setWasmRoot`) know about the new wit-bindgen/jco shapes at
// all. Split into `jianpuWasmRoot.ts` (singleton wiring), `jianpuWasmTypes.ts`
// (output shapes), `jianpuWasmConvert.ts` (shape conversion helpers), and
// `jianpuWasmApiCore.ts`/`jianpuWasmApiAudio.ts` (the actual public API,
// re-exported here so this stays the single import surface) purely to keep
// every file under the crate's 400-line-per-file cap.
//
// The actual type-safety win this migration is for — every function's
// *inputs* (previously `JsValue`/`any`, decoded via
// `serde_wasm_bindgen::from_value(...).unwrap_or_default()` with no
// compile-time checking at all) — is real here: every call in
// `jianpuWasmApiCore.ts`/`jianpuWasmApiAudio.ts` passes a genuinely-typed
// argument to the generated `Root` method. Only outputs are reshaped back to
// the old convention, since outputs were already `tsify`-generated and
// type-safe before this migration (see the plan's "Why" section) —
// reshaping them isn't required by the plan's own goal, and avoids a
// needless field-rename sweep across the whole app.

export * from './jianpuWasmApiAudio'
export * from './jianpuWasmApiCore'
export { setWasmRoot } from './jianpuWasmRoot'
export * from './jianpuWasmTypes'
