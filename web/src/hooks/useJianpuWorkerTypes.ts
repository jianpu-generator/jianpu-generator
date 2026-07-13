import type { SvgDocumentOut } from 'jianpu-wasm'
import type {
  Diagnostic,
  DiagnosticViewZone,
  MeasureSpan,
  PartDeclaration,
  PartInfo,
  PartMode,
  SectionRange,
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
  /** The `<audio>` element currently playing the selected measure range, if any; a new element each time playback starts. */
  measureAudioElement: HTMLAudioElement | null
  notifySelection: (startLine: number, endLine: number) => void
  playSelectedMeasures: () => void
  playFromCurrentMeasure: () => void
  stopMeasurePlayback: () => void
  highlightedDocuments: SvgDocumentOut[]
  measureSpans: MeasureSpan[]
  sectionRanges: SectionRange[]
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
}
