import {
  DEMO_FILE_NAMES,
  type FileStoreState,
  fileContent,
  fileIdForName,
  isReadOnlyFile,
  createFile as pureCreateFile,
  deleteFile as pureDeleteFile,
  duplicateFile as pureDuplicateFile,
  importSharedFile as pureImportSharedFile,
  renameFile as pureRenameFile,
  restoreFile as pureRestoreFile,
  updateActiveContent as pureUpdateActiveContent,
} from '../fileStore'
import { syncedShareWorkerOrigin } from '../syncedShare/workerUrl'
import {
  HttpStatusError,
  isConflictBody,
  NetworkFailure,
  request,
} from './cloudBackendHttp'
import {
  addedName,
  withNameCollisionRetry,
  withRenamedKey,
} from './cloudBackendNaming'
import type {
  CloudBackend,
  CloudBackendConfig,
  CloudBackendError,
  ListFilesResponseWire,
  UpdateFileContentResponseWire,
} from './cloudBackendTypes'
import type { SaveStatus } from './types'

export type {
  CloudBackend,
  CloudBackendConfig,
  CloudBackendError,
} from './cloudBackendTypes'

/**
 * `StorageBackend` implementation backed by `crates/live-share-worker`'s
 * `/files/*` routes (D1-backed), the replacement for the deleted GitHub
 * Contents API backend. Reuses `fileStore.ts`'s pure transforms exactly
 * like that backend did -- every structural op below
 * calls the matching pure transform, performs the corresponding HTTP call,
 * and returns the transform's resulting `FileStoreState` (reconciled by the
 * caller via `mergeBackendResult`, unchanged).
 *
 * Design notes, contrasted with the deleted GitHub backend:
 * - No sha-refetch-before-write: `revisionByFileId` tracks each file's last
 *   known `revision` (populated by `load()`, advanced after every
 *   successful content save), sent as `expectedRevision` on the next save --
 *   the server's atomic `UPDATE ... WHERE revision = ?` is the actual CAS
 *   guard (see `crate::files::classify_content_write`).
 * - Rename/delete/restore are each a single atomic server-side `UPDATE`,
 *   not two sequential calls -- no "left at both paths" window to recover
 *   from.
 * - A file's id is the real, durable D1 row id straight from the server
 *   (`load()`'s response) -- no more localStorage name -> UUID shim.
 *
 * Request framing (`request`/`NetworkFailure`/`HttpStatusError`), the
 * `409 {code: "name_taken"}` retry (`withNameCollisionRetry` and friends),
 * and the wire/public types live in `cloudBackendHttp.ts`,
 * `cloudBackendNaming.ts`, and `cloudBackendTypes.ts` respectively -- split
 * out of this file to stay under this repo's 400-line cap.
 */
