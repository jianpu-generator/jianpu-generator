import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { EditorSelection, RangeEditOperation } from '../types'
import type { WorkerRequest } from '../worker/jianpu.worker'
import type { RangeEditRequestTracker } from './useJianpuWorkerTypes'

interface UseJianpuWorkerEditRangeParams {
  workerRef: RefObject<Worker | null>
  sourceRef: RefObject<string>
  editRangeTracker: RangeEditRequestTracker
}

/** Sends a selection-scoped edit request (`operation`: octave shift or
 * slur toggle) for a disjoint set of byte ranges (a Monaco multicursor
 * selection, e.g. every measure a clicked part label's notes span, need not
 * be one contiguous range) to the worker and resolves once it replies with
 * the rewritten `.jianpu` source *and* the input ranges remapped onto that
 * new source. The `ranges` half is what lets the caller restore the editor
 * selection synchronously alongside the new source (see
 * `HANDOFF-octave-toolbar-part-label-selection-bug.md`). */
export function useJianpuWorkerEditRange({
  workerRef,
  sourceRef,
  editRangeTracker,
}: UseJianpuWorkerEditRangeParams) {
  const editRange = useCallback(
    (ranges: EditorSelection[], operation: RangeEditOperation) =>
      new Promise<{ source: string; ranges: EditorSelection[] }>((resolve) => {
        const worker = workerRef.current
        if (!worker) {
          resolve({ source: sourceRef.current, ranges: [] })
          return
        }
        const id = ++editRangeTracker.requestIdRef.current
        editRangeTracker.latestIdRef.current = id
        editRangeTracker.pendingRequestsRef.current.set(id, resolve)
        worker.postMessage({
          type: 'editRange',
          source: sourceRef.current,
          ranges,
          operation,
          id,
        } satisfies WorkerRequest)
      }),
    [workerRef, sourceRef, editRangeTracker],
  )

  return { editRange }
}
