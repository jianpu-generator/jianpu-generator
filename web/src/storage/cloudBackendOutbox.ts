import type { SendRequest } from '../jianpuWasm'
import {
  callWorker,
  type createWorkerClient,
} from '../syncedShare/workerClient'
import type { CloudBackendError } from './cloudBackendTypes'
import { type BaseSnapshots, createBaseSnapshots } from './outbox/baseSnapshots'
import { createMemoryOutboxStore } from './outbox/memoryOutboxStore'
import { runAsLeader } from './outbox/outboxLock'
import {
  createOutboxLooper,
  type OutboxLooper,
  type OutboxSnapshot,
} from './outbox/outboxLooper'
import { createScheduler } from './outbox/outboxScheduler'
import {
  createIndexedDbOutboxStore,
  type OutboxStore,
} from './outbox/outboxStore'
import { sendMessage } from './outbox/sendMessage'
import type { SaveStatus } from './types'

export interface CloudOutboxConfig {
  client: ReturnType<typeof createWorkerClient>
  identityToken: string
  /** Signed-in account; scopes the on-device outbox. */
  account: string
  /** The file's current local name, for a `RestoreFile` message. */
  restoreName(fileId: string): string
}

/** The looper hooks that keep `bases` equal to the last text the server has. */
export function baseSnapshotHooks(bases: BaseSnapshots) {
  return {
    async onAcknowledged(request: SendRequest): Promise<void> {
      const { message, fileId } = request
      if (message.tag === 'save-content' || message.tag === 'create-file') {
        await bases.recordBase(fileId, message.val.content)
      } else if (message.tag === 'trash-file') {
        await bases.dropBase(fileId)
      }
    },
    onDiscarded: (fileId: string): Promise<void> => bases.dropBase(fileId),
  }
}

/** Records the listed text of every file as its base, except for files with
 * unsent messages: their base must stay the last text the server is known to
 * have for them. */
export async function recordListedBases(
  looper: OutboxLooper,
  bases: BaseSnapshots,
  files: ReadonlyArray<{ id: string; content: string }>,
): Promise<void> {
  const unsent = new Set(
    looper
      .snapshot()
      .queue.lanes.filter((lane) => lane.messages.length > 0)
      .map((lane) => lane.fileId),
  )
  for (const file of files) {
    if (!unsent.has(file.id)) await bases.recordBase(file.id, file.content)
  }
}

/** The looper and bases to use: injected ones (tests) or the production pair. */
export function createCloudOutbox(
  config: CloudOutboxConfig,
  injected: { looper?: OutboxLooper; bases?: BaseSnapshots },
): CloudOutbox {
  if (!injected.looper) {
    const production = createCloudOutboxLooper(config)
    return {
      looper: production.looper,
      bases: injected.bases ?? production.bases,
    }
  }
  return {
    looper: injected.looper,
    bases: injected.bases ?? createBaseSnapshots(createMemoryOutboxStore()),
  }
}

export interface CloudOutbox {
  looper: OutboxLooper
  bases: BaseSnapshots
}

/** The production looper: IndexedDB store, leader-tab lock, real timers.
 * The looper and the base snapshots share one store. */
function createCloudOutboxLooper(config: CloudOutboxConfig): CloudOutbox {
  const store: OutboxStore = createIndexedDbOutboxStore(config.account)
  const bases = createBaseSnapshots(store)
  const { client, identityToken, restoreName } = config
  const fetchServerContent = async (fileId: string): Promise<string> => {
    const response = await callWorker(
      client.POST('/files/list', { body: { identityToken } }),
    )
    return response.files.find((file) => file.id === fileId)?.content ?? ''
  }
  const looper = createOutboxLooper({
    ...baseSnapshotHooks(bases),
    store,
    sendMessage: (request) =>
      sendMessage(request, {
        client,
        identityToken,
        fetchServerContent,
        restoreName,
      }),
    createScheduler: (onWake) =>
      createScheduler({
        now: () => Date.now(),
        setTimer: (callback, delayMs) => setTimeout(callback, delayMs),
        clearTimer: (handle) => clearTimeout(handle),
        onWake,
      }),
    runAsLeader,
    now: () => Date.now(),
    random: () => Math.random(),
  })
  return { looper, bases }
}

/** The save status the outbox implies, or null when it implies none. */
export function outboxSaveStatus(snapshot: OutboxSnapshot): SaveStatus | null {
  const reasons = snapshot.summary.lanes.map((lane) => lane.reason)
  if (snapshot.halted) return 'error'
  if (
    reasons.some((r) => ['needs-sign-in', 'needs-merge', 'failed'].includes(r))
  ) {
    return 'error'
  }
  if (reasons.some((r) => ['waiting-to-retry', 'possibly-stuck'].includes(r))) {
    return 'offline'
  }
  return reasons.length > 0 ? 'saving' : null
}

/** The first lane that needs attention, as the error it surfaces. */
export function outboxLaneError(
  snapshot: OutboxSnapshot,
): CloudBackendError | null {
  const { queue, summary } = snapshot
  for (const lane of queue.lanes) {
    if (lane.status.tag === 'needs-merge') {
      return {
        kind: 'conflict',
        currentRevision: Number(lane.status.val.currentRevision),
      }
    }
  }
  for (const lane of summary.lanes) {
    if (lane.reason === 'needs-sign-in') return { kind: 'auth' }
    if (lane.reason === 'failed') {
      return { kind: 'unknown', message: lane.failureReason ?? 'failed' }
    }
  }
  const waiting = summary.lanes.some(
    (lane) =>
      lane.reason === 'waiting-to-retry' || lane.reason === 'possibly-stuck',
  )
  return waiting ? { kind: 'network' } : null
}

/**
 * The text of the newest unsent save of each file, keyed by file id. A fresh
 * listing returns the server's text, which is older than these edits, so they
 * are laid over it to keep what the user typed.
 */
export function pendingContentByFileId(
  snapshot: OutboxSnapshot,
): Map<string, string> {
  const pending = new Map<string, string>()
  for (const lane of snapshot.queue.lanes) {
    const saves = lane.messages.flatMap(({ message }) =>
      message.tag === 'save-content' ? [message.val.content] : [],
    )
    const newest = saves.at(-1)
    if (newest !== undefined) pending.set(lane.fileId, newest)
  }
  return pending
}
