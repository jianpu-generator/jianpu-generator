import { useCallback, useEffect, useState } from 'react'
import { DEMO_FILE_NAME, type FileStoreState } from '../fileStore'
import { manifestAndFilesToFileStore } from '../github/manifest'

const EMPTY_GITHUB_STORE: FileStoreState = {
  active: DEMO_FILE_NAME,
  userFiles: {},
  bin: {},
  fileIds: {},
}

interface GitHubStoreResponse {
  manifest: {
    active: string
    fileIds: Record<string, string>
    bin: string[]
  }
  scoreFiles: Record<string, string>
  binFiles: Record<string, string>
}

export interface GitHubFileStoreMeta {
  loading: boolean
  error: string | null
  refresh: () => Promise<void>
}

type SetFileStore = (
  value: FileStoreState | ((previous: FileStoreState) => FileStoreState),
) => void

export function useGitHubFileStore(
  connected: boolean,
): [FileStoreState, SetFileStore, GitHubFileStoreMeta] {
  const [store, setStoreState] = useState<FileStoreState>(EMPTY_GITHUB_STORE)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    if (!connected) {
      setStoreState(EMPTY_GITHUB_STORE)
      setError(null)
      setLoading(false)
      return
    }

    setLoading(true)
    setError(null)
    try {
      const response = await fetch('/api/github/store')
      if (!response.ok) {
        const payload = (await response.json().catch(() => null)) as {
          error?: string
        } | null
        throw new Error(payload?.error ?? 'Failed to load GitHub store')
      }

      const payload = (await response.json()) as GitHubStoreResponse
      setStoreState(
        manifestAndFilesToFileStore(
          payload.manifest,
          payload.scoreFiles,
          payload.binFiles,
        ),
      )
    } catch (loadError) {
      setError(
        loadError instanceof Error
          ? loadError.message
          : 'Failed to load GitHub store',
      )
    } finally {
      setLoading(false)
    }
  }, [connected])

  useEffect(() => {
    void refresh()
  }, [refresh])

  const setStore = useCallback<SetFileStore>((value) => {
    setStoreState((previous) =>
      typeof value === 'function' ? value(previous) : value,
    )
  }, [])

  return [store, setStore, { loading, error, refresh }]
}
