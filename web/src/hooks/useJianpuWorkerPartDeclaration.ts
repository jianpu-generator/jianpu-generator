import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { PartMode } from '../types'
import type { WorkerRequest } from '../worker/jianpu.worker'

interface UseJianpuWorkerPartDeclarationParams {
  workerRef: RefObject<Worker | null>
  sourceRef: RefObject<string>
  updatePartDeclarationRequestIdRef: RefObject<number>
  latestUpdatePartDeclarationIdRef: RefObject<number>
  pendingPartDeclarationUpdatesRef: RefObject<
    Map<number, (source: string) => void>
  >
}

/** Sends a part-declaration edit (mode, follow target, soundfont, volume,
 * octave offset) to the worker and resolves once it replies with the
 * rewritten `.jianpu` source. */
export function useJianpuWorkerPartDeclaration({
  workerRef,
  sourceRef,
  updatePartDeclarationRequestIdRef,
  latestUpdatePartDeclarationIdRef,
  pendingPartDeclarationUpdatesRef,
}: UseJianpuWorkerPartDeclarationParams) {
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
    [
      workerRef,
      sourceRef,
      updatePartDeclarationRequestIdRef,
      latestUpdatePartDeclarationIdRef,
      pendingPartDeclarationUpdatesRef,
    ],
  )

  return { updatePartDeclaration }
}
