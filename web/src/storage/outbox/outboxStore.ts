import { type DBSchema, openDB } from 'idb'

export interface StoredRecord {
  key: string
  value: string
}

export interface OutboxChanges {
  puts: StoredRecord[]
  deletes: string[]
}

export interface OutboxStore {
  loadAll(): Promise<StoredRecord[]>
  /** Applies every put and delete in one atomic transaction. */
  apply(changes: OutboxChanges): Promise<void>
}

const DATABASE_NAME = 'jianpu-outbox'
const OBJECT_STORE_NAME = 'records'

interface OutboxSchema extends DBSchema {
  records: { key: string; value: string }
}

export function assertStoredRecord(record: StoredRecord): void {
  if (typeof record.key !== 'string' || typeof record.value !== 'string') {
    throw new TypeError(
      'Outbox records must have a string key and a string value',
    )
  }
}

/** One database per account, so edits queued by one account are never sent
 * with another account's token. */
export function createIndexedDbOutboxStore(account: string): OutboxStore {
  const databasePromise = openDB<OutboxSchema>(
    `${DATABASE_NAME}:${account}`,
    1,
    {
      upgrade(database) {
        database.createObjectStore(OBJECT_STORE_NAME)
      },
    },
  )

  return {
    async loadAll() {
      const database = await databasePromise
      const transaction = database.transaction(OBJECT_STORE_NAME, 'readonly')
      const [keys, values] = await Promise.all([
        transaction.store.getAllKeys(),
        transaction.store.getAll(),
      ])
      return keys.map((key, index) => ({ key, value: values[index] ?? '' }))
    },
    async apply(changes) {
      const database = await databasePromise
      const transaction = database.transaction(OBJECT_STORE_NAME, 'readwrite')
      // Requests queued before an abort reject too; `transaction.done` reports the failure.
      const settled = (request: Promise<unknown>) => {
        request.catch(() => undefined)
        return request
      }
      const done = settled(transaction.done)
      try {
        const operations = [
          ...changes.puts.map((record) => {
            assertStoredRecord(record)
            return settled(transaction.store.put(record.value, record.key))
          }),
          ...changes.deletes.map((key) =>
            settled(transaction.store.delete(key)),
          ),
        ]
        await Promise.all([...operations, done])
      } catch (error) {
        try {
          transaction.abort()
        } catch {
          // the transaction already finished or aborted
        }
        throw error
      }
    },
  }
}
