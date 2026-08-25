import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openBin,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '# metadata',
  'title = "Restore Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

Given(
  'the GitHub repo is seeded with only a binned file {string} for restoring',
  async ({ page }, path: string) => {
    // Seed only `trash/`, so the file loads straight into the bin without a
    // prior delete step in this test.
    await mockGithubContentsApi(
      page,
      { [path]: SOURCE },
      {
        // Slow enough for the restore button's pending spinner to be observable.
        mutationDelayMs: 300,
      },
    )
  },
)

When(
  'the app loads with the preview ready for the restore test',
  async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

When('I open the file list to check the restore', async ({ page }) => {
  await openFileList(page)
})

Then(
  'no {string} tab exists in the main list',
  async ({ page }, name: string) => {
    // The seeded file loads straight into the bin, not the main tab list.
    await expect(
      page.locator('.file-tabs .file-tab-name', { hasText: name }),
    ).toHaveCount(0)
  },
)

Then(
  'the file actions bin trigger shows {string} before restoring',
  async ({ page }, text: string) => {
    await openFileActions(page)
    await expect(page.locator('.file-tab-bar-bin-trigger')).toContainText(text)
  },
)

When('I open the bin to restore a file', async ({ page }) => {
  await openBin(page)
})

Then(
  'the bin lists the restorable file {string}',
  async ({ page }, name: string) => {
    await expect(page.locator('.file-tab-bar-bin-name')).toHaveText(name)
  },
)

When(
  'I click the restore-from-bin button for {string}',
  async ({ page }, name: string) => {
    const restoreButton = page.locator(
      `.file-tab-bar-restore[aria-label="Restore ${name}"]`,
    )
    await restoreButton.click()
  },
)

Then(
  'the restore-from-bin button shows a pending spinner',
  async ({ page }) => {
    // The pending `restoreFile` call (PUT scores/... + DELETE trash/...) shows
    // a spinner on the restore button — user-visible feedback that the op is
    // in flight, made observable by the mocked mutation delay (above).
    const restoreButton = page.locator(
      '.file-tab-bar-restore[aria-label="Restore original.jianpu"]',
    )
    await expect(restoreButton.locator('.file-tab-bar-spinner')).toBeVisible()
  },
)

Then(
  'the bin modal closes once the plain restore resolves',
  async ({ page }) => {
    // Once the restore resolves, the bin empties out and the modal — which
    // blocks interaction with the rest of the page while it's open — closes
    // itself automatically.
    await expect(page.locator('[data-testid="bin-modal"]')).toHaveCount(0, {
      timeout: 5_000,
    })
  },
)

Then(
  'the {string} tab reappears within 5 seconds',
  async ({ page }, name: string) => {
    // The restored file reappears as an active tab...
    const originalTab = page.locator('.file-tab-name', { hasText: name })
    await originalTab.waitFor({ timeout: 5_000 })
  },
)

Then(
  'the active tab is now the restored file {string}',
  async ({ page }, name: string) => {
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then(
  'the file actions bin trigger disappears after restoring',
  async ({ page }) => {
    // ...and the bin empties out, so its control no longer renders.
    await openFileActions(page)
    await expect(page.locator('.file-tab-bar-bin-trigger')).toHaveCount(0)
  },
)

When('I reload the page after restoring', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, so the restored file
  // staying in the main tab list and gone from the bin proves the backend's
  // PUT scores/... + DELETE trash/... pair actually landed in the fake
  // remote, not just in-memory React state.
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await openFileList(page)
})

Then(
  'the file list still shows {string} after the restore reload',
  async ({ page }, name: string) => {
    await page.locator('.file-tab-name', { hasText: name }).waitFor({
      timeout: 15_000,
    })
  },
)

Then(
  'the file actions bin trigger is gone after the restore reload',
  async ({ page }) => {
    await openFileActions(page)
    await expect(page.locator('.file-tab-bar-bin-trigger')).toHaveCount(0)
  },
)
