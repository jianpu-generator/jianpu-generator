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
  const [measureAudioElement, setMeasureAudioElement] =
    useState<HTMLAudioElement | null>(null)
  const currentMeasureAudioRef = useRef<HTMLAudioElement | null>(null)
  const measureAudioRequestIdRef = useRef(0)
  const latestMeasureAudioIdRef = useRef(0)
  const measureWavUrlRef = useRef<string | null>(null)

  const setNextMeasureWavUrl = useCallback(
    (next: string | null, nextMeasureTimes: number[] = []) => {
      if (currentMeasureAudioRef.current) {
        currentMeasureAudioRef.current.pause()
        currentMeasureAudioRef.current = null
      }
      if (measureWavUrlRef.current) {
        URL.revokeObjectURL(measureWavUrlRef.current)
      }
      measureWavUrlRef.current = next
      setMeasureAudioTimes(nextMeasureTimes)
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
    (startMeasureIndex: number, endMeasureIndex: number) => {
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
        enabledTracks: enabledTracksRef.current,
      } satisfies WorkerRequest)
    },
    [],
  )

  const playSelectedMeasures = useCallback(() => {
    if (selectedMeasureRange === null) return
    playMeasureRange(selectedMeasureRange.start, selectedMeasureRange.end)
  }, [selectedMeasureRange, playMeasureRange])

  const playFromCurrentMeasure = useCallback(() => {
    if (selectedMeasureRange === null || measureSpans.length === 0) return
    playMeasureRange(selectedMeasureRange.start, measureSpans.length - 1)
  }, [selectedMeasureRange, measureSpans, playMeasureRange])

  return {
    measureAudioGenerating,
    setMeasureAudioGenerating,
    measureAudioPlaying,
    measureAudioTimes,
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
