import type {
  GenerateWavResponse,
  ListPartsResponse,
  NoteTimingOut,
  NoteTimingsResponse,
  PartOut,
} from 'jianpu-wasm'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import { computeNoteSelectionTrimWindow } from '../utils/noteSelectionTrim'
import { remapNoteTimingsToVisiblePartIndex } from '../utils/noteTimingsPartIndex'
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

type ListPartsFn =
  | ((source: string, instruments: typeof GM_INSTRUMENTS) => ListPartsResponse)
  | null

/** Declaration-order parts, used only to rebuild the hidden-parts-compacted
 * index space `remapNoteTimingsToVisiblePartIndex` needs — see its doc
 * comment. */
function visiblePartsFromSource(
  listParts: ListPartsFn,
  source: string,
): PartOut[] {
  if (!listParts) return []
  const result = listParts(source, GM_INSTRUMENTS)
  return result.status === 'ok' ? result.parts : []
}

function noteTimingsFromSource(
  listNoteTimings:
    | ((source: string, enabledTracks?: string[]) => NoteTimingsResponse)
    | null,
  source: string,
  enabledTracks: string[] | undefined,
): NoteTimingOut[] {
  if (!listNoteTimings) return []
  const result = listNoteTimings(source, enabledTracks)
  return result.status === 'ok' ? result.timings : []
}

function noteTimingsForRangeFromSource(
  listNoteTimingsForRange:
    | ((
        source: string,
        startIndex: number,
        endIndex: number,
        extendToLastOccurrence: boolean,
        respectSequence: boolean,
        sequenceEntryStartIndex: number | undefined,
        sequenceEntryEndIndex: number | undefined,
        enabledTracks?: string[],
      ) => NoteTimingsResponse)
    | null,
  source: string,
  startMeasureIndex: number,
  endMeasureIndex: number,
  extendToLastOccurrence: boolean,
  respectSequence: boolean,
  sequenceEntryStartIndex: number | undefined,
  sequenceEntryEndIndex: number | undefined,
  enabledTracks: string[] | undefined,
): NoteTimingOut[] {
  if (!listNoteTimingsForRange) return []
  const result = listNoteTimingsForRange(
    source,
    startMeasureIndex,
    endMeasureIndex,
    extendToLastOccurrence,
    respectSequence,
    sequenceEntryStartIndex,
    sequenceEntryEndIndex,
    enabledTracks,
  )
  return result.status === 'ok' ? result.timings : []
}

type GenerateWavFn =
  | ((
      source: string,
      enabledTracks: string[] | undefined,
      soundfont: Uint8Array,
    ) => GenerateWavResponse)
  | null

type ListNoteTimingsFn =
  | ((source: string, enabledTracks?: string[]) => NoteTimingsResponse)
  | null

