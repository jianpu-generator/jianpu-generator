import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Message, SendRequest } from '../../jianpuWasm'
import { suggestNameAfterCollision } from '../cloudBackendNaming'
import { type SendMessageDeps, sendMessage } from './sendMessage'

interface FakeReply {
  status: number
  body?: unknown
}

type Post = (
  path: string,
  options: { signal?: AbortSignal; body: unknown },
) => Promise<unknown>

function replyWith({ status, body }: FakeReply) {
  const response = { ok: status >= 200 && status < 300, status } as Response
  return status >= 200 && status < 300
    ? { data: body, response }
    : { error: body, response }
}

function makeDeps(
  post: Post,
  overrides: Partial<SendMessageDeps> = {},
): SendMessageDeps {
  return {
    client: { POST: post } as unknown as SendMessageDeps['client'],
    identityToken: 'token',
    fetchServerContent: async () => 'server text',
    restoreName: () => 'restored.jianpu',
    ...overrides,
  }
}

function requestFor(message: Message, expectedRevision = 3n): SendRequest {
  return {
    fileId: 'file-1',
    messageId: 'message-1',
    message,
    expectedRevision,
  }
}

const saveContent = (content: string): Message => ({
  tag: 'save-content',
  val: { content },
})

const respond =
  (reply: FakeReply): Post =>
  async () =>
    replyWith(reply)

afterEach(() => {
  vi.useRealTimers()
})

describe('sendMessage success', () => {
  it('saves content and returns the new revision', async () => {
    const post = vi.fn<Post>(async () =>
      replyWith({ status: 200, body: { revision: 4 } }),
    )
    const result = await sendMessage(
      requestFor(saveContent('1 2 3')),
      makeDeps(post),
    )
    expect(result).toEqual({ tag: 'ok', val: { revision: 4n } })
    expect(post).toHaveBeenCalledWith(
      '/files/{id}/content',
      expect.objectContaining({
        body: { identityToken: 'token', content: '1 2 3', expectedRevision: 3 },
      }),
    )
  })

  it('creates a file with the lane id', async () => {
    const post = vi.fn<Post>(async () =>
      replyWith({ status: 200, body: { revision: 1 } }),
    )
    const result = await sendMessage(
      requestFor({
        tag: 'create-file',
        val: { name: 'a.jianpu', content: 'x' },
      }),
      makeDeps(post),
    )
    expect(result).toEqual({ tag: 'ok', val: { revision: 1n } })
    expect(post).toHaveBeenCalledWith(
      '/files',
      expect.objectContaining({
        body: {
          identityToken: 'token',
          id: 'file-1',
          name: 'a.jianpu',
          content: 'x',
        },
      }),
    )
  })

  it.each([
    [
      { tag: 'rename-file', val: { to: 'b.jianpu' } } as Message,
      '/files/{id}/rename',
    ],
    [{ tag: 'trash-file' } as Message, '/files/{id}/delete'],
    [{ tag: 'restore-file' } as Message, '/files/{id}/restore'],
  ])('routes %j and keeps the recorded revision for an empty body', async (message, path) => {
    const post = vi.fn<Post>(async () => replyWith({ status: 200 }))
    const result = await sendMessage(requestFor(message), makeDeps(post))
    expect(result).toEqual({ tag: 'ok', val: { revision: 3n } })
    expect(post).toHaveBeenCalledWith(path, expect.anything())
  })

  it('restores under the current local name', async () => {
    const post = vi.fn<Post>(async () => replyWith({ status: 200 }))
    await sendMessage(requestFor({ tag: 'restore-file' }), makeDeps(post))
    expect(post).toHaveBeenCalledWith(
      '/files/{id}/restore',
      expect.objectContaining({
        body: { identityToken: 'token', name: 'restored.jianpu' },
      }),
    )
  })
})

