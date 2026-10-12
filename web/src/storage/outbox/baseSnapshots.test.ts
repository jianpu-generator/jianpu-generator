import { describe, expect, it } from 'vitest'
import { createBaseSnapshots } from './baseSnapshots'
import { createMemoryOutboxStore } from './memoryOutboxStore'

describe('base snapshots', () => {
  it('reads back what was recorded and overwrites on re-record', async () => {
    const bases = createBaseSnapshots(createMemoryOutboxStore())
    await bases.recordBase('file-a', 'one')
    expect(await bases.readBase('file-a')).toBe('one')
    await bases.recordBase('file-a', 'two')
    expect(await bases.readBase('file-a')).toBe('two')
  })

  it('reads undefined for an unknown file', async () => {
    const bases = createBaseSnapshots(createMemoryOutboxStore())
    expect(await bases.readBase('missing')).toBeUndefined()
  })

  it('drops only the named file', async () => {
    const bases = createBaseSnapshots(createMemoryOutboxStore())
    await bases.recordBase('file-a', 'a')
    await bases.recordBase('file-b', 'b')
    await bases.dropBase('file-a')
    expect(await bases.readBase('file-a')).toBeUndefined()
    expect(await bases.readBase('file-b')).toBe('b')
  })

  it('stores an empty text as a real value', async () => {
    const bases = createBaseSnapshots(createMemoryOutboxStore())
    await bases.recordBase('file-a', '')
    expect(await bases.readBase('file-a')).toBe('')
  })
})
