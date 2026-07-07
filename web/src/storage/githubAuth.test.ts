import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  checkGithubAuthStatus,
  GITHUB_AUTH_STORAGE_KEY,
  readStoredGithubAuth,
} from './githubAuth'

const getAuthenticated = vi.fn()

vi.mock('@octokit/rest', () => ({
  Octokit: vi.fn().mockImplementation(function MockOctokit(this: unknown) {
    return {
      rest: {
        users: { getAuthenticated },
      },
    }
  }),
}))

function makeFakeLocalStorage(): Storage {
  const data = new Map<string, string>()
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => void data.set(key, value),
    removeItem: (key: string) => void data.delete(key),
    clear: () => data.clear(),
    key: () => null,
    get length() {
      return data.size
    },
  } as Storage
}

beforeEach(() => {
  vi.clearAllMocks()
  vi.stubGlobal('localStorage', makeFakeLocalStorage())
})

describe('readStoredGithubAuth', () => {
  it('returns null when nothing is stored', () => {
    expect(readStoredGithubAuth()).toBeNull()
  })

  it('returns null when the stored value is malformed JSON', () => {
    localStorage.setItem(GITHUB_AUTH_STORAGE_KEY, '{not json')
    expect(readStoredGithubAuth()).toBeNull()
  })

  it('returns null when the stored value has no string token', () => {
    localStorage.setItem(
      GITHUB_AUTH_STORAGE_KEY,
      JSON.stringify({ scopes: ['repo'] }),
    )
    expect(readStoredGithubAuth()).toBeNull()
  })

  it('defaults scopes to an empty array when omitted', () => {
    localStorage.setItem(
      GITHUB_AUTH_STORAGE_KEY,
      JSON.stringify({ token: 'tok' }),
    )
    expect(readStoredGithubAuth()).toEqual({ token: 'tok', scopes: [] })
  })
})

describe('checkGithubAuthStatus', () => {
  it('reports disconnected when no token is stored', async () => {
    await expect(checkGithubAuthStatus()).resolves.toEqual({
      state: 'disconnected',
    })
    expect(getAuthenticated).not.toHaveBeenCalled()
  })

  it('reports connected with the username when the token is valid', async () => {
    localStorage.setItem(
      GITHUB_AUTH_STORAGE_KEY,
      JSON.stringify({ token: 'tok', scopes: [] }),
    )
    getAuthenticated.mockResolvedValue({ data: { login: 'octocat' } })

    await expect(checkGithubAuthStatus()).resolves.toEqual({
      state: 'connected',
      username: 'octocat',
    })
  })

  it('clears the stored token and reports needs-reconnect on a 401', async () => {
    localStorage.setItem(
      GITHUB_AUTH_STORAGE_KEY,
      JSON.stringify({ token: 'stale', scopes: [] }),
    )
    getAuthenticated.mockRejectedValue(
      Object.assign(new Error('Bad credentials'), { status: 401 }),
    )

    await expect(checkGithubAuthStatus()).resolves.toEqual({
      state: 'needs-reconnect',
    })
    expect(readStoredGithubAuth()).toBeNull()
  })

  it('rethrows errors that are not a 401', async () => {
    localStorage.setItem(
      GITHUB_AUTH_STORAGE_KEY,
      JSON.stringify({ token: 'tok', scopes: [] }),
    )
    getAuthenticated.mockRejectedValue(
      Object.assign(new Error('boom'), { status: 500 }),
    )

    await expect(checkGithubAuthStatus()).rejects.toThrow('boom')
    expect(readStoredGithubAuth()).not.toBeNull()
  })
})
