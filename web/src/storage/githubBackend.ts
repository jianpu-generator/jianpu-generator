import { Octokit } from '@octokit/rest'
import {
  DEMO_FILE_NAME,
  type FileStoreState,
  fileContent,
  isReadOnlyFile,
  createFile as pureCreateFile,
  deleteFile as pureDeleteFile,
  duplicateFile as pureDuplicateFile,
  importSharedFile as pureImportSharedFile,
  renameFile as pureRenameFile,
  restoreFile as pureRestoreFile,
  updateActiveContent as pureUpdateActiveContent,
} from '../fileStore'
import {
  addedFileName,
  decodeBase64,
  encodeBase64,
  isNetworkError,
  joinPath,
  readStoredFileIds,
  statusOf,
  writeStoredFileIds,
} from './githubBackendUtils'
import type { SaveStatus, StorageBackend } from './types'

/**
 * Fixed top-level folder the app's files live under within the target repo,
 * alongside future siblings like `trash`/`metadata` — keeps the repo root
 * free for those rather than mixing everything in together.
 */
const SCORES_DIR = 'scores'
const TRASH_DIR = 'trash'

export interface GithubBackendConfig {
  token: string
  owner: string
  repo: string
  /** Defaults to the repo's default branch when omitted. */
  branch?: string
}

/**
 * Typed detail behind a `'error'`/`'offline'` `SaveStatus`, for a future
 * settings UI (step 4) to render a specific banner or conflict prompt.
 * Not part of `StorageBackend` itself since `SaveStatus` is shared with the
 * local backend, which never produces these — callers that care narrow to
 * `GithubBackend` to read it.
 */
export type GithubBackendError =
  | { kind: 'conflict'; path: string }
  | { kind: 'rate-limited' }
  | { kind: 'network' }
  | { kind: 'unknown'; message: string }

export interface GithubBackend extends StorageBackend {
  readonly kind: 'github'
  /** Detail behind the most recent `'error'`/`'offline'` status, if any. */
  lastError(): GithubBackendError | null
}

/**
 * `StorageBackend` implementation backed by GitHub's Contents API. Owns its
 * own `Octokit` instance (bearer auth); see `githubAuth.ts` for how the
 * token is obtained and persisted.
 *
 * v1 design notes (see `plan-github-step-3-github-backend-and-auth.md`):
 * - No sha cache: every update/delete refetches `sha` via `getContent`
 *   immediately before the write.
 * - Rename/delete/restore are two sequential Contents API calls
 *   (create-at-new-path, then delete-at-old-path), not an atomic Git Data
 *   API commit — an interruption between the two transiently leaves the
 *   file at both paths, recoverable by re-running the operation.
 * - Restore only undoes a delete (moves the file back from `trash/` with the
 *   content it had at delete time); it does not restore an older edited
 *   version of a still-active file.
 */
