import type { ByteRange } from '../jianpuWasm'
import { jianpuWasm, setWasmRoot } from '../jianpuWasm'
import type { EditorSelection, PartDeclaration } from '../types'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import { instantiateWasmComponentFromModule } from '../wasmInit'
import {
  handleGenerateAudio,
  handleGenerateMeasureRangeAudio,
  handleGenerateMp3,
} from './audioMessageHandlers'
import {
  handleGenerateMidi,
  handleGeneratePdf,
  handleGenerateSplitMidi,
  handleGenerateSplitMp3,
  handleGenerateSplitPdf,
  handleGenerateSplitWav,
} from './exportMessageHandlers'
import { handleImportFromFile } from './importMessageHandlers'
import type { WorkerRequest, WorkerResponse } from './messages'
import {
  handlePreviewInstrument,
  handlePreviewPercussion,
} from './previewMessageHandlers'

export type { WorkerRequest, WorkerResponse } from './messages'

let resolveWasmModule: (module: WebAssembly.Module) => void
const wasmModulePromise = new Promise<WebAssembly.Module>((resolve) => {
  resolveWasmModule = resolve
})

let initPromise: Promise<void> | null = null

function ensureInit(): Promise<void> {
  if (!initPromise) {
    initPromise = wasmModulePromise
      .then((module) => instantiateWasmComponentFromModule(module))
      .then((root) => {
        setWasmRoot(root)
        postMessage({
          type: 'ready',
          audioAvailable: true,
          pdfAvailable: true,
          midiAvailable: true,
          mp3Available: true,
        } satisfies WorkerResponse)
      })
  }
  return initPromise
}

// Applies the layout fonts to the wasm module as soon as they've both
// arrived (from the `loadPdfFonts` message, reusing the bytes the app
// already fetches for PDF export) and the module is ready to receive them.
// Deliberately not awaited by any render: renders that happen before the
// fonts land just use the character-bucket fallback for that render, the
// same graceful degradation as a font fetch that fails outright. Blocking
// render on a network fetch would turn a slow or failed fetch into a stuck
// preview instead of a merely imprecise one.
function applyCoreFontsWhenReady(fonts: {
  sc: Uint8Array
  tc: Uint8Array
  mono: Uint8Array
}): void {
  // `setLayoutFonts(directiveLineFont, lyricFont, monospaceFont)` —
  // directive-line text measures against `tc` (the `sansSerif` role's
  // font), lyrics against `sc` (the `serif` role's font, shared with the
  // song title) — see `fonts/fonts.json` and
  // `DIRECTIVE_LINE_FONT_FAMILY`/`SERIF_FONT_FAMILY` in
  // src/serializer/mod.rs.
  ensureInit().then(() =>
    jianpuWasm().setLayoutFonts(fonts.tc, fonts.sc, fonts.mono),
  )
}

let loadedSoundfont: Uint8Array | null = null
// `sc` holds the `serif` role's font — the song title/lyric font; `tc`
// holds the `sansSerif` role's font, the default/body font for everything
// else — see `fonts/fonts.json` and `useFontsLoader`.
let loadedFonts: {
  sc: Uint8Array
  tc: Uint8Array
  mono: Uint8Array
} | null = null

function toByteRanges(ranges: EditorSelection[]): ByteRange[] {
  return ranges.map((range) => ({ startByte: range.start, endByte: range.end }))
}

