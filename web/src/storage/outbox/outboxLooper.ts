import {
  type DecodeError,
  jianpuWasm,
  type Message,
  type Queue,
  type QueueSummary,
  type Resolution,
  type SendRequest,
  type SendResult,
  type StoredRecord,
} from '../../jianpuWasm'
import type { Scheduler } from './outboxScheduler'
import type { OutboxStore } from './outboxStore'

/** Only these keys belong to the queue codec; anything else in the store
 * (e.g. `base:<fileId>` snapshots) is left alone when persisting. */
const CODEC_KEY_PREFIXES = ['lane:', 'meta:']

function isCodecKey(key: string): boolean {
  return CODEC_KEY_PREFIXES.some((prefix) => key.startsWith(prefix))
}

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
  now(): number
  random(): number
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
  /** Resolves once the current drain (if any) has finished and persisted. */
  whenIdle(): Promise<void>
}

function decodeErrorOf(error: unknown): DecodeError | null {
  const payload = (error as { payload?: unknown } | null)?.payload
  if (
    typeof payload === 'object' &&
    payload !== null &&
    'tag' in payload &&
    (payload.tag === 'unsupported-version' || payload.tag === 'corrupt')
  ) {
    return payload as DecodeError
  }
  return null
}

export function createOutboxLooper(deps: OutboxLooperDeps): OutboxLooper {
  const { store, sendMessage, runAsLeader, now, random } = deps
  const nowMs = () => BigInt(now())

  let queue: Queue = { lanes: [], revisions: [] }
  let halted: DecodeError | null = null
  let started = false
  let isLeader = false
  let stopLeading: (() => void) | null = null
  let persisted = new Map<string, string>()
  let persistChain: Promise<void> = Promise.resolve()
  let drainPromise: Promise<void> | null = null
  let drainRequested = false
  const listeners = new Set<(snapshot: OutboxSnapshot) => void>()

  const snapshot = (): OutboxSnapshot => ({
    queue,
    summary: jianpuWasm().outboxSummarize(queue, nowMs()),
    halted,
  })

  const notify = () => {
    if (listeners.size === 0) return
    const current = snapshot()
    for (const listener of listeners) listener(current)
  }

  const persistNow = async (): Promise<void> => {
    if (halted) return
    const records = jianpuWasm().outboxEncodeQueue(queue)
    const next = new Map(records.map((record) => [record.key, record.value]))
    const puts = records.filter(
      (record) => persisted.get(record.key) !== record.value,
    )
    const deletes = [...persisted.keys()].filter((key) => !next.has(key))
    if (puts.length === 0 && deletes.length === 0) return
    await store.apply({ puts, deletes })
    persisted = next
  }

  /** Serialised so concurrent mutations never interleave their store writes;
   * each run encodes whatever the queue is by the time it executes. */
  const persist = (): Promise<void> => {
    persistChain = persistChain.then(persistNow, persistNow)
    return persistChain
  }

  const load = async (): Promise<void> => {
    const records: StoredRecord[] = await store.loadAll()
    const owned = records.filter((record) => isCodecKey(record.key))
    try {
      queue = jianpuWasm().outboxDecodeQueue(owned)
      halted = null
      persisted = new Map(owned.map((record) => [record.key, record.value]))
    } catch (error) {
      const decodeError = decodeErrorOf(error)
      if (!decodeError) throw error
      halted = decodeError
    }
  }

  const wakeAtNextDue = () => {
    const next = jianpuWasm().outboxNextWakeMs(queue)
    scheduler.wakeAt(next === undefined ? null : Number(next))
  }

  const drainOnce = async (signal: AbortSignal): Promise<void> => {
    while (!signal.aborted && !halted) {
      const begun = jianpuWasm().outboxBeginSend(queue, nowMs())
      queue = begun.queue
      await persist()
      notify()
      const { request } = begun
      if (!request) break
      const result = await sendMessage(request)
      queue = jianpuWasm().outboxOnResult(
        queue,
        request.fileId,
        request.messageId,
        result,
        nowMs(),
        random(),
      )
      await persist()
      notify()
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

  const scheduler = deps.createScheduler(() => {
    void requestDrain().catch(() => undefined)
  })

  const mutate = async (change: () => Queue): Promise<void> => {
    if (halted) {
      throw new Error(
        'The outbox could not be read, so it will not accept new messages',
      )
    }
    queue = change()
    await persist()
    notify()
    void requestDrain().catch(() => undefined)
  }

  return {
    async start() {
      if (started) return
      started = true
      await load()
      notify()
      stopLeading = runAsLeader(async (signal) => {
        // The previous leader may have changed the stored queue since we loaded.
        await load()
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
      isLeader = false
      scheduler.dispose()
    },
    enqueue: (fileId, message) =>
      mutate(() =>
        jianpuWasm().outboxEnqueue(
          queue,
          fileId,
          message,
          crypto.randomUUID(),
          nowMs(),
        ),
      ),
    resolve: (fileId, resolution) =>
      mutate(() => jianpuWasm().outboxResolve(queue, fileId, resolution)),
    resolveSignedIn: () =>
      mutate(() => jianpuWasm().outboxResolveSignedIn(queue)),
    recordRevision: (fileId, revision) =>
      mutate(() => jianpuWasm().outboxRecordRevision(queue, fileId, revision)),
    subscribe(listener) {
      listeners.add(listener)
      if (started) listener(snapshot())
      return () => listeners.delete(listener)
    },
    snapshot,
    async whenIdle() {
      await drainPromise
      await persistChain
    },
  }
}
