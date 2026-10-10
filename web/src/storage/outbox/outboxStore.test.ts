import 'fake-indexeddb/auto'
import { describe, expect, it } from 'vitest'
import { createMemoryOutboxStore } from './memoryOutboxStore'
import { createIndexedDbOutboxStore, type OutboxStore } from './outboxStore'

const factories: Array<{ name: string; create: () => OutboxStore }> = [
  {
    name: 'indexeddb',
    create: () => createIndexedDbOutboxStore('test-account'),
  },
  { name: 'memory', create: createMemoryOutboxStore },
]

describe.each(factories)('$name outbox store', ({ name, create }) => {
  const freshStore = () => {
    if (name === 'indexeddb') {
      globalThis.indexedDB = new IDBFactory()
    }
    return create()
  }

  it('loads an empty store', async () => {
    expect(await freshStore().loadAll()).toEqual([])
  })

  it('loads what was put', async () => {
    const store = freshStore()
    await store.apply({ puts: [{ key: 'a', value: '1' }], deletes: [] })
    expect(await store.loadAll()).toEqual([{ key: 'a', value: '1' }])
  })

  it('deletes records', async () => {
    const store = freshStore()
    await store.apply({
      puts: [
        { key: 'a', value: '1' },
        { key: 'b', value: '2' },
      ],
      deletes: [],
    })
    await store.apply({ puts: [], deletes: ['a'] })
    expect(await store.loadAll()).toEqual([{ key: 'b', value: '2' }])
  })

  it('applies puts and deletes together', async () => {
    const store = freshStore()
    await store.apply({ puts: [{ key: 'a', value: '1' }], deletes: [] })
    await store.apply({ puts: [{ key: 'b', value: '2' }], deletes: ['a'] })
    expect(await store.loadAll()).toEqual([{ key: 'b', value: '2' }])
  })

  it('leaves the previous state when a transaction fails midway', async () => {
    const store = freshStore()
    await store.apply({ puts: [{ key: 'a', value: '1' }], deletes: [] })
    const invalid = { key: 'c', value: 3 } as unknown as {
      key: string
      value: string
    }
    await expect(
      store.apply({
        puts: [{ key: 'b', value: '2' }, invalid],
        deletes: ['a'],
      }),
    ).rejects.toThrow()
    expect(await store.loadAll()).toEqual([{ key: 'a', value: '1' }])
  })
})
