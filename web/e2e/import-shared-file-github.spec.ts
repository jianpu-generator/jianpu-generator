import { expect, test } from '@playwright/test'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'
import { gotoShareUrl } from './shareUrlHelper'

const SHARED_FILENAME = 'shared-test.jianpu'
const SHARED_SOURCE = [
  '# metadata',
  'title = "Shared Score"',
  '',
  '# parts',
  'Melody = notes',
  '',
  '# score',
  '(time=4/4 key=C4 bpm=120)',
  '1 2 3 4',
].join('\n')

test('importing a shared score persists via the GitHub storage backend', async ({
  page,
}) => {
  const putBodies: { path: string; sha?: string }[] = []
  await mockGithubContentsApi(
    page,
    {},
    {
      onPut: (path, body) => putBodies.push({ path, sha: body.sha }),
      // Slow enough for the import to still be in flight when we assert on it.
      mutationDelayMs: 300,
    },
  )

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

  await gotoShareUrl(page, SHARED_FILENAME, SHARED_SOURCE)

  await expect(page.locator('.shared-preview-banner')).toContainText(
    SHARED_FILENAME,
  )
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await page.getByRole('button', { name: 'Import to my scores' }).click()

  // The banner is dismissed and the imported file becomes the active tab
  // once `backend.importFile`'s create-only `PUT` resolves.
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    SHARED_FILENAME,
  )
  await expect(page.locator('.shared-preview-banner')).toHaveCount(0)

  // Create-only: the PUT that lands the import must not carry a `sha` — a
  // `sha` would mean the backend fetched the file first, which importing
  // (like new/duplicate) should never do.
  expect(putBodies).toContainEqual({
    path: `scores/${SHARED_FILENAME}`,
    sha: undefined,
  })

  // Reloading re-fetches from the (mocked) GitHub API, so the imported file
  // persisting across a reload proves the backend's create-only `PUT`
  // actually landed in the fake remote, not just in in-memory React state.
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await page
    .locator('.file-tab-name', { hasText: SHARED_FILENAME })
    .waitFor({ timeout: 15_000 })
})
