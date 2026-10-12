import {
  assertStoredRecord,
  type OutboxChanges,
  type OutboxStore,
  type StoredRecord,
} from './outboxStore'

export function createMemoryOutboxStore(): OutboxStore {
  let records = new Map<string, string>()

  return {
    async loadAll(): Promise<StoredRecord[]> {
      return [...records].map(([key, value]) => ({ key, value }))
    },
    async apply(changes: OutboxChanges) {
      const staged = new Map(records)
      for (const record of changes.puts) {
        assertStoredRecord(record)
        staged.set(record.key, record.value)
      }
      for (const key of changes.deletes) {
        staged.delete(key)
      }
      records = staged
    },
  }
}
