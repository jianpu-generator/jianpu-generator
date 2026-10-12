import { describe, expect, it } from 'vitest'
import {
  applyPendingStructure,
  pendingContentByFileId,
} from './cloudBackendOutbox'
import type { OutboxSnapshot } from './outbox/outboxLooper'

function snapshotWith(
  lanes: Array<{ fileId: string; saves: string[] }>,
): OutboxSnapshot {
  return {
    halted: null,
    summary: {
      lanes: [],
      totalPending: 0,
      lanesNeedingAttention: 0,
      hasUnsynced: false,
    },
    queue: {
      revisions: [],
      lanes: lanes.map(({ fileId, saves }) => ({
        fileId,
        messages: saves.map((content, index) => ({
          message: { tag: 'save-content', val: { content } },
          messageId: `m${index}`,
        })),
        status: { tag: 'idle' },
      })),
    },
  } as unknown as OutboxSnapshot
}

describe('pendingContentByFileId', () => {
  it('returns the newest unsent save of each file', () => {
    const pending = pendingContentByFileId(
      snapshotWith([
        { fileId: 'a', saves: ['one', 'two'] },
        { fileId: 'b', saves: [] },
      ]),
    )
    expect([...pending]).toEqual([['a', 'two']])
  })
})

describe('applyPendingStructure', () => {
  const server = [
    { id: 'a', name: 'a.jianpu', content: 'A', revision: 1, trashedAt: null },
    { id: 'b', name: 'b.jianpu', content: 'B', revision: 1, trashedAt: null },
  ]
  const snapshotOf = (
    lanes: Array<{ fileId: string; messages: unknown[] }>,
  ): OutboxSnapshot =>
    ({
      queue: {
        revisions: [],
        lanes: lanes.map(({ fileId, messages }) => ({
          fileId,
          status: { tag: 'idle' },
          messages: messages.map((message, index) => ({
            message,
            messageId: `m${index}`,
          })),
        })),
      },
    }) as unknown as OutboxSnapshot

  it('keeps a file created but not yet sent', () => {
    const result = applyPendingStructure(
      server,
      snapshotOf([
        {
          fileId: 'c',
          messages: [
            { tag: 'create-file', val: { name: 'c.jianpu', content: 'C' } },
          ],
        },
      ]),
    )
    expect(result.map((file) => file.name)).toEqual([
      'a.jianpu',
      'b.jianpu',
      'c.jianpu',
    ])
  })

  it('applies an unsent rename, trash and restore in order', () => {
    const result = applyPendingStructure(
      server,
      snapshotOf([
        {
          fileId: 'a',
          messages: [{ tag: 'rename-file', val: { to: 'z.jianpu' } }],
        },
        {
          fileId: 'b',
          messages: [{ tag: 'trash-file' }, { tag: 'restore-file' }],
        },
      ]),
    )
    expect(result.find((file) => file.id === 'a')?.name).toBe('z.jianpu')
    expect(result.find((file) => file.id === 'b')?.trashedAt).toBeNull()
  })
})
