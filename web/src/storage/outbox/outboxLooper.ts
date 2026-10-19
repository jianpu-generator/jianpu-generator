import {
  type DecodeError,
  type FileName,
  type HistoryFilter,
  type HistoryRow,
  jianpuWasm,
  type Message,
  type Queue,
  type QueueSummary,
  type Resolution,
  type SendRequest,
  type SendResult,
  type StoredRecord,
} from '../../jianpuWasm'
import { type StoreLock, serialLock } from './outboxLock'
import type { Scheduler } from './outboxScheduler'
import type { OutboxChanges, OutboxStore } from './outboxStore'
import {
  type TransactOptions,
  transact as transactStored,
} from './outboxTransaction'

export interface OutboxSnapshot {
  queue: Queue
  summary: QueueSummary
  /** Set when the stored queue could not be decoded. The looper then neither
   * sends nor writes, so the stored data is never lost. */
  halted: DecodeError | null
}

export interface OutboxLooperDeps {
  store: OutboxStore
  /** Delivers one request. Decides nothing: classification is `sendMessage`'s job. */
  sendMessage(request: SendRequest): Promise<SendResult>
  /** Builds the wake-up scheduler around the looper's own wake callback. */
  createScheduler(onWake: () => void): Scheduler
  runAsLeader(task: (signal: AbortSignal) => Promise<void>): () => void
  /** Held by every read-change-write of the stored queue, across every tab
   * of the account. Defaults to one tab's tasks running one at a time. */
  withStoreLock?: StoreLock
  now(): number
  random(): number
  /** Called after a successful send has been applied and persisted. A thrown
   * error is ignored so it never breaks the drain loop. */
  onAcknowledged?(request: SendRequest, revision: bigint): void | Promise<void>
  /** Called after a discard resolution has been persisted. Errors are ignored. */
  onDiscarded?(fileId: string): void | Promise<void>
  /** Tells the other tabs of this account that the stored queue changed. */
  changes?: OutboxChangeChannel
}

export interface OutboxChangeChannel {
  announce(): void
  listen(onChange: () => void): () => void
}

export interface OutboxLooper {
  start(): Promise<void>
  stop(): void
  enqueue(fileId: string, message: Message): Promise<void>
  resolve(fileId: string, resolution: Resolution): Promise<void>
  resolveSignedIn(): Promise<void>
  recordRevision(fileId: string, revision: bigint): Promise<void>
  subscribe(listener: (snapshot: OutboxSnapshot) => void): () => void
  snapshot(): OutboxSnapshot
  /** Queued and delivered messages that pass the filter, newest first. The
   * delivered ones are those last read from the store; `refreshHistory`
   * re-reads them (e.g. after another tab delivered something). */
  historyRows(fileNames: FileName[], filter: HistoryFilter): HistoryRow[]
  refreshHistory(): Promise<void>
  /** Resolves once the current drain (if any) has finished and persisted. */
  whenIdle(): Promise<void>
}

