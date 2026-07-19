import type { NoteTimingOut } from 'jianpu-wasm'
import type { RefObject } from 'react'
import { useCallback, useRef, useState } from 'react'
import type { MeasureSpan } from '../types'
import type { WorkerRequest } from '../worker/jianpu.worker'

interface UseMeasureAudioPlaybackParams {
  workerRef: RefObject<Worker | null>
  sourceRef: RefObject<string>
  enabledTracksRef: RefObject<string[] | undefined>
  selectedMeasureRange: { start: number; end: number } | null
  measureSpans: MeasureSpan[]
}

/** Manages generating and playing back audio for a range of measures (e.g. the currently selected measures). */
export function useMeasureAudioPlayback({
  workerRef,
  sourceRef,
  enabledTracksRef,
  selectedMeasureRange,
  measureSpans,
}: UseMeasureAudioPlaybackParams) {
  const [measureAudioGenerating, setMeasureAudioGenerating] = useState(false)
  const [measureAudioPlaying, setMeasureAudioPlaying] = useState(false)
  const [measureAudioTimes, setMeasureAudioTimes] = useState<number[]>([])
  const [measureAudioWrittenIndices, setMeasureAudioWrittenIndices] = useState<
    number[]
  >([])
  const [measureAudioNoteTimings, setMeasureAudioNoteTimings] = useState<
    NoteTimingOut[]
  >([])
  const [measureAudioElement, setMeasureAudioElement] =
    useState<HTMLAudioElement | null>(null)
  const currentMeasureAudioRef = useRef<HTMLAudioElement | null>(null)
  const measureAudioRequestIdRef = useRef(0)
  const latestMeasureAudioIdRef = useRef(0)
  const measureWavUrlRef = useRef<string | null>(null)

  const setNextMeasureWavUrl = useCallback(
    (
      next: string | null,
      nextMeasureTimes: number[] = [],
      nextWrittenIndices: number[] = [],
      nextNoteTimings: NoteTimingOut[] = [],
    ) => {
      if (currentMeasureAudioRef.current) {
        currentMeasureAudioRef.current.pause()
        currentMeasureAudioRef.current = null
      }
      if (measureWavUrlRef.current) {
        URL.revokeObjectURL(measureWavUrlRef.current)
      }
      measureWavUrlRef.current = next
      setMeasureAudioTimes(nextMeasureTimes)
      setMeasureAudioWrittenIndices(nextWrittenIndices)
      setMeasureAudioNoteTimings(nextNoteTimings)
      if (next) {
        const audio = new Audio(next)
        currentMeasureAudioRef.current = audio
        setMeasureAudioElement(audio)
        audio.addEventListener('play', () => setMeasureAudioPlaying(true))
        audio.addEventListener('ended', () => {
          setMeasureAudioPlaying(false)
          currentMeasureAudioRef.current = null
          setMeasureAudioElement(null)
        })
        audio.addEventListener('pause', () => setMeasureAudioPlaying(false))
        audio.play().catch(() => {})
      } else {
        setMeasureAudioElement(null)
      }
    },
    [],
  )

  const stopMeasurePlayback = useCallback(() => {
    if (currentMeasureAudioRef.current) {
      currentMeasureAudioRef.current.pause()
      currentMeasureAudioRef.current = null
    }
    setMeasureAudioPlaying(false)
  }, [])

  // biome-ignore lint/correctness/useExhaustiveDependencies: workerRef/sourceRef/enabledTracksRef are stable refs passed in as params
  const playMeasureRange = useCallback(
    (
      startMeasureIndex: number,
      endMeasureIndex: number,
      extendToLastOccurrence: boolean,
      respectSequence: boolean,
    ) => {
      const worker = workerRef.current
      if (!worker) return
      const id = ++measureAudioRequestIdRef.current
      latestMeasureAudioIdRef.current = id
      setMeasureAudioGenerating(true)
      worker.postMessage({
        type: 'generateMeasureRangeAudio',
        source: sourceRef.current,
        id,
        startMeasureIndex,
        endMeasureIndex,
        extendToLastOccurrence,
        respectSequence,
        enabledTracks: enabledTracksRef.current,
      } satisfies WorkerRequest)
    },
    [],
  )

  const playSelectedMeasures = useCallback(() => {
    if (selectedMeasureRange === null) return
    // Exact range: stop at the end measure's first occurrence, so a
    // single-measure selection (e.g. "play current measure") doesn't
    // overrun into a later D.C./D.S. al Coda repeat pass. Ignore
    // # sequence/D.C./D.S. entirely, so "play current measure" always plays
    // exactly what is written, regardless of any part omission a # sequence
    // entry might apply to this measure's occurrence(s).
    playMeasureRange(
      selectedMeasureRange.start,
      selectedMeasureRange.end,
      false,
      false,
    )
  }, [selectedMeasureRange, playMeasureRange])

  const playFromCurrentMeasure = useCallback(() => {
    if (selectedMeasureRange === null || measureSpans.length === 0) return
    // "Play to the end": follow every repeat/jump through to the true end
    // of the performance instead of stopping at the last written measure's
    // first occurrence, respecting # sequence/D.C./D.S. navigation.
    playMeasureRange(
      selectedMeasureRange.start,
      measureSpans.length - 1,
      true,
      true,
    )
  }, [selectedMeasureRange, measureSpans, playMeasureRange])

  return {
    measureAudioGenerating,
    setMeasureAudioGenerating,
    measureAudioPlaying,
    measureAudioTimes,
    measureAudioWrittenIndices,
    measureAudioNoteTimings,
    measureAudioElement,
    setNextMeasureWavUrl,
    stopMeasurePlayback,
    playMeasureRange,
    playSelectedMeasures,
    playFromCurrentMeasure,
    latestMeasureAudioIdRef,
    measureWavUrlRef,
  }
}
