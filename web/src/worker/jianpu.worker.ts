import init, * as jianpuWasm from 'jianpu-wasm'
import {
  list_measure_spans,
  list_parts,
  list_score_line_hints,
  render,
} from 'jianpu-wasm'
import type {
  Diagnostic,
  DiagnosticViewZone,
  MeasureSpan,
  PartInfo,
  ScoreLineHint,
} from '../types'

const generateWav =
  'generate_wav' in jianpuWasm ? jianpuWasm.generate_wav : null

const generateWavForMeasureRange =
  'generate_wav_for_measure_range' in jianpuWasm
    ? jianpuWasm.generate_wav_for_measure_range
    : null

const renderWithHighlightRange =
  'render_with_highlight_range' in jianpuWasm
    ? jianpuWasm.render_with_highlight_range
    : null

const generatePdf =
  'generate_pdf' in jianpuWasm ? jianpuWasm.generate_pdf : null

const generateSplitPdfs =
  'generate_split_pdfs' in jianpuWasm ? jianpuWasm.generate_split_pdfs : null

export type WorkerRequest =
  | {
      type: 'render'
      source: string
      id: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
  | { type: 'listParts'; source: string; id: number }
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
      type: 'renderWithHighlightRange'
      source: string
      id: number
      startMeasureIndex: number
      endMeasureIndex: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
  | { type: 'listMeasureSpans'; source: string; id: number }
  | { type: 'listScoreLineHints'; source: string; id: number }

export type WorkerResponse =
  | { type: 'ready'; audioAvailable: boolean; pdfAvailable: boolean }
  | {
      type: 'ok'
      id: number
      svgs: string[]
      diagnostics: Diagnostic[]
      diagnosticViewZones: DiagnosticViewZone[]
    }
  | { type: 'audio'; id: number; wav: ArrayBuffer }
  | { type: 'audioErr'; id: number }
  | {
      type: 'err'
      id: number
      diagnostics: Diagnostic[]
      diagnosticViewZones: DiagnosticViewZone[]
    }
  | { type: 'parts'; id: number; parts: PartInfo[] }
  | { type: 'pdf'; id: number; pdf: ArrayBuffer }
  | { type: 'pdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitPdf'; id: number; zip: ArrayBuffer }
  | { type: 'splitPdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'measureRangeAudio'; id: number; wav: ArrayBuffer }
  | { type: 'measureRangeAudioErr'; id: number }
  | { type: 'highlightRangeOk'; id: number; svgs: string[] }
  | { type: 'highlightRangeErr'; id: number; diagnostics: Diagnostic[] }
  | {
      type: 'measureSpans'
      id: number
      status: 'ok' | 'err'
      spans: MeasureSpan[]
    }
  | {
      type: 'scoreLineHints'
      id: number
      status: 'ok' | 'err'
      hints: ScoreLineHint[]
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
    } satisfies WorkerResponse)
  }
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
  await ensureInit()

  if (msg.type === 'listParts') {
    const result = list_parts(msg.source)
    if (result.status === 'ok') {
      postMessage({
        type: 'parts',
        id: msg.id,
        parts: result.parts,
      } satisfies WorkerResponse)
      return
    }

    postMessage({
      type: 'parts',
      id: msg.id,
      parts: [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'generatePdf') {
    if (!generatePdf) {
      postMessage({
        type: 'pdfErr',
        id: msg.id,
        diagnostics: [
          {
            severity: 'error',
            message: 'PDF export is not available in this build.',
            span: { start: 0, end: 0 },
          },
        ],
      } satisfies WorkerResponse)
      return
    }

    const result = generatePdf(
      msg.source,
      msg.enabledTracks,
      msg.disabledLyrics,
    )
    if (result.status === 'ok') {
      const pdfBuffer = binaryBufferFromResult(result.pdf)
      postMessage(
        {
          type: 'pdf',
          id: msg.id,
          pdf: pdfBuffer,
        } satisfies WorkerResponse,
        { transfer: [pdfBuffer] },
      )
      return
    }

    postMessage({
      type: 'pdfErr',
      id: msg.id,
      diagnostics: result.diagnostics,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'generateSplitPdf') {
    if (!generateSplitPdfs) {
      postMessage({
        type: 'splitPdfErr',
        id: msg.id,
        diagnostics: [
          {
            severity: 'error',
            message: 'Split PDF export is not available in this build.',
            span: { start: 0, end: 0 },
          },
        ],
      } satisfies WorkerResponse)
      return
    }

    const result = generateSplitPdfs(msg.source, msg.baseName)
    if (result.status === 'ok') {
      const zipBuffer = binaryBufferFromResult(result.zip)
      postMessage(
        {
          type: 'splitPdf',
          id: msg.id,
          zip: zipBuffer,
        } satisfies WorkerResponse,
        { transfer: [zipBuffer] },
      )
      return
    }

    postMessage({
      type: 'splitPdfErr',
      id: msg.id,
      diagnostics: result.diagnostics,
    } satisfies WorkerResponse)
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

    const wavResult = generateWav(msg.source, msg.enabledTracks)
    if (wavResult.status === 'ok' && wavResult.wav != null) {
      const wavBuffer = binaryBufferFromResult(wavResult.wav)
      postMessage(
        {
          type: 'audio',
          id: msg.id,
          wav: wavBuffer,
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
    const wavResult = generateWavForMeasureRange(
      msg.source,
      msg.startMeasureIndex,
      msg.endMeasureIndex,
      msg.enabledTracks,
    )
    if (wavResult.status === 'ok' && wavResult.wav != null) {
      const wavBuffer = binaryBufferFromResult(wavResult.wav)
      postMessage(
        {
          type: 'measureRangeAudio',
          id: msg.id,
          wav: wavBuffer,
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
    )
    if (result.status === 'ok') {
      postMessage({
        type: 'highlightRangeOk',
        id: msg.id,
        svgs: result.svgs,
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
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'listScoreLineHints') {
    const result = list_score_line_hints(msg.source)
    postMessage({
      type: 'scoreLineHints',
      id: msg.id,
      status: result.status,
      hints: result.status === 'ok' ? result.hints : [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type !== 'render') return

  const result = render(msg.source, msg.enabledTracks, msg.disabledLyrics)
  if (result.status === 'ok') {
    postMessage({
      type: 'ok',
      id: msg.id,
      svgs: result.svgs,
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