export function handleGenerateAudio(
  msg: Extract<WorkerRequest, { type: 'generateAudio' }>,
  generateWav: GenerateWavFn,
  listNoteTimings: ListNoteTimingsFn,
  listParts: ListPartsFn,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!generateWav || !loadedSoundfont) {
    postMessage({ type: 'audioErr', id: msg.id } satisfies WorkerResponse)
    return
  }
  const wavResult = generateWav(msg.source, msg.enabledTracks, loadedSoundfont)
  if (wavResult.status === 'ok' && wavResult.wav != null) {
    const wavBuffer = binaryBufferFromResult(wavResult.wav)
    // `enabledTracks` here is always the part-visibility toggle's state
    // (never a playback-only mute override — this handler has no such
    // concept), so it doubles as the compaction basis.
    const noteTimings = remapNoteTimingsToVisiblePartIndex(
      noteTimingsFromSource(listNoteTimings, msg.source, msg.enabledTracks),
      visiblePartsFromSource(listParts, msg.source),
      msg.enabledTracks,
    )
    postMessage(
      {
        type: 'audio',
        id: msg.id,
        wav: wavBuffer,
        noteTimings,
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
      respectSequence: boolean,
      sequenceEntryStartIndex: number | undefined,
      sequenceEntryEndIndex: number | undefined,
      enabledTracks: string[] | undefined,
      trimStartS: number | undefined,
      trimEndS: number | undefined,
      trimNextNoteStartS: number | undefined,
      soundfont: Uint8Array,
    ) => GenerateWavResponse)
  | null

type ListNoteTimingsForRangeFn =
  | ((
      source: string,
      startIndex: number,
      endIndex: number,
      extendToLastOccurrence: boolean,
      respectSequence: boolean,
      sequenceEntryStartIndex: number | undefined,
      sequenceEntryEndIndex: number | undefined,
      enabledTracks?: string[],
    ) => NoteTimingsResponse)
  | null

/** Shifts every timing's `start_s`/`end_s` back by `trimStartS`, so they
 * stay relative to the start of a clip that Rust has sample-accurately
 * trimmed down to `[trimStartS, trimEndS]` (see `crate::wav::TrimWindow`)
 * instead of the full, untrimmed measure-range clip they were originally
 * computed against. */
function shiftNoteTimings(
  timings: NoteTimingOut[],
  trimStartS: number,
): NoteTimingOut[] {
  return timings.map((t) => ({
    ...t,
    start_s: t.start_s - trimStartS,
    end_s: t.end_s - trimStartS,
  }))
}

export function handleGenerateMeasureRangeAudio(
  msg: Extract<WorkerRequest, { type: 'generateMeasureRangeAudio' }>,
  generateWavForMeasureRange: GenerateWavForMeasureRangeFn,
  listNoteTimingsForRange: ListNoteTimingsForRangeFn,
  listParts: ListPartsFn,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!generateWavForMeasureRange || !loadedSoundfont) {
    postMessage({
      type: 'measureRangeAudioErr',
      id: msg.id,
    } satisfies WorkerResponse)
    return
  }
  // Rebuild the hidden-parts-compacted index space (matching the currently
  // rendered SVG's `data-part-index`) before this clip's own, possibly
  // narrower, playback mute (`msg.enabledTracks` — e.g. "play selection")
  // gets applied — see `remapNoteTimingsToVisiblePartIndex`'s doc comment.
  const fullRangeNoteTimings = remapNoteTimingsToVisiblePartIndex(
    noteTimingsForRangeFromSource(
      listNoteTimingsForRange,
      msg.source,
      msg.startMeasureIndex,
      msg.endMeasureIndex,
      msg.extendToLastOccurrence,
      msg.respectSequence,
      msg.sequenceEntryStartIndex,
      msg.sequenceEntryEndIndex,
      msg.enabledTracks,
    ),
    visiblePartsFromSource(listParts, msg.source),
    msg.visibleTracks,
  )
  // "Play selection": narrow the clip Rust synthesizes down to exactly the
  // drag-selected notes' elapsed-seconds span (sample-accurate trim/fade —
  // see `crate::wav::TrimWindow`), derived from the full range's note
  // timings fetched above. `undefined` for a plain measure-range play
  // (every other caller), which always plays the range in full.
  const trim = msg.trimToSelectedNoteCells
    ? computeNoteSelectionTrimWindow(
        msg.trimToSelectedNoteCells,
        fullRangeNoteTimings,
      )
    : null
  const wavResult = generateWavForMeasureRange(
    msg.source,
    msg.startMeasureIndex,
    msg.endMeasureIndex,
    msg.extendToLastOccurrence,
    msg.respectSequence,
    msg.sequenceEntryStartIndex,
    msg.sequenceEntryEndIndex,
    msg.enabledTracks,
    trim?.start,
    trim?.end,
    trim?.nextNoteStart,
    loadedSoundfont,
  )
  if (wavResult.status === 'ok' && wavResult.wav != null) {
    const wavBuffer = binaryBufferFromResult(wavResult.wav)
    postMessage(
      {
        type: 'measureRangeAudio',
        id: msg.id,
        wav: wavBuffer,
        noteTimings: trim
          ? shiftNoteTimings(fullRangeNoteTimings, trim.start)
          : fullRangeNoteTimings,
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
