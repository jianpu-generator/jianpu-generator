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
import { callWorker, createWorkerClient } from '../syncedShare/workerClient'
import { ensureWasmInit } from '../wasmInit'
import { addedName } from './cloudBackendNaming'
import {
  applyPendingStructure,
  createCloudOutbox,
  outboxLaneError,
  outboxSaveStatus,
  pendingContentByFileId,
  recordListedBases,
} from './cloudBackendOutbox'
import type {
  CloudBackend,
  CloudBackendConfig,
  CloudBackendError,
  LocalRename,
} from './cloudBackendTypes'
import type { BaseSnapshots } from './outbox/baseSnapshots'
import { type ConflictFlow, createConflictFlow } from './outbox/conflictFlow'
import type { OutboxLooper, OutboxSnapshot } from './outbox/outboxLooper'
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
 * handlers). Every write (content or structural) is applied to local state
 * at once and queued in the outbox (`outbox/`), which delivers it in the
 * background. A `name_taken` answer is resolved by the outbox rewriting the
 * queued name; this backend then reports the local rename through
 * `onLocalRename`. Naming helpers and the public types live in
 * `cloudBackendNaming.ts` and `cloudBackendTypes.ts` -- split out of this
 * file to stay under this repo's 400-line cap.
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

  const renameListeners = new Set<(change: LocalRename) => void>()
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
  // alongside every structural op, for the same reason.
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

  /** The current local name of `fileId`, for a `RestoreFile` message. */
  function trashedNameOf(fileId: string): string {
    for (const [name, id] of activeIdByName) if (id === fileId) return name
    for (const [name, id] of trashedIdByName) if (id === fileId) return name
    return ''
  }

  const { looper, bases } = createCloudOutbox(
    {
      client,
      identityToken,
      account: config.account,
      restoreName: trashedNameOf,
    },
    dependencies,
  )
  const ensureWasm = dependencies.ensureWasm ?? ensureWasmInit
  let ready: Promise<void> | null = null
  async function fetchServerFile(fileId: string) {
    const response = await callWorker(
      client.POST('/files/list', { body: { identityToken } }),
    )
    const file = response.files.find(({ id }) => id === fileId)
    return file && { revision: file.revision, content: file.content }
  }

  const noticeListeners = new Set<(message: string) => void>()
  let conflictFlow: ConflictFlow | null = null
  let startGeneration = 0
  let stopReconciling: (() => void) | null = null

  /** Initializes wasm and starts the looper, once. */
  function whenReady(): Promise<void> {
    if (ready) return ready
    const generation = startGeneration
    ready = ensureWasm().then(async () => {
      // Disposed while wasm was loading: do not take the lock.
      if (generation !== startGeneration) return
      await looper.start()
      // A backend only exists with a valid sign-in, so lanes that stopped
      // for sign-in can go again.
      if (!looper.snapshot().halted) await looper.resolveSignedIn()
      stopReconciling = looper.subscribe(reconcileLocalNames)
      conflictFlow = createConflictFlow({
        looper,
        bases,
        fetchServerFile,
        onNotice: (message) => {
          for (const listener of noticeListeners) listener(message)
        },
      })
    })
    return ready
  }

  /** The name the last queued `CreateFile`/`RenameFile` of each lane asks
   * for. When the outbox rewrote one after a `name_taken` answer, it differs
   * from the local name, which is then renamed to match. */
  function reconcileLocalNames({ queue }: OutboxSnapshot): void {
    for (const lane of queue.lanes) {
      const wanted = lane.messages
        .map(({ message }) =>
          message.tag === 'create-file'
            ? message.val.name
            : message.tag === 'rename-file'
              ? message.val.to
              : undefined,
        )
        .findLast((name) => name !== undefined)
      const current = [...activeIdByName].find(
        ([, id]) => id === lane.fileId,
      )?.[0]
      if (wanted === undefined || current === undefined || wanted === current) {
        continue
      }
      activeIdByName.delete(current)
      activeIdByName.set(wanted, lane.fileId)
      for (const listener of renameListeners) {
        listener({ fileId: lane.fileId, from: current, to: wanted })
      }
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

  /** Queues the creation of the file `nextState` added relative to `state`
   * (create, import, duplicate); the file already exists locally. */
  async function createRemoteFile(
    state: FileStoreState,
    nextState: FileStoreState,
  ): Promise<FileStoreState> {
    const name = addedName(state, nextState)
    if (!name) return nextState
    const id = fileIdForName(nextState, name)
    activeIdByName.set(name, id)
    await whenReady()
    await looper.enqueue(id, {
      tag: 'create-file',
      val: { name, content: nextState.userFiles[name] ?? '' },
    })
    return nextState
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
      const listed = looper.snapshot().halted
        ? response.files
        : applyPendingStructure(response.files, looper.snapshot())
      for (const file of listed) {
        fileIds[file.name] = file.id
        if (file.trashedAt === null) {
          userFiles[file.name] = file.content
          activeIdByName.set(file.name, file.id)
        } else {
          bin[file.name] = file.content
          trashedIdByName.set(file.name, file.id)
        }
      }
      // An unreadable outbox must not stop the files from loading.
      if (!looper.snapshot().halted) {
        await recordServerRevisions(response.files)
        await recordListedBases(looper, bases, response.files)
      }
      for (const [id, content] of pendingContentByFileId(looper.snapshot())) {
        const name = [...activeIdByName].find(([, value]) => value === id)?.[0]
        if (name !== undefined) userFiles[name] = content
      }
      // A successful listing proves the backend is reachable and current,
      // so any stale error/conflict from a previous save (e.g. "discard
      // mine", which reloads via this method without going through
      // `runOp`/`saveContentImpl`) no longer applies -- same reasoning as
      // the deleted GitHub backend's `load()`.
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
      activeIdByName.delete(from)
      activeIdByName.set(newName, id)
      await whenReady()
      await looper.enqueue(id, { tag: 'rename-file', val: { to: newName } })
      return nextState
    },

    async deleteFile(
      state: FileStoreState,
      name: string,
    ): Promise<FileStoreState> {
      const nextState = pureDeleteFile(state, name)
      if (nextState === state) return nextState
      const id = activeFileId(state, name)
      activeIdByName.delete(name)
      trashedIdByName.set(name, id)
      await whenReady()
      await looper.enqueue(id, { tag: 'trash-file' })
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
      trashedIdByName.delete(name)
      activeIdByName.set(newName, id)
      await whenReady()
      await looper.enqueue(id, { tag: 'restore-file' })
      return nextState
    },

    updateActiveContent: (
      state: FileStoreState,
      content: string,
    ): FileStoreState => pureUpdateActiveContent(state, content),

    saveContent,

    fetchServerFile,

    conflicts: () => conflictFlow,

    onNotice(listener: (message: string) => void): () => void {
      noticeListeners.add(listener)
      return () => noticeListeners.delete(listener)
    },

    outbox: () => looper,
    dispose() {
      stopReconciling?.()
      stopReconciling = null
      conflictFlow?.stop()
      conflictFlow = null
      startGeneration += 1
      looper.stop()
      ready = null
    },
    bases: () => bases,

    status: (): SaveStatus => outboxSaveStatus(looper.snapshot()) ?? 'idle',

    lastError: (): CloudBackendError | null =>
      outboxLaneError(looper.snapshot()),

    onLocalRename(listener: (change: LocalRename) => void): () => void {
      renameListeners.add(listener)
      return () => renameListeners.delete(listener)
    },

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
