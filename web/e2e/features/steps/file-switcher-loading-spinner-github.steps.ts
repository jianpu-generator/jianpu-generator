import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from '../../fileSwitcherHelpers'
import { API_PREFIX, mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

Given(
  'the GitHub repo is seeded with a file named {string} for the file-switcher spinner',
  async ({ page }, path: string) => {
    await mockGithubContentsApi(page, {
      [path]: [
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
  'the GitHub directory listing GET is delayed by 1 second for the file switcher',
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
  'the app loads with the editor ready while GitHub files load',
  async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
  },
)

Then('the file switcher trigger shows a loading spinner', async ({ page }) => {
  const trigger = fileSwitcherTrigger(page)
  await expect(trigger.locator('.file-tab-bar-spinner')).toBeVisible({
    timeout: 5_000,
  })
})

Then(
  'opening the file list shows the hint {string}',
  async ({ page }, hint: string) => {
    await openFileList(page)
    await expect(page.locator('.file-tab-bar-hint')).toHaveText(hint)
  },
)

When('the GitHub directory listing resolves', async ({ page }) => {
  const trigger = fileSwitcherTrigger(page)
  await expect(trigger.locator('.file-tab-bar-spinner')).toHaveCount(0, {
    timeout: 15_000,
  })
})

Then(
  'the file switcher trigger spinner is gone and the caret is visible',
  async ({ page }) => {
    const trigger = fileSwitcherTrigger(page)
    await expect(trigger.locator('.export-menu-caret')).toBeVisible()
  },
)

Then('the file list shows {string}', async ({ page }, name: string) => {
  await openFileList(page)
  await expect(
    page.locator('.file-tabs .file-tab-name', { hasText: name }),
  ).toBeVisible()
})
