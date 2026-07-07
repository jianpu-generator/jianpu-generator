import { expect, test } from '@playwright/test'
import type { Route } from '@playwright/test'

const OWNER = 'octo-test-user'
const REPO = 'jianpu-generator-storage'
const API_PREFIX = `https://api.github.com/repos/${OWNER}/${REPO}/contents/`

const SOURCE = [
  '# metadata',
  'title = "Rename Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

function encodeBase64(text: string): string {
  return Buffer.from(text, 'utf-8').toString('base64')
}

/**
 * Fakes the slice of GitHub's Contents API the `github` storage backend
 * calls (`githubBackend.ts`): directory listing, file read, create
 * (`PUT`), and delete (`DELETE`). Backed by an in-memory path -> content
 * map seeded with a single `original.jianpu`, so a full load + rename
 * round-trip never touches the real network.
 */
async function mockGithubContentsApi(
  page: import('@playwright/test').Page,
): Promise<void> {
  const files = new Map<string, string>([['scores/original.jianpu', SOURCE]])

  await page.route(`${API_PREFIX}**`, async (route: Route) => {
    const request = route.request()
    const url = new URL(request.url())
    const path = decodeURIComponent(
      url.pathname.slice(url.pathname.indexOf('/contents/') + '/contents/'.length),
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
      const body = request.postDataJSON() as { content: string }
      files.set(path, Buffer.from(body.content, 'base64').toString('utf-8'))
      return route.fulfill({
        status: 201,
        json: { content: { sha: `sha-${path}` }, commit: {} },
      })
    }

    if (request.method() === 'DELETE') {
      files.delete(path)
      return route.fulfill({ status: 200, json: { commit: {} } })
    }

    throw new Error(`Unexpected ${request.method()} ${request.url()}`)
  })
}

test('renaming a file persists via the GitHub storage backend', async ({
  page,
}) => {
  await mockGithubContentsApi(page)

  await page.addInitScript(
    ({ owner }) => {
      localStorage.setItem(
        'jianpu:storage-backend:v1',
        JSON.stringify({ backend: 'github', github: { owner } }),
      )
      localStorage.setItem(
        'jianpu:github-auth:v1',
        JSON.stringify({ token: 'fake-token', scopes: ['repo'] }),
      )
    },
    { owner: OWNER },
  )

  await page.goto('/')

  // The GitHub-backed file list loads asynchronously; wait for the seeded
  // file's tab to appear alongside the read-only demo tab.
  const originalTab = page.locator('.file-tab-name', {
    hasText: 'original.jianpu',
  })
  await originalTab.waitFor({ timeout: 15_000 })

  // Select it (it isn't active by default — the backend always loads onto
  // the demo file), then double-click to enter rename mode.
  await originalTab.click()
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'original.jianpu',
  )
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await originalTab.dblclick()
  const input = page.locator('.file-tab--active input.file-tab-name')
  await input.fill('renamed.jianpu')
  await input.press('Enter')

  // The tab reflects the new name, and its content/preview survive the
  // rename (i.e. the rename resolved through the mocked create+delete
  // Contents API calls rather than getting stuck or reverting).
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'renamed.jianpu',
  )
  await expect(page.locator('.preview-page').first()).toBeVisible({
    timeout: 5_000,
  })

  // Reloading re-fetches from the (mocked) GitHub API, so the renamed tab
  // persisting across a reload proves the backend's create+delete pair
  // actually landed in the fake remote, not just in in-memory React state.
  await page.reload()
  await page.locator('.file-tab-name', { hasText: 'renamed.jianpu' }).waitFor({
    timeout: 15_000,
  })
  await expect(
    page.locator('.file-tab-name', { hasText: 'original.jianpu' }),
  ).toHaveCount(0)
})
