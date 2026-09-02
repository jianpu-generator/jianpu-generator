import * as jianpuWasm from 'jianpu-wasm'

// Older cached wasm builds (e.g. a stale service-worker asset) may not yet
// export newer functions, so every export the worker depends on is looked up
// defensively via `in` rather than imported directly — a missing export
// degrades the corresponding feature to unavailable instead of crashing the
// worker's module load. Grouped here, out of `jianpu.worker.ts`, to keep that
// file under its line-count cap.

export const generateWav =
  'generate_wav' in jianpuWasm ? jianpuWasm.generate_wav : null

export const generateWavForMeasureRange =
  'generate_wav_for_measure_range' in jianpuWasm
    ? jianpuWasm.generate_wav_for_measure_range
    : null

export const listNoteTimings =
  'list_note_timings' in jianpuWasm ? jianpuWasm.list_note_timings : null

export const listNoteTimingsForRange =
  'list_note_timings_for_range' in jianpuWasm
    ? jianpuWasm.list_note_timings_for_range
    : null

export const renderWithHighlightRange =
  'render_with_highlight_range' in jianpuWasm
    ? jianpuWasm.render_with_highlight_range
    : null

export const generatePdf =
  'generate_pdf' in jianpuWasm ? jianpuWasm.generate_pdf : null

export const generateSplitPdfs =
  'generate_split_pdfs' in jianpuWasm ? jianpuWasm.generate_split_pdfs : null

export const generateMidi =
  'generate_midi' in jianpuWasm ? jianpuWasm.generate_midi : null

export const generateSplitMidis =
  'generate_split_midis' in jianpuWasm ? jianpuWasm.generate_split_midis : null

export const generateSplitWavs =
  'generate_split_wavs' in jianpuWasm ? jianpuWasm.generate_split_wavs : null

export const generateMp3 =
  'generate_mp3' in jianpuWasm ? jianpuWasm.generate_mp3 : null

export const generateSplitMp3s =
  'generate_split_mp3s' in jianpuWasm ? jianpuWasm.generate_split_mp3s : null

export const generateInstrumentPreviewWav =
  'generate_instrument_preview_wav' in jianpuWasm
    ? jianpuWasm.generate_instrument_preview_wav
    : null

export const generatePercussionPreviewWav =
  'generate_percussion_preview_wav' in jianpuWasm
    ? jianpuWasm.generate_percussion_preview_wav
    : null
