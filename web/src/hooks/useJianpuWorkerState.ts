import type {
  LyricsVerseRangesOut,
  NoteTimingOut,
  PartMeasureRangesOut,
  SvgDocumentOut,
} from 'jianpu-wasm'
import { useMemo, useRef, useState } from 'react'
import type {
  Diagnostic,
  DiagnosticViewZone,
  MeasureSpan,
  PartDeclaration,
  PartInfo,
  SectionRange,
  SequenceEntry,
} from '../types'
import {
  disabledLyricsForRender,
  enabledPartNamesForFilename,
  enabledTracksForRender,
  wavFilenameFromActiveFile,
} from './workerHelpers'

/** All the plain state, refs, and derived values `useJianpuWorker` shares
 * across its sub-hooks (lifecycle, render requests, exports, measure audio,
 * instrument preview). Kept together because most of it is read and written
 * from several of those sub-hooks via refs that must stay in sync with the
 * latest render's state. */
export function useJianpuWorkerState(
  source: string,
  activeFile: string,
  disabledParts: ReadonlySet<string>,
  disabledLyrics: ReadonlySet<string>,
  soloedParts: ReadonlySet<string>,
) {
  const [parts, setParts] = useState<PartInfo[]>([])
  const [partDeclarations, setPartDeclarations] = useState<PartDeclaration[]>(
    [],
  )
  const [partsLoading, setPartsLoading] = useState(false)
  const [documents, setDocuments] = useState<SvgDocumentOut[]>([])
  const [wavUrl, setWavUrl] = useState<string | null>(null)
  const [noteTimings, setNoteTimings] = useState<NoteTimingOut[]>([])
  const [audioAvailable, setAudioAvailable] = useState(false)
  const [pdfAvailable, setPdfAvailable] = useState(false)
  const [pdfExporting, setPdfExporting] = useState(false)
  const [splitPdfExporting, setSplitPdfExporting] = useState(false)
  const [midiAvailable, setMidiAvailable] = useState(false)
  const [midiExporting, setMidiExporting] = useState(false)
  const [splitMidiExporting, setSplitMidiExporting] = useState(false)
  const [splitWavExporting, setSplitWavExporting] = useState(false)
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([])
  const [diagnosticViewZones, setDiagnosticViewZones] = useState<
    DiagnosticViewZone[]
  >([])
  const [rendering, setRendering] = useState(false)
  const [audioGenerating, setAudioGenerating] = useState(false)
  const [selectedMeasureRange, setSelectedMeasureRange] = useState<{
    start: number
    end: number
  } | null>(null)
  const [highlightedDocuments, setHighlightedDocuments] = useState<
    SvgDocumentOut[]
  >([])
  const [measureSpans, setMeasureSpans] = useState<MeasureSpan[]>([])
  // A one-time snapshot of `extract_unzipped_text(source)`, taken only when
  // Unzipped view is switched on (see effect in useJianpuWorker) — NOT
  // re-derived on every source change, so the Unzipped editor's displayed
  // text doesn't get clobbered by rest-padding while the user is mid-edit.
  // Contrast with `partMeasureRanges`, which is intentionally re-derived on
  // every source change inside `useJianpuWorkerRenderRequests` for
  // (stale-tolerated) cursor->measure highlighting.
  const [unzippedText, setUnzippedText] = useState('')
  const [partMeasureRanges, setPartMeasureRanges] = useState<
    PartMeasureRangesOut[]
  >([])
  const [lyricsVerseRanges, setLyricsVerseRanges] = useState<
    LyricsVerseRangesOut[]
  >([])
  const [sectionRanges, setSectionRanges] = useState<SectionRange[]>([])
  const [sequenceEntries, setSequenceEntries] = useState<SequenceEntry[]>([])
  const highlightRenderRequestIdRef = useRef(0)
  const latestHighlightRenderIdRef = useRef(0)
  const measureSpansRequestIdRef = useRef(0)
  const latestMeasureSpansIdRef = useRef(0)
  const measureSpansRef = useRef<MeasureSpan[]>([])
  const partMeasureRangesRef = useRef<PartMeasureRangesOut[]>([])
  const lyricsVerseRangesRef = useRef<LyricsVerseRangesOut[]>([])
  const workerRef = useRef<Worker | null>(null)
  const wavUrlRef = useRef<string | null>(null)
  const partsRequestIdRef = useRef(0)
  const updatePartDeclarationRequestIdRef = useRef(0)
  const latestUpdatePartDeclarationIdRef = useRef(0)
  const pendingPartDeclarationUpdatesRef = useRef(
    new Map<number, (source: string) => void>(),
  )
  const formatScoreRequestIdRef = useRef(0)
  const latestFormatScoreIdRef = useRef(0)
  const pendingFormatScoreRequestsRef = useRef(
    new Map<number, (source: string) => void>(),
  )
  const importRequestIdRef = useRef(0)
  const pendingImportsRef = useRef(
    new Map<
      number,
      { resolve: (source: string) => void; reject: (error: Error) => void }
    >(),
  )
  const renderRequestIdRef = useRef(0)
  const audioRequestIdRef = useRef(0)
  const pdfRequestIdRef = useRef(0)
  const splitPdfRequestIdRef = useRef(0)
  const midiRequestIdRef = useRef(0)
  const splitMidiRequestIdRef = useRef(0)
  const splitWavRequestIdRef = useRef(0)
  const latestPartsIdRef = useRef(0)
  const latestRenderIdRef = useRef(0)
  const latestAudioIdRef = useRef(0)
  const latestPdfIdRef = useRef(0)
  const latestSplitPdfIdRef = useRef(0)
  const latestMidiIdRef = useRef(0)
  const latestSplitMidiIdRef = useRef(0)
  const latestSplitWavIdRef = useRef(0)
  const sourceRef = useRef(source)
  const activeFileRef = useRef(activeFile)
  const enabledTracksRef = useRef<string[] | undefined>(undefined)
  const enabledPartNamesRef = useRef<string[] | undefined>(undefined)
  const disabledLyricsRef = useRef<string[] | undefined>(undefined)
  const audioAvailableRef = useRef(false)
  const cursorOffsetTimerRef = useRef<number | null>(null)
  const lastSelectionRef = useRef<{
    start: number
    end: number
    mode: 'source' | 'unzipped'
  } | null>(null)

  const effectiveDisabledParts = useMemo(() => {
    if (soloedParts.size === 0) return disabledParts
    return new Set(
      parts
        .map((part) => part.abbreviation)
        .filter((abbr) => !soloedParts.has(abbr)),
    )
  }, [soloedParts, parts, disabledParts])

  const enabledTracks = useMemo(
    () => enabledTracksForRender(parts, effectiveDisabledParts),
    [parts, effectiveDisabledParts],
  )
  const enabledPartNames = useMemo(
    () => enabledPartNamesForFilename(parts, effectiveDisabledParts),
    [parts, effectiveDisabledParts],
  )
  const disabledLyricsTracks = useMemo(
    () => disabledLyricsForRender(parts, disabledLyrics),
    [parts, disabledLyrics],
  )
  const wavFilename = useMemo(
    () => wavFilenameFromActiveFile(activeFile, enabledPartNames),
    [activeFile, enabledPartNames],
  )

  sourceRef.current = source
  activeFileRef.current = activeFile
  enabledTracksRef.current = enabledTracks
  enabledPartNamesRef.current = enabledPartNames
  disabledLyricsRef.current = disabledLyricsTracks
  measureSpansRef.current = measureSpans
  partMeasureRangesRef.current = partMeasureRanges
  lyricsVerseRangesRef.current = lyricsVerseRanges

  return {
    parts,
    setParts,
    partDeclarations,
    setPartDeclarations,
    partsLoading,
    setPartsLoading,
    documents,
    setDocuments,
    wavUrl,
    setWavUrl,
    noteTimings,
    setNoteTimings,
    audioAvailable,
    setAudioAvailable,
    pdfAvailable,
    setPdfAvailable,
    pdfExporting,
    setPdfExporting,
    splitPdfExporting,
    setSplitPdfExporting,
    midiAvailable,
    setMidiAvailable,
    midiExporting,
    setMidiExporting,
    splitMidiExporting,
    setSplitMidiExporting,
    splitWavExporting,
    setSplitWavExporting,
    diagnostics,
    setDiagnostics,
    diagnosticViewZones,
    setDiagnosticViewZones,
    rendering,
    setRendering,
    audioGenerating,
    setAudioGenerating,
    selectedMeasureRange,
    setSelectedMeasureRange,
    highlightedDocuments,
    setHighlightedDocuments,
    measureSpans,
    setMeasureSpans,
    unzippedText,
    setUnzippedText,
    partMeasureRanges,
    setPartMeasureRanges,
    lyricsVerseRanges,
    setLyricsVerseRanges,
    sectionRanges,
    setSectionRanges,
    sequenceEntries,
    setSequenceEntries,
    highlightRenderRequestIdRef,
    latestHighlightRenderIdRef,
    measureSpansRequestIdRef,
    latestMeasureSpansIdRef,
    measureSpansRef,
    partMeasureRangesRef,
    lyricsVerseRangesRef,
    workerRef,
    wavUrlRef,
    partsRequestIdRef,
    updatePartDeclarationRequestIdRef,
    latestUpdatePartDeclarationIdRef,
    pendingPartDeclarationUpdatesRef,
    formatScoreRequestIdRef,
    latestFormatScoreIdRef,
    pendingFormatScoreRequestsRef,
    importRequestIdRef,
    pendingImportsRef,
    renderRequestIdRef,
    audioRequestIdRef,
    pdfRequestIdRef,
    splitPdfRequestIdRef,
    midiRequestIdRef,
    splitMidiRequestIdRef,
    splitWavRequestIdRef,
    latestPartsIdRef,
    latestRenderIdRef,
    latestAudioIdRef,
    latestPdfIdRef,
    latestSplitPdfIdRef,
    latestMidiIdRef,
    latestSplitMidiIdRef,
    latestSplitWavIdRef,
    sourceRef,
    activeFileRef,
    enabledTracksRef,
    enabledPartNamesRef,
    disabledLyricsRef,
    audioAvailableRef,
    cursorOffsetTimerRef,
    lastSelectionRef,
    enabledTracks,
    enabledPartNames,
    disabledLyricsTracks,
    wavFilename,
  }
}
