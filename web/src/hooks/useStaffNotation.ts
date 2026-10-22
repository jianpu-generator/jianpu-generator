import type { RefObject } from 'react'
import { useCallback, useEffect, useState } from 'react'
import type { ResolvedPartVisibility } from '../types'
import type {
  Notation,
  StaffRenderResult,
  StaffScore,
  WorkerRequest,
} from '../worker/messages'
import type { StaffRenderRequestTracker } from './useJianpuWorkerTypes'

export type { Notation }

/** The jianpu/staff preview toggle plus what the staff view shows. */
export interface StaffNotation {
  notation: Notation
  setNotation: (notation: Notation) => void
  /** The engraved current source; no pages until the first staff render
   * lands. */
  score: StaffScore
  /** True while a staff render (including the first, which also downloads
   * Verovio) is in flight. */
  loading: boolean
  error: string | null
}

const EMPTY_SCORE: StaffScore = { pages: [], measures: [] }

const STORAGE_KEY = 'jianpu:preview-notation:v1'

function load(): Notation {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    return stored === 'staff' || stored === 'abc' || stored === 'guitar'
      ? stored
      : 'jianpu'
  } catch {
    return 'jianpu'
  }
}

function save(notation: Notation) {
  try {
    localStorage.setItem(STORAGE_KEY, notation)
  } catch {
    // Storage may be blocked; the toggle just won't persist.
  }
}

interface UseStaffNotationParams {
  workerRef: RefObject<Worker | null>
  source: string
  visibility: ResolvedPartVisibility
  staffRenderTracker: StaffRenderRequestTracker
  debounceMs: number
}

/** Remembers the preview's notation in this browser only (a per-viewer UI
 * preference, like `useExpandedFolders`), and while it's `staff` asks the
 * worker to engrave the current source with Verovio. Nothing is requested —
 * and Verovio is never downloaded — until the user first switches to staff. */
export function useStaffNotation({
  workerRef,
  source,
  visibility,
  staffRenderTracker,
  debounceMs,
}: UseStaffNotationParams): StaffNotation {
  const { requestIdRef, latestIdRef, pendingRequestsRef } = staffRenderTracker
  const [notation, setNotationState] = useState<Notation>(load)
  const [score, setScore] = useState<StaffScore>(EMPTY_SCORE)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const setNotation = useCallback((next: Notation) => {
    save(next)
    setNotationState(next)
  }, [])

  useEffect(() => {
    const worker = workerRef.current
    if (notation !== 'staff' || !worker) return
    setLoading(true)
    const timer = window.setTimeout(() => {
      const id = ++requestIdRef.current
      latestIdRef.current = id
      pendingRequestsRef.current.set(id, (result: StaffRenderResult) => {
        setLoading(false)
        if (result.status === 'ok') {
          setScore(result.score)
          setError(null)
          return
        }
        setError(result.message)
      })
      worker.postMessage({
        type: 'renderStaff',
        source,
        id,
        visibility,
      } satisfies WorkerRequest)
    }, debounceMs)
    return () => window.clearTimeout(timer)
  }, [
    notation,
    source,
    visibility,
    debounceMs,
    workerRef,
    requestIdRef,
    latestIdRef,
    pendingRequestsRef,
  ])

  return { notation, setNotation, score, loading, error }
}
