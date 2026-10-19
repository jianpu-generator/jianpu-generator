import {
  type DecodeError,
  jianpuWasm,
  type Queue,
  type StoredRecord,
} from '../../jianpuWasm'
import type { StoreLock } from './outboxLock'
import type { OutboxChanges, OutboxStore } from './outboxStore'

/** Only these keys belong to the queue codec; anything else in the store
 * (e.g. `base:<fileId>` snapshots) is left alone when persisting. */
const CODEC_KEY_PREFIXES = ['lane:', 'meta:']
/** Delivered messages kept for the history view; written by the looper but
 * not part of the queue codec. */
const SENT_KEY_PREFIX = 'sent:'

function isCodecKey(key: string): boolean {
  return CODEC_KEY_PREFIXES.some((prefix) => key.startsWith(prefix))
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

export interface TransactOptions {
  /** Clear every `in_flight` and wake `Waiting` lanes: only for a tab
   * taking over as leader, since the previous one's sends are over. */
  recover?: boolean
  /** More records to write in the same transaction, from the stored queue
   * and delivered messages before the change. A throw only drops these. */
  alsoWrite?(stored: Queue, sentRecords: StoredRecord[]): OutboxChanges
}

/** What a transaction left in the store. */
export type Transacted =
  | { tag: 'written'; queue: Queue; sentRecords: StoredRecord[] }
  | { tag: 'halted'; error: DecodeError }

/**
 * Reads the stored queue, changes it and writes back what changed, all
 * while holding `withStoreLock`, so no tab ever writes over a change it has
 * not read (specs/outbox-tabs). Nothing is written when the stored queue
 * cannot be decoded.
 */
export function transact(
  store: OutboxStore,
  withStoreLock: StoreLock,
  change: (stored: Queue) => Queue,
  options: TransactOptions = {},
): Promise<Transacted> {
  return withStoreLock(async (): Promise<Transacted> => {
    const records = await store.loadAll()
    const sentRecords = records.filter((record) =>
      record.key.startsWith(SENT_KEY_PREFIX),
    )
    const owned = records.filter((record) => isCodecKey(record.key))
    let stored: Queue
    try {
      stored = options.recover
        ? jianpuWasm().outboxDecodeQueue(owned)
        : jianpuWasm().outboxDecodeStoredQueue(owned)
    } catch (error) {
      const decodeError = decodeErrorOf(error)
      if (!decodeError) throw error
      return { tag: 'halted', error: decodeError }
    }
    let also: OutboxChanges = { puts: [], deletes: [] }
    try {
      also = options.alsoWrite?.(stored, sentRecords) ?? also
    } catch {
      // Extra records are a courtesy; the queue change still goes ahead.
    }
    const queue = change(stored)
    const before = new Map(owned.map((record) => [record.key, record.value]))
    const encoded = jianpuWasm().outboxEncodeQueue(queue)
    const kept = new Set(encoded.map((record) => record.key))
    const puts = [
      ...encoded.filter((record) => before.get(record.key) !== record.value),
      ...also.puts,
    ]
    const deletes = [
      ...[...before.keys()].filter((key) => !kept.has(key)),
      ...also.deletes,
    ]
    if (puts.length > 0 || deletes.length > 0) {
      await store.apply({ puts, deletes })
    }
    const deleted = new Set(also.deletes)
    return {
      tag: 'written',
      queue,
      sentRecords: [
        ...sentRecords.filter((record) => !deleted.has(record.key)),
        ...also.puts,
      ],
    }
  })
}