export function createGithubBackend(
  config: GithubBackendConfig,
): GithubBackend {
  const octokit = new Octokit({ auth: config.token })
  const { owner, repo, branch } = config

  const mainDir = SCORES_DIR
  const binDir = TRASH_DIR
  const filePath = (name: string) => joinPath(SCORES_DIR, name)
  const binFilePath = (name: string) => joinPath(TRASH_DIR, name)

  let status: SaveStatus = 'idle'
  let lastError: GithubBackendError | null = null
  let inFlightSave: Promise<void> | null = null
  let pendingRetryState: FileStoreState | null = null

  if (typeof window !== 'undefined') {
    window.addEventListener('online', () => {
      if (status === 'offline' && pendingRetryState) {
        const retryState = pendingRetryState
        pendingRetryState = null
        void saveContent(retryState)
      }
    })
  }

  async function fetchSha(path: string): Promise<string | undefined> {
    try {
      const { data } = await octokit.rest.repos.getContent({
        owner,
        repo,
        path,
        ref: branch,
      })
      return !Array.isArray(data) && 'sha' in data ? data.sha : undefined
    } catch (error) {
      if (statusOf(error) === 404) return undefined
      throw error
    }
  }

  async function fetchFileContent(path: string): Promise<string> {
    const { data } = await octokit.rest.repos.getContent({
      owner,
      repo,
      path,
      ref: branch,
    })
    if (Array.isArray(data) || data.type !== 'file' || !data.content) {
      throw new Error(`githubBackend: expected a file at ${path}`)
    }
    return decodeBase64(data.content)
  }

  async function listJianpuFiles(
    dirPath: string,
  ): Promise<Record<string, string>> {
    let entries: { name: string; path: string; type: string }[]
    try {
      const { data } = await octokit.rest.repos.getContent({
        owner,
        repo,
        path: dirPath,
        ref: branch,
      })
      entries = Array.isArray(data) ? data : []
    } catch (error) {
      if (statusOf(error) === 404) return {}
      throw error
    }

    const files = entries.filter(
      (entry) => entry.type === 'file' && entry.name.endsWith('.jianpu'),
    )
    const contents = await Promise.all(
      files.map((file) => fetchFileContent(file.path)),
    )
    const result: Record<string, string> = {}
    files.forEach((file, index) => {
      result[file.name] = contents[index] ?? ''
    })
    return result
  }

  /** Create-only write, no sha lookup — for paths guaranteed not to exist
   * yet (new file, rename/restore destination, duplicate destination). */
  async function createOnly(
    path: string,
    content: string,
    message: string,
  ): Promise<void> {
    await octokit.rest.repos.createOrUpdateFileContents({
      owner,
      repo,
      path,
      message,
      content: encodeBase64(content),
      branch,
    })
  }

  /** Fetch-sha-then-write, for paths that may already exist (active-file
   * saves, and the `trash/` destination of a delete — which can already hold
   * a stale entry from an earlier restore-then-delete cycle). */
  async function putFile(
    path: string,
    content: string,
    message: string,
  ): Promise<void> {
    const sha = await fetchSha(path)
    await octokit.rest.repos.createOrUpdateFileContents({
      owner,
      repo,
      path,
      message,
      content: encodeBase64(content),
      sha,
      branch,
    })
  }

  async function deleteFileAt(path: string, message: string): Promise<void> {
    const sha = await fetchSha(path)
    if (!sha) return
    await octokit.rest.repos.deleteFile({
      owner,
      repo,
      path,
      message,
      sha,
      branch,
    })
  }

  /** Classifies a thrown error into the `GithubBackendError`/`SaveStatus`
   * pair it should surface. Returns the pair rather than mutating `status`
   * directly so callers can branch on the freshly computed status (e.g. to
   * decide whether to retry) without relying on TypeScript narrowing the
   * `status` closure variable across a separate function call.
   */
  function classifyError(
    error: unknown,
    path?: string,
  ): { status: SaveStatus; error: GithubBackendError } {
    if (statusOf(error) === 409) {
      return { status: 'error', error: { kind: 'conflict', path: path ?? '' } }
    }
    if (statusOf(error) === 403) {
      return { status: 'error', error: { kind: 'rate-limited' } }
    }
    if (isNetworkError(error)) {
      return { status: 'offline', error: { kind: 'network' } }
    }
    return {
      status: 'error',
      error: {
        kind: 'unknown',
        message: error instanceof Error ? error.message : String(error),
      },
    }
  }

  /** Runs a structural operation's API calls, translating failures into the
   * same status/error tracking `saveContent` uses so `status()` reflects
   * both autosave and structural-op failures uniformly. */
  async function runOp<T>(operation: () => Promise<T>): Promise<T> {
    try {
      const result = await operation()
      status = 'idle'
      lastError = null
      return result
    } catch (error) {
      const classified = classifyError(error)
      status = classified.status
      lastError = classified.error
      throw error
    }
  }

  async function saveContentImpl(state: FileStoreState): Promise<void> {
    if (isReadOnlyFile(state.active)) return
    const path = filePath(state.active)
    status = 'saving'
    try {
      await putFile(
        path,
        fileContent(state, state.active),
        `jianpu: update ${state.active}`,
      )
      status = 'idle'
      lastError = null
      pendingRetryState = null
    } catch (error) {
      const classified = classifyError(error, path)
      status = classified.status
      lastError = classified.error
      if (classified.status === 'offline') pendingRetryState = state
      throw error
    }
  }

  /** Serializes autosave calls: only one file is ever actively edited at a
   * time, so a single in-flight promise (rather than a per-path map) is
   * enough to guarantee the next save waits for the previous one. */
  function saveContent(state: FileStoreState): Promise<void> {
    const previous = inFlightSave ?? Promise.resolve()
    const next = previous.catch(() => {}).then(() => saveContentImpl(state))
    inFlightSave = next.catch(() => {})
    return next
  }

  return {
    kind: 'github',

    async load(): Promise<FileStoreState> {
      const [userFiles, bin] = await Promise.all([
        listJianpuFiles(mainDir),
        listJianpuFiles(binDir),
      ])
      const stored = readStoredFileIds(owner, repo)
      const fileIds: Record<string, string> = {}
      for (const name of [...Object.keys(userFiles), ...Object.keys(bin)]) {
        fileIds[name] = stored[name] ?? crypto.randomUUID()
      }
      writeStoredFileIds(owner, repo, fileIds)
      // A successful listing proves the backend is reachable and current,
      // so any stale error/conflict from a previous save (e.g. "discard
      // mine", which reloads via this method without going through
      // `runOp`/`saveContentImpl`) no longer applies.
      status = 'idle'
      lastError = null
      return { active: DEMO_FILE_NAME, userFiles, bin, fileIds }
    },

    async createFile(state: FileStoreState): Promise<FileStoreState> {
      const nextState = pureCreateFile(state)
      const name = addedFileName(state, nextState)
      if (!name) return nextState
      await runOp(() =>
        createOnly(
          filePath(name),
          nextState.userFiles[name] ?? '',
          `jianpu: create ${name}`,
        ),
      )
      writeStoredFileIds(owner, repo, nextState.fileIds)
      return nextState
    },

    async importFile(
      state: FileStoreState,
      filename: string,
      content: string,
    ): Promise<FileStoreState> {
      const nextState = pureImportSharedFile(state, filename, content)
      const name = addedFileName(state, nextState)
      if (!name) return nextState
      await runOp(() =>
        createOnly(
          filePath(name),
          nextState.userFiles[name] ?? '',
          `jianpu: import ${name}`,
        ),
      )
      writeStoredFileIds(owner, repo, nextState.fileIds)
      return nextState
    },

    async duplicateFile(state: FileStoreState): Promise<FileStoreState> {
      const nextState = pureDuplicateFile(state)
      const name = addedFileName(state, nextState)
      if (!name) return nextState
      await runOp(() =>
        createOnly(
          filePath(name),
          nextState.userFiles[name] ?? '',
          `jianpu: duplicate ${state.active} as ${name}`,
        ),
      )
      writeStoredFileIds(owner, repo, nextState.fileIds)
      return nextState
    },

    async renameFile(
      state: FileStoreState,
      from: string,
      to: string,
    ): Promise<FileStoreState> {
      const nextState = pureRenameFile(state, from, to)
      const newName = addedFileName(state, nextState)
      if (!newName) return nextState
      await runOp(async () => {
        await createOnly(
          filePath(newName),
          nextState.userFiles[newName] ?? '',
          `jianpu: rename ${from} to ${newName}`,
        )
        await deleteFileAt(
          filePath(from),
          `jianpu: rename ${from} to ${newName}`,
        )
      })
      writeStoredFileIds(owner, repo, nextState.fileIds)
      return nextState
    },

    async deleteFile(
      state: FileStoreState,
      name: string,
    ): Promise<FileStoreState> {
      const nextState = pureDeleteFile(state, name)
      if (nextState === state) return nextState
      const content = nextState.bin[name] ?? ''
      await runOp(async () => {
        await putFile(binFilePath(name), content, `jianpu: delete ${name}`)
        await deleteFileAt(filePath(name), `jianpu: delete ${name}`)
      })
      return nextState
    },

    async restoreFile(
      state: FileStoreState,
      name: string,
    ): Promise<FileStoreState> {
      const nextState = pureRestoreFile(state, name)
      const newName = addedFileName(state, nextState)
      if (!newName) return nextState
      await runOp(async () => {
        await createOnly(
          filePath(newName),
          nextState.userFiles[newName] ?? '',
          `jianpu: restore ${newName}`,
        )
        await deleteFileAt(binFilePath(name), `jianpu: restore ${newName}`)
      })
      writeStoredFileIds(owner, repo, nextState.fileIds)
      return nextState
    },

    updateActiveContent: (
      state: FileStoreState,
      content: string,
    ): FileStoreState => pureUpdateActiveContent(state, content),

    saveContent,

    status: (): SaveStatus => status,

    lastError: (): GithubBackendError | null => lastError,
  }
}
