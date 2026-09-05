import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { gotoShareUrl } from '../../shareUrlHelper'
import { Given, Then, When } from './fixtures'

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

let putBodies: { path: string; sha?: string }[] = []

Given(
  'the GitHub Contents API is mocked with no files for a shared import',
  async ({ page }) => {
    putBodies = []
    await mockGithubContentsApi(
      page,
      {},
      {
        onPut: (path, body) => putBodies.push({ path, sha: body.sha }),
        // Slow enough for the import to still be in flight when we assert on it.
        mutationDelayMs: 300,
      },
    )
  },
)

When(
  'I navigate to the share URL for {string}',
  async ({ page }, filename: string) => {
    await gotoShareUrl(page, filename, SHARED_SOURCE)
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then('the shared-preview banner is visible', async ({ page }) => {
  await expect(page.locator('.shared-preview-banner')).toBeVisible()
})

When('I click the {string} button', async ({ page }, buttonName: string) => {
  await page.getByRole('button', { name: buttonName }).click()
})

Then(
  'the active tab becomes the imported file {string}',
  async ({ page }, name: string) => {
    // The banner is dismissed and the imported file becomes the active tab
    // once `backend.importFile`'s create-only `PUT` resolves.
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then('the shared-preview banner is gone', async ({ page }) => {
  await expect(page.locator('.shared-preview-banner')).toHaveCount(0)
})

Then(
  'the import-create PUT for {string} carries no sha',
  async ({}, path: string) => {
    // Create-only: the PUT that lands the import must not carry a `sha` — a
    // `sha` would mean the backend fetched the file first, which importing
    // (like new/duplicate) should never do.
    expect(putBodies).toContainEqual({ path, sha: undefined })
  },
)

When('I reload the page after importing', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, so the imported file
  // persisting across a reload proves the backend's create-only `PUT`
  // actually landed in the fake remote, not just in in-memory React state.
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Then(
  'the file list shows {string} after reload',
  async ({ page }, filename: string) => {
    await openFileList(page)
    await page
      .locator('.file-tab-name', { hasText: filename })
      .waitFor({ timeout: 15_000 })
  },
)
