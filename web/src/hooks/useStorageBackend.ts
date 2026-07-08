import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useDebouncedCallback } from 'use-debounce'
import { useLocalStorage } from 'usehooks-ts'
import {
  DEMO_FILE_NAME,
  FILE_STORE_KEY,
  type FileStoreState,
  fileContent,
} from '../fileStore'
import { useGithubAuthToken } from '../storage/githubAuth'
import { createGithubBackend } from '../storage/githubBackend'
import {
  deserializeStoreSync,
  localBackend,
  readInitialStoreSync,
} from '../storage/localBackend'
import type { SaveStatus, StorageBackend } from '../storage/types'

/**
 * Idle interval at which a changed active file's content is flushed to the
 * backend via `saveContent()`. Shared by both backends — immaterial for
 * `localBackend` (whose `saveContent` is a no-op, since `useLocalStorage`
 * already persists every state change) — so there is a single autosave
 * cadence instead of a redundant per-backend config knob. Chosen to keep
 * GitHub commit frequency low while a file is being actively edited.
 */
export const AUTOSAVE_DEBOUNCE_MS = 20_000

/**
 * Fixed name of the repo the GitHub backend always targets, under the
 * authenticated user's own account. See `StorageSettingsModal.tsx`'s
 * `ensureStorageRepo` for the get-or-create logic that ensures it exists.
 */
export const GITHUB_STORAGE_REPO = 'jianpu-generator-storage'

const STORAGE_BACKEND_PREFERENCE_KEY = 'jianpu:storage-backend:v1'

/**
 * Persisted choice of backend plus the GitHub-specific connection details
 * needed to reconstruct a `GithubBackend` synchronously on reload. The OAuth
 * token itself is stored separately under `githubAuth.ts`'s own key so it
 * can be shared with `checkGithubAuthStatus`/`useGithubAuthToken`.
 */
export interface StorageBackendPreference {
  backend: 'local' | 'github'
  github?: { owner: string }
}

const DEFAULT_PREFERENCE: StorageBackendPreference = { backend: 'local' }

/** Placeholder `FileStoreState` shown for the brief window between
 * selecting the GitHub backend and `backend.load()` resolving. Identical in
 * shape to `fileStore.ts`'s own `DEFAULT_FILE_STORE` (not exported from
 * there, so reconstructed here). */
const EMPTY_STORE: FileStoreState = {
  active: DEMO_FILE_NAME,
  userFiles: {},
  bin: {},
  fileIds: {},
}

export type StorageBackendTarget =
  | { kind: 'local' }
  | { kind: 'github'; owner: string }

export interface UseStorageBackendResult {
  store: FileStoreState
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void
  backend: StorageBackend
  saveStatus: SaveStatus
  /** Currently persisted backend choice, exposed so `StorageSettingsModal`
   * can reflect the active selection without re-deriving it from
   * `backend.kind` alone. */
  preference: StorageBackendPreference
  /**
   * Switches the active backend. If leaving GitHub with a pending debounced
   * save, forces it to flush (and awaits it) before switching. Always lands
   * on the demo file — there is no per-backend "last active file" memory.
   */
  switchBackend: (target: StorageBackendTarget) => Promise<void>
  /**
   * Immediately persists the active file, bypassing the autosave debounce
   * interval. Wired to Ctrl/Cmd+S so users aren't stuck waiting out
   * `AUTOSAVE_DEBOUNCE_MS` when they explicitly ask to save. A no-op for
   * `localBackend` (already persisted synchronously by `useLocalStorage`),
   * but harmless to call unconditionally.
   */
  forceSave: () => void
  /**
   * Flushes a pending debounced save without cancelling the debounce timer
   * itself (unlike `forceSave`, which also bypasses scheduling a new one).
   * Wired to file-tab switches so an edit made just before switching away
   * from a file is persisted immediately instead of waiting out
   * `AUTOSAVE_DEBOUNCE_MS` — `shouldScheduleAutosave` deliberately does not
   * schedule a new save on such switches, so this is the only flush path
   * for that case. A no-op for `localBackend` or when nothing is pending.
   */
  flushPendingSave: () => void
}

