import {
  ORIGIN_MIGRATED_KEY,
  type OriginMigrationPayload,
  SAVED_LOCAL_STORAGE_PREFIX,
  type SavedDatabase,
} from '../originMigration/originMigrationPayload'
import {
  createIndexedDbOutboxStore,
  OUTBOX_DATABASE_PREFIX,
} from '../storage/outbox/outboxStore'

/** Everything this origin saved: the app's `localStorage` entries and every
 * account's cloud outbox. Storage that can't be read counts as empty. */
export async function collectSavedData(): Promise<OriginMigrationPayload> {
  return {
    version: 1,
    localStorage: savedLocalStorage(),
    indexedDb: await savedOutboxes(),
  }
}

function savedLocalStorage(): Record<string, string> {
  try {
    return Object.fromEntries(
      Array.from({ length: localStorage.length }, (_, index) =>
        localStorage.key(index),
      )
        .filter(
          (key): key is string =>
            key?.startsWith(SAVED_LOCAL_STORAGE_PREFIX) === true &&
            key !== ORIGIN_MIGRATED_KEY,
        )
        .flatMap((key) => {
          const value = localStorage.getItem(key)
          return value === null ? [] : [[key, value] as const]
        }),
    )
  } catch {
    return {}
  }
}

async function savedOutboxes(): Promise<SavedDatabase[]> {
  const databases = await indexedDB.databases().catch(() => [])
  const names = databases
    .map((database) => database.name)
    .filter(
      (name): name is string =>
        name?.startsWith(OUTBOX_DATABASE_PREFIX) === true,
    )
  return Promise.all(
    names.map(async (name) => ({
      name,
      records: await createIndexedDbOutboxStore(
        name.slice(OUTBOX_DATABASE_PREFIX.length),
      )
        .loadAll()
        .catch(() => []),
    })),
  )
}
