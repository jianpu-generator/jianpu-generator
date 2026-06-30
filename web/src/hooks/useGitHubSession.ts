import { useCallback, useEffect, useState } from 'react'
import { enableGitHubSync } from '../env'

export interface GitHubSessionState {
  connected: boolean
  username?: string
  repo?: string
}

interface GitHubSessionResult extends GitHubSessionState {
  loading: boolean
  refresh: () => Promise<void>
}

const DISCONNECTED: GitHubSessionState = { connected: false }

async function fetchSession(): Promise<GitHubSessionState> {
  const response = await fetch('/api/github/session')
  if (!response.ok) {
    throw new Error('Failed to fetch GitHub session')
  }
  return (await response.json()) as GitHubSessionState
}

export function useGitHubSession(): GitHubSessionResult {
  const [session, setSession] = useState<GitHubSessionState>(DISCONNECTED)
  const [loading, setLoading] = useState(enableGitHubSync)

  const refresh = useCallback(async () => {
    if (!enableGitHubSync) {
      setSession(DISCONNECTED)
      setLoading(false)
      return
    }

    setLoading(true)
    try {
      setSession(await fetchSession())
    } catch {
      setSession(DISCONNECTED)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void refresh()
  }, [refresh])

  useEffect(() => {
    if (!enableGitHubSync) {
      return
    }

    const onFocus = () => {
      void refresh()
    }
    window.addEventListener('focus', onFocus)
    return () => window.removeEventListener('focus', onFocus)
  }, [refresh])

  return {
    ...session,
    loading,
    refresh,
  }
}
