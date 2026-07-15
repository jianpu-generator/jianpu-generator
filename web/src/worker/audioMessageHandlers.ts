import type {
  GenerateWavResponse,
  ListMeasureTimesResponse,
  WrittenMeasureIndicesResponse,
} from 'jianpu-wasm'
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

function measureTimesFromSource(
  listMeasureTimes:
    | ((source: string, enabledTracks?: string[]) => ListMeasureTimesResponse)
    | null,
  source: string,
  enabledTracks: string[] | undefined,
): number[] {
  if (!listMeasureTimes) return []
  const result = listMeasureTimes(source, enabledTracks)
  return result.status === 'ok' ? result.times : []
}

function measureTimesForRangeFromSource(
  listMeasureTimesForRange:
    | ((
        source: string,
        startIndex: number,
        endIndex: number,
        extendToLastOccurrence: boolean,
        enabledTracks?: string[],
      ) => ListMeasureTimesResponse)
    | null,
  source: string,
  startMeasureIndex: number,
  endMeasureIndex: number,
  extendToLastOccurrence: boolean,
  enabledTracks: string[] | undefined,
): number[] {
  if (!listMeasureTimesForRange) return []
  const result = listMeasureTimesForRange(
    source,
    startMeasureIndex,
    endMeasureIndex,
    extendToLastOccurrence,
    enabledTracks,
  )
  return result.status === 'ok' ? result.times : []
}

function writtenMeasureIndicesFromSource(
  writtenMeasureIndices:
    | ((
        source: string,
        enabledTracks?: string[],
      ) => WrittenMeasureIndicesResponse)
    | null,
  source: string,
  enabledTracks: string[] | undefined,
): number[] {
  if (!writtenMeasureIndices) return []
  const result = writtenMeasureIndices(source, enabledTracks)
  return result.status === 'ok' ? result.indices : []
}

function writtenMeasureIndicesForRangeFromSource(
  writtenMeasureIndicesForRange:
    | ((
        source: string,
        startIndex: number,
        endIndex: number,
        extendToLastOccurrence: boolean,
        enabledTracks?: string[],
      ) => WrittenMeasureIndicesResponse)
    | null,
  source: string,
  startMeasureIndex: number,
  endMeasureIndex: number,
  extendToLastOccurrence: boolean,
  enabledTracks: string[] | undefined,
): number[] {
  if (!writtenMeasureIndicesForRange) return []
  const result = writtenMeasureIndicesForRange(
    source,
    startMeasureIndex,
    endMeasureIndex,
    extendToLastOccurrence,
    enabledTracks,
  )
  return result.status === 'ok' ? result.indices : []
}

type GenerateWavFn =
  | ((
      source: string,
      enabledTracks: string[] | undefined,
      soundfont: Uint8Array,
    ) => GenerateWavResponse)
  | null

type ListMeasureTimesFn =
  | ((source: string, enabledTracks?: string[]) => ListMeasureTimesResponse)
  | null

type WrittenMeasureIndicesFn =
  | ((
      source: string,
      enabledTracks?: string[],
    ) => WrittenMeasureIndicesResponse)
  | null

export function handleGenerateAudio(
  msg: Extract<WorkerRequest, { type: 'generateAudio' }>,
  generateWav: GenerateWavFn,
  listMeasureTimes: ListMeasureTimesFn,
  writtenMeasureIndices: WrittenMeasureIndicesFn,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!generateWav || !loadedSoundfont) {
    postMessage({ type: 'audioErr', id: msg.id } satisfies WorkerResponse)
    return
  }
  const wavResult = generateWav(msg.source, msg.enabledTracks, loadedSoundfont)
  if (wavResult.status === 'ok' && wavResult.wav != null) {
    const wavBuffer = binaryBufferFromResult(wavResult.wav)
    postMessage(
      {
        type: 'audio',
        id: msg.id,
        wav: wavBuffer,
        measureTimes: measureTimesFromSource(
          listMeasureTimes,
          msg.source,
          msg.enabledTracks,
        ),
        writtenMeasureIndices: writtenMeasureIndicesFromSource(
          writtenMeasureIndices,
          msg.source,
          msg.enabledTracks,
        ),
      } satisfies WorkerResponse,
      { transfer: [wavBuffer] },
    )
    return
  }
  postMessage({ type: 'audioErr', id: msg.id } satisfies WorkerResponse)
}

type GenerateWavForMeasureRangeFn =
  | ((
      source: string,
      startIndex: number,
      endIndex: number,
      extendToLastOccurrence: boolean,
      enabledTracks: string[] | undefined,
      soundfont: Uint8Array,
    ) => GenerateWavResponse)
  | null

type ListMeasureTimesForRangeFn =
  | ((
      source: string,
      startIndex: number,
      endIndex: number,
      extendToLastOccurrence: boolean,
      enabledTracks?: string[],
    ) => ListMeasureTimesResponse)
  | null

type WrittenMeasureIndicesForRangeFn =
  | ((
      source: string,
      startIndex: number,
      endIndex: number,
      extendToLastOccurrence: boolean,
      enabledTracks?: string[],
    ) => WrittenMeasureIndicesResponse)
  | null

export function handleGenerateMeasureRangeAudio(
  msg: Extract<WorkerRequest, { type: 'generateMeasureRangeAudio' }>,
  generateWavForMeasureRange: GenerateWavForMeasureRangeFn,
  listMeasureTimesForRange: ListMeasureTimesForRangeFn,
  writtenMeasureIndicesForRange: WrittenMeasureIndicesForRangeFn,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!generateWavForMeasureRange || !loadedSoundfont) {
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
    msg.extendToLastOccurrence,
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
          listMeasureTimesForRange,
          msg.source,
          msg.startMeasureIndex,
          msg.endMeasureIndex,
          msg.extendToLastOccurrence,
          msg.enabledTracks,
        ),
        writtenMeasureIndices: writtenMeasureIndicesForRangeFromSource(
          writtenMeasureIndicesForRange,
          msg.source,
          msg.startMeasureIndex,
          msg.endMeasureIndex,
          msg.extendToLastOccurrence,
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
}
