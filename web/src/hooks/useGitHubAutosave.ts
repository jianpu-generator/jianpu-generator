import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from 'react'
import type { FileStoreState } from '../fileStore'
import { buildSyncPlan, executeSyncPlan, syncPlanIsEmpty } from '../github/sync'

export type SyncStatus = 'idle' | 'saving' | 'saved' | 'error'

export interface GitHubAutosaveState {
  status: SyncStatus
  error: string | null
}

const DEBOUNCE_MS = 1500
const SAVED_DISPLAY_MS = 2000

export function useGitHubAutosave(
  enabled: boolean,
  store: FileStoreState,
  baselineToken: number,
  loading: boolean,
): GitHubAutosaveState {
  const [status, setStatus] = useState<SyncStatus>('idle')
  const [error, setError] = useState<string | null>(null)
  const baselineRef = useRef<FileStoreState>(store)
  const storeRef = useRef(store)
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const savedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const savingRef = useRef(false)
  const prevLoadingRef = useRef(loading)

  storeRef.current = store

  const clearDebounce = useCallback(() => {
    if (debounceTimerRef.current !== null) {
      clearTimeout(debounceTimerRef.current)
      debounceTimerRef.current = null
    }
  }, [])

  const clearSavedTimer = useCallback(() => {
    if (savedTimerRef.current !== null) {
      clearTimeout(savedTimerRef.current)
      savedTimerRef.current = null
    }
  }, [])

  const resetBaseline = useCallback(
    (nextStore: FileStoreState) => {
      baselineRef.current = nextStore
      clearDebounce()
      clearSavedTimer()
      setStatus('idle')
      setError(null)
    },
    [clearDebounce, clearSavedTimer],
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: baselineToken triggers reset after workspace pull
  useLayoutEffect(() => {
    resetBaseline(storeRef.current)
  }, [baselineToken, resetBaseline])

  useLayoutEffect(() => {
    const finishedLoading = prevLoadingRef.current && !loading
    prevLoadingRef.current = loading
    if (finishedLoading && enabled) {
      resetBaseline(storeRef.current)
    }
  }, [loading, enabled, resetBaseline])

  useEffect(() => {
    if (!enabled) {
      clearDebounce()
      clearSavedTimer()
      setStatus('idle')
      setError(null)
    }
  }, [enabled, clearDebounce, clearSavedTimer])

  const flush = useCallback(async () => {
    if (!enabled || savingRef.current) {
      return
    }

    savingRef.current = true
    try {
      while (enabled) {
        const baseline = baselineRef.current
        const current = storeRef.current
        const plan = buildSyncPlan(baseline, current)

        if (syncPlanIsEmpty(plan)) {
          setStatus('idle')
          setError(null)
          return
        }

        setStatus('saving')
        setError(null)

        try {
          await executeSyncPlan(plan)
        } catch (syncError) {
          setStatus('error')
          setError(
            syncError instanceof Error
              ? syncError.message
              : 'Failed to sync with GitHub',
          )
          return
        }

        baselineRef.current = current
        const followUp = buildSyncPlan(current, storeRef.current)
        if (!syncPlanIsEmpty(followUp)) {
          continue
        }

        setStatus('saved')
        clearSavedTimer()
        savedTimerRef.current = setTimeout(() => {
          setStatus('idle')
          savedTimerRef.current = null
        }, SAVED_DISPLAY_MS)
        return
      }
    } finally {
      savingRef.current = false
    }
  }, [enabled, clearSavedTimer])

  useEffect(() => {
    if (!enabled || loading) {
      return
    }

    const plan = buildSyncPlan(baselineRef.current, store)
    if (syncPlanIsEmpty(plan)) {
      return
    }

    clearDebounce()
    debounceTimerRef.current = setTimeout(() => {
      debounceTimerRef.current = null
      void flush()
    }, DEBOUNCE_MS)

    return clearDebounce
  }, [enabled, loading, store, flush, clearDebounce])

  useEffect(
    () => () => {
      clearDebounce()
      clearSavedTimer()
    },
    [clearDebounce, clearSavedTimer],
  )

  return { status, error }
}
