import { expect, test } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from './fileSwitcherHelpers'
import {
  API_PREFIX,
  mockGithubContentsApi,
  OWNER,
} from './github-contents-mock'

test('header file switcher shows a loading spinner while GitHub files load', async ({
  page,
}) => {
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

  const trigger = fileSwitcherTrigger(page)
  await expect(trigger.locator('.file-tab-bar-spinner')).toBeVisible({
    timeout: 5_000,
  })

  await openFileList(page)
  await expect(page.locator('.file-tab-bar-hint')).toHaveText(
    'Loading files from GitHub…',
  )

  await expect(trigger.locator('.file-tab-bar-spinner')).toHaveCount(0, {
    timeout: 15_000,
  })
  await expect(trigger.locator('.export-menu-caret')).toBeVisible()

  await openFileList(page)
  await expect(
    page.locator('.file-tabs .file-tab-name', { hasText: 'loading.jianpu' }),
  ).toBeVisible()
})
