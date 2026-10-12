import type { OutboxStore } from './outboxStore'

const BASE_KEY_PREFIX = 'base:'

const baseKey = (fileId: string): string => `${BASE_KEY_PREFIX}${fileId}`

/** The last text the server is known to have, per file. The text is opaque:
 * this module only stores it and never decides anything from it. */
export interface BaseSnapshots {
  recordBase(fileId: string, content: string): Promise<void>
  readBase(fileId: string): Promise<string | undefined>
  dropBase(fileId: string): Promise<void>
}

export function createBaseSnapshots(store: OutboxStore): BaseSnapshots {
  return {
    recordBase: (fileId, content) =>
      store.apply({
        puts: [{ key: baseKey(fileId), value: content }],
        deletes: [],
      }),
    async readBase(fileId) {
      const key = baseKey(fileId)
      return (await store.loadAll()).find((record) => record.key === key)?.value
    },
    dropBase: (fileId) => store.apply({ puts: [], deletes: [baseKey(fileId)] }),
  }
}
