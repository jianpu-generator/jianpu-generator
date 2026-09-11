import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  completeSyncedShareGithubSignInFromCallback,
  SYNCED_SHARE_GITHUB_PKCE_STORAGE_KEY,
  syncedShareGithubCallbackPathname,
} from './syncedShareGithubAuth.ts'

// Regression test for a bug where the redirect URI GitHub is sent (and the
// pathname `main.tsx` checks to short-circuit into the callback page) was
// built from `window.location.origin` alone, silently dropping the app's
// configured base path -- correct on Cloudflare Pages (served from the
// domain root, `BASE_URL` `/`) but producing a URL GitHub Pages has no
// content at all on (served under a `/jianpu-generator/` subpath, and this
// repo has no SPA-fallback `404.html` to recover from that).
describe('syncedShareGithubCallbackPathname', () => {
  afterEach(() => {
    vi.unstubAllEnvs()
  })

  it('is just the redirect path when the app is served from the domain root', () => {
    vi.stubEnv('BASE_URL', '/')
    expect(syncedShareGithubCallbackPathname()).toBe(
      '/synced-share/github-callback',
    )
  })

  it('is prefixed with the base path when served under a repo-name subpath', () => {
    vi.stubEnv('BASE_URL', '/jianpu-generator/')
    expect(syncedShareGithubCallbackPathname()).toBe(
      '/jianpu-generator/synced-share/github-callback',
    )
  })

  it('tolerates a base path missing its trailing slash', () => {
    vi.stubEnv('BASE_URL', '/jianpu-generator')
    expect(syncedShareGithubCallbackPathname()).toBe(
      '/jianpu-generator/synced-share/github-callback',
    )
  })
})

// Regression test for a bug where a `200 OK` response with a body that
// couldn't be parsed as JSON (e.g. a truncated response while the local
// worker restarts mid-request) threw out of `response.json()` uncaught,
// rejecting `completeSyncedShareGithubSignInFromCallback`'s promise instead
// of resolving a `{ ok: false }` result. `SyncedShareGithubCallbackPage` has
// no `.catch` on that promise, so the rejection was unhandled and the popup
// stayed on "Signing in with GitHub…" forever -- it never reached the
// `.then` that flips `status` and calls `window.close()`.
function makeFakeStorage(): Storage {
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

describe('completeSyncedShareGithubSignInFromCallback', () => {
  const pending = {
    codeVerifier: 'test-code-verifier',
    state: 'test-state',
    redirectUri: 'http://localhost:5173/synced-share/github-callback',
  }

  beforeEach(() => {
    // This module reads `window.opener`/`window.location` (there is no real
    // `window` global under vitest's default `node` environment, per this
    // repo's convention of stubbing just what a test needs rather than
    // switching to a jsdom environment -- see `githubAuth.test.ts`).
    vi.stubGlobal('window', {
      opener: null,
      location: { origin: 'http://localhost:5173' },
    })
    vi.stubGlobal('sessionStorage', makeFakeStorage())
    sessionStorage.setItem(
      SYNCED_SHARE_GITHUB_PKCE_STORAGE_KEY,
      JSON.stringify(pending),
    )
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('resolves ok:false instead of throwing when the worker returns an unparseable 200 body', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response('not json', {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    )

    const result = await completeSyncedShareGithubSignInFromCallback({
      host: 'localhost:8787',
      search: `?code=abc123&state=${pending.state}`,
    })

    expect(result.ok).toBe(false)
    if (!result.ok) {
      expect(result.error).toMatch(/unreadable response/)
    }
  })
})
