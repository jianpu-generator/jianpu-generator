import init, * as jianpuWasm from 'jianpu-wasm'
import { list_parts, render } from 'jianpu-wasm'
import type {
  Diagnostic,
  GenerateWavResult,
  ListPartsResult,
  PartInfo,
  RenderResult,
} from '../types'

const generateWav =
  'generate_wav' in jianpuWasm
    ? (jianpuWasm.generate_wav as (
        source: string,
        enabledTracks?: string[],
      ) => GenerateWavResult)
    : null

export type WorkerRequest =
  | { type: 'render'; source: string; id: number; enabledTracks?: string[] }
  | { type: 'listParts'; source: string; id: number }

export type WorkerResponse =
  | { type: 'ready'; audioAvailable: boolean }
  | { type: 'ok'; id: number; svgs: string[]; wav?: ArrayBuffer }
  | { type: 'err'; id: number; diagnostics: Diagnostic[] }
  | { type: 'parts'; id: number; parts: PartInfo[] }

let initialized = false

async function ensureInit() {
  if (!initialized) {
    await init()
    initialized = true
    postMessage({
      type: 'ready',
      audioAvailable: generateWav !== null,
    } satisfies WorkerResponse)
  }
}

function wavBufferFromResult(wav: Uint8Array | number[]): ArrayBuffer {
  const bytes = wav instanceof Uint8Array ? wav : new Uint8Array(wav)
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
    const result = list_parts(msg.source) as ListPartsResult
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

  if (msg.type !== 'render') return

  const result = render(msg.source, msg.enabledTracks) as RenderResult
  if (result.status === 'ok') {
    let wavBuffer: ArrayBuffer | undefined
    if (generateWav) {
      const wavResult = generateWav(msg.source, msg.enabledTracks)
      if (wavResult.status === 'ok') {
        wavBuffer = wavBufferFromResult(wavResult.wav)
      }
    }

    if (wavBuffer) {
      postMessage(
        {
          type: 'ok',
          id: msg.id,
          svgs: result.svgs,
          wav: wavBuffer,
        } satisfies WorkerResponse,
        { transfer: [wavBuffer] },
      )
      return
    }

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
