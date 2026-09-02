import { expect } from '@playwright/test'
import { openFileActions } from '../../fileSwitcherHelpers'
import {
  API_PREFIX,
  mockGithubContentsApi,
  mockGithubUser,
  OWNER,
} from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

Given(
  'the local storage backend is active with GitHub auth already seeded',
  async ({ page }) => {
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
  },
)

Given(
  'the mocked GitHub user exists for the mocked owner',
  async ({ page }) => {
    await mockGithubUser(page, OWNER)
  },
)

Given(
  'the GitHub Contents API is mocked with a seeded file for the storage-switch spinner',
  async ({ page }) => {
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
  },
)

Given(
  'the GitHub directory listing GET is delayed by 1 second when switching backend',
  async ({ page }) => {
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
  },
)

When(
  'the app loads on the local backend with the editor ready',
  async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
  },
)

When(
  'I open the storage settings modal to switch backend',
  async ({ page }) => {
    await openFileActions(page)
    await page.getByRole('menuitem', { name: 'Storage…' }).click()
    await page.getByTestId('storage-settings-modal').waitFor()
  },
)

When(
  'I select the {string} storage radio option',
  async ({ page }, label: string) => {
    await page.getByRole('button', { name: label }).click()
  },
)

Then('the github loading spinner is visible', async ({ page }) => {
  const spinner = page.getByTestId('github-loading-spinner')
  await expect(spinner).toBeVisible({ timeout: 5_000 })
})

Then(
  'the github loading spinner disappears once loading finishes',
  async ({ page }) => {
    const spinner = page.getByTestId('github-loading-spinner')
    await expect(spinner).toHaveCount(0, { timeout: 15_000 })
  },
)

Then(
  'the storage settings modal shows connected as the mocked owner',
  async ({ page }) => {
    await expect(page.getByTestId('github-connected')).toBeVisible()
  },
)