/**
 * Decides whether an autosave should be scheduled for a `store` transition.
 * Pure (no backend/React access) so it's unit-testable in isolation: a save
 * is only warranted when the *same* active file's content actually changed
 * — not merely when the user switched which file is active (whose content
 * naturally differs from the previously active file's) — and never for a
 * non-GitHub backend, since only GitHub's `saveContent` does real work. A
 * switch away from a file with a still-*pending* debounced save is not
 * covered by "already saved" — that's instead handled by `flushPendingSave`,
 * which callers should invoke before changing the active file.
 */
export function shouldScheduleAutosave(
  backendKind: StorageBackend['kind'],
  previous: { active: string; content: string } | null,
  next: { active: string; content: string },
): boolean {
  if (backendKind !== 'github') return false
  if (!previous) return false
  return previous.active === next.active && previous.content !== next.content
}

/**
 * Decides whether the `beforeunload` handler should warn about unsaved
 * changes. Pure for the same testability reason as `shouldScheduleAutosave`.
 * True in two disjoint windows: `isPending` (a debounced save is armed but
 * hasn't fired yet) and `saveStatus === 'saving'` (it fired — via the debounce
 * timer, `flush()`, or `forceSave()` — but the underlying multi-request
 * Octokit call hasn't resolved). Never true for `localBackend`, which has no
 * debounce and no in-flight network call to lose.
 */
export function shouldWarnBeforeUnload(
  backendKind: StorageBackend['kind'],
  isPending: boolean,
  saveStatus: SaveStatus,
): boolean {
  if (backendKind !== 'github') return false
  return isPending || saveStatus === 'saving'
}

/**
 * Holds the file store's in-memory state and wires it to a `StorageBackend`,
 * switchable between `local` and `github`. `store`/`setStore` keep the same
 * ergonomics `useFileStore` used to expose, so callers still perform sync
 * updates (e.g. selecting a file, or applying `backend.updateActiveContent`)
 * via `setStore`. Structural operations (create/duplicate/rename/delete/
 * restore) are modeled as async on `StorageBackend`, so callers
 * `await backend.xxxFile(store)` and then reconcile the result into the
 * latest state with `fileStore.ts`'s `mergeBackendResult` via a functional
 * `setStore(prev => ...)` update — never a plain `setStore(next)`, which
 * would discard any edits made to `prev` while the await was in flight.
 * Those calls hit the backend immediately, unlike content edits (see
 * `AUTOSAVE_DEBOUNCE_MS`).
 */
