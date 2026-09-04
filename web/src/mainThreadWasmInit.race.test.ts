import { describe, expect, it, vi } from 'vitest'

/**
 * `renameSymbol.ts`, `shareUrl.ts`, and `utils/metadataDefaults.ts` each
 * call the shared `ensureWasmInit()` (`wasmInit.ts`) before touching
 * `jianpuWasm.ts`'s functions. Because `ensureWasmInit()` memoizes a single
 * in-flight promise, calling into two of them before the first
 * instantiate() call resolves — e.g. `parseShareFromHash()` on mount racing
 * a rename-symbol lookup — must still only trigger one fetch/instantiate of
 * the wasm component, not one per caller. This is the main-thread
 * counterpart of the worker-side race covered by
 * `worker/jianpu.worker.raceInit.test.ts`.
 */

const hoisted = vi.hoisted(() => ({
  instantiateCallCount: 0,
  resolveInstantiate: undefined as (() => void) | undefined,
}))

vi.mock('../../crates/jianpu-wasm/pkg-component/jianpu_wasm.js', () => ({
  instantiate: vi.fn(() => {
    hoisted.instantiateCallCount++
    return new Promise((resolve) => {
      hoisted.resolveInstantiate = () =>
        resolve({
          listSymbols: vi.fn(() => ({ tag: 'ok', val: { symbols: [] } })),
          renameSymbol: vi.fn(),
          compressSharePayload: vi.fn(() => new Uint8Array()),
          decompressSharePayload: vi.fn(() => ''),
          getMetadataDefaults: vi.fn(() => ({
            title: {},
            subtitle: {},
            author: {},
            sequence: {},
            partLegend: {},
            measureNumber: {},
            sectionLabel: {},
            partLabel: {},
            pageNumber: {},
            lyrics: {},
            notes: {},
            chords: {},
            noteDash: {},
          })),
          getDefaultLyricsFontSize: vi.fn(() => 0),
        })
    })
  }),
}))

describe('main-thread wasm init', () => {
  it('only instantiates the wasm component once across renameSymbol/shareUrl/metadataDefaults, even when they race', async () => {
    const fetchMock = vi.fn(() => Promise.resolve(new Response()))
    vi.stubGlobal('fetch', fetchMock)
    const compileStreamingMock = vi.fn(() => Promise.resolve({}))
    vi.stubGlobal('WebAssembly', {
      ...WebAssembly,
      compileStreaming: compileStreamingMock,
    })

    const { listRenameSymbols } = await import('./renameSymbol')
    const { encodeShareHashSuffix } = await import('./shareUrl')
    const { loadMetadataDefaults } = await import('./utils/metadataDefaults')
    const { ensureWasmModule } = await import('./wasmInit')

    // Fire all entry points before the first instantiate() resolves, the
    // same way independent app-startup call sites (including the
    // worker-lifecycle hand-off, represented here by ensureWasmModule())
    // would race in practice.
    void listRenameSymbols('')
    void encodeShareHashSuffix('a.jianpu', '')
    void loadMetadataDefaults()
    void ensureWasmModule()

    // The shared `ensureWasmInit()` chain (fetch -> compileStreaming ->
    // instantiate) resolves over several microtask hops; wait for
    // instantiate() to actually fire rather than hardcoding a tick count
    // that drifts with the chain's shape.
    await vi.waitFor(() => expect(hoisted.instantiateCallCount).toBe(1))
    expect(compileStreamingMock).toHaveBeenCalledTimes(1)
    expect(fetchMock).toHaveBeenCalledTimes(1)

    hoisted.resolveInstantiate?.()
    vi.unstubAllGlobals()
  })
})
