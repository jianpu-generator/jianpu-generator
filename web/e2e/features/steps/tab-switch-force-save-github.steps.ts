import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

const SOURCE_A = [
  '# metadata',
  'title = "Tab Switch Test A"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

const SOURCE_B = [
  '# metadata',
  'title = "Tab Switch Test B"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '5 6 7 1',
].join('\n')

let putBodies: { path: string; content: string }[] = []

Given(
  'the GitHub repo is seeded with files {string} and {string} for a tab-switch save',
  async ({ page }, pathA: string, pathB: string) => {
    putBodies = []
    await mockGithubContentsApi(
      page,
      { [pathA]: SOURCE_A, [pathB]: SOURCE_B },
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
  'a fake clock is installed to prevent an autosave race with the tab switch',
  async ({ page }) => {
    // Install the fake clock so the debounce timer never elapses on its own —
    // the test relies on the tab switch itself forcing the flush, not the
    // debounce interval happening to run out.
    await page.clock.install()
  },
)

When(
  'the app loads the GitHub-backed file list for a tab-switch save',
  async ({ page }) => {
    await page.goto('/')
    await openFileList(page)
  },
)

When(
  'I select the {string} tab to test the tab switch',
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
  'I append {string} to the editor to trigger a tab-switch save',
  async ({ page }, text: string) => {
    await typeAtEditorEnd(page, text)
  },
)

Then('no PUT has been sent yet before the tab switch', async () => {
  // Right after the edit, the debounce hasn't fired yet: no PUT sent, and
  // the save-status badge shows the pending "Unsaved" countdown.
  expect(putBodies).toHaveLength(0)
})

Then(
  'the tab-switch status badge shows {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toContainText(text)
  },
)

When(
  'I switch to the {string} tab from the file list',
  async ({ page }, name: string) => {
    await openFileList(page)
    await page.locator('.file-tab-name', { hasText: name }).click()
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then(
  'the tab-switch PUT lands for {string} containing {string}',
  async ({}, path: string, content: string) => {
    // No `page.clock.fastForward` call anywhere in this test: the PUT firing
    // here proves the tab switch itself forced the flush.
    await expect
      .poll(() => putBodies.find((body) => body.path === path))
      .toMatchObject({ content: expect.stringContaining(content) })
  },
)

When('I reload the page after the tab-switch save', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, so the edit surviving
  // a reload proves the flush actually landed in the fake remote, not just
  // in-memory React state.
  await page.reload()
  await openFileList(page)
})

Then(
  'the tab-switch-saved file list still shows {string} after reload',
  async ({ page }, name: string) => {
    await page.locator('.file-tab-name', { hasText: name }).waitFor({
      timeout: 15_000,
    })
    await page.locator('.file-tab-name', { hasText: name }).click()
  },
)

Then(
  'the reloaded editor still contains the tab-switch-saved edit {string}',
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)