export function useStorageBackend(): UseStorageBackendResult {
  const [preference, setPreference] = useLocalStorage<StorageBackendPreference>(
    STORAGE_BACKEND_PREFERENCE_KEY,
    DEFAULT_PREFERENCE,
  )
  const [authToken] = useGithubAuthToken()

  const [localStore, setLocalStore] = useLocalStorage<FileStoreState>(
    FILE_STORE_KEY,
    readInitialStoreSync,
    { deserializer: deserializeStoreSync },
  )
  const [githubStore, setGithubStore] = useState<FileStoreState | null>(null)
  const [saveStatus, setSaveStatus] = useState<SaveStatus>('idle')

  const backend = useMemo<StorageBackend>(() => {
    if (preference.backend === 'github' && authToken && preference.github) {
      return createGithubBackend({
        token: authToken.token,
        owner: preference.github.owner,
        repo: GITHUB_STORAGE_REPO,
      })
    }
    return localBackend
  }, [preference, authToken])

  // (Re)loads the GitHub listing whenever the backend identity changes
  // (kind, owner, or token) — exactly when a fresh listing is
  // needed. `localBackend`'s state instead lives in `localStore` above,
  // seeded synchronously, so no such effect is needed for it.
  useEffect(() => {
    if (backend.kind !== 'github') return
    let cancelled = false
    setGithubStore(null)
    backend.load().then((state) => {
      if (!cancelled) setGithubStore(state)
    })
    return () => {
      cancelled = true
    }
  }, [backend])

  useEffect(() => {
    setSaveStatus(backend.status())
  }, [backend])

  const store =
    backend.kind === 'github' ? (githubStore ?? EMPTY_STORE) : localStore

  const setStore = useCallback(
    (value: FileStoreState | ((prev: FileStoreState) => FileStoreState)) => {
      if (backend.kind === 'github') {
        setGithubStore((prev) => {
          const base = prev ?? EMPTY_STORE
          return typeof value === 'function' ? value(base) : value
        })
      } else {
        setLocalStore(value)
      }
    },
    [backend, setLocalStore],
  )

  const pendingSaveRef = useRef<Promise<void> | null>(null)
  const runSave = useCallback(
    (state: FileStoreState) => {
      setSaveStatus('saving')
      const promise = backend
        .saveContent(state)
        .then(() => {
          const status = backend.status()
          setSaveStatus(status === 'idle' ? 'saved' : status)
        })
        .catch(() => {
          setSaveStatus(backend.status())
        })
      pendingSaveRef.current = promise
    },
    [backend],
  )
  const debouncedSave = useDebouncedCallback(runSave, AUTOSAVE_DEBOUNCE_MS)

  const lastContentRef = useRef<{ active: string; content: string } | null>(
    null,
  )
  useEffect(() => {
    const next = {
      active: store.active,
      content: fileContent(store, store.active),
    }
    if (shouldScheduleAutosave(backend.kind, lastContentRef.current, next)) {
      debouncedSave(store)
    }
    lastContentRef.current = next
  }, [store, backend, debouncedSave])

  // Flushes a pending debounced save on blur/tab-hide so edits made just
  // before closing the tab aren't lost waiting for the idle interval.
  useEffect(() => {
    const flush = () => debouncedSave.flush()
    window.addEventListener('blur', flush)
    document.addEventListener('visibilitychange', flush)
    return () => {
      window.removeEventListener('blur', flush)
      document.removeEventListener('visibilitychange', flush)
    }
  }, [debouncedSave])

  // Warns before closing/reloading the tab if a GitHub save hasn't landed
  // yet — the blur/visibilitychange flush above only *starts* the save; it
  // can't guarantee the underlying multi-request Octokit call (fetch sha,
  // then PUT) finishes before the page actually unloads. Native
  // confirmation is the only backstop for that gap. No-op (and thus no
  // dialog) whenever nothing is pending or in flight, including for
  // `localBackend`, which never has a pending save.
  useEffect(() => {
    const handler = (event: BeforeUnloadEvent) => {
      if (
        shouldWarnBeforeUnload(
          backend.kind,
          debouncedSave.isPending(),
          saveStatus,
        )
      ) {
        event.preventDefault()
      }
    }
    window.addEventListener('beforeunload', handler)
    return () => window.removeEventListener('beforeunload', handler)
  }, [backend, debouncedSave, saveStatus])

  const forceSave = useCallback(() => {
    debouncedSave.cancel()
    runSave(store)
  }, [debouncedSave, runSave, store])

  const flushPendingSave = useCallback(() => {
    if (backend.kind === 'github' && debouncedSave.isPending()) {
      debouncedSave.flush()
    }
  }, [backend, debouncedSave])

  const switchBackend = useCallback(
    async (target: StorageBackendTarget) => {
      if (backend.kind === 'github' && debouncedSave.isPending()) {
        debouncedSave.flush()
        await pendingSaveRef.current
      }
      if (target.kind === 'local') {
        setPreference({ backend: 'local' })
        setLocalStore((prev) => ({ ...prev, active: DEMO_FILE_NAME }))
      } else {
        setPreference({
          backend: 'github',
          github: { owner: target.owner },
        })
      }
    },
    [backend, debouncedSave, setPreference, setLocalStore],
  )

  return {
    store,
    setStore,
    backend,
    saveStatus,
    preference,
    switchBackend,
    forceSave,
    flushPendingSave,
  }
}