export function createCloudBackend(config: CloudBackendConfig): CloudBackend {
  const { token } = config
  const origin = syncedShareWorkerOrigin(config.workerHost)

  let status: SaveStatus = 'idle'
  let lastError: CloudBackendError | null = null
  let inFlightSave: Promise<void> | null = null
  let pendingRetryState: FileStoreState | null = null
  const revisionByFileId = new Map<string, number>()
  // `FileStoreState.fileIds` is one flat map keyed by display name, shared
  // across `userFiles` and `bin` -- sound for every *pure* transform in
  // `fileStore.ts` (which never lets the same name occupy both buckets at
  // once, see `reservedNames`), but this backend's own `load()` response can
  // -- in principle -- list an active and a trashed row under the same
  // name (the real D1 schema's `idx_files_owner_name` unique index makes
  // that impossible for a genuine, unmodified server response, but nothing
  // in this client trusts that invariant blindly, and this project's own
  // e2e coverage deliberately manufactures exactly that response shape to
  // exercise `fileStore.ts`'s `restoreFile` collision-avoidance -- see
  // `files-cloud-backend.steps.ts`'s restore-collision `Given`). Tracking
  // each bucket's ids separately here means `renameFile`/`deleteFile`
  // (always operating on a *known-active* name) and `restoreFile` (always
  // operating on a *known-trashed* name) never resolve the wrong row's id
  // out of a name that happens to collide across buckets. Kept in sync
  // alongside `revisionByFileId` after every successful structural op,
  // for the same reason that map is.
  const activeIdByName = new Map<string, string>()
  const trashedIdByName = new Map<string, string>()

  /** Id of the *active* file named `name`, preferring the bucket-aware
   * `activeIdByName` (accurate even when a same-named trashed row exists)
   * over the ambiguous `state.fileIds`, which is only consulted as a
   * fallback (e.g. before the first `load()` has populated the map). */
  function activeFileId(state: FileStoreState, name: string): string {
    return activeIdByName.get(name) ?? fileIdForName(state, name)
  }

  /** Id of the *trashed* file named `name` -- see `activeFileId` above. */
  function trashedFileId(state: FileStoreState, name: string): string {
    return trashedIdByName.get(name) ?? fileIdForName(state, name)
  }

  if (typeof window !== 'undefined') {
    window.addEventListener('online', () => {
      if (status === 'offline' && pendingRetryState) {
        const retryState = pendingRetryState
        pendingRetryState = null
        void saveContent(retryState)
      }
    })
  }

  function post(path: string, body: object): Promise<unknown> {
    return request(origin, path, { identityToken: token, ...body })
  }

  /** Classifies a thrown error into the `CloudBackendError`/`SaveStatus`
   * pair it should surface. Returns the pair rather than mutating `status`
   * directly, same reasoning as the deleted GitHub backend's
   * `classifyError`. */
  function classifyError(error: unknown): {
    status: SaveStatus
    error: CloudBackendError
  } {
    if (error instanceof NetworkFailure) {
      return { status: 'offline', error: { kind: 'network' } }
    }
    if (error instanceof HttpStatusError) {
      if (error.httpStatus === 409 && isConflictBody(error.body)) {
        return {
          status: 'error',
          error: {
            kind: 'conflict',
            currentRevision: error.body.currentRevision,
          },
        }
      }
      if (error.httpStatus === 401) {
        return { status: 'error', error: { kind: 'auth' } }
      }
    }
    return {
      status: 'error',
      error: {
        kind: 'unknown',
        message: error instanceof Error ? error.message : String(error),
      },
    }
  }

  /** Runs a structural operation's API call(s), translating a failure into
   * the same status/error tracking `saveContent` uses, and resetting to
   * `'idle'`/`null` on success -- mirrors the deleted GitHub backend's
   * `runOp`. */
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
    const id = fileIdForName(state, state.active)
    const expectedRevision = revisionByFileId.get(id) ?? 0
    status = 'saving'
    try {
      const json = await post(`/files/${id}/content`, {
        content: fileContent(state, state.active),
        expectedRevision,
      })
      const response = json as UpdateFileContentResponseWire
      revisionByFileId.set(id, response.revision)
      status = 'idle'
      lastError = null
      pendingRetryState = null
    } catch (error) {
      if (
        error instanceof HttpStatusError &&
        error.httpStatus === 409 &&
        isConflictBody(error.body)
      ) {
        // Deliberately does not rethrow, unlike every other failure here --
        // see `CloudBackend.forceOverwrite`'s doc comment. The revision map
        // is left untouched so `forceOverwrite` can realign it to
        // `currentRevision` before retrying.
        status = 'error'
        lastError = {
          kind: 'conflict',
          currentRevision: error.body.currentRevision,
        }
        return
      }
      const classified = classifyError(error)
      status = classified.status
      lastError = classified.error
      if (classified.status === 'offline') pendingRetryState = state
      throw error
    }
  }

  /** Serializes autosave calls: only one file is ever actively edited at a
   * time, so a single in-flight promise (rather than a per-id map) is
   * enough to guarantee the next save waits for the previous one -- same
   * shape as the deleted GitHub backend's `saveContent`. */
  function saveContent(state: FileStoreState): Promise<void> {
    const previous = inFlightSave ?? Promise.resolve()
    const next = previous.catch(() => {}).then(() => saveContentImpl(state))
    inFlightSave = next.catch(() => {})
    return next
  }

  return {
    kind: 'cloud',

    async load(): Promise<FileStoreState> {
      const json = await post('/files/list', {})
      const response = json as ListFilesResponseWire
      const userFiles: Record<string, string> = {}
      const bin: Record<string, string> = {}
      const fileIds: Record<string, string> = {}
      activeIdByName.clear()
      trashedIdByName.clear()
      for (const file of response.files) {
        fileIds[file.name] = file.id
        revisionByFileId.set(file.id, file.revision)
        if (file.trashedAt === null) {
          userFiles[file.name] = file.content
          activeIdByName.set(file.name, file.id)
        } else {
          bin[file.name] = file.content
          trashedIdByName.set(file.name, file.id)
        }
      }
      // A successful listing proves the backend is reachable and current,
      // so any stale error/conflict from a previous save (e.g. "discard
      // mine", which reloads via this method without going through
      // `runOp`/`saveContentImpl`) no longer applies -- same reasoning as
      // the deleted GitHub backend's `load()`.
      status = 'idle'
      lastError = null
      return { active: DEMO_FILE_NAMES[0] ?? '', userFiles, bin, fileIds }
    },

    async createFile(state: FileStoreState): Promise<FileStoreState> {
      const nextState = pureCreateFile(state)
      const name = addedName(state, nextState)
      if (!name) return nextState
      const id = fileIdForName(nextState, name)
      const content = nextState.userFiles[name] ?? ''
      const finalName = await runOp(() =>
        withNameCollisionRetry(name, state, (attemptName) =>
          post('/files', { id, name: attemptName, content }),
        ),
      )
      revisionByFileId.set(id, 0)
      activeIdByName.set(finalName, id)
      return finalName === name
        ? nextState
        : withRenamedKey(nextState, name, finalName)
    },

    async importFile(
      state: FileStoreState,
      filename: string,
      content: string,
    ): Promise<FileStoreState> {
      const nextState = pureImportSharedFile(state, filename, content)
      const name = addedName(state, nextState)
      if (!name) return nextState
      const id = fileIdForName(nextState, name)
      const fileContentToSend = nextState.userFiles[name] ?? ''
      const finalName = await runOp(() =>
        withNameCollisionRetry(name, state, (attemptName) =>
          post('/files', { id, name: attemptName, content: fileContentToSend }),
        ),
      )
      revisionByFileId.set(id, 0)
      activeIdByName.set(finalName, id)
      return finalName === name
        ? nextState
        : withRenamedKey(nextState, name, finalName)
    },

    async duplicateFile(state: FileStoreState): Promise<FileStoreState> {
      const nextState = pureDuplicateFile(state)
      const name = addedName(state, nextState)
      if (!name) return nextState
      const id = fileIdForName(nextState, name)
      const content = nextState.userFiles[name] ?? ''
      const finalName = await runOp(() =>
        withNameCollisionRetry(name, state, (attemptName) =>
          post('/files', { id, name: attemptName, content }),
        ),
      )
      revisionByFileId.set(id, 0)
      activeIdByName.set(finalName, id)
      return finalName === name
        ? nextState
        : withRenamedKey(nextState, name, finalName)
    },

    async renameFile(
      state: FileStoreState,
      from: string,
      to: string,
    ): Promise<FileStoreState> {
      const nextState = pureRenameFile(state, from, to)
      const newName = addedName(state, nextState)
      if (!newName) return nextState
      const id = activeFileId(state, from)
      const finalName = await runOp(() =>
        withNameCollisionRetry(newName, state, (attemptName) =>
          post(`/files/${id}/rename`, { name: attemptName }),
        ),
      )
      activeIdByName.delete(from)
      activeIdByName.set(finalName, id)
      return finalName === newName
        ? nextState
        : withRenamedKey(nextState, newName, finalName)
    },

    async deleteFile(
      state: FileStoreState,
      name: string,
    ): Promise<FileStoreState> {
      const nextState = pureDeleteFile(state, name)
      if (nextState === state) return nextState
      const id = activeFileId(state, name)
      await runOp(() => post(`/files/${id}/delete`, {}))
      activeIdByName.delete(name)
      trashedIdByName.set(name, id)
      return nextState
    },

    async restoreFile(
      state: FileStoreState,
      name: string,
    ): Promise<FileStoreState> {
      const nextState = pureRestoreFile(state, name)
      const newName = addedName(state, nextState)
      if (!newName) return nextState
      const id = trashedFileId(state, name)
      const finalName = await runOp(() =>
        withNameCollisionRetry(newName, state, (attemptName) =>
          post(`/files/${id}/restore`, { name: attemptName }),
        ),
      )
      trashedIdByName.delete(name)
      activeIdByName.set(finalName, id)
      return finalName === newName
        ? nextState
        : withRenamedKey(nextState, newName, finalName)
    },

    updateActiveContent: (
      state: FileStoreState,
      content: string,
    ): FileStoreState => pureUpdateActiveContent(state, content),

    saveContent,

    status: (): SaveStatus => status,

    lastError: (): CloudBackendError | null => lastError,

    async forceOverwrite(state: FileStoreState): Promise<void> {
      if (lastError?.kind === 'conflict') {
        const id = fileIdForName(state, state.active)
        revisionByFileId.set(id, lastError.currentRevision)
      }
      return saveContent(state)
    },
  }
}
