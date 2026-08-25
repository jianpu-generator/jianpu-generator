import { expect } from '@playwright/test'
import {
  closeBin,
  fileSwitcherTrigger,
  openBin,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '# metadata',
  'title = "Delete Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

const deleteButton = ({ page }: { page: import('@playwright/test').Page }) =>
  page.getByRole('menuitem', { name: 'Delete' })

Given(
  'the GitHub repo is seeded with a file named {string} for deletion',
  async ({ page }, path: string) => {
    await mockGithubContentsApi(
      page,
      { [path]: SOURCE },
      {
        // Slow enough for the delete button's pending spinner to be observable.
        mutationDelayMs: 300,
      },
    )
  },
)

When(
  'the app loads the GitHub-backed file list for deletion',
  async ({ page }) => {
    await page.goto('/')

    // The GitHub-backed file list loads asynchronously; wait for the seeded
    // file's tab to appear (the read-only demo files live in a nested "Demo"
    // submenu, so they don't share this top-level list).
    await openFileList(page)
  },
)

When(
  'I select the {string} tab to delete it',
  async ({ page }, name: string) => {
    const tab = page.locator('.file-tab-name', { hasText: name })
    await tab.waitFor({ timeout: 15_000 })

    // Select it (it isn't active by default — the backend always loads onto
    // the demo file). Selecting closes the file-switcher dropdown.
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(name)
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

When('I delete the active file via the file actions menu', async ({ page }) => {
  await openFileActions(page)
  await deleteButton({ page }).click()
})

Then('the delete button shows a pending spinner', async ({ page }) => {
  // The "⋯" dropdown stays open while the delete is pending, so its spinner
  // (PUT trash/... + DELETE scores/...) is visible without reopening —
  // user-visible feedback that the op is in flight, given time to render by
  // the mocked mutation delay (above).
  await expect(
    deleteButton({ page }).locator('.file-tab-bar-spinner'),
  ).toBeVisible()
})

Then(
  'the delete button disappears once the delete resolves',
  async ({ page }) => {
    // Once the delete resolves, the "⋯" dropdown closes itself (it's now a
    // "Bin" menu item away from the deleted file's now-gone Delete button).
    await expect(deleteButton({ page })).toHaveCount(0, { timeout: 5_000 })
  },
)

Then(
  'the file actions bin trigger shows {string} after deleting',
  async ({ page }, text: string) => {
    // ...and the deleted file moves into the bin, listed by name.
    await openFileActions(page)
    await expect(page.locator('.file-tab-bar-bin-trigger')).toContainText(text)
  },
)

Then(
  'the bin lists the deleted file {string}',
  async ({ page }, name: string) => {
    await openBin(page)
    await expect(page.locator('.file-tab-bar-bin-name')).toHaveText(name)
    await closeBin(page)
  },
)

Then(
  'the active tab falls back to {string}',
  async ({ page }, name: string) => {
    // With no other user files remaining, the active tab falls back to the
    // (first) read-only demo file, and the trigger reflects it directly.
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

When('I reload the page after deleting', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, so the deleted file
  // staying gone from the main tab list and present in the bin proves the
  // backend's PUT trash/... + DELETE scores/... pair actually landed in the
  // fake remote, not just in in-memory React state.
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await openFileList(page)
})

Then(
  'the file list no longer shows the deleted file {string} after reload',
  async ({ page }, name: string) => {
    await expect(
      page.locator('.file-tabs .file-tab-name', { hasText: name }),
    ).toHaveCount(0)
  },
)

Then(
  'the file actions bin trigger shows {string} after reload',
  async ({ page }, text: string) => {
    await openFileActions(page)
    await expect(page.locator('.file-tab-bar-bin-trigger')).toContainText(text)
  },
)

Then('the bin lists {string} after reload', async ({ page }, name: string) => {
  await openBin(page)
  await expect(page.locator('.file-tab-bar-bin-name')).toHaveText(name)
})
