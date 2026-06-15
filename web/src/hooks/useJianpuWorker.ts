import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { findIndex, findLastIndex } from 'remeda'
import type { Diagnostic, PartInfo } from '../types'
import type { WorkerRequest, WorkerResponse } from '../worker/jianpu.worker'

function measureRangeInSpan(
  spans: Array<{ start: number; end: number }>,
  selStart: number,
  selEnd: number,
): { start: number; end: number } | null {
  const effective = selStart === selEnd ? selEnd + 1 : selEnd
  const overlaps = (span: { start: number; end: number }) =>
    span.start < effective && span.end > selStart
  const start = findIndex(spans, overlaps)
  const end = findLastIndex(spans, overlaps)
  return start === -1 ? null : { start, end }
}

function enabledTracksForRender(
  parts: PartInfo[],
  disabledParts: ReadonlySet<string>,
): string[] | undefined {
  if (parts.length === 0) return undefined
  const enabled = parts
    .filter((part) => !disabledParts.has(part.abbreviation))
    .map((part) => part.abbreviation)
  if (enabled.length === parts.length) return undefined
  return enabled
}

function disabledLyricsForRender(
  parts: PartInfo[],
  disabledLyrics: ReadonlySet<string>,
): string[] | undefined {
  const lyricParts = parts.filter((part) => part.has_lyrics)
  if (lyricParts.length === 0) return undefined
  const disabled = lyricParts
    .filter((part) => disabledLyrics.has(part.abbreviation))
    .map((part) => part.abbreviation)
  if (disabled.length === 0) return undefined
  return disabled
}

interface JianpuWorkerState {
  parts: PartInfo[]
  partsLoading: boolean
  svgs: string[]
  wavUrl: string | null
  audioAvailable: boolean
  pdfAvailable: boolean
  pdfExporting: boolean
  splitPdfExporting: boolean
  diagnostics: Diagnostic[]
  rendering: boolean
  audioGenerating: boolean
  exportPdf: () => void
  exportSplitPdf: () => void
  selectedMeasureRange: { start: number; end: number } | null
  measureAudioGenerating: boolean
  notifySelection: (startOffset: number, endOffset: number) => void
  playSelectedMeasures: () => void
  highlightedSvgs: string[]
  measureSpans: Array<{ start: number; end: number }>
}

function downloadPdf(bytes: ArrayBuffer, filename: string) {
  const url = URL.createObjectURL(
    new Blob([bytes], { type: 'application/pdf' }),
  )
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
}

function pdfFilenameFromActiveFile(activeFile: string): string {
  if (activeFile.endsWith('.jianpu')) {
    return activeFile.replace(/\.jianpu$/, '.pdf')
  }
  return `${activeFile}.pdf`
}

function zipFilenameFromActiveFile(activeFile: string): string {
  if (activeFile.endsWith('.jianpu')) {
    return activeFile.replace(/\.jianpu$/, '.zip')
  }
  return `${activeFile}.zip`
}

function baseNameFromActiveFile(activeFile: string): string {
  if (activeFile.endsWith('.jianpu')) {
    return activeFile.replace(/\.jianpu$/, '')
  }
  return activeFile
}

function downloadZip(bytes: ArrayBuffer, filename: string) {
  const url = URL.createObjectURL(
    new Blob([bytes], { type: 'application/zip' }),
  )
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
}

