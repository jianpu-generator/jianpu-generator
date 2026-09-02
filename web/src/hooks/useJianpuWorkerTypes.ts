import type { NoteTimingOut, SvgDocumentOut } from 'jianpu-wasm'
import type { RefObject } from 'react'
import type {
  Diagnostic,
  DiagnosticViewZone,
  LyricSpan,
  MeasureSpan,
  NoteSpan,
  PartDeclaration,
  PartInfo,
  PartMode,
  SectionRange,
  SequenceEntry,
} from '../types'
import type { NoteCell } from '../utils/noteSpanSelection'

/** Tracks one in-flight "send source, get rewritten source back" round trip
 * to the render worker, so a stale reply (superseded by a newer request for
 * the same action) can be dropped instead of resolving out of order. */
export interface TextRequestTracker {
  requestIdRef: RefObject<number>
  latestIdRef: RefObject<number>
  pendingRequestsRef: RefObject<Map<number, (source: string) => void>>
}

export interface JianpuWorkerState {
  parts: PartInfo[]
  partDeclarations: PartDeclaration[]
  partsLoading: boolean
  documents: SvgDocumentOut[]
  wavUrl: string | null
  wavFilename: string
  /** The full-score preview MP3 URL, mirroring `wavUrl` — mutually exclusive
   * with it (generating one revokes and clears the other), so `Preview`
   * shows whichever was most recently generated. */
  mp3Url: string | null
  mp3Filename: string
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
  mp3Available: boolean
  mp3Exporting: boolean
  splitMp3Exporting: boolean
  diagnostics: Diagnostic[]
  diagnosticViewZones: DiagnosticViewZone[]
  rendering: boolean
  audioGenerating: boolean
  exportPdf: () => void
  exportSplitPdf: () => void
  exportMidi: () => void
  exportSplitMidi: () => void
  exportSplitWav: () => void
  exportMp3: () => void
  exportSplitMp3: () => void
  generateFullAudio: () => void
  selectedMeasureRange: {
    start: number
    end: number
    /** Which measure the preview should scroll to for this selection, when
     * it differs from `start` — a section/sequence chain selection can
     * resolve to a written-measure range whose start (in document order)
     * isn't where the user actually navigated to (e.g. dragging from an
     * early section down to a later one out of chain order). See
     * `notifySelection`'s `revealLine` parameter. */
    revealMeasureIndex: number
    /** The exact disjoint measure ranges to highlight in the SVG preview,
     * when they differ from the single `[start, end]` span above — a `#
     * sequence` chain selection spanning out-of-order entries (e.g. "C" and
     * a later repeat of "A", but not "B" in between). Only ever populated
     * by a sequence-chain selection; every other selection kind leaves this
     * unset and falls back to the caret-only single-range highlight. */
    highlightRanges?: { start: number; end: number }[]
  } | null
  measureAudioGenerating: boolean
  measureAudioPlaying: boolean
  /** Elapsed-seconds start/end of every sounding note/rest for the selected range's audio, keyed by `(source_part_index, note_id)`. */
  measureAudioNoteTimings: NoteTimingOut[]
  /** The `<audio>` element currently playing the selected measure range, if any; a new element each time playback starts. */
  measureAudioElement: HTMLAudioElement | null
  notifySelection: (
    startLine: number,
    endLine: number,
    isEmpty: boolean,
    /** The line the preview should scroll to, if it differs from
     * `startLine` — see `revealMeasureIndex` above. Defaults to
     * `startLine`. */
    revealLine?: number,
    /** The exact disjoint measure ranges to highlight in the SVG preview —
     * see `highlightRanges` above. */
    measureRanges?: { start: number; end: number }[],
  ) => void
  playSelectedMeasures: () => void
  playFromCurrentMeasure: () => void
  /** Plays only `selectedPartNames`, muting the rest, over
   * `[minMeasureIndex, maxMeasureIndex]`, then trims playback to the exact
   * elapsed-seconds window of `selectedCells` — see `useNoteSelection`'s
   * `selectedNoteRangePlaybackInfo`/`selectedNoteCells`. */
  playNoteSelection: (
    minMeasureIndex: number,
    maxMeasureIndex: number,
    selectedPartNames: string[],
    selectedCells: NoteCell[],
  ) => void
  /** Plays the whole score from its first measure through the last written
   * one, following any D.C./D.S./`# sequence` repeat structure — the "Play
   * All" button. */
  playAll: () => void
  stopMeasurePlayback: () => void
  highlightedDocuments: SvgDocumentOut[]
  measureSpans: MeasureSpan[]
  /** Source byte span of every note/chord/percussion/rest event, keyed by
   * `(source_part_index, note_id)` matching the SVG's `data-part-index`/
   * `data-note-id` attributes — see `useNoteSelection`. */
  noteSpans: NoteSpan[]
  /** Source byte span of every lyric syllable, keyed by
   * `(source_part_index, note_id, verse)` matching the SVG's
   * `data-part-index`/`data-note-id`/`data-verse` attributes — see
   * `useLyricSelection`. */
  lyricSpans: LyricSpan[]
  sectionRanges: SectionRange[]
  sequenceEntries: SequenceEntry[]
  /** The current mute/solo filter as sent to the `listNoteSpans`/
   * `listLyricSpans`/render worker messages — `undefined` means every part
   * is enabled. `undefined`-vs-empty-array (rather than always an array)
   * matches `enabledTracksForRender`'s contract. Needed by `useNoteSelection`
   * to resolve `sourcePartIndex` against the same visible-parts-only index
   * space `noteSpans` was compiled with. */
  enabledTracks: string[] | undefined
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
   * Zipped-view "Format" action: drops `# score` `[Key]` data lines that are
   * entirely redundant with implicit-fill, and collapses whitespace on
   * every surviving directive/data line (see `format_source::format_score`).
   */
  formatScore: (source: string) => Promise<string>
  /**
   * Rewrites the `'`/`,` octave marker on every note in the named part by
   * `delta` octaves (see `source_edit::shift_part_octave`) — the "notation
   * octave" control, distinct from the MIDI-only `octaveOffset` in
   * `updatePartDeclaration`. Resolves with the updated source; a `follow[X]`
   * part or unknown abbreviation resolves with the source unchanged.
   */
  shiftPartOctave: (abbreviation: string, delta: number) => Promise<string>
  /**
   * Recovers the `.jianpu` source embedded in a previously exported SVG/PDF
   * file (see `source_embed::extract_embedded_source`). Rejects if the file
   * has no embedded source.
   */
  importFromFile: (file: File) => Promise<string>
}
