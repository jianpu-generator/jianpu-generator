import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import {
  currentContentSaveTracker,
  installContentSaveTracking,
  lastContentSaveFor,
} from './cloud-content-save-tracking'
import { Given, Then, When } from './fixtures'

// Autosave coverage for the cloud storage backend, split out of
// `files-cloud-backend.steps.ts` to stay under this repo's 400-line cap.

Given(
  'a fake clock is installed before navigating to test autosave',
  async ({ page }) => {
    await page.clock.install()
  },
)

When(
  'the app loads the cloud-backed file list for autosave',
  async ({ page }) => {
    installContentSaveTracking(page)
    await gotoCloudApp(page)
    await openFileList(page)
  },
)

When(
  'I select the {string} tab to test autosave',
  async ({ page }, name: string) => {
    const tab = fileTabByExactName(page, name)
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

Then(
  'no content request has been sent yet for the debounced edit',
  async () => {
    expect(currentContentSaveTracker().records).toHaveLength(0)
  },
)

Then(
  'the autosave status badge shows {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toContainText(text)
  },
)

// Mirrors `useStorageBackend.ts`'s `AUTOSAVE_DEBOUNCE_MS`. Not imported
// directly -- that module transitively pulls in `fileStore.ts`'s Vite-only
// `?raw` import, which Playwright's test loader can't resolve.
const AUTOSAVE_DEBOUNCE_MS = 20_000

When(
  'I fast-forward the clock past the autosave debounce interval to trigger it',
  async ({ page }) => {
    await page.clock.fastForward(AUTOSAVE_DEBOUNCE_MS)
  },
)

Then(
  'the content save lands for {string} containing {string}',
  async ({}, name: string, text: string) => {
    await expect
      .poll(() => lastContentSaveFor(currentContentSaveTracker(), name), {
        timeout: 10_000,
      })
      .toEqual(expect.stringContaining(text))
  },
)

When('I reload the page after the autosave', async ({ page }) => {
  await page.reload()
  await openFileList(page)
})

Then(
  'the reloaded editor still contains the autosaved edit {string}',
  async ({ page }, text: string) => {
    await openFileList(page)
    const tab = fileTabByExactName(page, 'auto')
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)
