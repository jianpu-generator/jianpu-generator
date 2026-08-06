import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { WorkerRequest } from '../worker/jianpu.worker'

interface UseJianpuWorkerAudioActionsParams {
  workerRef: RefObject<Worker | null>
  sourceRef: RefObject<string>
  enabledTracksRef: RefObject<string[] | undefined>
  wavUrlRef: RefObject<string | null>
  setWavUrl: (next: string | null) => void
  audioGenerating: boolean
  setAudioGenerating: (generating: boolean) => void
  audioRequestIdRef: RefObject<number>
  latestAudioIdRef: RefObject<number>
}

/** Revokes/replaces the current full-score preview WAV URL, and sends the
 * "generateAudio" request to the worker. */
export function useJianpuWorkerAudioActions({
  workerRef,
  sourceRef,
  enabledTracksRef,
  wavUrlRef,
  setWavUrl,
  audioGenerating,
  setAudioGenerating,
  audioRequestIdRef,
  latestAudioIdRef,
}: UseJianpuWorkerAudioActionsParams) {
  const setNextWavUrl = useCallback(
    (next: string | null) => {
      if (wavUrlRef.current) {
        URL.revokeObjectURL(wavUrlRef.current)
      }
      wavUrlRef.current = next
      setWavUrl(next)
    },
    [wavUrlRef, setWavUrl],
  )

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
  }, [
    audioGenerating,
    enabledTracksRef,
    workerRef,
    sourceRef,
    setAudioGenerating,
    latestAudioIdRef,
    audioRequestIdRef,
  ])

  return { setNextWavUrl, generateFullAudio }
}
