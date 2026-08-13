import type {
  LyricsVerseRangesOut,
  PartMeasureRangesOut,
  SvgDocumentOut,
} from 'jianpu-wasm'
import { extract_unzipped_text } from 'jianpu-wasm'
import type { RefObject } from 'react'
import { useCallback, useEffect } from 'react'
import type { Diagnostic, MeasureSpan } from '../types'
import { ensureWasmInit } from '../wasmInit'
import type { WorkerRequest } from '../worker/jianpu.worker'
import { measureRangeInSpan, measureRangeInUnzippedText } from './workerHelpers'

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
  noteSpansRequestIdRef: RefObject<number>
  latestNoteSpansIdRef: RefObject<number>
  cursorOffsetTimerRef: RefObject<number | null>
  lastSelectionRef: RefObject<{
    start: number
    end: number
    mode: 'source' | 'unzipped'
  } | null>
  /** Whether the whole-document Unzipped view is currently shown; gates the
   * `extract_unzipped_text` re-fetch below. */
  unzippedView: boolean
  partMeasureRangesRef: RefObject<PartMeasureRangesOut[]>
  setPartMeasureRanges: (value: PartMeasureRangesOut[]) => void
  lyricsVerseRangesRef: RefObject<LyricsVerseRangesOut[]>
  setLyricsVerseRanges: (value: LyricsVerseRangesOut[]) => void
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
  noteSpansRequestIdRef,
  latestNoteSpansIdRef,
  cursorOffsetTimerRef,
  lastSelectionRef,
  unzippedView,
  partMeasureRangesRef,
  setPartMeasureRanges,
  lyricsVerseRangesRef,
  setLyricsVerseRanges,
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

  // Re-derives `partMeasureRanges` (used for stale-but-tolerated
  // cursor->measure highlighting in Unzipped view) on every source change.
  // This intentionally does NOT feed the Unzipped editor's displayed text —
  // that comes from a one-time snapshot taken when Unzipped view is
  // switched on (see `useJianpuWorker`'s `unzippedText` snapshot effect) so
  // that in-progress edits aren't clobbered by rest-padding from repeated
  // merge/re-extract cycles while the user is still typing.
  // biome-ignore lint/correctness/useExhaustiveDependencies: setPartMeasureRanges is a stable setState function passed in as a param
  useEffect(() => {
    if (!unzippedView) return
    let cancelled = false
    const timer = window.setTimeout(() => {
      ensureWasmInit().then(() => {
        if (cancelled) return
        const result = extract_unzipped_text(source)
        setPartMeasureRanges(
          result.status === 'ok' ? result.part_measure_ranges : [],
        )
        setLyricsVerseRanges(
          result.status === 'ok' ? result.lyrics_verse_ranges : [],
        )
      })
    }, debounceMs)
    return () => {
      cancelled = true
      window.clearTimeout(timer)
    }
  }, [source, unzippedView, debounceMs])

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
      lastSelectionRef.current = {
        start: startLine,
        end: endLine,
        mode: 'source',
      }
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

  // Unzipped view text's byte offsets map to measure indices via
  // `partMeasureRanges`, not through the full source's lines/byte offsets.
  // biome-ignore lint/correctness/useExhaustiveDependencies: lastSelectionRef/cursorOffsetTimerRef/partMeasureRangesRef are stable refs passed in as params
  const notifyUnzippedSelection = useCallback(
    (startOffset: number, endOffset: number) => {
      lastSelectionRef.current = {
        start: startOffset,
        end: endOffset,
        mode: 'unzipped',
      }
      if (cursorOffsetTimerRef.current !== null) {
        window.clearTimeout(cursorOffsetTimerRef.current)
      }
      cursorOffsetTimerRef.current = window.setTimeout(() => {
        cursorOffsetTimerRef.current = null
        const startRange = measureRangeInUnzippedText(
          partMeasureRangesRef.current,
          startOffset,
          lyricsVerseRangesRef.current,
        )
        const endRange = measureRangeInUnzippedText(
          partMeasureRangesRef.current,
          endOffset,
          lyricsVerseRangesRef.current,
        )
        if (startRange === null || endRange === null) {
          setSelectedMeasureRange(null)
          return
        }
        setSelectedMeasureRange({
          start: Math.min(startRange.start, endRange.start),
          end: Math.max(startRange.end, endRange.end),
        })
      }, debounceMs)
    },
    [debounceMs],
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: lastSelectionRef/partMeasureRangesRef are stable refs passed in as params
  useEffect(() => {
    const sel = lastSelectionRef.current
    if (!sel) return
    if (sel.mode === 'unzipped') {
      const startRange = measureRangeInUnzippedText(
        partMeasureRangesRef.current,
        sel.start,
        lyricsVerseRangesRef.current,
      )
      const endRange = measureRangeInUnzippedText(
        partMeasureRangesRef.current,
        sel.end,
        lyricsVerseRangesRef.current,
      )
      setSelectedMeasureRange(
        startRange === null || endRange === null
          ? null
          : {
              start: Math.min(startRange.start, endRange.start),
              end: Math.max(startRange.end, endRange.end),
            },
      )
    } else {
      setSelectedMeasureRange(
        measureRangeInSpan(measureSpans, sel.start, sel.end),
      )
    }
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

  // biome-ignore lint/correctness/useExhaustiveDependencies: workerRef/noteSpansRequestIdRef/latestNoteSpansIdRef are stable refs passed in as params
  useEffect(() => {
    const worker = workerRef.current
    if (!worker) return

    const id = ++noteSpansRequestIdRef.current
    latestNoteSpansIdRef.current = id

    const timer = window.setTimeout(() => {
      worker.postMessage({
        type: 'listNoteSpans',
        source,
        id,
      } satisfies WorkerRequest)
    }, debounceMs)

    return () => window.clearTimeout(timer)
  }, [source, debounceMs])

  return { notifySelection, notifyUnzippedSelection }
}
