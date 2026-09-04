// Public API: generate-audio/pdf/midi, note-timing, share-payload, and
// source-embed functions, matching the old (`tsify`-generated)
// `pkg/jianpu_wasm.d.ts` names/shapes exactly — split out of
// `jianpuWasm.ts` purely to stay under the 400-line-per-file cap. See
// `jianpuWasmApiCore.ts` for the render/list/group/symbol cluster.

import type {
  GenerateMidiResponse as WitGenerateMidiResponse,
  GenerateMp3Response as WitGenerateMp3Response,
  GeneratePdfResponse as WitGeneratePdfResponse,
  GenerateSplitMidisResponse as WitGenerateSplitMidisResponse,
  GenerateSplitMp3sResponse as WitGenerateSplitMp3sResponse,
  GenerateSplitPdfsResponse as WitGenerateSplitPdfsResponse,
  GenerateSplitWavsResponse as WitGenerateSplitWavsResponse,
  GenerateWavResponse as WitGenerateWavResponse,
  NoteTimingsResponse as WitNoteTimingsResponse,
} from '../../crates/jianpu-wasm/pkg-component/jianpu_wasm.js'
import { convertNoteTiming, diagnosticsErrOk, opt } from './jianpuWasmConvert'
import { root } from './jianpuWasmRoot'
import type {
  GenerateMidiResponse,
  GenerateMp3Response,
  GeneratePdfResponse,
  GenerateSplitMidisResponse,
  GenerateSplitMp3sResponse,
  GenerateSplitPdfsResponse,
  GenerateSplitWavsResponse,
  GenerateWavResponse,
  NoteTimingsResponse,
} from './jianpuWasmTypes'

export function generate_wav(
  source: string,
  enabled_tracks: string[] | null | undefined,
  soundfont: Uint8Array,
): GenerateWavResponse {
  const resp: WitGenerateWavResponse = root().generateWav(
    source,
    opt(enabled_tracks),
    soundfont,
  )
  return diagnosticsErrOk(resp, (v) => ({ wav: v.wav }))
}

export function generate_wav_for_measure_range(
  source: string,
  start_index: number,
  end_index: number,
  extend_to_last_occurrence: boolean,
  respect_sequence: boolean,
  sequence_entry_start_index: number | null | undefined,
  sequence_entry_end_index: number | null | undefined,
  enabled_tracks: string[] | null | undefined,
  trim_start_s: number | null | undefined,
  trim_end_s: number | null | undefined,
  trim_next_note_start_s: number | null | undefined,
  soundfont: Uint8Array,
): GenerateWavResponse {
  const resp: WitGenerateWavResponse = root().generateWavForMeasureRange(
    source,
    start_index,
    end_index,
    extend_to_last_occurrence,
    respect_sequence,
    opt(sequence_entry_start_index),
    opt(sequence_entry_end_index),
    opt(enabled_tracks),
    opt(trim_start_s),
    opt(trim_end_s),
    opt(trim_next_note_start_s),
    soundfont,
  )
  return diagnosticsErrOk(resp, (v) => ({ wav: v.wav }))
}

export function generate_instrument_preview_wav(
  program_number: number,
  soundfont: Uint8Array,
): GenerateWavResponse {
  const resp: WitGenerateWavResponse = root().generateInstrumentPreviewWav(
    program_number,
    soundfont,
  )
  return diagnosticsErrOk(resp, (v) => ({ wav: v.wav }))
}

export function generate_percussion_preview_wav(
  key: number,
  soundfont: Uint8Array,
): GenerateWavResponse {
  const resp: WitGenerateWavResponse = root().generatePercussionPreviewWav(
    key,
    soundfont,
  )
  return diagnosticsErrOk(resp, (v) => ({ wav: v.wav }))
}

export function list_note_timings(
  source: string,
  visible_tracks?: string[] | null,
  enabled_tracks?: string[] | null,
): NoteTimingsResponse {
  const resp: WitNoteTimingsResponse = root().listNoteTimings(
    source,
    opt(visible_tracks),
    opt(enabled_tracks),
  )
  return diagnosticsErrOk(resp, (v) => ({
    timings: v.timings.map(convertNoteTiming),
  }))
}

export function list_note_timings_for_range(
  source: string,
  start_index: number,
  end_index: number,
  extend_to_last_occurrence: boolean,
  respect_sequence: boolean,
  sequence_entry_start_index?: number | null,
  sequence_entry_end_index?: number | null,
  visible_tracks?: string[] | null,
  enabled_tracks?: string[] | null,
): NoteTimingsResponse {
  const resp: WitNoteTimingsResponse = root().listNoteTimingsForRange(
    source,
    start_index,
    end_index,
    extend_to_last_occurrence,
    respect_sequence,
    opt(sequence_entry_start_index),
    opt(sequence_entry_end_index),
    opt(visible_tracks),
    opt(enabled_tracks),
  )
  return diagnosticsErrOk(resp, (v) => ({
    timings: v.timings.map(convertNoteTiming),
  }))
}

