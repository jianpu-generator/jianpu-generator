import {
  callWorker,
  type createWorkerClient,
} from '../syncedShare/workerClient'
import type { CloudBackendError } from './cloudBackendTypes'
import { runAsLeader } from './outbox/outboxLock'
import {
  createOutboxLooper,
  type OutboxLooper,
  type OutboxSnapshot,
} from './outbox/outboxLooper'
import { createScheduler } from './outbox/outboxScheduler'
import { createIndexedDbOutboxStore } from './outbox/outboxStore'
import { sendMessage } from './outbox/sendMessage'
import type { SaveStatus } from './types'

export interface CloudOutboxConfig {
  client: ReturnType<typeof createWorkerClient>
  identityToken: string
  /** The file's current local name, for a `RestoreFile` message. */
  restoreName(fileId: string): string
}

/** The production looper: IndexedDB store, leader-tab lock, real timers. */
export function createCloudOutboxLooper(
  config: CloudOutboxConfig,
): OutboxLooper {
  const { client, identityToken, restoreName } = config
  const fetchServerContent = async (fileId: string): Promise<string> => {
    const response = await callWorker(
      client.POST('/files/list', { body: { identityToken } }),
    )
    return response.files.find((file) => file.id === fileId)?.content ?? ''
  }
  return createOutboxLooper({
    store: createIndexedDbOutboxStore(),
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
