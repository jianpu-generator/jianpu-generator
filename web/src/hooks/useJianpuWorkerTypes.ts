import type {
  LyricsVerseRangesOut,
  NoteTimingOut,
  PartMeasureRangesOut,
  SvgDocumentOut,
} from 'jianpu-wasm'
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
  /** Elapsed-seconds start/end of every sounding note/rest for `wavUrl`'s audio, keyed by `(source_part_index, note_id)`. Drives the per-part, per-note playback cursor. */
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
  /** Elapsed-seconds start/end of every sounding note/rest for the selected range's audio, keyed by `(source_part_index, note_id)`. */
  measureAudioNoteTimings: NoteTimingOut[]
  /** The `<audio>` element currently playing the selected measure range, if any; a new element each time playback starts. */
  measureAudioElement: HTMLAudioElement | null
  notifySelection: (startLine: number, endLine: number) => void
  /** Same as `notifySelection`, but for Unzipped view text, whose byte offsets map
   * to measure indices via `partMeasureRanges` instead of `measureSpans`. */
  notifyUnzippedSelection: (startOffset: number, endOffset: number) => void
  /** The whole-document Unzipped view projection of `source` (empty until
   * `unzippedView` is enabled), recomputed alongside `measureSpans`. */
  unzippedText: string
  /** Per declared part, per measure index: byte range within `unzippedText`
   * covering that measure's tokens. */
  partMeasureRanges: PartMeasureRangesOut[]
  /** Per declared part, per lyrics verse, per measure index: byte range
   * within `unzippedText` covering that verse's tagged `[Abbrev:lyrics:N]`
   * block tokens for that measure. */
  lyricsVerseRanges: LyricsVerseRangesOut[]
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
