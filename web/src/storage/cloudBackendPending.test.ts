import { describe, expect, it } from 'vitest'
import { pendingContentByFileId } from './cloudBackendOutbox'
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
