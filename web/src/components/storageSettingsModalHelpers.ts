import type { Octokit } from '@octokit/rest'
import type { FileStoreState } from '../fileStore'
import { GITHUB_STORAGE_REPO } from '../hooks/useStorageBackend'
import type {
  GithubBackend,
  GithubBackendError,
} from '../storage/githubBackend'
import type { StorageBackend } from '../storage/types'

function statusOf(error: unknown): number | undefined {
  if (typeof error === 'object' && error !== null && 'status' in error) {
    const status = (error as { status: unknown }).status
    return typeof status === 'number' ? status : undefined
  }
  return undefined
}

/**
 * Get-or-create the fixed app repo under the authenticated user's account.
 * No confirmation prompt either way: the common case (existing repo) is the
 * same user reconnecting, and even a name collision is non-destructive
 * since the app only ever touches its own `scores/` folder within the repo.
 *
 * Lives here (not `githubBackend.ts`/`githubAuth.ts`) since it only ever
 * runs once, right after `connectWithDeviceFlow` resolves and this modal
 * already has both the fresh `Octokit` instance and the username on hand —
 * threading it through the backend/auth modules would add a second entry
 * point for the same one-shot setup step.
 */
export async function ensureStorageRepo(
  octokit: Octokit,
  owner: string,
): Promise<void> {
  try {
    await octokit.rest.repos.get({ owner, repo: GITHUB_STORAGE_REPO })
  } catch (error) {
    if (statusOf(error) !== 404) throw error
    await octokit.rest.repos.createForAuthenticatedUser({
      name: GITHUB_STORAGE_REPO,
      private: true,
    })
  }
}

export type ConflictResolution = 'overwrite-mine' | 'discard-mine'

/**
 * Resolves a `409` conflict on the active file with a minimal "last write
 * wins" choice (no 3-way merge, per the v1 limitations): `overwrite-mine`
 * re-pushes the current in-memory content (the retried `saveContent` fetches
 * a fresh `sha` first, so it succeeds even though the previous attempt
 * raced); `discard-mine` reloads the backend's file listing and replaces the
 * active file's in-memory content with whatever is now on GitHub.
 */
export async function resolveGithubConflict(
  resolution: ConflictResolution,
  backend: GithubBackend,
  store: FileStoreState,
): Promise<FileStoreState> {
  if (resolution === 'overwrite-mine') {
    await backend.saveContent(store)
    return store
  }
  const reloaded = await backend.load()
  const remoteContent = reloaded.userFiles[store.active] ?? ''
  return backend.updateActiveContent(store, remoteContent)
}

export function isGithubBackend(
  backend: StorageBackend,
): backend is GithubBackend {
  return backend.kind === 'github'
}

export function errorBannerMessage(
  error: GithubBackendError | null,
): string | null {
  if (!error) return null
  if (error.kind === 'rate-limited') {
    return 'GitHub API rate limit reached. Autosave is paused until it resets.'
  }
  if (error.kind === 'network') {
    return "You appear to be offline. Changes will save once you're back online."
  }
  return null
}
