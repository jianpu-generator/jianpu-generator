import type { SvgDocumentOut } from 'jianpu-wasm'
import init, * as jianpuWasm from 'jianpu-wasm'
import {
  list_measure_spans,
  list_parts,
  render,
  update_part_declaration,
} from 'jianpu-wasm'
import type {
  Diagnostic,
  DiagnosticViewZone,
  MeasureSpan,
  PartDeclaration,
  PartInfo,
  PartMode,
  SectionRange,
} from '../types'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import {
  handleGenerateMidi,
  handleGeneratePdf,
  handleGenerateSplitMidi,
  handleGenerateSplitPdf,
  handleGenerateSplitWav,
} from './exportMessageHandlers'

const generateWav =
  'generate_wav' in jianpuWasm ? jianpuWasm.generate_wav : null

const generateWavForMeasureRange =
  'generate_wav_for_measure_range' in jianpuWasm
    ? jianpuWasm.generate_wav_for_measure_range
    : null

const listMeasureTimes =
  'list_measure_times' in jianpuWasm ? jianpuWasm.list_measure_times : null

const listMeasureTimesForRange =
  'list_measure_times_for_range' in jianpuWasm
    ? jianpuWasm.list_measure_times_for_range
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

const renderPcmStreamingForMeasureRange =
  'render_pcm_streaming_for_measure_range' in jianpuWasm
    ? jianpuWasm.render_pcm_streaming_for_measure_range
    : null

// Must match `SAMPLE_RATE` in `src/wav.rs`.
const STREAMED_PCM_SAMPLE_RATE = 44100

