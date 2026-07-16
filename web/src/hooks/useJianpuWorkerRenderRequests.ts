import type { SvgDocumentOut } from 'jianpu-wasm'
import type { RefObject } from 'react'
import { useCallback, useEffect } from 'react'
import type { Diagnostic, MeasureSpan } from '../types'
import type { WorkerRequest } from '../worker/jianpu.worker'
import { measureRangeInSpan } from './workerHelpers'

interface UseJianpuWorkerRenderRequestsParams {
  workerRef: RefObject<Worker | null>
  sourceRef: RefObject<string>
  source: string
  activeFile: string
  debounceMs: number
  enabledTracks: string[] | undefined
  disabledLyricsTracks: string[] | undefined
  setDocuments: (value: SvgDocumentOut[]) => void
  setNextWavUrl: (value: string | null) => void
  setDiagnostics: (value: Diagnostic[]) => void
  setPartsLoading: (value: boolean) => void
  partsRequestIdRef: RefObject<number>
  latestPartsIdRef: RefObject<number>
  setRendering: (value: boolean) => void
  renderRequestIdRef: RefObject<number>
  latestRenderIdRef: RefObject<number>
  selectedMeasureRange: { start: number; end: number } | null
  setSelectedMeasureRange: (
    value: { start: number; end: number } | null,
  ) => void
  setHighlightedDocuments: (value: SvgDocumentOut[]) => void
  highlightRenderRequestIdRef: RefObject<number>
  latestHighlightRenderIdRef: RefObject<number>
  measureSpans: MeasureSpan[]
  measureSpansRef: RefObject<MeasureSpan[]>
  measureSpansRequestIdRef: RefObject<number>
  latestMeasureSpansIdRef: RefObject<number>
  cursorOffsetTimerRef: RefObject<number | null>
  lastSelectionRef: RefObject<{ start: number; end: number } | null>
}

/** Debounced worker requests that keep parts, rendered documents, highlighted documents and
 * measure spans in sync with the current source, plus the selection-to-measure-range mapping. */
export function useJianpuWorkerRenderRequests({
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
}: UseJianpuWorkerRenderRequestsParams) {
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

  // biome-ignore lint/correctness/useExhaustiveDependencies: workerRef/partsRequestIdRef/latestPartsIdRef are stable refs passed in as params
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

  // biome-ignore lint/correctness/useExhaustiveDependencies: activeFile triggers re-render after rename (content unchanged but activeFile changes); workerRef/renderRequestIdRef/latestRenderIdRef are stable refs passed in as params
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

  // biome-ignore lint/correctness/useExhaustiveDependencies: lastSelectionRef/cursorOffsetTimerRef/measureSpansRef are stable refs passed in as params
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

  // biome-ignore lint/correctness/useExhaustiveDependencies: lastSelectionRef is a stable ref passed in as a param
  useEffect(() => {
    const sel = lastSelectionRef.current
    if (!sel) return
    setSelectedMeasureRange(
      measureRangeInSpan(measureSpans, sel.start, sel.end),
    )
  }, [measureSpans])

  // biome-ignore lint/correctness/useExhaustiveDependencies: workerRef/sourceRef/highlightRenderRequestIdRef/latestHighlightRenderIdRef are stable refs passed in as params
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

  // biome-ignore lint/correctness/useExhaustiveDependencies: workerRef/measureSpansRequestIdRef/latestMeasureSpansIdRef are stable refs passed in as params
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

  return { notifySelection }
}
