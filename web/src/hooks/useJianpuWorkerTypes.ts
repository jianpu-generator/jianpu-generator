import type { NoteTimingOut, SvgDocumentOut } from 'jianpu-wasm'
import type {
  Diagnostic,
  DiagnosticViewZone,
  MeasureSpan,
  PartDeclaration,
  PartInfo,
  PartMode,
  SectionRange,
  SequenceEntry,
} from '../types'

export interface JianpuWorkerState {
  parts: PartInfo[]
  partDeclarations: PartDeclaration[]
  partsLoading: boolean
  documents: SvgDocumentOut[]
  wavUrl: string | null
  wavFilename: string
  /** Elapsed-seconds offset of each measure boundary for `wavUrl`'s audio, length = measure count + 1. */
  measureTimes: number[]
  /** Written measure index to highlight at each playback position of `measureTimes`, following D.C. al Coda navigation; entry `i` pairs with `measureTimes[i]`. */
  writtenMeasureIndices: number[]
  /** Cumulative pixel-weight column boundaries of every rendered measure, entry `i` pairs with `data-measure-index="i"`. Used to map a linear time position within a measure onto its actual (density-weighted) pixel position. */
  columnBoundaries: number[][]
  /** Elapsed-seconds start/end of every sounding note/rest for `wavUrl`'s audio, keyed by `(source_part_index, note_id)`. Drives the per-part, per-note playback highlight. */
  noteTimings: NoteTimingOut[]
  audioAvailable: boolean
  pdfAvailable: boolean
  pdfExporting: boolean
  splitPdfExporting: boolean
  midiAvailable: boolean
  midiExporting: boolean
  splitMidiExporting: boolean
  splitWavExporting: boolean
  diagnostics: Diagnostic[]
  diagnosticViewZones: DiagnosticViewZone[]
  rendering: boolean
  audioGenerating: boolean
  exportPdf: () => void
  exportSplitPdf: () => void
  exportMidi: () => void
  exportSplitMidi: () => void
  exportSplitWav: () => void
  generateFullAudio: () => void
  selectedMeasureRange: { start: number; end: number } | null
  measureAudioGenerating: boolean
  measureAudioPlaying: boolean
  /** Elapsed-seconds offset of each measure boundary within the selected range's audio, relative to the range start. */
  measureAudioTimes: number[]
  /** Written measure index to highlight at each playback position of `measureAudioTimes`, following D.C. al Coda navigation; entry `i` pairs with `measureAudioTimes[i]`. */
  measureAudioWrittenIndices: number[]
  /** Elapsed-seconds start/end of every sounding note/rest for the selected range's audio, keyed by `(source_part_index, note_id)`. */
  measureAudioNoteTimings: NoteTimingOut[]
  /** The `<audio>` element currently playing the selected measure range, if any; a new element each time playback starts. */
  measureAudioElement: HTMLAudioElement | null
  notifySelection: (startLine: number, endLine: number) => void
  playSelectedMeasures: () => void
  playFromCurrentMeasure: () => void
  stopMeasurePlayback: () => void
  highlightedDocuments: SvgDocumentOut[]
  measureSpans: MeasureSpan[]
  sectionRanges: SectionRange[]
  sequenceEntries: SequenceEntry[]
  selectedSequenceRange: { start: number; end: number } | null
  sequenceJumpToolbarProps: {
    sequenceEntries: SequenceEntry[]
    dragStartIndex: number | null
    setDragStartIndex: (index: number | null) => void
    setDragCurrentIndex: (index: number | null) => void
    activeHighlightedIndices: Set<number>
    handleSequenceEntryClick: (index: number) => void
    handleSequenceEntryRangeSelect: (indexA: number, indexB: number) => void
  }
  previewInstrument: (programNumber: number) => void
  previewPercussion: (key: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
  updatePartDeclaration: (
    abbreviation: string,
    mode: PartMode,
    followTarget: string | null,
    soundfont: string | null,
    volume: number | null,
    octaveOffset: number | null,
  ) => Promise<string>
  /**
   * Recovers the `.jianpu` source embedded in a previously exported SVG/PDF
   * file (see `source_embed::extract_embedded_source`). Rejects if the file
   * has no embedded source.
   */
  importFromFile: (file: File) => Promise<string>
}
