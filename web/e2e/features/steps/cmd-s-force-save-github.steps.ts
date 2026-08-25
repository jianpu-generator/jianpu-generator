import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '# metadata',
  'title = "Cmd+S Force Save Test"',
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
  'the GitHub repo is seeded with a file named {string} for a force save',
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
  'a fake clock is installed to prevent an autosave race with force save',
  async ({ page }) => {
    // Install the fake clock so the debounce timer never elapses on its own —
    // the test relies on Cmd/Ctrl+S itself forcing the flush, not the
    // debounce interval happening to run out.
    await page.clock.install()
  },
)

When(
  'the app loads the GitHub-backed file list for a force save',
  async ({ page }) => {
    await page.goto('/')
    await openFileList(page)
  },
)

When(
  'I select the {string} tab to test the force save',
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
  'I append {string} to the editor to trigger a force save',
  async ({ page }, text: string) => {
    await typeAtEditorEnd(page, text)
  },
)

Then('no PUT has been sent yet before the force save', async () => {
  // Right after the edit, the debounce hasn't fired yet: no PUT sent, and
  // the badge shows the pending "Unsaved" countdown.
  expect(putBodies).toHaveLength(0)
})

Then(
  'the force-save status badge shows {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toContainText(text)
  },
)

When('I press Cmd\\/Ctrl+S', async ({ page }) => {
  // No `page.clock.fastForward` call anywhere in this test: the PUT firing
  // here proves the shortcut itself forced the flush. (Other specs, e.g.
  // `conflict-resolution-github.spec.ts`, use the same `Meta+s` chord.)
  await page.keyboard.press('Meta+s')
})

Then(
  'the force-save PUT lands for {string} containing {string}',
  async ({}, path: string, content: string) => {
    await expect
      .poll(() => putBodies.find((body) => body.path === path))
      .toMatchObject({ content: expect.stringContaining(content) })
  },
)

When('I reload the page after the force save', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, so the edit surviving
  // a reload proves the flush actually landed in the fake remote, not just
  // in-memory React state.
  await page.reload()
  await openFileList(page)
})

Then(
  'the force-saved file list still shows {string} after reload',
  async ({ page }, name: string) => {
    await page.locator('.file-tab-name', { hasText: name }).waitFor({
      timeout: 15_000,
    })
    await page.locator('.file-tab-name', { hasText: name }).click()
  },
)

Then(
  'the reloaded editor still contains the force-saved edit {string}',
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)