export function createOutboxLooper(deps: OutboxLooperDeps): OutboxLooper {
  const { store, sendMessage, runAsLeader, now, random } = deps
  const withStoreLock = deps.withStoreLock ?? serialLock()
  const nowMs = () => BigInt(now())

  /** The stored queue as this tab last read or wrote it. Only shown: every
   * change re-reads the store first. */
  let queue: Queue = { lanes: [], revisions: [] }
  let halted: DecodeError | null = null
  let started = false
  let isLeader = false
  let stopLeading: (() => void) | null = null
  let stopListening: (() => void) | null = null
  let sentRecords: StoredRecord[] = []
  let lastTransaction: Promise<unknown> = Promise.resolve()
  let drainPromise: Promise<void> | null = null
  let drainRequested = false
  const listeners = new Set<(snapshot: OutboxSnapshot) => void>()

  /** Until the stored queue has been read (which needs the wasm component),
   * there is nothing to summarise, and asking wasm would throw. */
  let loaded = false

  const snapshot = (): OutboxSnapshot => ({
    queue,
    summary: loaded
      ? jianpuWasm().outboxSummarize(queue, nowMs())
      : {
          lanes: [],
          totalPending: 0,
          lanesNeedingAttention: 0,
          hasUnsynced: false,
        },
    halted,
  })

  const notify = () => {
    if (listeners.size === 0) return
    const current = snapshot()
    for (const listener of listeners) listener(current)
  }

  /** `transact` on the store, keeping what it read for the UI. Resolves to
   * false, having written nothing, when the stored queue cannot be decoded. */
  const transact = (
    change: (stored: Queue) => Queue,
    options: TransactOptions = {},
  ): Promise<boolean> => {
    const run = transactStored(store, withStoreLock, change, options).then(
      (outcome) => {
        if (outcome.tag === 'halted') {
          halted = outcome.error
          return false
        }
        queue = outcome.queue
        sentRecords = outcome.sentRecords
        halted = null
        return true
      },
    )
    lastTransaction = run.catch(() => undefined)
    return run
  }

  /** Re-reads the stored queue, e.g. after another tab changed it. */
  const refresh = (): Promise<boolean> => transact((stored) => stored)

  /** Keeps the delivered message for the history view and prunes old ones,
   * in the same transaction that drops it from the queue. */
  const recordSent =
    (request: SendRequest, revision: bigint) =>
    (stored: Queue, sent: StoredRecord[]): OutboxChanges => {
      const write = jianpuWasm().outboxRecordSent(
        stored,
        sent,
        request.fileId,
        request.messageId,
        revision,
        nowMs(),
      )
      return { puts: write.put ? [write.put] : [], deletes: write.deletes }
    }

  const wakeAtNextDue = () => {
    const next = jianpuWasm().outboxNextWakeMs(queue)
    scheduler?.wakeAt(next === undefined ? null : Number(next))
  }

  const runHook = async (hook: () => void | Promise<void>): Promise<void> => {
    try {
      await hook()
    } catch {
      // A failing hook must never break the drain loop.
    }
  }

  const drainOnce = async (signal: AbortSignal): Promise<void> => {
    while (!signal.aborted && !halted) {
      const begun: { request?: SendRequest } = {}
      await transact((stored) => {
        const next = jianpuWasm().outboxBeginSend(stored, nowMs())
        begun.request = next.request
        return next.queue
      })
      notify()
      const { request } = begun
      if (!request) break
      const result = await sendMessage(request)
      await transact(
        (stored) =>
          jianpuWasm().outboxOnResult(
            stored,
            request.fileId,
            request.messageId,
            result,
            nowMs(),
            random(),
          ),
        result.tag === 'ok'
          ? { alsoWrite: recordSent(request, result.val.revision) }
          : {},
      )
      notify()
      if (result.tag === 'ok') {
        await runHook(() => deps.onAcknowledged?.(request, result.val.revision))
      }
    }
    wakeAtNextDue()
  }

  let leaderSignal: AbortSignal | null = null

  const requestDrain = (): Promise<void> => {
    if (!isLeader || !leaderSignal) return Promise.resolve()
    if (drainPromise) {
      drainRequested = true
      return drainPromise
    }
    const signal = leaderSignal
    drainPromise = (async () => {
      try {
        do {
          drainRequested = false
          await drainOnce(signal)
        } while (drainRequested && !signal.aborted)
      } finally {
        drainPromise = null
      }
    })()
    return drainPromise
  }

  /** Exists only while started: stopping disposes it, starting rebuilds it. */
  let scheduler: Scheduler | null = null

  const mutate = async (change: (stored: Queue) => Queue): Promise<void> => {
    if (halted) {
      throw new Error(
        'The outbox could not be read, so it will not accept new messages',
      )
    }
    if (!(await transact(change))) {
      throw new Error('The outbox could not be read')
    }
    notify()
    deps.changes?.announce()
    void requestDrain().catch(() => undefined)
  }

  /** Another tab changed the stored queue: show it, and deliver it if this
   * tab leads. */
  const adoptExternalChange = async (): Promise<void> => {
    await refresh()
    notify()
    void requestDrain().catch(() => undefined)
  }

  return {
    async start() {
      if (started) return
      started = true
      scheduler = deps.createScheduler(() => {
        void requestDrain().catch(() => undefined)
      })
      await refresh()
      if (!started) return
      loaded = true
      notify()
      stopListening =
        deps.changes?.listen(() => {
          void adoptExternalChange().catch(() => undefined)
        }) ?? null
      stopLeading = runAsLeader(async (signal) => {
        // Whatever the previous leader was sending, it no longer is.
        await transact((stored) => stored, { recover: true })
        notify()
        isLeader = true
        leaderSignal = signal
        void requestDrain().catch(() => undefined)
        await new Promise<void>((resolve) => {
          if (signal.aborted) resolve()
          else signal.addEventListener('abort', () => resolve(), { once: true })
        })
        isLeader = false
        leaderSignal = null
      })
    },
    stop() {
      started = false
      stopLeading?.()
      stopLeading = null
      stopListening?.()
      stopListening = null
      isLeader = false
      scheduler?.dispose()
      scheduler = null
    },
    enqueue: (fileId, message) =>
      mutate((stored) =>
        jianpuWasm().outboxEnqueue(
          stored,
          fileId,
          message,
          crypto.randomUUID(),
          nowMs(),
        ),
      ),
    resolve: async (fileId, resolution) => {
      await mutate((stored) =>
        jianpuWasm().outboxResolve(stored, fileId, resolution),
      )
      if (resolution.tag === 'discard') {
        await runHook(() => deps.onDiscarded?.(fileId))
      }
    },
    resolveSignedIn: () =>
      mutate((stored) => jianpuWasm().outboxResolveSignedIn(stored)),
    recordRevision: (fileId, revision) =>
      mutate((stored) =>
        jianpuWasm().outboxRecordRevision(stored, fileId, revision),
      ),
    subscribe(listener) {
      listeners.add(listener)
      if (started) listener(snapshot())
      return () => listeners.delete(listener)
    },
    snapshot,
    historyRows: (fileNames, filter) =>
      loaded
        ? jianpuWasm().outboxHistoryRows(
            queue,
            sentRecords,
            fileNames,
            filter,
            nowMs(),
          )
        : [],
    async refreshHistory() {
      await refresh()
      notify()
    },
    async whenIdle() {
      await drainPromise
      await lastTransaction
    },
  }
}
