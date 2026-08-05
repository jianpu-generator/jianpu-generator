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

export type WorkerRequest =
  | { type: 'wasmModule'; module: WebAssembly.Module }
  | {
      type: 'loadSoundfont'
      soundfont: ArrayBuffer
    }
  | {
      type: 'loadPdfFonts'
      scFont: ArrayBuffer
      tcFont: ArrayBuffer
      monoFont: ArrayBuffer
    }
  | {
      type: 'render'
      source: string
      id: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
  | { type: 'listParts'; source: string; id: number }
  | {
      type: 'updatePartDeclaration'
      source: string
      abbreviation: string
      mode: PartMode
      followTarget: string | null
      soundfont: string | null
      volume: number | null
      octaveOffset: number | null
      id: number
    }
  | {
      type: 'generatePdf'
      source: string
      id: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
  | {
      type: 'generateSplitPdf'
      source: string
      id: number
      baseName: string
    }
  | {
      type: 'generateMidi'
      source: string
      id: number
      enabledTracks?: string[]
    }
  | {
      type: 'generateSplitMidi'
      source: string
      id: number
      baseName: string
    }
  | {
      type: 'generateSplitWav'
      source: string
      id: number
      baseName: string
    }
  | {
      type: 'generateAudio'
      source: string
      id: number
      enabledTracks?: string[]
    }
  | {
      type: 'generateMeasureRangeAudio'
      source: string
      id: number
      startMeasureIndex: number
      endMeasureIndex: number
      extendToLastOccurrence: boolean
      respectSequence: boolean
      /**
       * 0-based index range into `# sequence` (the order entries are
       * written in `# sequence`) naming the exact entry/entries selected,
       * so a repeated label (e.g. `A, B(-x), B`) resolves to the clicked
       * occurrence instead of always the first one sharing that written
       * measure range. Omit both when the range isn't a `# sequence`
       * selection (e.g. "play current measure").
       */
      sequenceEntryStartIndex?: number
      sequenceEntryEndIndex?: number
      enabledTracks?: string[]
    }
  | {
      type: 'renderWithHighlightRange'
      source: string
      id: number
      startMeasureIndex: number
      endMeasureIndex: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
  | { type: 'listMeasureSpans'; source: string; id: number }
  | { type: 'previewInstrument'; id: number; programNumber: number }
  | { type: 'previewPercussion'; id: number; key: number }
  | {
      type: 'importFromFile'
      id: number
      bytes: ArrayBuffer
      kind: 'svg' | 'pdf'
    }
  | {
      type: 'formatScore'
      source: string
      id: number
    }

export type WorkerResponse =
  | {
      type: 'ready'
      audioAvailable: boolean
      pdfAvailable: boolean
      midiAvailable: boolean
    }
  | {
      type: 'ok'
      id: number
      documents: SvgDocumentOut[]
      diagnostics: Diagnostic[]
      diagnosticViewZones: DiagnosticViewZone[]
    }
  | {
      type: 'audio'
      id: number
      wav: ArrayBuffer
      noteTimings: NoteTimingOut[]
    }
  | { type: 'audioErr'; id: number }
  | {
      type: 'err'
      id: number
      diagnostics: Diagnostic[]
      diagnosticViewZones: DiagnosticViewZone[]
    }
  | {
      type: 'parts'
      id: number
      parts: PartInfo[]
      declarations: PartDeclaration[]
    }
  | {
      type: 'partDeclarationUpdated'
      id: number
      source: string
      declarations: PartDeclaration[]
    }
  | { type: 'pdf'; id: number; pdf: ArrayBuffer }
  | { type: 'pdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitPdf'; id: number; zip: ArrayBuffer }
  | { type: 'splitPdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'midi'; id: number; midi: ArrayBuffer }
  | { type: 'midiErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitMidi'; id: number; zip: ArrayBuffer }
  | { type: 'splitMidiErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitWav'; id: number; zip: ArrayBuffer }
  | { type: 'splitWavErr'; id: number; diagnostics: Diagnostic[] }
  | {
      type: 'measureRangeAudio'
      id: number
      wav: ArrayBuffer
      noteTimings: NoteTimingOut[]
    }
  | { type: 'measureRangeAudioErr'; id: number }
  | { type: 'instrumentPreview'; id: number; wav: ArrayBuffer }
  | { type: 'instrumentPreviewErr'; id: number }
  | { type: 'percussionPreview'; id: number; wav: ArrayBuffer }
  | { type: 'percussionPreviewErr'; id: number }
  | { type: 'highlightRangeOk'; id: number; documents: SvgDocumentOut[] }
  | { type: 'highlightRangeErr'; id: number; diagnostics: Diagnostic[] }
  | {
      type: 'measureSpans'
      id: number
      status: 'ok' | 'err'
      spans: MeasureSpan[]
      sectionRanges: SectionRange[]
      sequenceEntries: SequenceEntry[]
    }
  | { type: 'importOk'; id: number; source: string }
  | { type: 'importErr'; id: number }
  | { type: 'scoreFormatted'; id: number; source: string }
