import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileActions,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import {
  API_PREFIX,
  mockGithubContentsApi,
  OWNER,
} from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

// Mirrors `useStorageBackend.ts`'s `AUTOSAVE_DEBOUNCE_MS`. Not imported
// directly — that module transitively pulls in `fileStore.ts`'s Vite-only
// `?raw` import, which Playwright's test loader can't resolve.
const AUTOSAVE_DEBOUNCE_MS = 20_000

const SOURCE = [
  '# metadata',
  'title = "Error Banner Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

let putBodies: { path: string; content: string }[] = []

Given(
  'the GitHub repo is seeded with a file named {string} for a rate-limit banner',
  async ({ page }, path: string) => {
    putBodies = []
    await mockGithubContentsApi(
      page,
      { [path]: SOURCE },
      {
        onPut: (path, body) =>
          putBodies.push({
            path,
            content: Buffer.from(body.content, 'base64').toString('utf-8'),
          }),
      },
    )
  },
)

Given(
  'the GitHub repo is seeded with a file named {string} for an offline banner',
  async ({ page }, path: string) => {
    putBodies = []
    await mockGithubContentsApi(
      page,
      { [path]: SOURCE },
      {
        onPut: (path, body) =>
          putBodies.push({
            path,
            content: Buffer.from(body.content, 'base64').toString('utf-8'),
          }),
      },
    )
  },
)

Given(
  'the first autosave PUT will fail with a 403 rate-limit response',
  async ({ page }) => {
    // Registered after the base mock above, so Playwright routes it first: it
    // fails the first PUT with a 403 (GitHub's rate-limit response), then
    // `route.fallback()`s to the base mock for every request after — same
    // one-shot pattern as `file-op-error-github.spec.ts`.
    let putCount = 0
    await page.route(`${API_PREFIX}**`, async (route) => {
      const request = route.request()
      if (request.method() === 'PUT') {
        putCount += 1
        if (putCount === 1) {
          return route.fulfill({
            status: 403,
            json: { message: 'API rate limit exceeded' },
          })
        }
      }
      return route.fallback()
    })
  },
)

Given(
  'the first autosave PUT will be aborted as a network failure',
  async ({ page }) => {
    // Same one-shot pattern, but aborting the request (rather than fulfilling
    // with a status) produces the network failure that `isNetworkError()`
    // classifies as offline, matching a real offline `fetch`.
    let putCount = 0
    await page.route(`${API_PREFIX}**`, async (route) => {
      const request = route.request()
      if (request.method() === 'PUT') {
        putCount += 1
        if (putCount === 1) {
          return route.abort('failed')
        }
      }
      return route.fallback()
    })
  },
)

Given(
  'I open and edit {string} with suffix {string} and a fake clock installed',
  async ({ page }, filename: string, suffix: string) => {
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

    // Install the fake clock before navigating, same as `autosave-github.spec.ts`
    // — lets us jump straight past the debounce interval instead of waiting it
    // out for real.
    await page.clock.install()

    await page.goto('/')

    await openFileList(page)
    const tab = page.locator('.file-tab-name', { hasText: filename })
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(filename)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })

    await typeAtEditorEnd(page, suffix)

    // Waits for the edit to actually land in the DOM before the caller jumps
    // the fake clock — otherwise the clock can advance past the debounce
    // deadline before React's effect has re-run and armed the debounced
    // timer for this edit, so the fast-forward wouldn't trigger a save yet.
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(
      `1 2 3 4${suffix}`,
      { timeout: 10_000 },
    )
  },
)

When(
  'I fast-forward the clock past the autosave debounce interval for the error banner',
  async ({ page }) => {
    await page.clock.fastForward(AUTOSAVE_DEBOUNCE_MS)
  },
)

When(
  'I open the storage settings modal to check the error banner',
  async ({ page }) => {
    await openFileActions(page)
    await page.getByRole('menuitem', { name: 'Storage…' }).click()
    await page.getByTestId('storage-settings-modal').waitFor()
  },
)

Then(
  'the status banner is visible and mentions {string}',
  async ({ page }, text: string) => {
    const banner = page.getByTestId('status-banner')
    await expect(banner).toBeVisible({ timeout: 10_000 })
    await expect(banner).toContainText(text)
  },
)

When('I close the storage settings modal', async ({ page }) => {
  // Close the modal so its overlay stops intercepting clicks on the editor.
  await page.keyboard.press('Escape')
  await expect(page.getByTestId('storage-settings-modal')).toHaveCount(0)
})

When(
  'I append {string} to the editor to trigger a recovering autosave',
  async ({ page }, text: string) => {
    await typeAtEditorEnd(page, text)
  },
)

Then('the editor contains {string}', async ({ page }, text: string) => {
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(text, {
    timeout: 10_000,
  })
})

Then(
  'the recovering-autosave PUT lands for {string} containing {string}',
  async ({}, path: string, content: string) => {
    await expect
      .poll(() => putBodies.find((body) => body.path === path), {
        timeout: 10_000,
      })
      .toMatchObject({ content: expect.stringContaining(content) })
  },
)

Then('the status banner is gone', async ({ page }) => {
  // The one-shot failed route has already fired and now falls back to the
  // base mock, so the next autosave succeeds and should clear the banner
  // (`lastError` reset to `null` in `saveContentImpl`'s success path).
  await expect(page.getByTestId('status-banner')).toHaveCount(0)
})