export function generate_pdf(
  source: string,
  enabled_tracks: string[] | null | undefined,
  disabled_lyrics: string[] | null | undefined,
  sans_serif_sc: Uint8Array,
  sans_serif_tc: Uint8Array,
  monospace: Uint8Array,
): GeneratePdfResponse {
  const resp: WitGeneratePdfResponse = root().generatePdf(
    source,
    opt(enabled_tracks),
    opt(disabled_lyrics),
    sans_serif_sc,
    sans_serif_tc,
    monospace,
  )
  return diagnosticsErrOk(resp, (v) => ({ pdf: v.pdf }))
}

export function generate_split_pdfs(
  source: string,
  base_name: string,
  sans_serif_sc: Uint8Array,
  sans_serif_tc: Uint8Array,
  monospace: Uint8Array,
): GenerateSplitPdfsResponse {
  const resp: WitGenerateSplitPdfsResponse = root().generateSplitPdfs(
    source,
    base_name,
    sans_serif_sc,
    sans_serif_tc,
    monospace,
  )
  return diagnosticsErrOk(resp, (v) => ({ zip: v.zip }))
}

export function generate_midi(
  source: string,
  enabled_tracks?: string[] | null,
): GenerateMidiResponse {
  const resp: WitGenerateMidiResponse = root().generateMidi(
    source,
    opt(enabled_tracks),
  )
  return diagnosticsErrOk(resp, (v) => ({ midi: v.midi }))
}

export function generate_split_midis(
  source: string,
  base_name: string,
): GenerateSplitMidisResponse {
  const resp: WitGenerateSplitMidisResponse = root().generateSplitMidis(
    source,
    base_name,
  )
  return diagnosticsErrOk(resp, (v) => ({ zip: v.zip }))
}

export function generate_split_wavs(
  source: string,
  base_name: string,
  soundfont: Uint8Array,
): GenerateSplitWavsResponse {
  const resp: WitGenerateSplitWavsResponse = root().generateSplitWavs(
    source,
    base_name,
    soundfont,
  )
  return diagnosticsErrOk(resp, (v) => ({ zip: v.zip }))
}

export function generate_mp3(
  source: string,
  enabled_tracks: string[] | null | undefined,
  soundfont: Uint8Array,
): GenerateMp3Response {
  const resp: WitGenerateMp3Response = root().generateMp3(
    source,
    opt(enabled_tracks),
    soundfont,
  )
  return diagnosticsErrOk(resp, (v) => ({ mp3: v.mp3 }))
}

export function generate_mp3_for_measure_range(
  source: string,
  start_index: number,
  end_index: number,
  extend_to_last_occurrence: boolean,
  respect_sequence: boolean,
  sequence_entry_start_index: number | null | undefined,
  sequence_entry_end_index: number | null | undefined,
  enabled_tracks: string[] | null | undefined,
  trim_start_s: number | null | undefined,
  trim_end_s: number | null | undefined,
  trim_next_note_start_s: number | null | undefined,
  soundfont: Uint8Array,
): GenerateMp3Response {
  const resp: WitGenerateMp3Response = root().generateMp3ForMeasureRange(
    source,
    start_index,
    end_index,
    extend_to_last_occurrence,
    respect_sequence,
    opt(sequence_entry_start_index),
    opt(sequence_entry_end_index),
    opt(enabled_tracks),
    opt(trim_start_s),
    opt(trim_end_s),
    opt(trim_next_note_start_s),
    soundfont,
  )
  return diagnosticsErrOk(resp, (v) => ({ mp3: v.mp3 }))
}

export function generate_split_mp3s(
  source: string,
  base_name: string,
  soundfont: Uint8Array,
): GenerateSplitMp3sResponse {
  const resp: WitGenerateSplitMp3sResponse = root().generateSplitMp3s(
    source,
    base_name,
    soundfont,
  )
  return diagnosticsErrOk(resp, (v) => ({ zip: v.zip }))
}

export function extract_source_from_svg(
  svg_bytes: Uint8Array,
): string | undefined {
  return root().extractSourceFromSvg(svg_bytes)
}

export function extract_source_from_pdf(
  pdf_bytes: Uint8Array,
): string | undefined {
  return root().extractSourceFromPdf(pdf_bytes)
}

export function compress_share_payload(payload: string): Uint8Array {
  return root().compressSharePayload(payload)
}

export function decompress_share_payload(
  bytes: Uint8Array,
): string | undefined {
  return root().decompressSharePayload(bytes)
}
