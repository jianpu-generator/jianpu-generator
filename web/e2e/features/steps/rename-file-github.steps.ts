import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '# metadata',
  'title = "Rename Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

Given(
  'the GitHub repo is seeded with a file named {string} for renaming',
  async ({ page }, path: string) => {
    await mockGithubContentsApi(
      page,
      { [path]: SOURCE },
      {
        // Slow enough for the renaming tab's pending spinner to be observable.
        mutationDelayMs: 300,
      },
    )
  },
)

When('the app loads the GitHub-backed file list', async ({ page }) => {
  await page.goto('/')

  // The GitHub-backed file list loads asynchronously; wait for the seeded
  // file's tab to appear (the read-only demo files live in their own
  // dropdown now, so they no longer share this list).
  await openFileList(page)
})

When(
  'I select the {string} tab from the file list',
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

When(
  'I rename the active tab to {string}',
  async ({ page }, newName: string) => {
    // Reopen the dropdown, since selecting the tab above closed it.
    await openFileList(page)
    const activeTab = page.locator('.file-tab--active .file-tab-name')
    await activeTab.dblclick()
    const input = page.locator('.file-tab--active input.file-tab-name')
    await input.fill(newName)
    await input.press('Enter')
  },
)

Then('the active tab shows a pending rename spinner', async ({ page }) => {
  // The pending `renameFile` call shows a spinner on the tab being renamed —
  // this is user-visible feedback that the op is in flight, and the mocked
  // create+delete pair's artificial delay (above) gives it time to render.
  const activeTabName = page.locator('.file-tab--active .file-tab-name')
  await expect(activeTabName.locator('.file-tab-bar-spinner')).toBeVisible()
})

Then(
  'the active tab is renamed to {string}',
  async ({ page }, name: string) => {
    // The tab reflects the new name, and its content/preview survive the
    // rename (i.e. the rename resolved through the mocked create+delete
    // Contents API calls rather than getting stuck or reverting).
    const activeTabName = page.locator('.file-tab--active .file-tab-name')
    await expect(activeTabName).toHaveText(name)
  },
)

Then("the renamed file's preview is visible", async ({ page }) => {
  await expect(page.locator('.preview-page').first()).toBeVisible({
    timeout: 5_000,
  })
})

When('I reload the page after renaming', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, so the renamed tab
  // persisting across a reload proves the backend's create+delete pair
  // actually landed in the fake remote, not just in in-memory React state.
  await page.reload()
})

Then(
  'the file list still shows {string} after reload',
  async ({ page }, name: string) => {
    await openFileList(page)
    await page.locator('.file-tab-name', { hasText: name }).waitFor({
      timeout: 15_000,
    })
  },
)

Then(
  'the file list no longer shows {string} after reload',
  async ({ page }, name: string) => {
    await expect(page.locator('.file-tab-name', { hasText: name })).toHaveCount(
      0,
    )
  },
)