function listDeclarationsFromSource(source: string): PartDeclaration[] {
  const result = jianpuWasm().listPartDeclarations(source, GM_INSTRUMENTS)
  return result.tag === 'ok' ? result.val.declarations : []
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const msg = event.data

  if (msg.type === 'wasmModule') {
    resolveWasmModule(msg.module)
    return
  }

  if (msg.type === 'loadSoundfont') {
    loadedSoundfont = new Uint8Array(msg.soundfont)
    return
  }

  if (msg.type === 'loadPdfFonts') {
    const sc = new Uint8Array(msg.scFont)
    const tc = new Uint8Array(msg.tcFont)
    const mono = new Uint8Array(msg.monoFont)
    loadedFonts = { sc, tc, mono }
    applyCoreFontsWhenReady({ sc, tc, mono })
    return
  }

  await ensureInit()

  if (msg.type === 'listParts') {
    const result = jianpuWasm().listParts(msg.source, GM_INSTRUMENTS)
    if (result.tag === 'ok') {
      postMessage({
        type: 'parts',
        id: msg.id,
        parts: result.val.parts,
        declarations: result.val.declarations,
      } satisfies WorkerResponse)
      return
    }

    postMessage({
      type: 'parts',
      id: msg.id,
      parts: [],
      declarations: [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'updatePartDeclaration') {
    const newSource = jianpuWasm().updatePartDeclaration(
      msg.source,
      msg.abbreviation,
      msg.settings,
    )
    postMessage({
      type: 'partDeclarationUpdated',
      id: msg.id,
      source: newSource,
      declarations: listDeclarationsFromSource(newSource),
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'formatScore') {
    postMessage({
      type: 'scoreFormatted',
      id: msg.id,
      source: jianpuWasm().formatScore(msg.source),
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'shiftPartOctave') {
    postMessage({
      type: 'partOctaveShifted',
      id: msg.id,
      source: jianpuWasm().shiftPartOctave(
        msg.source,
        msg.abbreviation,
        msg.delta,
      ),
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'editRange') {
    const { operation } = msg
    const ranges = toByteRanges(msg.ranges)
    const result =
      operation.kind === 'shiftOctave'
        ? jianpuWasm().shiftRangeOctave(msg.source, ranges, operation.delta)
        : operation.kind === 'toggleSlur'
          ? jianpuWasm().toggleRangeSlur(msg.source, ranges)
          : jianpuWasm().toggleRangeTie(msg.source, ranges)
    postMessage({
      type: 'rangeEdited',
      id: msg.id,
      source: result.source,
      ranges: result.ranges.map((range) => ({
        start: range.startByte,
        end: range.endByte,
      })),
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'describeSelection') {
    postMessage({
      type: 'selectionDescribed',
      id: msg.id,
      description:
        jianpuWasm().describeSelection(msg.source, toByteRanges(msg.ranges)) ??
        null,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'generatePdf') {
    handleGeneratePdf(msg, loadedFonts)
    return
  }

  if (msg.type === 'generateSplitPdf') {
    handleGenerateSplitPdf(msg, loadedFonts)
    return
  }

  if (msg.type === 'generateMidi') {
    handleGenerateMidi(msg)
    return
  }

  if (msg.type === 'generateSplitMidi') {
    handleGenerateSplitMidi(msg)
    return
  }

  if (msg.type === 'generateSplitWav') {
    handleGenerateSplitWav(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'generateMp3') {
    handleGenerateMp3(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'generateSplitMp3') {
    handleGenerateSplitMp3(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'generateAudio') {
    handleGenerateAudio(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'generateMeasureRangeAudio') {
    handleGenerateMeasureRangeAudio(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'previewInstrument') {
    handlePreviewInstrument(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'previewPercussion') {
    handlePreviewPercussion(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'renderWithHighlightRange') {
    const result = jianpuWasm().renderSvgWithHighlightRange(
      msg.source,
      msg.ranges,
      msg.enabledTracks,
      msg.disabledLyrics,
      GM_INSTRUMENTS,
    )
    if (result.tag === 'ok') {
      postMessage({
        type: 'highlightRangeOk',
        id: msg.id,
        documents: result.val.documents,
      } satisfies WorkerResponse)
      return
    }
    postMessage({
      type: 'highlightRangeErr',
      id: msg.id,
      diagnostics: result.val.diagnostics,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'importFromFile') {
    handleImportFromFile(msg)
    return
  }

  if (msg.type === 'listMeasureSpans') {
    const result = jianpuWasm().listMeasureSpans(msg.source)
    postMessage({
      type: 'measureSpans',
      id: msg.id,
      status: result.tag,
      spans: result.tag === 'ok' ? result.val.spans : [],
      sectionRanges: result.tag === 'ok' ? result.val.sectionRanges : [],
      sequenceEntries: result.tag === 'ok' ? result.val.sequenceEntries : [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'listNoteSpans') {
    const result = jianpuWasm().listNoteSpans(msg.source, msg.enabledTracks)
    postMessage({
      type: 'noteSpans',
      id: msg.id,
      status: result.tag,
      spans: result.tag === 'ok' ? result.val.spans : [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'listLyricSpans') {
    const result = jianpuWasm().listLyricSpans(msg.source, msg.enabledTracks)
    postMessage({
      type: 'lyricSpans',
      id: msg.id,
      status: result.tag,
      spans: result.tag === 'ok' ? result.val.spans : [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type !== 'render') return

  const result = jianpuWasm().renderSvg(
    msg.source,
    msg.enabledTracks,
    msg.disabledLyrics,
    GM_INSTRUMENTS,
  )
  if (result.tag === 'ok') {
    postMessage({
      type: 'ok',
      id: msg.id,
      documents: result.val.documents,
      diagnostics: result.val.diagnostics,
      diagnosticViewZones: result.val.diagnosticViewZones,
    } satisfies WorkerResponse)
    return
  }

  postMessage({
    type: 'err',
    id: msg.id,
    diagnostics: result.val.diagnostics,
    diagnosticViewZones: result.val.diagnosticViewZones,
  } satisfies WorkerResponse)
}
