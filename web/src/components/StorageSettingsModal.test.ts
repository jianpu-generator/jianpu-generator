import type { Octokit } from '@octokit/rest'
import { describe, expect, it, vi } from 'vitest'
import type { FileStoreState } from '../fileStore'
import type { GithubBackend } from '../storage/githubBackend'
import {
  ensureStorageRepo,
  resolveGithubConflict,
} from './StorageSettingsModal'

function notFound(): Promise<never> {
  return Promise.reject(Object.assign(new Error('Not Found'), { status: 404 }))
}

function makeOctokit(overrides: {
  get?: () => Promise<unknown>
  create?: () => Promise<unknown>
}): Octokit {
  return {
    rest: {
      repos: {
        get: vi.fn(overrides.get ?? (() => Promise.resolve({ data: {} }))),
        createForAuthenticatedUser: vi.fn(
          overrides.create ?? (() => Promise.resolve({ data: {} })),
        ),
      },
    },
  } as unknown as Octokit
}

describe('ensureStorageRepo', () => {
  it('does nothing when the repo already exists', async () => {
    const octokit = makeOctokit({})
    await ensureStorageRepo(octokit, 'octo')

    expect(octokit.rest.repos.get).toHaveBeenCalledWith({
      owner: 'octo',
      repo: 'jianpu-generator-storage',
    })
    expect(octokit.rest.repos.createForAuthenticatedUser).not.toHaveBeenCalled()
  })

  it('creates the repo as private when a 404 is returned', async () => {
    const octokit = makeOctokit({ get: notFound })
    await ensureStorageRepo(octokit, 'octo')

    expect(octokit.rest.repos.createForAuthenticatedUser).toHaveBeenCalledWith({
      name: 'jianpu-generator-storage',
      private: true,
    })
  })

  it('rethrows non-404 errors without attempting to create', async () => {
    const octokit = makeOctokit({
      get: () =>
        Promise.reject(Object.assign(new Error('boom'), { status: 500 })),
    })

    await expect(ensureStorageRepo(octokit, 'octo')).rejects.toThrow('boom')
    expect(octokit.rest.repos.createForAuthenticatedUser).not.toHaveBeenCalled()
  })
})

function makeStore(active: string, content: string): FileStoreState {
  return {
    active,
    userFiles: { [active]: content },
    bin: {},
    fileIds: { [active]: 'id-1' },
  }
}

describe('resolveGithubConflict', () => {
  it('overwrite-mine re-pushes the current in-memory content and keeps the store unchanged', async () => {
    const saveContent = vi.fn().mockResolvedValue(undefined)
    const backend = { saveContent } as unknown as GithubBackend
    const store = makeStore('a.jianpu', 'mine')

    const result = await resolveGithubConflict('overwrite-mine', backend, store)

    expect(saveContent).toHaveBeenCalledWith(store)
    expect(result).toBe(store)
  })

  it('discard-mine reloads and replaces the active file content with the remote version', async () => {
    const load = vi.fn().mockResolvedValue({
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'theirs' },
      bin: {},
      fileIds: {},
    })
    const updateActiveContent = vi.fn(
      (state: FileStoreState, content: string) => ({
        ...state,
        userFiles: { ...state.userFiles, [state.active]: content },
      }),
    )
    const backend = { load, updateActiveContent } as unknown as GithubBackend
    const store = makeStore('a.jianpu', 'mine')

    const result = await resolveGithubConflict('discard-mine', backend, store)

    expect(load).toHaveBeenCalled()
    expect(updateActiveContent).toHaveBeenCalledWith(store, 'theirs')
    expect(result.userFiles['a.jianpu']).toBe('theirs')
  })
})
