import init, * as jianpuWasm from 'jianpu-wasm'
import {
  extract_source_from_pdf,
  extract_source_from_svg,
  list_measure_spans,
  list_parts,
  render,
  update_part_declaration,
} from 'jianpu-wasm'
import type { PartDeclaration, PartMode } from '../types'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import {
  handleGenerateAudio,
  handleGenerateMeasureRangeAudio,
} from './audioMessageHandlers'
import {
  handleGenerateMidi,
  handleGeneratePdf,
  handleGenerateSplitMidi,
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

const generateWav =
  'generate_wav' in jianpuWasm ? jianpuWasm.generate_wav : null

const generateWavForMeasureRange =
  'generate_wav_for_measure_range' in jianpuWasm
    ? jianpuWasm.generate_wav_for_measure_range
    : null

const listNoteTimings =
  'list_note_timings' in jianpuWasm ? jianpuWasm.list_note_timings : null

const listNoteTimingsForRange =
  'list_note_timings_for_range' in jianpuWasm
    ? jianpuWasm.list_note_timings_for_range
    : null

const renderWithHighlightRange =
  'render_with_highlight_range' in jianpuWasm
    ? jianpuWasm.render_with_highlight_range
    : null

const generatePdf =
  'generate_pdf' in jianpuWasm ? jianpuWasm.generate_pdf : null

const generateSplitPdfs =
  'generate_split_pdfs' in jianpuWasm ? jianpuWasm.generate_split_pdfs : null

const generateMidi =
  'generate_midi' in jianpuWasm ? jianpuWasm.generate_midi : null

const generateSplitMidis =
  'generate_split_midis' in jianpuWasm ? jianpuWasm.generate_split_midis : null

const generateSplitWavs =
  'generate_split_wavs' in jianpuWasm ? jianpuWasm.generate_split_wavs : null

const generateInstrumentPreviewWav =
  'generate_instrument_preview_wav' in jianpuWasm
    ? jianpuWasm.generate_instrument_preview_wav
    : null

const generatePercussionPreviewWav =
  'generate_percussion_preview_wav' in jianpuWasm
    ? jianpuWasm.generate_percussion_preview_wav
    : null

let resolveWasmModule: (module: WebAssembly.Module) => void
const wasmModulePromise = new Promise<WebAssembly.Module>((resolve) => {
  resolveWasmModule = resolve
})

let initPromise: Promise<void> | null = null

function ensureInit(): Promise<void> {
  if (!initPromise) {
    initPromise = wasmModulePromise
      .then((module) => init({ module_or_path: module }))
      .then(() => {
        postMessage({
          type: 'ready',
          audioAvailable: generateWav !== null,
          pdfAvailable: generatePdf !== null,
          midiAvailable: generateMidi !== null,
        } satisfies WorkerResponse)
      })
  }
  return initPromise
}

let loadedSoundfont: Uint8Array | null = null
let loadedFonts: { sc: Uint8Array; tc: Uint8Array; mono: Uint8Array } | null =
  null

function modeToWasmString(mode: PartMode, followTarget: string | null): string {
  if (mode === 'follow') {
    return `follow[${followTarget ?? ''}]`
  }
  return mode
}

function octaveOffsetToWasmString(octaveOffset: number | null): string {
  if (octaveOffset == null || octaveOffset === 0) return ''
  return octaveOffset > 0 ? `+${octaveOffset}` : String(octaveOffset)
}

function listDeclarationsFromSource(source: string): PartDeclaration[] {
  if (!('list_part_declarations' in jianpuWasm)) return []
  const result = jianpuWasm.list_part_declarations(source, GM_INSTRUMENTS)
  return result.status === 'ok' ? result.declarations : []
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
    loadedFonts = {
      sc: new Uint8Array(msg.scFont),
      tc: new Uint8Array(msg.tcFont),
      mono: new Uint8Array(msg.monoFont),
    }
    return
  }

  await ensureInit()

  if (msg.type === 'listParts') {
    const result = list_parts(msg.source, GM_INSTRUMENTS)
    if (result.status === 'ok') {
      postMessage({
        type: 'parts',
        id: msg.id,
        parts: result.parts,
        declarations: result.declarations,
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
    const newSource = update_part_declaration(
      msg.source,
      msg.abbreviation,
      modeToWasmString(msg.mode, msg.followTarget),
      msg.soundfont ?? '',
      msg.volume != null ? String(msg.volume) : '',
      octaveOffsetToWasmString(msg.octaveOffset),
    )
    postMessage({
      type: 'partDeclarationUpdated',
      id: msg.id,
      source: newSource,
      declarations: listDeclarationsFromSource(newSource),
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'generatePdf') {
    handleGeneratePdf(msg, generatePdf, loadedFonts)
    return
  }

  if (msg.type === 'generateSplitPdf') {
    handleGenerateSplitPdf(msg, generateSplitPdfs, loadedFonts)
    return
  }

  if (msg.type === 'generateMidi') {
    handleGenerateMidi(msg, generateMidi)
    return
  }

  if (msg.type === 'generateSplitMidi') {
    handleGenerateSplitMidi(msg, generateSplitMidis)
    return
  }

  if (msg.type === 'generateSplitWav') {
    handleGenerateSplitWav(msg, generateSplitWavs, loadedSoundfont)
    return
  }

  if (msg.type === 'generateAudio') {
    handleGenerateAudio(msg, generateWav, listNoteTimings, loadedSoundfont)
    return
  }

  if (msg.type === 'generateMeasureRangeAudio') {
    handleGenerateMeasureRangeAudio(
      msg,
      generateWavForMeasureRange,
      listNoteTimingsForRange,
      loadedSoundfont,
    )
    return
  }

  if (msg.type === 'previewInstrument') {
    handlePreviewInstrument(msg, generateInstrumentPreviewWav, loadedSoundfont)
    return
  }

  if (msg.type === 'previewPercussion') {
    handlePreviewPercussion(msg, generatePercussionPreviewWav, loadedSoundfont)
    return
  }

  if (msg.type === 'renderWithHighlightRange') {
    if (!renderWithHighlightRange) {
      postMessage({
        type: 'highlightRangeErr',
        id: msg.id,
        diagnostics: [
          {
            severity: 'error',
            message:
              'render_with_highlight_range is not available in this build.',
            span: { start: 0, end: 0 },
          },
        ],
      } satisfies WorkerResponse)
      return
    }
    const result = renderWithHighlightRange(
      msg.source,
      msg.startMeasureIndex,
      msg.endMeasureIndex,
      msg.enabledTracks,
      msg.disabledLyrics,
      GM_INSTRUMENTS,
    )
    if (result.status === 'ok') {
      postMessage({
        type: 'highlightRangeOk',
        id: msg.id,
        documents: result.documents,
      } satisfies WorkerResponse)
      return
    }
    postMessage({
      type: 'highlightRangeErr',
      id: msg.id,
      diagnostics: result.diagnostics,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'importFromFile') {
    handleImportFromFile(msg, extract_source_from_svg, extract_source_from_pdf)
    return
  }

  if (msg.type === 'listMeasureSpans') {
    const result = list_measure_spans(msg.source)
    postMessage({
      type: 'measureSpans',
      id: msg.id,
      status: result.status,
      spans: result.status === 'ok' ? result.spans : [],
      sectionRanges: result.status === 'ok' ? result.section_ranges : [],
      sequenceEntries: result.status === 'ok' ? result.sequence_entries : [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type !== 'render') return

  const result = render(
    msg.source,
    msg.enabledTracks,
    msg.disabledLyrics,
    GM_INSTRUMENTS,
  )
  if (result.status === 'ok') {
    postMessage({
      type: 'ok',
      id: msg.id,
      documents: result.documents,
      diagnostics: result.diagnostics,
      diagnosticViewZones: result.diagnostic_view_zones,
    } satisfies WorkerResponse)
    return
  }

  postMessage({
    type: 'err',
    id: msg.id,
    diagnostics: result.diagnostics,
    diagnosticViewZones: result.diagnostic_view_zones,
  } satisfies WorkerResponse)
}
