import type {
  GenerateMidiResponse,
  GeneratePdfResponse,
  GenerateSplitMidisResponse,
  GenerateSplitPdfsResponse,
  GenerateSplitWavsResponse,
} from 'jianpu-wasm'
import type { WorkerRequest, WorkerResponse } from './jianpu.worker'

type LoadedFonts = { sc: Uint8Array; tc: Uint8Array; mono: Uint8Array } | null

function binaryBufferFromResult(
  bytes: Uint8Array | ArrayBuffer | ArrayLike<number>,
): ArrayBuffer {
  if (bytes instanceof ArrayBuffer) {
    return bytes.slice(0)
  }
  const view = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes)
  return view.slice().buffer
}

export function handleGeneratePdf(
  msg: Extract<WorkerRequest, { type: 'generatePdf' }>,
  generatePdf:
    | ((
        source: string,
        enabledTracks: string[] | undefined,
        disabledLyrics: string[] | undefined,
        sansSerifSc: Uint8Array,
        sansSerifTc: Uint8Array,
        monospace: Uint8Array,
      ) => GeneratePdfResponse)
    | null,
  loadedFonts: LoadedFonts,
): void {
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

  if (!loadedFonts) {
    postMessage({
      type: 'pdfErr',
      id: msg.id,
      diagnostics: [
        {
          severity: 'error',
          message: 'Fonts are not yet loaded.',
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
    loadedFonts.sc,
    loadedFonts.tc,
    loadedFonts.mono,
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
}

export function handleGenerateSplitPdf(
  msg: Extract<WorkerRequest, { type: 'generateSplitPdf' }>,
  generateSplitPdfs:
    | ((
        source: string,
        baseName: string,
        sansSerifSc: Uint8Array,
        sansSerifTc: Uint8Array,
        monospace: Uint8Array,
      ) => GenerateSplitPdfsResponse)
    | null,
  loadedFonts: LoadedFonts,
): void {
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

  if (!loadedFonts) {
    postMessage({
      type: 'splitPdfErr',
      id: msg.id,
      diagnostics: [
        {
          severity: 'error',
          message: 'Fonts are not yet loaded.',
          span: { start: 0, end: 0 },
        },
      ],
    } satisfies WorkerResponse)
    return
  }
  const result = generateSplitPdfs(
    msg.source,
    msg.baseName,
    loadedFonts.sc,
    loadedFonts.tc,
    loadedFonts.mono,
  )
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
}

export function handleGenerateMidi(
  msg: Extract<WorkerRequest, { type: 'generateMidi' }>,
  generateMidi:
    | ((
        source: string,
        enabledTracks: string[] | undefined,
      ) => GenerateMidiResponse)
    | null,
): void {
  if (!generateMidi) {
    postMessage({
      type: 'midiErr',
      id: msg.id,
      diagnostics: [
        {
          severity: 'error',
          message: 'MIDI export is not available in this build.',
          span: { start: 0, end: 0 },
        },
      ],
    } satisfies WorkerResponse)
    return
  }
  const result = generateMidi(msg.source, msg.enabledTracks)
  if (result.status === 'ok') {
    const midiBuffer = binaryBufferFromResult(result.midi)
    postMessage(
      {
        type: 'midi',
        id: msg.id,
        midi: midiBuffer,
      } satisfies WorkerResponse,
      { transfer: [midiBuffer] },
    )
    return
  }

  postMessage({
    type: 'midiErr',
    id: msg.id,
    diagnostics: result.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateSplitMidi(
  msg: Extract<WorkerRequest, { type: 'generateSplitMidi' }>,
  generateSplitMidis:
    | ((source: string, baseName: string) => GenerateSplitMidisResponse)
    | null,
): void {
  if (!generateSplitMidis) {
    postMessage({
      type: 'splitMidiErr',
      id: msg.id,
      diagnostics: [
        {
          severity: 'error',
          message: 'Split MIDI export is not available in this build.',
          span: { start: 0, end: 0 },
        },
      ],
    } satisfies WorkerResponse)
    return
  }
  const result = generateSplitMidis(msg.source, msg.baseName)
  if (result.status === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.zip)
    postMessage(
      {
        type: 'splitMidi',
        id: msg.id,
        zip: zipBuffer,
      } satisfies WorkerResponse,
      { transfer: [zipBuffer] },
    )
    return
  }

  postMessage({
    type: 'splitMidiErr',
    id: msg.id,
    diagnostics: result.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateSplitWav(
  msg: Extract<WorkerRequest, { type: 'generateSplitWav' }>,
  generateSplitWavs:
    | ((
        source: string,
        baseName: string,
        soundfont: Uint8Array,
      ) => GenerateSplitWavsResponse)
    | null,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!generateSplitWavs) {
    postMessage({
      type: 'splitWavErr',
      id: msg.id,
      diagnostics: [
        {
          severity: 'error',
          message: 'Split WAV export is not available in this build.',
          span: { start: 0, end: 0 },
        },
      ],
    } satisfies WorkerResponse)
    return
  }
  if (!loadedSoundfont) {
    postMessage({
      type: 'splitWavErr',
      id: msg.id,
      diagnostics: [
        {
          severity: 'error',
          message: 'Soundfont is not yet loaded.',
          span: { start: 0, end: 0 },
        },
      ],
    } satisfies WorkerResponse)
    return
  }
  const result = generateSplitWavs(msg.source, msg.baseName, loadedSoundfont)
  if (result.status === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.zip)
    postMessage(
      {
        type: 'splitWav',
        id: msg.id,
        zip: zipBuffer,
      } satisfies WorkerResponse,
      { transfer: [zipBuffer] },
    )
    return
  }

  postMessage({
    type: 'splitWavErr',
    id: msg.id,
    diagnostics: result.diagnostics,
  } satisfies WorkerResponse)
}
