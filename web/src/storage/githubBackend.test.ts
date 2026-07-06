import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { FileStoreState } from '../fileStore'
import { createGithubBackend } from './githubBackend'

const getContent = vi.fn()
const createOrUpdateFileContents = vi.fn()
const deleteFile = vi.fn()

vi.mock('@octokit/rest', () => ({
  Octokit: vi.fn().mockImplementation(function MockOctokit(this: unknown) {
    return {
      rest: {
        repos: { getContent, createOrUpdateFileContents, deleteFile },
      },
    }
  }),
}))

function encodeBase64(text: string): string {
  const bytes = new TextEncoder().encode(text)
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return btoa(binary)
}

function notFound(): Promise<never> {
  return Promise.reject(Object.assign(new Error('Not Found'), { status: 404 }))
}

function fileResponse(content: string, sha = 'sha-1') {
  return Promise.resolve({
    data: { type: 'file', content: encodeBase64(content), encoding: 'base64', sha },
  })
}

function dirResponse(entries: { name: string; path: string }[]) {
  return Promise.resolve({
    data: entries.map((entry) => ({ ...entry, type: 'file' })),
  })
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

const config = { token: 'test-token', owner: 'octo', repo: 'jianpu-storage' }

beforeEach(() => {
  vi.clearAllMocks()
})

describe('createGithubBackend: path construction', () => {
  it('scopes directory listing to the fixed scores/ folder and the trash/ folder', async () => {
    getContent.mockImplementation(() => dirResponse([]))
    const backend = createGithubBackend(config)

    await backend.load()

    const paths = getContent.mock.calls.map((call) => call[0].path)
    expect(paths).toContain('scores')
    expect(paths).toContain('trash')
  })

  it('loads files from the main folder and treats a missing trash/ as empty', async () => {
    getContent.mockImplementation(({ path }: { path: string }) => {
      if (path === 'scores') {
        return dirResponse([
          { name: 'a.jianpu', path: 'scores/a.jianpu' },
          { name: 'notes.txt', path: 'scores/notes.txt' },
        ])
      }
      if (path === 'scores/a.jianpu') return fileResponse('1 2 3')
      if (path === 'trash') return notFound()
      throw new Error(`unexpected path ${path}`)
    })

    const backend = createGithubBackend(config)
    const state = await backend.load()

    expect(state.userFiles).toEqual({ 'a.jianpu': '1 2 3' })
    expect(state.bin).toEqual({})
  })
})

describe('createGithubBackend: sha-refetch-before-write', () => {
  it('refetches sha via getContent immediately before saveContent writes', async () => {
    getContent.mockImplementation(() => fileResponse('old', 'sha-abc'))
    createOrUpdateFileContents.mockResolvedValue({})
    const backend = createGithubBackend(config)

    const state: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'new content' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-1' },
    }
    await backend.saveContent(state)

    expect(getContent).toHaveBeenCalledWith(
      expect.objectContaining({ path: 'scores/a.jianpu' }),
    )
    expect(createOrUpdateFileContents).toHaveBeenCalledWith(
      expect.objectContaining({ path: 'scores/a.jianpu', sha: 'sha-abc' }),
    )
  })

  it('creates without a sha lookup when the destination is guaranteed new', async () => {
    createOrUpdateFileContents.mockResolvedValue({})
    const backend = createGithubBackend(config)

    const state: FileStoreState = {
      active: 'demo',
      userFiles: {},
      bin: {},
      fileIds: {},
    }
    await backend.createFile(state)

    expect(getContent).not.toHaveBeenCalled()
    expect(createOrUpdateFileContents).toHaveBeenCalledTimes(1)
    expect(createOrUpdateFileContents.mock.calls[0][0]).not.toHaveProperty('sha')
  })
})

describe('createGithubBackend: rename as create-then-delete', () => {
  it('creates at the new path before deleting the old path', async () => {
    const order: string[] = []
    createOrUpdateFileContents.mockImplementation(async ({ path }) => {
      order.push(`create:${path}`)
      return {}
    })
    getContent.mockImplementation(({ path }: { path: string }) => {
      order.push(`getContent:${path}`)
      return fileResponse('content', 'sha-old')
    })
    deleteFile.mockImplementation(async ({ path }) => {
      order.push(`delete:${path}`)
      return {}
    })

    const backend = createGithubBackend(config)
    const state: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'content' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-1' },
    }

    await backend.renameFile(state, 'a.jianpu', 'b.jianpu')

    expect(order).toEqual([
      'create:scores/b.jianpu',
      'getContent:scores/a.jianpu',
      'delete:scores/a.jianpu',
    ])
  })

  it('is a no-op against the API when the pure rename rejects the change', async () => {
    const backend = createGithubBackend(config)
    const state: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'content' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-1' },
    }

    // Renaming to the (read-only) demo file's name is rejected by the pure
    // fileStore transform, so no API calls should happen at all.
    await backend.renameFile(state, 'a.jianpu', 'reference.jianpu')

    expect(createOrUpdateFileContents).not.toHaveBeenCalled()
    expect(deleteFile).not.toHaveBeenCalled()
  })
})

describe('createGithubBackend: single-flight save serialization', () => {
  it('never runs two saveContent network sequences concurrently', async () => {
    let active = 0
    let maxConcurrent = 0

    async function track<T>(result: T): Promise<T> {
      active++
      maxConcurrent = Math.max(maxConcurrent, active)
      await delay(10)
      active--
      return result
    }

    getContent.mockImplementation(() => track({ data: { type: 'file', sha: 'sha-1', content: '', encoding: 'base64' } }))
    createOrUpdateFileContents.mockImplementation(() => track({}))

    const backend = createGithubBackend(config)
    const stateA: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'A' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-a' },
    }
    const stateB: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'B' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-a' },
    }

    await Promise.all([backend.saveContent(stateA), backend.saveContent(stateB)])

    expect(maxConcurrent).toBe(1)
    expect(createOrUpdateFileContents).toHaveBeenCalledTimes(2)
  })
})

describe('createGithubBackend: 409 conflict surfacing', () => {
  it('reports a conflict status and detail when the write is rejected with 409', async () => {
    getContent.mockImplementation(() => fileResponse('old', 'sha-stale'))
    createOrUpdateFileContents.mockRejectedValue(
      Object.assign(new Error('Conflict'), { status: 409 }),
    )

    const backend = createGithubBackend(config)
    const state: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'new content' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-1' },
    }

    await expect(backend.saveContent(state)).rejects.toThrow('Conflict')

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toEqual({ kind: 'conflict', path: 'scores/a.jianpu' })
  })
})
