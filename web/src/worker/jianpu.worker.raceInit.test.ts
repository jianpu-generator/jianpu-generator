import { describe, expect, it, vi } from 'vitest'
import type { WorkerRequest } from './messages'

/**
 * Reproduces the production bug where the jianpu-wasm binary is fetched
 * multiple times. `ensureInit()` in jianpu.worker.ts guards on a boolean
 * that is only flipped to `true` *after* `await init()` resolves. The
 * generated wasm-bindgen `init()` (crates/jianpu-wasm/pkg/jianpu_wasm.js)
 * has no in-flight-promise cache of its own either — it only short-circuits
 * once fully finished — so every worker message that reaches `ensureInit()`
 * before the first `init()` call settles triggers its own independent
 * `fetch()` of the wasm binary.
 *
 * This covers every message type that funnels through `ensureInit()`
 * (everything except `loadSoundfont`/`loadPdfFonts`, which return before
 * reaching it), dispatched in a single burst, so the test guards the
 * invariant itself ("init() runs exactly once no matter which/how many
 * messages race in first") rather than one specific message pairing.
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
  list_parts: vi.fn(() => ({ status: 'ok', parts: [], declarations: [] })),
  render: vi.fn(() => ({
    status: 'ok',
    documents: [],
    diagnostics: [],
    diagnostic_view_zones: [],
  })),
  update_part_declaration: vi.fn(),
  list_measure_spans: vi.fn(() => ({
    status: 'ok',
    spans: [],
    section_ranges: [],
    sequence_entries: [],
  })),
  extract_source_from_pdf: vi.fn(),
  extract_source_from_svg: vi.fn(),
}))

const raceMessages: WorkerRequest[] = [
  { type: 'render', source: '', id: 1 },
  { type: 'listParts', source: '', id: 2 },
  {
    type: 'updatePartDeclaration',
    source: '',
    abbreviation: 'S',
    mode: 'notes',
    followTarget: null,
    soundfont: null,
    volume: null,
    octaveOffset: null,
    id: 3,
  },
  { type: 'generatePdf', source: '', id: 4 },
  { type: 'generateSplitPdf', source: '', id: 5, baseName: 'x' },
  { type: 'generateMidi', source: '', id: 6 },
  { type: 'generateSplitMidi', source: '', id: 7, baseName: 'x' },
  { type: 'generateSplitWav', source: '', id: 8, baseName: 'x' },
  { type: 'generateAudio', source: '', id: 9 },
  {
    type: 'generateMeasureRangeAudio',
    source: '',
    id: 10,
    startMeasureIndex: 0,
    endMeasureIndex: 0,
    extendToLastOccurrence: false,
    respectSequence: false,
  },
  {
    type: 'renderWithHighlightRange',
    source: '',
    id: 11,
    startMeasureIndex: 0,
    endMeasureIndex: 0,
  },
  { type: 'listMeasureSpans', source: '', id: 12 },
  { type: 'previewInstrument', id: 13, programNumber: 0 },
  { type: 'previewPercussion', id: 14, key: 0 },
  { type: 'importFromFile', id: 15, bytes: new ArrayBuffer(0), kind: 'svg' },
]

describe('jianpu.worker ensureInit', () => {
  it('only calls wasm init() once, no matter how many messages race in before it resolves', async () => {
    vi.stubGlobal('self', globalThis)
    const postMessageMock = vi.fn()
    vi.stubGlobal('postMessage', postMessageMock)

    await import('./jianpu.worker')

    const onmessage = (self as unknown as { onmessage: (e: unknown) => void })
      .onmessage

    onmessage({ data: { type: 'wasmModule', module: {} } })

    for (const data of raceMessages) {
      onmessage({ data })
    }

    // The shared ensureInit() chain (wasmModulePromise -> init() -> the
    // "ready" postMessage) resolves over several microtask hops; wait for
    // init() to actually fire rather than hardcoding a tick count.
    await vi.waitFor(() => expect(hoisted.initCallCount).toBe(1))

    hoisted.resolveInit?.()
    // Likewise, wait for the "ready" postMessage that init() resolving
    // triggers before unstubbing postMessage out from under it.
    await vi.waitFor(() => expect(postMessageMock).toHaveBeenCalled())
    vi.unstubAllGlobals()
  })
})
