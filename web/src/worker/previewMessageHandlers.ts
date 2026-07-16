import type { GenerateWavResponse } from 'jianpu-wasm'
import type { WorkerRequest, WorkerResponse } from './jianpu.worker'

function binaryBufferFromResult(
  bytes: Uint8Array | ArrayBuffer | ArrayLike<number>,
): ArrayBuffer {
  if (bytes instanceof ArrayBuffer) {
    return bytes.slice(0)
  }
  const view = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes)
  return view.slice().buffer
}

export function handlePreviewInstrument(
  msg: Extract<WorkerRequest, { type: 'previewInstrument' }>,
  generateInstrumentPreviewWav:
    | ((programNumber: number, soundfont: Uint8Array) => GenerateWavResponse)
    | null,
  loadedSoundfont: Uint8Array | null,
): void {
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
}

export function handlePreviewPercussion(
  msg: Extract<WorkerRequest, { type: 'previewPercussion' }>,
  generatePercussionPreviewWav:
    | ((key: number, soundfont: Uint8Array) => GenerateWavResponse)
    | null,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!generatePercussionPreviewWav || !loadedSoundfont) {
    postMessage({
      type: 'percussionPreviewErr',
      id: msg.id,
    } satisfies WorkerResponse)
    return
  }
  const result = generatePercussionPreviewWav(msg.key, loadedSoundfont)
  if (result.status === 'ok' && result.wav != null) {
    const wavBuffer = binaryBufferFromResult(result.wav)
    postMessage(
      {
        type: 'percussionPreview',
        id: msg.id,
        wav: wavBuffer,
      } satisfies WorkerResponse,
      { transfer: [wavBuffer] },
    )
    return
  }
  postMessage({
    type: 'percussionPreviewErr',
    id: msg.id,
  } satisfies WorkerResponse)
}