describe('sendMessage failures', () => {
  it('treats a network error as transient', async () => {
    const post: Post = async () => {
      throw new TypeError('Failed to fetch')
    }
    const result = await sendMessage(
      requestFor(saveContent('x')),
      makeDeps(post),
    )
    expect(result.tag).toBe('transient')
  })

  it('aborts after the timeout and reports transient', async () => {
    vi.useFakeTimers()
    const post: Post = (_path, { signal }) =>
      new Promise((_resolve, reject) => {
        signal?.addEventListener('abort', () => reject(new Error('aborted')))
      })
    const pending = sendMessage(
      requestFor(saveContent('x')),
      makeDeps(post, { timeoutMs: 15_000 }),
    )
    await vi.advanceTimersByTimeAsync(15_000)
    expect((await pending).tag).toBe('transient')
  })

  it.each([500, 503, 429])('treats status %i as transient', async (status) => {
    const result = await sendMessage(
      requestFor(saveContent('x')),
      makeDeps(respond({ status, body: { code: 'internal', message: 'm' } })),
    )
    expect(result.tag).toBe('transient')
  })

  it('treats a 5xx with a non-JSON body as transient', async () => {
    const result = await sendMessage(
      requestFor(saveContent('x')),
      makeDeps(respond({ status: 502 })),
    )
    expect(result.tag).toBe('transient')
  })

  it('maps 401 to unauthorized', async () => {
    const result = await sendMessage(
      requestFor(saveContent('x')),
      makeDeps(respond({ status: 401 })),
    )
    expect(result).toEqual({ tag: 'unauthorized' })
  })

  it('maps the unauthorized error code to unauthorized', async () => {
    const result = await sendMessage(
      requestFor(saveContent('x')),
      makeDeps(
        respond({
          status: 403,
          body: { code: 'unauthorized', attempts: 1, failedAt: 0, reason: 'r' },
        }),
      ),
    )
    expect(result).toEqual({ tag: 'unauthorized' })
  })

  it('maps any other 4xx to permanent', async () => {
    const result = await sendMessage(
      requestFor(saveContent('x')),
      makeDeps(respond({ status: 400, body: { code: 'bad_request' } })),
    )
    expect(result).toEqual({
      tag: 'permanent',
      val: { status: 400, reason: 'bad_request' },
    })
  })
})

describe('sendMessage revision conflict', () => {
  const conflict = respond({
    status: 409,
    body: { code: 'revision_conflict', currentRevision: 9 },
  })

  it('reports a conflict when the server text differs', async () => {
    const result = await sendMessage(
      requestFor(saveContent('mine')),
      makeDeps(conflict),
    )
    expect(result).toEqual({ tag: 'conflict', val: { currentRevision: 9n } })
  })

  it('treats equal server text as a lost acknowledgement', async () => {
    const result = await sendMessage(
      requestFor(saveContent('server text')),
      makeDeps(conflict),
    )
    expect(result).toEqual({ tag: 'ok', val: { revision: 9n } })
  })

  it('reports transient when the server text cannot be fetched', async () => {
    const result = await sendMessage(
      requestFor(saveContent('mine')),
      makeDeps(conflict, {
        fetchServerContent: async () => {
          throw new TypeError('offline')
        },
      }),
    )
    expect(result.tag).toBe('transient')
  })
})

describe('sendMessage name_taken', () => {
  const taken = respond({ status: 409, body: { code: 'name_taken' } })

  it('suggests a new name for a create', async () => {
    const result = await sendMessage(
      requestFor({
        tag: 'create-file',
        val: { name: 'a.jianpu', content: '' },
      }),
      makeDeps(taken),
    )
    expect(result).toEqual({
      tag: 'name-taken',
      val: { suggestedName: 'a 2.jianpu' },
    })
  })

  it('suggests a new name for a rename', async () => {
    const result = await sendMessage(
      requestFor({ tag: 'rename-file', val: { to: 'b.jianpu' } }),
      makeDeps(taken),
    )
    expect(result).toEqual({
      tag: 'name-taken',
      val: { suggestedName: 'b 2.jianpu' },
    })
  })

  it('is permanent for a message kind with no name', async () => {
    const result = await sendMessage(
      requestFor(saveContent('x')),
      makeDeps(taken),
    )
    expect(result).toEqual({
      tag: 'permanent',
      val: { status: 409, reason: 'name_taken' },
    })
  })
})

describe('suggestNameAfterCollision', () => {
  it('appends 2, then increments', () => {
    expect(suggestNameAfterCollision('a.jianpu')).toBe('a 2.jianpu')
    expect(suggestNameAfterCollision('a 2.jianpu')).toBe('a 3.jianpu')
  })
})
