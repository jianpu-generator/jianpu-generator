import type { SvgDocumentOut } from 'jianpu-wasm'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
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
import { createWorkerMessageHandler } from './useJianpuWorkerMessageHandler'
import type { JianpuWorkerState } from './useJianpuWorkerTypes'
import {
  baseNameFromActiveFile,
  disabledLyricsForRender,
  enabledTracksForRender,
  measureRangeInSpan,
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
  const [audioAvailable, setAudioAvailable] = useState(false)
  const [pdfAvailable, setPdfAvailable] = useState(false)
  const [pdfExporting, setPdfExporting] = useState(false)
  const [splitPdfExporting, setSplitPdfExporting] = useState(false)
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
  const [measureAudioGenerating, setMeasureAudioGenerating] = useState(false)
  const [measureAudioPlaying, setMeasureAudioPlaying] = useState(false)
  const [previewAudioPlaying, setPreviewAudioPlaying] = useState(false)
  const currentMeasureAudioRef = useRef<HTMLAudioElement | null>(null)
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
  const latestPartsIdRef = useRef(0)
  const latestRenderIdRef = useRef(0)
  const latestAudioIdRef = useRef(0)
  const latestPdfIdRef = useRef(0)
  const latestSplitPdfIdRef = useRef(0)
  const sourceRef = useRef(source)
  const activeFileRef = useRef(activeFile)
  const enabledTracksRef = useRef<string[] | undefined>(undefined)
  const disabledLyricsRef = useRef<string[] | undefined>(undefined)
  const audioAvailableRef = useRef(false)
  const cursorOffsetTimerRef = useRef<number | null>(null)
  const lastSelectionRef = useRef<{ start: number; end: number } | null>(null)
  const measureAudioRequestIdRef = useRef(0)
  const latestMeasureAudioIdRef = useRef(0)
  const measureWavUrlRef = useRef<string | null>(null)
  const previewAudioRequestIdRef = useRef(0)
  const latestPreviewAudioIdRef = useRef(0)
  const currentPreviewAudioRef = useRef<HTMLAudioElement | null>(null)

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
  const disabledLyricsTracks = useMemo(
    () => disabledLyricsForRender(parts, disabledLyrics),
    [parts, disabledLyrics],
  )

  sourceRef.current = source
  activeFileRef.current = activeFile
  enabledTracksRef.current = enabledTracks
  disabledLyricsRef.current = disabledLyricsTracks
  measureSpansRef.current = measureSpans

  const setNextWavUrl = useCallback((next: string | null) => {
    if (wavUrlRef.current) {
      URL.revokeObjectURL(wavUrlRef.current)
    }
    wavUrlRef.current = next
    setWavUrl(next)
  }, [])

  const setNextMeasureWavUrl = useCallback((next: string | null) => {
    if (currentMeasureAudioRef.current) {
      currentMeasureAudioRef.current.pause()
      currentMeasureAudioRef.current = null
    }
    if (measureWavUrlRef.current) {
      URL.revokeObjectURL(measureWavUrlRef.current)
    }
    measureWavUrlRef.current = next
    if (next) {
      const audio = new Audio(next)
      currentMeasureAudioRef.current = audio
      audio.addEventListener('play', () => setMeasureAudioPlaying(true))
      audio.addEventListener('ended', () => {
        setMeasureAudioPlaying(false)
        currentMeasureAudioRef.current = null
      })
      audio.addEventListener('pause', () => setMeasureAudioPlaying(false))
      audio.play().catch(() => {})
    }
  }, [])

  useEffect(() => {
    const worker = new Worker(
      new URL('../worker/jianpu.worker.ts', import.meta.url),
      { type: 'module' },
    )
    workerRef.current = worker

    worker.onmessage = createWorkerMessageHandler({
      audioAvailableRef,
      setAudioAvailable,
      setPdfAvailable,
      latestPartsIdRef,
      setPartsLoading,
      setParts,
      setPartDeclarations,
      latestUpdatePartDeclarationIdRef,
      pendingPartDeclarationUpdatesRef,
      latestPdfIdRef,
      setPdfExporting,
      activeFileRef,
      setDiagnostics,
      latestSplitPdfIdRef,
      setSplitPdfExporting,
      latestRenderIdRef,
      setRendering,
      setDocuments,
      setDiagnosticViewZones,
      latestAudioIdRef,
      setAudioGenerating,
      setNextWavUrl,
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

    return () => {
      worker.terminate()
      workerRef.current = null
      if (wavUrlRef.current) {
        URL.revokeObjectURL(wavUrlRef.current)
        wavUrlRef.current = null
      }
      if (measureWavUrlRef.current) {
        URL.revokeObjectURL(measureWavUrlRef.current)
        measureWavUrlRef.current = null
      }
      if (cursorOffsetTimerRef.current !== null) {
        window.clearTimeout(cursorOffsetTimerRef.current)
      }
    }
  }, [setNextWavUrl, setNextMeasureWavUrl])

  useEffect(() => {
    const worker = workerRef.current
    if (!worker || !soundfontBytes) return
    worker.postMessage({
      type: 'loadSoundfont',
      soundfont: soundfontBytes.buffer as ArrayBuffer,
    } satisfies WorkerRequest)
  }, [soundfontBytes])

  useEffect(() => {
    const worker = workerRef.current
    if (!worker || !fontBytes) return
    worker.postMessage({
      type: 'loadPdfFonts',
      scFont: fontBytes.sc.buffer as ArrayBuffer,
      tcFont: fontBytes.tc.buffer as ArrayBuffer,
      monoFont: fontBytes.mono.buffer as ArrayBuffer,
    } satisfies WorkerRequest)
  }, [fontBytes])

  // biome-ignore lint/correctness/useExhaustiveDependencies: activeFile is intentional trigger
  useEffect(() => {
    setDocuments([])
    setNextWavUrl(null)
    setDiagnostics([])
  }, [activeFile, setNextWavUrl])

  // biome-ignore lint/correctness/useExhaustiveDependencies: source is intentional trigger
  useEffect(() => {
    setSelectedMeasureRange(null)
  }, [source])

  useEffect(() => {
    const worker = workerRef.current
    if (!worker) return

    const id = ++partsRequestIdRef.current
    latestPartsIdRef.current = id
    setPartsLoading(true)

    const timer = window.setTimeout(() => {
      const payload: WorkerRequest = { type: 'listParts', source, id }
      worker.postMessage(payload)
    }, debounceMs)

    return () => window.clearTimeout(timer)
  }, [source, debounceMs])

  // biome-ignore lint/correctness/useExhaustiveDependencies: activeFile triggers re-render after rename (content unchanged but activeFile changes)
  useEffect(() => {
    const worker = workerRef.current
    if (!worker) return

    const id = ++renderRequestIdRef.current
    latestRenderIdRef.current = id
    setRendering(true)

    const payload: WorkerRequest = {
      type: 'render',
      source,
      id,
      enabledTracks,
      disabledLyrics: disabledLyricsTracks,
    }
    worker.postMessage(payload)
  }, [source, activeFile, enabledTracks, disabledLyricsTracks])

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

  const notifySelection = useCallback(
    (startLine: number, endLine: number) => {
      lastSelectionRef.current = { start: startLine, end: endLine }
      if (cursorOffsetTimerRef.current !== null) {
        window.clearTimeout(cursorOffsetTimerRef.current)
      }
      cursorOffsetTimerRef.current = window.setTimeout(() => {
        cursorOffsetTimerRef.current = null
        setSelectedMeasureRange(
          measureRangeInSpan(measureSpansRef.current, startLine, endLine),
        )
      }, debounceMs)
    },
    [debounceMs],
  )

  useEffect(() => {
    const sel = lastSelectionRef.current
    if (!sel) return
    setSelectedMeasureRange(
      measureRangeInSpan(measureSpans, sel.start, sel.end),
    )
  }, [measureSpans])

  useEffect(() => {
    if (selectedMeasureRange === null) {
      setHighlightedDocuments([])
      return
    }
    const worker = workerRef.current
    if (!worker) return
    const id = ++highlightRenderRequestIdRef.current
    latestHighlightRenderIdRef.current = id
    worker.postMessage({
      type: 'renderWithHighlightRange',
      source: sourceRef.current,
      id,
      startMeasureIndex: selectedMeasureRange.start,
      endMeasureIndex: selectedMeasureRange.end,
      enabledTracks,
      disabledLyrics: disabledLyricsTracks,
    } satisfies WorkerRequest)
  }, [selectedMeasureRange, enabledTracks, disabledLyricsTracks])

  useEffect(() => {
    const worker = workerRef.current
    if (!worker) return

    const id = ++measureSpansRequestIdRef.current
    latestMeasureSpansIdRef.current = id

    const timer = window.setTimeout(() => {
      worker.postMessage({
        type: 'listMeasureSpans',
        source,
        id,
      } satisfies WorkerRequest)
    }, debounceMs)

    return () => window.clearTimeout(timer)
  }, [source, debounceMs])

  const stopMeasurePlayback = useCallback(() => {
    if (currentMeasureAudioRef.current) {
      currentMeasureAudioRef.current.pause()
      currentMeasureAudioRef.current = null
    }
    setMeasureAudioPlaying(false)
  }, [])

  const playSelectedMeasures = useCallback(() => {
    const worker = workerRef.current
    if (!worker || selectedMeasureRange === null) return
    const id = ++measureAudioRequestIdRef.current
    latestMeasureAudioIdRef.current = id
    setMeasureAudioGenerating(true)
    worker.postMessage({
      type: 'generateMeasureRangeAudio',
      source: sourceRef.current,
      id,
      startMeasureIndex: selectedMeasureRange.start,
      endMeasureIndex: selectedMeasureRange.end,
      enabledTracks: enabledTracksRef.current,
    } satisfies WorkerRequest)
  }, [selectedMeasureRange])

  const exportPdf = useCallback(() => {
    const worker = workerRef.current
    if (!worker || pdfExporting || splitPdfExporting) return

    const id = ++pdfRequestIdRef.current
    latestPdfIdRef.current = id
    setPdfExporting(true)

    const payload: WorkerRequest = {
      type: 'generatePdf',
      source: sourceRef.current,
      id,
      enabledTracks: enabledTracksRef.current,
      disabledLyrics: disabledLyricsRef.current,
    }
    worker.postMessage(payload)
  }, [pdfExporting, splitPdfExporting])

  const previewInstrument = useCallback((programNumber: number) => {
    const worker = workerRef.current
    if (!worker) return
    const id = ++previewAudioRequestIdRef.current
    latestPreviewAudioIdRef.current = id
    worker.postMessage({
      type: 'previewInstrument',
      id,
      programNumber,
    } satisfies WorkerRequest)
  }, [])

  const stopPreviewInstrument = useCallback(() => {
    if (currentPreviewAudioRef.current) {
      currentPreviewAudioRef.current.pause()
    }
  }, [])

  const exportSplitPdf = useCallback(() => {
    const worker = workerRef.current
    if (!worker || pdfExporting || splitPdfExporting) return

    const id = ++splitPdfRequestIdRef.current
    latestSplitPdfIdRef.current = id
    setSplitPdfExporting(true)

    const payload: WorkerRequest = {
      type: 'generateSplitPdf',
      source: sourceRef.current,
      id,
      baseName: baseNameFromActiveFile(activeFileRef.current),
    }
    worker.postMessage(payload)
  }, [pdfExporting, splitPdfExporting])

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
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    splitPdfExporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    exportPdf,
    exportSplitPdf,
    generateFullAudio,
    selectedMeasureRange,
    measureAudioGenerating,
    measureAudioPlaying,
    notifySelection,
    playSelectedMeasures,
    stopMeasurePlayback,
    highlightedDocuments,
    measureSpans,
    sectionRanges,
    previewInstrument,
    stopPreviewInstrument,
    previewAudioPlaying,
    updatePartDeclaration,
  }
}
