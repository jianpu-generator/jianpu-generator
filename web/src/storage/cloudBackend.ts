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
import {
  callWorker,
  createWorkerClient,
  NetworkFailure,
  WorkerRequestError,
} from '../syncedShare/workerClient'
import { ensureWasmInit } from '../wasmInit'
import {
  addedName,
  withNameCollisionRetry,
  withRenamedKey,
} from './cloudBackendNaming'
import {
  createCloudOutbox,
  outboxLaneError,
  outboxSaveStatus,
  recordListedBases,
} from './cloudBackendOutbox'
import type {
  CloudBackend,
  CloudBackendConfig,
  CloudBackendError,
} from './cloudBackendTypes'
import type { BaseSnapshots } from './outbox/baseSnapshots'
import type { OutboxLooper } from './outbox/outboxLooper'
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
 * Requests go through the worker's generated client (`workerClient.ts` --
 * paths, bodies and the `ApiError` failure union all come from the Rust
 * handlers). The `name_taken` retry (`withNameCollisionRetry` and friends)
 * and the public types live in `cloudBackendNaming.ts` and
 * `cloudBackendTypes.ts` -- split out of this file to stay under this
 * repo's 400-line cap.
 */
export interface CloudBackendDependencies {
  looper?: OutboxLooper
  bases?: BaseSnapshots
  ensureWasm?: () => Promise<void>
}

export function createCloudBackend(
  config: CloudBackendConfig,
  dependencies: CloudBackendDependencies = {},
): CloudBackend {
  const identityToken = config.token
  const client = createWorkerClient(config.workerHost)

  // Status of the last structural operation; content saves report through
  // the outbox (see `status()` and `lastError()` below).
  let operationStatus: SaveStatus = 'idle'
  let operationError: CloudBackendError | null = null
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

  /** The local name of the trashed file `fileId`, for a `RestoreFile` message. */
  function trashedNameOf(fileId: string): string {
    for (const [name, id] of trashedIdByName) if (id === fileId) return name
    for (const [name, id] of activeIdByName) if (id === fileId) return name
    return ''
  }

  const { looper, bases } = createCloudOutbox(
    { client, identityToken, restoreName: trashedNameOf },
    dependencies,
  )
  const ensureWasm = dependencies.ensureWasm ?? ensureWasmInit
  let ready: Promise<void> | null = null
  /** Initializes wasm and starts the looper, once. */
  function whenReady(): Promise<void> {
    ready ??= ensureWasm().then(() => looper.start())
    return ready
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
    if (error instanceof WorkerRequestError) {
      switch (error.apiError?.code) {
        case 'revision_conflict':
          return {
            status: 'error',
            error: {
              kind: 'conflict',
              currentRevision: error.apiError.currentRevision,
            },
          }
        case 'unauthorized':
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
      operationStatus = 'idle'
      operationError = null
      return result
    } catch (error) {
      const classified = classifyError(error)
      operationStatus = classified.status
      operationError = classified.error
      throw error
    }
  }

  /** Queues the active file's content in the outbox and returns once it is
   * persisted locally -- delivery happens in the background. */
  async function saveContent(state: FileStoreState): Promise<void> {
    if (isReadOnlyFile(state.active)) return
    await whenReady()
    await looper.enqueue(fileIdForName(state, state.active), {
      tag: 'save-content',
      val: { content: fileContent(state, state.active) },
    })
  }

  /** Tells the outbox each file's server revision, except for files with
   * unsent messages: their recorded revision is what conflict detection
   * compares against. */
  async function recordServerRevisions(
    files: ReadonlyArray<{ id: string; revision: number }>,
  ): Promise<void> {
    const pending = new Set(
      looper
        .snapshot()
        .queue.lanes.filter((lane) => lane.messages.length > 0)
        .map((lane) => lane.fileId),
    )
    for (const file of files) {
      if (!pending.has(file.id)) {
        await looper.recordRevision(file.id, BigInt(file.revision))
      }
    }
  }

  /** Persists the file `nextState` added relative to `state` (create,
   * import, duplicate), renaming it if the server reports a name collision. */
  async function createRemoteFile(
    state: FileStoreState,
    nextState: FileStoreState,
  ): Promise<FileStoreState> {
    const name = addedName(state, nextState)
    if (!name) return nextState
    const id = fileIdForName(nextState, name)
    const content = nextState.userFiles[name] ?? ''
    const finalName = await runOp(() =>
      withNameCollisionRetry(name, state, (attemptName) =>
        callWorker(
          client.POST('/files', {
            body: { identityToken, id, name: attemptName, content },
          }),
        ),
      ),
    )
    await whenReady()
    await looper.recordRevision(id, 0n)
    activeIdByName.set(finalName, id)
    return finalName === name
      ? nextState
      : withRenamedKey(nextState, name, finalName)
  }

  return {
    kind: 'cloud',

    async load(): Promise<FileStoreState> {
      await whenReady()
      const response = await callWorker(
        client.POST('/files/list', { body: { identityToken } }),
      )
      const userFiles: Record<string, string> = {}
      const bin: Record<string, string> = {}
      const fileIds: Record<string, string> = {}
      activeIdByName.clear()
      trashedIdByName.clear()
      for (const file of response.files) {
        fileIds[file.name] = file.id
        if (file.trashedAt === null) {
          userFiles[file.name] = file.content
          activeIdByName.set(file.name, file.id)
        } else {
          bin[file.name] = file.content
          trashedIdByName.set(file.name, file.id)
        }
      }
      await recordServerRevisions(response.files)
      await recordListedBases(looper, bases, response.files)
      // A successful listing proves the backend is reachable and current,
      // so any stale error/conflict from a previous save (e.g. "discard
      // mine", which reloads via this method without going through
      // `runOp`/`saveContentImpl`) no longer applies -- same reasoning as
      // the deleted GitHub backend's `load()`.
      operationStatus = 'idle'
      operationError = null
      return { active: DEMO_FILE_NAMES[0] ?? '', userFiles, bin, fileIds }
    },

    createFile: (state) => createRemoteFile(state, pureCreateFile(state)),

    importFile: (state, filename, content) =>
      createRemoteFile(state, pureImportSharedFile(state, filename, content)),

    duplicateFile: (state) => createRemoteFile(state, pureDuplicateFile(state)),

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
          callWorker(
            client.POST('/files/{id}/rename', {
              params: { path: { id } },
              body: { identityToken, name: attemptName },
            }),
          ),
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
      await runOp(() =>
        callWorker(
          client.POST('/files/{id}/delete', {
            params: { path: { id } },
            body: { identityToken },
          }),
        ),
      )
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
          callWorker(
            client.POST('/files/{id}/restore', {
              params: { path: { id } },
              body: { identityToken, name: attemptName },
            }),
          ),
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

    outbox: () => looper,
    bases: () => bases,

    status: (): SaveStatus =>
      outboxSaveStatus(looper.snapshot()) ?? operationStatus,

    lastError: (): CloudBackendError | null =>
      outboxLaneError(looper.snapshot()) ?? operationError,

    async forceOverwrite(state: FileStoreState): Promise<void> {
      const id = fileIdForName(state, state.active)
      const conflict = looper
        .snapshot()
        .queue.lanes.find(
          (lane) => lane.fileId === id && lane.status.tag === 'needs-merge',
        )
      if (conflict?.status.tag === 'needs-merge') {
        await looper.recordRevision(id, conflict.status.val.currentRevision)
        await looper.resolve(id, { tag: 'retry' })
        return
      }
      return saveContent(state)
    },
  }
}
