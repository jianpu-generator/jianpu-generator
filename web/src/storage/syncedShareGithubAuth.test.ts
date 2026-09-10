import { afterEach, describe, expect, it, vi } from 'vitest'
import { syncedShareGithubCallbackPathname } from './syncedShareGithubAuth.ts'

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
