import { describe, expect, it, vi } from 'vitest'

/**
 * `renameSymbol.ts`, `shareUrl.ts`, and `utils/metadataDefaults.ts` each
 * define their own private `wasmReady`/`ensureWasmInit()` pair wrapping the
 * same underlying `jianpu-wasm` `init()`. Because those caches are separate
 * per file, calling into two of them before the first `init()` call
 * resolves — e.g. `parseShareFromHash()` on mount racing a rename-symbol
 * lookup — causes each file to independently see "not started yet" and
 * fire its own `init()`, each doing a full fetch of the wasm binary. This
 * is a second, independent source of the duplicated-wasm-download bug,
 * distinct from the worker-side race in jianpu.worker.ts.
 */

const hoisted = vi.hoisted(() => ({
  initCallCount: 0,
  resolveInit: undefined as (() => void) | undefined,
}))

vi.mock('jianpu-wasm', () => ({
  default: vi.fn(() => {
    hoisted.initCallCount++
    return new Promise<void>((resolve) => {
      hoisted.resolveInit = resolve
    })
  }),
  list_symbols: vi.fn(() => ({ status: 'ok', symbols: [] })),
  rename_symbol: vi.fn(),
  compress_share_payload: vi.fn(() => new Uint8Array()),
  decompress_share_payload: vi.fn(() => ''),
  get_metadata_defaults: vi.fn(() => ({})),
  get_default_lyrics_font_size: vi.fn(() => 0),
}))

describe('main-thread wasm init', () => {
  it('only calls wasm init() once across renameSymbol/shareUrl/metadataDefaults, even when they race', async () => {
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

    // Fire all entry points before the first init() resolves, the same way
    // independent app-startup call sites (including the worker-lifecycle
    // hand-off, represented here by ensureWasmModule()) would race in practice.
    void listRenameSymbols('')
    void encodeShareHashSuffix('a.jianpu', '')
    void loadMetadataDefaults()
    void ensureWasmModule()

    // The shared `ensureWasmInit()` chain (fetch -> compileStreaming -> init)
    // resolves over several microtask hops; wait for `init()` to actually
    // fire rather than hardcoding a tick count that drifts with the chain's
    // shape.
    await vi.waitFor(() => expect(hoisted.initCallCount).toBe(1))
    expect(compileStreamingMock).toHaveBeenCalledTimes(1)
    expect(fetchMock).toHaveBeenCalledTimes(1)

    hoisted.resolveInit?.()
    vi.unstubAllGlobals()
  })
})
