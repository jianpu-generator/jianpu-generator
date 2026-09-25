import { type Diagnostic, jianpuWasm } from '../jianpuWasm'
import type { WorkerRequest, WorkerResponse } from './jianpu.worker'

// `sc` holds the `title` role's font — the song title/lyric font; `tc`
// holds the `sansSerif` role's font, the default/body font for everything
// else — see `fonts/fonts.json` and `useFontsLoader`.
type LoadedFonts = {
  sc: Uint8Array
  tc: Uint8Array
  mono: Uint8Array
} | null

export function binaryBufferFromResult(bytes: Uint8Array): ArrayBuffer {
  return bytes.slice().buffer
}

export function singleErrorDiagnostic(message: string): Diagnostic[] {
  return [{ severity: 'error', message, span: { start: 0, end: 0 } }]
}

export function handleGeneratePdf(
  msg: Extract<WorkerRequest, { type: 'generatePdf' }>,
  loadedFonts: LoadedFonts,
): void {
  if (!loadedFonts) {
    postMessage({
      type: 'pdfErr',
      id: msg.id,
      diagnostics: singleErrorDiagnostic('Fonts are not yet loaded.'),
    } satisfies WorkerResponse)
    return
  }
  const result = jianpuWasm().generatePdf(
    msg.source,
    msg.enabledTracks,
    msg.disabledLyrics,
    loadedFonts.sc,
    loadedFonts.tc,
    loadedFonts.mono,
  )
  if (result.tag === 'ok') {
    const pdfBuffer = binaryBufferFromResult(result.val.pdf)
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
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateSplitPdf(
  msg: Extract<WorkerRequest, { type: 'generateSplitPdf' }>,
  loadedFonts: LoadedFonts,
): void {
  if (!loadedFonts) {
    postMessage({
      type: 'splitPdfErr',
      id: msg.id,
      diagnostics: singleErrorDiagnostic('Fonts are not yet loaded.'),
    } satisfies WorkerResponse)
    return
  }
  const result = jianpuWasm().generateSplitPdfs(
    msg.source,
    msg.baseName,
    loadedFonts.sc,
    loadedFonts.tc,
    loadedFonts.mono,
  )
  if (result.tag === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.val.zip)
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
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateMidi(
  msg: Extract<WorkerRequest, { type: 'generateMidi' }>,
): void {
  const result = jianpuWasm().generateMidi(msg.source, msg.enabledTracks)
  if (result.tag === 'ok') {
    const midiBuffer = binaryBufferFromResult(result.val.midi)
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
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateSplitMidi(
  msg: Extract<WorkerRequest, { type: 'generateSplitMidi' }>,
): void {
  const result = jianpuWasm().generateSplitMidis(msg.source, msg.baseName)
  if (result.tag === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.val.zip)
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
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateSplitWav(
  msg: Extract<WorkerRequest, { type: 'generateSplitWav' }>,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!loadedSoundfont) {
    postMessage({
      type: 'splitWavErr',
      id: msg.id,
      diagnostics: singleErrorDiagnostic('Soundfont is not yet loaded.'),
    } satisfies WorkerResponse)
    return
  }
  const result = jianpuWasm().generateSplitWavs(
    msg.source,
    msg.baseName,
    loadedSoundfont,
  )
  if (result.tag === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.val.zip)
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
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateSplitMp3(
  msg: Extract<WorkerRequest, { type: 'generateSplitMp3' }>,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!loadedSoundfont) {
    postMessage({
      type: 'splitMp3Err',
      id: msg.id,
      diagnostics: singleErrorDiagnostic('Soundfont is not yet loaded.'),
    } satisfies WorkerResponse)
    return
  }
  const result = jianpuWasm().generateSplitMp3s(
    msg.source,
    msg.baseName,
    loadedSoundfont,
  )
  if (result.tag === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.val.zip)
    postMessage(
      {
        type: 'splitMp3',
        id: msg.id,
        zip: zipBuffer,
      } satisfies WorkerResponse,
      { transfer: [zipBuffer] },
    )
    return
  }

  postMessage({
    type: 'splitMp3Err',
    id: msg.id,
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}
