import { expect, test } from '@playwright/test'
import { openFileActions } from './fileSwitcherHelpers'
import {
  API_PREFIX,
  mockGithubContentsApi,
  mockGithubUser,
  OWNER,
} from './github-contents-mock'

test('switching to GitHub repository storage shows a loading spinner while files load', async ({
  page,
}) => {
  await page.addInitScript(() => {
    localStorage.setItem(
      'jianpu:storage-backend:v1',
      JSON.stringify({ backend: 'local' }),
    )
    localStorage.setItem(
      'jianpu:github-auth:v1',
      JSON.stringify({ token: 'fake-token', scopes: ['repo'] }),
    )
  })

  await mockGithubUser(page, OWNER)
  await mockGithubContentsApi(page, {
    'scores/loading.jianpu': [
      '# metadata',
      'title = "Loading Spinner Test"',
      '',
      '# parts',
      'Melody [M] = notes',
      '',
      '# score',
      '(bpm=120 key=C4 time=4/4)',
      '1 2 3 4',
    ].join('\n'),
  })

  // Delays the directory-listing GET requests `backend.load()` issues so the
  // spinner has a window to be observed before the listing resolves. Routed
  // after the base mock above so Playwright tries this handler first.
  await page.route(`${API_PREFIX}**`, async (route) => {
    const request = route.request()
    if (request.method() === 'GET') {
      await new Promise((resolve) => setTimeout(resolve, 1000))
    }
    return route.fallback()
  })

  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })

  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Storage…' }).click()
  await page.getByTestId('storage-settings-modal').waitFor()

  await page
    .locator('label', { hasText: 'GitHub repository' })
    .locator('input[type="radio"]')
    .check()

  const spinner = page.getByTestId('github-loading-spinner')
  await expect(spinner).toBeVisible({ timeout: 5_000 })

  await expect(spinner).toHaveCount(0, { timeout: 15_000 })
  await expect(page.getByTestId('github-connected')).toBeVisible()
})