export function useJianpuWorker(
  source: string,
  disabledParts: ReadonlySet<string>,
  disabledLyrics: ReadonlySet<string>,
  activeFile: string,
  debounceMs = 300,
): JianpuWorkerState {
  const [parts, setParts] = useState<PartInfo[]>([])
  const [partsLoading, setPartsLoading] = useState(false)
  const [svgs, setSvgs] = useState<string[]>([])
  const [wavUrl, setWavUrl] = useState<string | null>(null)
  const [audioAvailable, setAudioAvailable] = useState(false)
  const [pdfAvailable, setPdfAvailable] = useState(false)
  const [pdfExporting, setPdfExporting] = useState(false)
  const [splitPdfExporting, setSplitPdfExporting] = useState(false)
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([])
  const [rendering, setRendering] = useState(false)
  const [audioGenerating, setAudioGenerating] = useState(false)
  const [selectedMeasureRange, setSelectedMeasureRange] = useState<{
    start: number
    end: number
  } | null>(null)
  const [measureAudioGenerating, setMeasureAudioGenerating] = useState(false)
  const [highlightedSvgs, setHighlightedSvgs] = useState<string[]>([])
  const [measureSpans, setMeasureSpans] = useState<
    Array<{ start: number; end: number }>
  >([])
  const highlightRenderRequestIdRef = useRef(0)
  const latestHighlightRenderIdRef = useRef(0)
  const measureSpansRequestIdRef = useRef(0)
  const latestMeasureSpansIdRef = useRef(0)

  const measureSpansRef = useRef<Array<{ start: number; end: number }>>([])

  const workerRef = useRef<Worker | null>(null)
  const wavUrlRef = useRef<string | null>(null)
  const partsRequestIdRef = useRef(0)
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

  const enabledTracks = useMemo(
    () => enabledTracksForRender(parts, disabledParts),
    [parts, disabledParts],
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
    if (measureWavUrlRef.current) {
      URL.revokeObjectURL(measureWavUrlRef.current)
    }
    measureWavUrlRef.current = next
    if (next) {
      const audio = new Audio(next)
      audio.play().catch(() => {})
    }
  }, [])

  useEffect(() => {
    const worker = new Worker(
      new URL('../worker/jianpu.worker.ts', import.meta.url),
      { type: 'module' },
    )
    workerRef.current = worker

    worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
      const msg = event.data
      if (msg.type === 'ready') {
        audioAvailableRef.current = msg.audioAvailable
        setAudioAvailable(msg.audioAvailable)
        setPdfAvailable(msg.pdfAvailable)
        return
      }

      if (msg.type === 'parts') {
        if (msg.id !== latestPartsIdRef.current) return
        setPartsLoading(false)
        setParts(msg.parts)
        return
      }

      if (msg.type === 'pdf') {
        if (msg.id !== latestPdfIdRef.current) return
        setPdfExporting(false)
        downloadPdf(msg.pdf, pdfFilenameFromActiveFile(activeFileRef.current))
        return
      }

      if (msg.type === 'pdfErr') {
        if (msg.id !== latestPdfIdRef.current) return
        setPdfExporting(false)
        setDiagnostics(msg.diagnostics)
        return
      }

      if (msg.type === 'splitPdf') {
        if (msg.id !== latestSplitPdfIdRef.current) return
        setSplitPdfExporting(false)
        downloadZip(msg.zip, zipFilenameFromActiveFile(activeFileRef.current))
        return
      }

      if (msg.type === 'splitPdfErr') {
        if (msg.id !== latestSplitPdfIdRef.current) return
        setSplitPdfExporting(false)
        setDiagnostics(msg.diagnostics)
        return
      }

      if (msg.type === 'ok') {
        if (msg.id !== latestRenderIdRef.current) return
        setRendering(false)
        setSvgs(msg.svgs)
        setDiagnostics([])
        return
      }

      if (msg.type === 'audio') {
        if (msg.id !== latestAudioIdRef.current) return
        setAudioGenerating(false)
        setNextWavUrl(
          URL.createObjectURL(new Blob([msg.wav], { type: 'audio/wav' })),
        )
        return
      }

      if (msg.type === 'audioErr') {
        if (msg.id !== latestAudioIdRef.current) return
        setAudioGenerating(false)
        return
      }

      if (msg.type === 'measureRangeAudio') {
        if (msg.id !== latestMeasureAudioIdRef.current) return
        setMeasureAudioGenerating(false)
        setNextMeasureWavUrl(
          URL.createObjectURL(new Blob([msg.wav], { type: 'audio/wav' })),
        )
        return
      }

      if (msg.type === 'measureRangeAudioErr') {
        if (msg.id !== latestMeasureAudioIdRef.current) return
        setMeasureAudioGenerating(false)
        return
      }

      if (msg.type === 'highlightRangeOk') {
        if (msg.id !== latestHighlightRenderIdRef.current) return
        setHighlightedSvgs(msg.svgs)
        return
      }

      if (msg.type === 'highlightRangeErr') {
        if (msg.id !== latestHighlightRenderIdRef.current) return
        return
      }

      if (msg.type === 'measureSpans') {
        if (msg.id !== latestMeasureSpansIdRef.current) return
        if (msg.status === 'ok') {
          setMeasureSpans(msg.spans)
        }
        return
      }

      if (msg.type === 'err') {
        if (msg.id !== latestRenderIdRef.current) return
        setRendering(false)
        setDiagnostics(msg.diagnostics)
      }
    }

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

  // biome-ignore lint/correctness/useExhaustiveDependencies: activeFile is intentional trigger
  useEffect(() => {
    setSvgs([])
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
  }, [source, enabledTracks, disabledLyricsTracks])

  useEffect(() => {
    if (!audioAvailable) return

    const worker = workerRef.current
    if (!worker) return

    const timer = window.setTimeout(() => {
      const id = ++audioRequestIdRef.current
      latestAudioIdRef.current = id
      setAudioGenerating(true)

      const payload: WorkerRequest = {
        type: 'generateAudio',
        source,
        id,
        enabledTracks,
      }
      worker.postMessage(payload)
    }, debounceMs)

    return () => window.clearTimeout(timer)
  }, [source, enabledTracks, debounceMs, audioAvailable])

  const notifySelection = useCallback(
    (startOffset: number, endOffset: number) => {
      lastSelectionRef.current = { start: startOffset, end: endOffset }
      if (cursorOffsetTimerRef.current !== null) {
        window.clearTimeout(cursorOffsetTimerRef.current)
      }
      cursorOffsetTimerRef.current = window.setTimeout(() => {
        cursorOffsetTimerRef.current = null
        setSelectedMeasureRange(
          measureRangeInSpan(measureSpansRef.current, startOffset, endOffset),
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
      setHighlightedSvgs([])
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
      enabledTracks: enabledTracksRef.current,
      disabledLyrics: disabledLyricsRef.current,
    } satisfies WorkerRequest)
  }, [selectedMeasureRange])

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

  return {
    parts,
    partsLoading,
    svgs,
    wavUrl,
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    splitPdfExporting,
    diagnostics,
    rendering,
    audioGenerating,
    exportPdf,
    exportSplitPdf,
    selectedMeasureRange,
    measureAudioGenerating,
    notifySelection,
    playSelectedMeasures,
    highlightedSvgs,
    measureSpans,
  }
}