export type WorkerRequest =
  | {
      type: 'loadSoundfont'
      soundfont: ArrayBuffer
    }
  | {
      type: 'loadPdfFonts'
      scFont: ArrayBuffer
      tcFont: ArrayBuffer
      monoFont: ArrayBuffer
    }
  | {
      type: 'render'
      source: string
      id: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
  | { type: 'listParts'; source: string; id: number }
  | {
      type: 'updatePartDeclaration'
      source: string
      abbreviation: string
      mode: PartMode
      followTarget: string | null
      soundfont: string | null
      volume: number | null
      octaveOffset: number | null
      id: number
    }
  | {
      type: 'generatePdf'
      source: string
      id: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
  | {
      type: 'generateSplitPdf'
      source: string
      id: number
      baseName: string
    }
  | {
      type: 'generateMidi'
      source: string
      id: number
      enabledTracks?: string[]
    }
  | {
      type: 'generateSplitMidi'
      source: string
      id: number
      baseName: string
    }
  | {
      type: 'generateSplitWav'
      source: string
      id: number
      baseName: string
    }
  | {
      type: 'generateAudio'
      source: string
      id: number
      enabledTracks?: string[]
    }
  | {
      type: 'generateMeasureRangeAudio'
      source: string
      id: number
      startMeasureIndex: number
      endMeasureIndex: number
      enabledTracks?: string[]
    }
  | {
      type: 'streamMeasureRangeAudio'
      source: string
      id: number
      startMeasureIndex: number
      endMeasureIndex: number
      enabledTracks?: string[]
    }
  | {
      type: 'renderWithHighlightRange'
      source: string
      id: number
      startMeasureIndex: number
      endMeasureIndex: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
  | { type: 'listMeasureSpans'; source: string; id: number }
  | { type: 'previewInstrument'; id: number; programNumber: number }

export type WorkerResponse =
  | {
      type: 'ready'
      audioAvailable: boolean
      pdfAvailable: boolean
      midiAvailable: boolean
    }
  | {
      type: 'ok'
      id: number
      documents: SvgDocumentOut[]
      diagnostics: Diagnostic[]
      diagnosticViewZones: DiagnosticViewZone[]
    }
  | { type: 'audio'; id: number; wav: ArrayBuffer; measureTimes: number[] }
  | { type: 'audioErr'; id: number }
  | {
      type: 'err'
      id: number
      diagnostics: Diagnostic[]
      diagnosticViewZones: DiagnosticViewZone[]
    }
  | {
      type: 'parts'
      id: number
      parts: PartInfo[]
      declarations: PartDeclaration[]
    }
  | {
      type: 'partDeclarationUpdated'
      id: number
      source: string
      declarations: PartDeclaration[]
    }
  | { type: 'pdf'; id: number; pdf: ArrayBuffer }
  | { type: 'pdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitPdf'; id: number; zip: ArrayBuffer }
  | { type: 'splitPdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'midi'; id: number; midi: ArrayBuffer }
  | { type: 'midiErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitMidi'; id: number; zip: ArrayBuffer }
  | { type: 'splitMidiErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitWav'; id: number; zip: ArrayBuffer }
  | { type: 'splitWavErr'; id: number; diagnostics: Diagnostic[] }
  | {
      type: 'measureRangeAudio'
      id: number
      wav: ArrayBuffer
      measureTimes: number[]
    }
  | { type: 'measureRangeAudioErr'; id: number }
  | {
      type: 'measureAudioChunk'
      id: number
      measureIndex: number
      pcm: ArrayBuffer
      sampleRate: number
      isFinal: boolean
    }
  | { type: 'measureAudioChunkErr'; id: number }
  | { type: 'instrumentPreview'; id: number; wav: ArrayBuffer }
  | { type: 'instrumentPreviewErr'; id: number }
  | { type: 'highlightRangeOk'; id: number; documents: SvgDocumentOut[] }
  | { type: 'highlightRangeErr'; id: number; diagnostics: Diagnostic[] }
  | {
      type: 'measureSpans'
      id: number
      status: 'ok' | 'err'
      spans: MeasureSpan[]
      sectionRanges: SectionRange[]
    }

let initialized = false

async function ensureInit() {
  if (!initialized) {
    await init()
    initialized = true
    postMessage({
      type: 'ready',
      audioAvailable: generateWav !== null,
      pdfAvailable: generatePdf !== null,
      midiAvailable: generateMidi !== null,
    } satisfies WorkerResponse)
  }
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

function measureTimesFromSource(
  source: string,
  enabledTracks: string[] | undefined,
): number[] {
  if (!listMeasureTimes) return []
  const result = listMeasureTimes(source, enabledTracks)
  return result.status === 'ok' ? result.times : []
}

function measureTimesForRangeFromSource(
  source: string,
  startMeasureIndex: number,
  endMeasureIndex: number,
  enabledTracks: string[] | undefined,
): number[] {
  if (!listMeasureTimesForRange) return []
  const result = listMeasureTimesForRange(
    source,
    startMeasureIndex,
    endMeasureIndex,
    enabledTracks,
  )
  return result.status === 'ok' ? result.times : []
}

function binaryBufferFromResult(
  bytes: Uint8Array | ArrayBuffer | ArrayLike<number>,
): ArrayBuffer {
  if (bytes instanceof ArrayBuffer) {
    return bytes.slice(0)
  }
  const view = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes)
  return view.slice().buffer
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const msg = event.data

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
    if (!generateWav) {
      postMessage({
        type: 'audioErr',
        id: msg.id,
      } satisfies WorkerResponse)
      return
    }

    if (!loadedSoundfont) {
      postMessage({ type: 'audioErr', id: msg.id } satisfies WorkerResponse)
      return
    }
    const wavResult = generateWav(
      msg.source,
      msg.enabledTracks,
      loadedSoundfont,
    )
    if (wavResult.status === 'ok' && wavResult.wav != null) {
      const wavBuffer = binaryBufferFromResult(wavResult.wav)
      postMessage(
        {
          type: 'audio',
          id: msg.id,
          wav: wavBuffer,
          measureTimes: measureTimesFromSource(msg.source, msg.enabledTracks),
        } satisfies WorkerResponse,
        { transfer: [wavBuffer] },
      )
      return
    }

    postMessage({
      type: 'audioErr',
      id: msg.id,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'generateMeasureRangeAudio') {
    if (!generateWavForMeasureRange) {
      postMessage({
        type: 'measureRangeAudioErr',
        id: msg.id,
      } satisfies WorkerResponse)
      return
    }
    if (!loadedSoundfont) {
      postMessage({
        type: 'measureRangeAudioErr',
        id: msg.id,
      } satisfies WorkerResponse)
      return
    }
    const wavResult = generateWavForMeasureRange(
      msg.source,
      msg.startMeasureIndex,
      msg.endMeasureIndex,
      msg.enabledTracks,
      loadedSoundfont,
    )
    if (wavResult.status === 'ok' && wavResult.wav != null) {
      const wavBuffer = binaryBufferFromResult(wavResult.wav)
      postMessage(
        {
          type: 'measureRangeAudio',
          id: msg.id,
          wav: wavBuffer,
          measureTimes: measureTimesForRangeFromSource(
            msg.source,
            msg.startMeasureIndex,
            msg.endMeasureIndex,
            msg.enabledTracks,
          ),
        } satisfies WorkerResponse,
        { transfer: [wavBuffer] },
      )
      return
    }
    postMessage({
      type: 'measureRangeAudioErr',
      id: msg.id,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'streamMeasureRangeAudio') {
    if (!renderPcmStreamingForMeasureRange || !loadedSoundfont) {
      postMessage({
        type: 'measureAudioChunkErr',
        id: msg.id,
      } satisfies WorkerResponse)
      return
    }
    const result = renderPcmStreamingForMeasureRange(
      msg.source,
      msg.startMeasureIndex,
      msg.endMeasureIndex,
      msg.enabledTracks,
      loadedSoundfont,
      (measureIndex: number, samples: Float32Array, isFinal: boolean) => {
        const pcm = samples.slice().buffer
        postMessage(
          {
            type: 'measureAudioChunk',
            id: msg.id,
            measureIndex,
            pcm,
            sampleRate: STREAMED_PCM_SAMPLE_RATE,
            isFinal,
          } satisfies WorkerResponse,
          { transfer: [pcm] },
        )
      },
    )
    if (result.status !== 'ok') {
      postMessage({
        type: 'measureAudioChunkErr',
        id: msg.id,
      } satisfies WorkerResponse)
    }
    return
  }

  if (msg.type === 'previewInstrument') {
    if (!generateInstrumentPreviewWav || !loadedSoundfont) {
      postMessage({
        type: 'instrumentPreviewErr',
        id: msg.id,
      } satisfies WorkerResponse)
      return
    }
    const result = generateInstrumentPreviewWav(
      msg.programNumber,
      loadedSoundfont,
    )
    if (result.status === 'ok' && result.wav != null) {
      const wavBuffer = binaryBufferFromResult(result.wav)
      postMessage(
        {
          type: 'instrumentPreview',
          id: msg.id,
          wav: wavBuffer,
        } satisfies WorkerResponse,
        { transfer: [wavBuffer] },
      )
      return
    }
    postMessage({
      type: 'instrumentPreviewErr',
      id: msg.id,
    } satisfies WorkerResponse)
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

  if (msg.type === 'listMeasureSpans') {
    const result = list_measure_spans(msg.source)
    postMessage({
      type: 'measureSpans',
      id: msg.id,
      status: result.status,
      spans: result.status === 'ok' ? result.spans : [],
      sectionRanges: result.status === 'ok' ? result.section_ranges : [],
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
