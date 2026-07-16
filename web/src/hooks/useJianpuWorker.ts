import type { SvgDocumentOut } from 'jianpu-wasm'
import { useCallback, useMemo, useRef, useState } from 'react'
import type {
  Diagnostic,
  DiagnosticViewZone,
  MeasureSpan,
  PartDeclaration,
  PartInfo,
  PartMode,
  SectionRange,
} from '../types'
import type { WorkerRequest } from '../worker/jianpu.worker'
import { useInstrumentPreview } from './useInstrumentPreview'
import { useJianpuWorkerExports } from './useJianpuWorkerExports'
import { useJianpuWorkerLifecycle } from './useJianpuWorkerLifecycle'
import { useJianpuWorkerRenderRequests } from './useJianpuWorkerRenderRequests'
import type { JianpuWorkerState } from './useJianpuWorkerTypes'
import { useMeasureAudioPlayback } from './useMeasureAudioPlayback'
import {
  disabledLyricsForRender,
  enabledPartNamesForFilename,
  enabledTracksForRender,
  wavFilenameFromActiveFile,
} from './workerHelpers'

export type { JianpuWorkerState } from './useJianpuWorkerTypes'

export function useJianpuWorker(
  source: string,
  disabledParts: ReadonlySet<string>,
  disabledLyrics: ReadonlySet<string>,
  soloedParts: ReadonlySet<string>,
  activeFile: string,
  soundfontBytes: Uint8Array | null,
  fontBytes: { sc: Uint8Array; tc: Uint8Array; mono: Uint8Array } | null,
  debounceMs = 300,
): JianpuWorkerState {
  const [parts, setParts] = useState<PartInfo[]>([])
  const [partDeclarations, setPartDeclarations] = useState<PartDeclaration[]>(
    [],
  )
  const [partsLoading, setPartsLoading] = useState(false)
  const [documents, setDocuments] = useState<SvgDocumentOut[]>([])
  const [wavUrl, setWavUrl] = useState<string | null>(null)
  const [measureTimes, setMeasureTimes] = useState<number[]>([])
  const [writtenMeasureIndices, setWrittenMeasureIndices] = useState<number[]>(
    [],
  )
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
  const [sectionRanges, setSectionRanges] = useState<SectionRange[]>([])
  const highlightRenderRequestIdRef = useRef(0)
  const latestHighlightRenderIdRef = useRef(0)
  const measureSpansRequestIdRef = useRef(0)
  const latestMeasureSpansIdRef = useRef(0)
  const measureSpansRef = useRef<MeasureSpan[]>([])
  const workerRef = useRef<Worker | null>(null)
  const wavUrlRef = useRef<string | null>(null)
  const partsRequestIdRef = useRef(0)
  const updatePartDeclarationRequestIdRef = useRef(0)
  const latestUpdatePartDeclarationIdRef = useRef(0)
  const pendingPartDeclarationUpdatesRef = useRef(
    new Map<number, (source: string) => void>(),
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
  const lastSelectionRef = useRef<{ start: number; end: number } | null>(null)

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

  const setNextWavUrl = useCallback((next: string | null) => {
    if (wavUrlRef.current) {
      URL.revokeObjectURL(wavUrlRef.current)
    }
    wavUrlRef.current = next
    setWavUrl(next)
  }, [])

  const {
    measureAudioGenerating,
    setMeasureAudioGenerating,
    measureAudioPlaying,
    measureAudioTimes,
    measureAudioWrittenIndices,
    measureAudioElement,
    setNextMeasureWavUrl,
    stopMeasurePlayback,
    playSelectedMeasures,
    playFromCurrentMeasure,
    latestMeasureAudioIdRef,
    measureWavUrlRef,
  } = useMeasureAudioPlayback({
    workerRef,
    sourceRef,
    enabledTracksRef,
    selectedMeasureRange,
    measureSpans,
  })

  const {
    previewAudioPlaying,
    setPreviewAudioPlaying,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    latestPreviewAudioIdRef,
    currentPreviewAudioRef,
  } = useInstrumentPreview({ workerRef })

  useJianpuWorkerLifecycle({
    workerRef,
    wavUrlRef,
    measureWavUrlRef,
    cursorOffsetTimerRef,
    soundfontBytes,
    fontBytes,
    audioAvailableRef,
    setAudioAvailable,
    setPdfAvailable,
    setMidiAvailable,
    latestPartsIdRef,
    setPartsLoading,
    setParts,
    setPartDeclarations,
    latestUpdatePartDeclarationIdRef,
    pendingPartDeclarationUpdatesRef,
    latestPdfIdRef,
    setPdfExporting,
    activeFileRef,
    enabledPartNamesRef,
    setDiagnostics,
    latestSplitPdfIdRef,
    setSplitPdfExporting,
    latestMidiIdRef,
    setMidiExporting,
    latestSplitMidiIdRef,
    setSplitMidiExporting,
    latestSplitWavIdRef,
    setSplitWavExporting,
    latestRenderIdRef,
    setRendering,
    setDocuments,
    setDiagnosticViewZones,
    latestAudioIdRef,
    setAudioGenerating,
    setNextWavUrl,
    setMeasureTimes,
    setWrittenMeasureIndices,
    latestMeasureAudioIdRef,
    setMeasureAudioGenerating,
    setNextMeasureWavUrl,
    latestHighlightRenderIdRef,
    setHighlightedDocuments,
    latestMeasureSpansIdRef,
    setMeasureSpans,
    setSectionRanges,
    latestPreviewAudioIdRef,
    currentPreviewAudioRef,
    setPreviewAudioPlaying,
  })

  const { notifySelection } = useJianpuWorkerRenderRequests({
    workerRef,
    sourceRef,
    source,
    activeFile,
    debounceMs,
    enabledTracks,
    disabledLyricsTracks,
    setDocuments,
    setNextWavUrl,
    setDiagnostics,
    setPartsLoading,
    partsRequestIdRef,
    latestPartsIdRef,
    setRendering,
    renderRequestIdRef,
    latestRenderIdRef,
    selectedMeasureRange,
    setSelectedMeasureRange,
    setHighlightedDocuments,
    highlightRenderRequestIdRef,
    latestHighlightRenderIdRef,
    measureSpans,
    measureSpansRef,
    measureSpansRequestIdRef,
    latestMeasureSpansIdRef,
    cursorOffsetTimerRef,
    lastSelectionRef,
  })

  const generateFullAudio = useCallback(() => {
    const worker = workerRef.current
    if (!worker || audioGenerating) return
    const id = ++audioRequestIdRef.current
    latestAudioIdRef.current = id
    setAudioGenerating(true)
    worker.postMessage({
      type: 'generateAudio',
      source: sourceRef.current,
      id,
      enabledTracks: enabledTracksRef.current,
    } satisfies WorkerRequest)
  }, [audioGenerating])

  const {
    exportPdf,
    exportSplitPdf,
    exportMidi,
    exportSplitMidi,
    exportSplitWav,
  } = useJianpuWorkerExports({
    workerRef,
    sourceRef,
    activeFileRef,
    enabledTracksRef,
    disabledLyricsRef,
    pdfExporting,
    splitPdfExporting,
    midiExporting,
    splitMidiExporting,
    splitWavExporting,
    setPdfExporting,
    setSplitPdfExporting,
    setMidiExporting,
    setSplitMidiExporting,
    setSplitWavExporting,
    pdfRequestIdRef,
    latestPdfIdRef,
    splitPdfRequestIdRef,
    latestSplitPdfIdRef,
    midiRequestIdRef,
    latestMidiIdRef,
    splitMidiRequestIdRef,
    latestSplitMidiIdRef,
    splitWavRequestIdRef,
    latestSplitWavIdRef,
  })

  const updatePartDeclaration = useCallback(
    (
      abbreviation: string,
      mode: PartMode,
      followTarget: string | null,
      soundfont: string | null,
      volume: number | null,
      octaveOffset: number | null,
    ) =>
      new Promise<string>((resolve) => {
        const worker = workerRef.current
        if (!worker) {
          resolve(sourceRef.current)
          return
        }
        const id = ++updatePartDeclarationRequestIdRef.current
        latestUpdatePartDeclarationIdRef.current = id
        pendingPartDeclarationUpdatesRef.current.set(id, resolve)
        worker.postMessage({
          type: 'updatePartDeclaration',
          source: sourceRef.current,
          abbreviation,
          mode,
          followTarget,
          soundfont,
          volume,
          octaveOffset,
          id,
        } satisfies WorkerRequest)
      }),
    [],
  )

  return {
    parts,
    partDeclarations,
    partsLoading,
    documents,
    wavUrl,
    wavFilename,
    measureTimes,
    writtenMeasureIndices,
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    splitPdfExporting,
    midiAvailable,
    midiExporting,
    splitMidiExporting,
    splitWavExporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    exportPdf,
    exportSplitPdf,
    exportMidi,
    exportSplitMidi,
    exportSplitWav,
    generateFullAudio,
    selectedMeasureRange,
    measureAudioGenerating,
    measureAudioPlaying,
    measureAudioTimes,
    measureAudioWrittenIndices,
    measureAudioElement,
    notifySelection,
    playSelectedMeasures,
    playFromCurrentMeasure,
    stopMeasurePlayback,
    highlightedDocuments,
    measureSpans,
    sectionRanges,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    previewAudioPlaying,
    updatePartDeclaration,
  }
}
