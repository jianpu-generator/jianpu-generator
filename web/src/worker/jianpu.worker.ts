import init, * as jianpuWasm from 'jianpu-wasm'
import {
  get_measure_index_at_offset,
  list_measure_spans,
  list_parts,
  render,
} from 'jianpu-wasm'
import type { Diagnostic, PartInfo } from '../types'

const generateWav =
  'generate_wav' in jianpuWasm ? jianpuWasm.generate_wav : null

const generateWavForMeasure =
  'generate_wav_for_measure' in jianpuWasm
    ? jianpuWasm.generate_wav_for_measure
    : null

const renderWithHighlight =
  'render_with_highlight' in jianpuWasm
    ? jianpuWasm.render_with_highlight
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
      type: 'getMeasureAtOffset'
      source: string
      id: number
      byteOffset: number
    }
  | {
      type: 'generateMeasureAudio'
      source: string
      id: number
      measureIndex: number
      enabledTracks?: string[]
    }
  | {
      type: 'renderWithHighlight'
      source: string
      id: number
      highlightedMeasureIndex: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
  | { type: 'listMeasureSpans'; source: string; id: number }

export type WorkerResponse =
  | { type: 'ready'; audioAvailable: boolean; pdfAvailable: boolean }
  | { type: 'ok'; id: number; svgs: string[] }
  | { type: 'audio'; id: number; wav: ArrayBuffer }
  | { type: 'audioErr'; id: number }
  | { type: 'err'; id: number; diagnostics: Diagnostic[] }
  | { type: 'parts'; id: number; parts: PartInfo[] }
  | { type: 'pdf'; id: number; pdf: ArrayBuffer }
  | { type: 'pdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitPdf'; id: number; zip: ArrayBuffer }
  | { type: 'splitPdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'measureAtOffset'; id: number; measureIndex: number | null }
  | { type: 'measureAudio'; id: number; wav: ArrayBuffer }
  | { type: 'measureAudioErr'; id: number }
  | { type: 'highlightOk'; id: number; svgs: string[] }
  | { type: 'highlightErr'; id: number; diagnostics: Diagnostic[] }
  | {
      type: 'measureSpans'
      id: number
      status: 'ok' | 'err'
      spans: Array<{ start: number; end: number }>
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

function binaryBufferFromResult(bytes: Uint8Array): ArrayBuffer {
  if (bytes.byteOffset === 0 && bytes.byteLength === bytes.buffer.byteLength) {
    return bytes.buffer as ArrayBuffer
  }
  return bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength,
  ) as ArrayBuffer
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
    if (wavResult.status === 'ok') {
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

  if (msg.type === 'getMeasureAtOffset') {
    const result = get_measure_index_at_offset(msg.source, msg.byteOffset)
    postMessage({
      type: 'measureAtOffset',
      id: msg.id,
      measureIndex: result.status === 'ok' ? result.measure_index : null,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'generateMeasureAudio') {
    if (!generateWavForMeasure) {
      postMessage({
        type: 'measureAudioErr',
        id: msg.id,
      } satisfies WorkerResponse)
      return
    }
    const wavResult = generateWavForMeasure(
      msg.source,
      msg.measureIndex,
      msg.enabledTracks,
    )
    if (wavResult.status === 'ok') {
      const wavBuffer = binaryBufferFromResult(wavResult.wav)
      postMessage(
        {
          type: 'measureAudio',
          id: msg.id,
          wav: wavBuffer,
        } satisfies WorkerResponse,
        { transfer: [wavBuffer] },
      )
      return
    }
    postMessage({
      type: 'measureAudioErr',
      id: msg.id,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'renderWithHighlight') {
    if (!renderWithHighlight) {
      postMessage({
        type: 'highlightErr',
        id: msg.id,
        diagnostics: [
          {
            severity: 'error',
            message: 'render_with_highlight is not available in this build.',
            span: { start: 0, end: 0 },
          },
        ],
      } satisfies WorkerResponse)
      return
    }
    const result = renderWithHighlight(
      msg.source,
      msg.highlightedMeasureIndex,
      msg.enabledTracks,
      msg.disabledLyrics,
    )
    if (result.status === 'ok') {
      postMessage({
        type: 'highlightOk',
        id: msg.id,
        svgs: result.svgs,
      } satisfies WorkerResponse)
      return
    }
    postMessage({
      type: 'highlightErr',
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

  if (msg.type !== 'render') return

  const result = render(msg.source, msg.enabledTracks, msg.disabledLyrics)
  if (result.status === 'ok') {
    postMessage({
      type: 'ok',
      id: msg.id,
      svgs: result.svgs,
    } satisfies WorkerResponse)
    return
  }

  postMessage({
    type: 'err',
    id: msg.id,
    diagnostics: result.diagnostics,
  } satisfies WorkerResponse)
}
