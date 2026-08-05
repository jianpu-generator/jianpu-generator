import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { WorkerRequest } from '../worker/jianpu.worker'

interface UseJianpuWorkerFormatParams {
  workerRef: RefObject<Worker | null>
  formatScoreRequestIdRef: RefObject<number>
  latestFormatScoreIdRef: RefObject<number>
  pendingFormatScoreRequestsRef: RefObject<
    Map<number, (source: string) => void>
  >
}

/** Sends the Zipped-view "Format" request to the worker and resolves once it
 * replies with the reformatted `.jianpu` source. */
export function useJianpuWorkerFormat({
  workerRef,
  formatScoreRequestIdRef,
  latestFormatScoreIdRef,
  pendingFormatScoreRequestsRef,
}: UseJianpuWorkerFormatParams) {
  const formatScore = useCallback(
    (source: string) =>
      new Promise<string>((resolve) => {
        const worker = workerRef.current
        if (!worker) {
          resolve(source)
          return
        }
        const id = ++formatScoreRequestIdRef.current
        latestFormatScoreIdRef.current = id
        pendingFormatScoreRequestsRef.current.set(id, resolve)
        worker.postMessage({
          type: 'formatScore',
          source,
          id,
        } satisfies WorkerRequest)
      }),
    [
      workerRef,
      formatScoreRequestIdRef,
      latestFormatScoreIdRef,
      pendingFormatScoreRequestsRef,
    ],
  )

  return { formatScore }
}
