import type { Route } from '@playwright/test'

export const OWNER = 'octo-test-user'
export const REPO = 'jianpu-generator-storage'
export const API_PREFIX = `https://api.github.com/repos/${OWNER}/${REPO}/contents/`

export function encodeBase64(text: string): string {
  return Buffer.from(text, 'utf-8').toString('base64')
}

function delay(ms: number): Promise<void> {
  return ms > 0
    ? new Promise((resolve) => setTimeout(resolve, ms))
    : Promise.resolve()
}

export interface MockGithubContentsApiOptions {
  onPut?: (path: string, body: { content: string; sha?: string }) => void
  /** Delays PUT/DELETE responses by this many milliseconds, giving a test a
   * window to assert the app's pending-state spinner is actually shown
   * rather than the op resolving before the UI ever paints it. */
  mutationDelayMs?: number
}

export interface GithubContentsApiController {
  /** Makes the next `PUT` to `path` fail with a `409` (GitHub's response
   * when the `sha` sent no longer matches HEAD), simulating a concurrent
   * edit on GitHub racing the app's save. Consumed after one use so a
   * retried `PUT` to the same path succeeds normally, matching real
   * GitHub's behavior once the client re-fetches a fresh `sha`. */
  failNextPutWith409(path: string): void
  /** Directly overwrites the in-memory file the mock serves, simulating a
   * change landing on GitHub from elsewhere (e.g. another device) without
   * going through this page's own `PUT` calls. */
  setRemoteContent(path: string, content: string): void
}

/**
 * Fakes the slice of GitHub's Contents API the `github` storage backend
 * calls (`githubBackend.ts`): directory listing, file read, create (`PUT`),
 * and delete (`DELETE`). Backed by an in-memory path -> content map, so a
 * full load + file-op round-trip never touches the real network.
 */
export async function mockGithubContentsApi(
  page: import('@playwright/test').Page,
  seed: Record<string, string>,
  options: MockGithubContentsApiOptions = {},
): Promise<GithubContentsApiController> {
  const { onPut, mutationDelayMs = 0 } = options
  const files = new Map<string, string>(Object.entries(seed))
  const conflictPaths = new Set<string>()

  await page.route(`${API_PREFIX}**`, async (route: Route) => {
    const request = route.request()
    const url = new URL(request.url())
    const path = decodeURIComponent(
      url.pathname.slice(
        url.pathname.indexOf('/contents/') + '/contents/'.length,
      ),
    )

    if (request.method() === 'GET') {
      const dirPrefix = `${path}/`
      const entries = [...files.keys()]
        .filter((key) => key.startsWith(dirPrefix))
        .map((key) => ({
          name: key.slice(dirPrefix.length),
          path: key,
          type: 'file',
          sha: `sha-${key}`,
        }))
      if (entries.length > 0) {
        return route.fulfill({ status: 200, json: entries })
      }
      const content = files.get(path)
      if (content === undefined) {
        return route.fulfill({
          status: 404,
          json: { message: 'Not Found' },
        })
      }
      return route.fulfill({
        status: 200,
        json: {
          type: 'file',
          path,
          sha: `sha-${path}`,
          content: encodeBase64(content),
          encoding: 'base64',
        },
      })
    }

    if (request.method() === 'PUT') {
      const body = request.postDataJSON() as { content: string; sha?: string }
      onPut?.(path, body)
      if (conflictPaths.has(path)) {
        conflictPaths.delete(path)
        await delay(mutationDelayMs)
        return route.fulfill({
          status: 409,
          json: { message: 'sha does not match' },
        })
      }
      files.set(path, Buffer.from(body.content, 'base64').toString('utf-8'))
      await delay(mutationDelayMs)
      return route.fulfill({
        status: 201,
        json: { content: { sha: `sha-${path}` }, commit: {} },
      })
    }

    if (request.method() === 'DELETE') {
      files.delete(path)
      await delay(mutationDelayMs)
      return route.fulfill({ status: 200, json: { commit: {} } })
    }

    throw new Error(`Unexpected ${request.method()} ${request.url()}`)
  })

  return {
    failNextPutWith409: (path) => conflictPaths.add(path),
    setRemoteContent: (path, content) => files.set(path, content),
  }
}

/**
 * Fakes `GET https://api.github.com/user`, which
 * `checkGithubAuthStatus`/`handleConnect` both call to resolve the
 * authenticated username.
 */
export async function mockGithubUser(
  page: import('@playwright/test').Page,
  login: string,
): Promise<void> {
  await page.route('https://api.github.com/user', async (route: Route) => {
    await route.fulfill({ status: 200, json: { login } })
  })
}

/**
 * Fakes `GET https://api.github.com/repos/{owner}/{repo}`, which
 * `ensureStorageRepo` calls right after connecting to check whether the
 * app's storage repo already exists (avoiding a `createForAuthenticatedUser`
 * call in the common case).
 */
export async function mockGithubRepoExists(
  page: import('@playwright/test').Page,
  owner: string,
  repo: string,
): Promise<void> {
  await page.route(
    `https://api.github.com/repos/${owner}/${repo}`,
    async (route: Route) => {
      await route.fulfill({
        status: 200,
        json: { name: repo, full_name: `${owner}/${repo}`, private: true },
      })
    },
  )
}
