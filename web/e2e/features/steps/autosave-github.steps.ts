import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

// Mirrors `useStorageBackend.ts`'s `AUTOSAVE_DEBOUNCE_MS`. Not imported
// directly — that module transitively pulls in `fileStore.ts`'s Vite-only
// `?raw` import, which Playwright's test loader can't resolve.
const AUTOSAVE_DEBOUNCE_MS = 20_000

const SOURCE = [
  '# metadata',
  'title = "Autosave Test"',
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
  'the GitHub repo is seeded with a file named {string} for autosave',
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
  'a fake clock is installed before navigating to test autosave',
  async ({ page }) => {
    // Install the fake clock before navigating. It keeps ticking in step with
    // real time until `fastForward` below, which lets us jump straight past
    // the debounce interval instead of waiting it out for real.
    await page.clock.install()
  },
)

When(
  'the app loads the GitHub-backed file list for autosave',
  async ({ page }) => {
    await page.goto('/')
    await openFileList(page)
  },
)

When(
  'I select the {string} tab to test autosave',
  async ({ page }, name: string) => {
    const tab = page.locator('.file-tab-name', { hasText: name })
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(name)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

When(
  'I append {string} to the editor to trigger an autosave',
  async ({ page }, text: string) => {
    await typeAtEditorEnd(page, text)
  },
)

Then('no autosave PUT has been sent yet for the debounced edit', async () => {
  // Right after the edit, the debounce hasn't fired yet: no PUT sent, and
  // the save-status badge shows the pending "Unsaved" countdown rather than
  // the stale "Saved" from before the edit (see `FileSwitcher.tsx`'s
  // `SaveStatusBadge`).
  expect(putBodies).toHaveLength(0)
})

Then(
  'the autosave status badge shows {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toContainText(text)
  },
)

When(
  'I fast-forward the clock past the autosave debounce interval to trigger it',
  async ({ page }) => {
    await page.clock.fastForward(AUTOSAVE_DEBOUNCE_MS)
  },
)

Then(
  'the autosave PUT lands for {string} containing {string}',
  async ({}, path: string, content: string) => {
    await expect
      .poll(() => putBodies.find((body) => body.path === path))
      .toMatchObject({ content: expect.stringContaining(content) })
  },
)

When('I reload the page after the autosave', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, so the edit surviving
  // a reload proves the autosave actually landed in the fake remote, not
  // just in-memory React state.
  await page.reload()
  await openFileList(page)
})

Then(
  'the autosaved file list still shows {string} after reload',
  async ({ page }, name: string) => {
    await page.locator('.file-tab-name', { hasText: name }).waitFor({
      timeout: 15_000,
    })
    await page.locator('.file-tab-name', { hasText: name }).click()
  },
)

Then(
  'the reloaded editor still contains the autosaved edit {string}',
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)
