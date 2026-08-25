import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '# metadata',
  'title = "New File Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

let putBodies: { path: string; sha?: string }[] = []
const newButton = ({ page }: { page: import('@playwright/test').Page }) =>
  page.locator('.export-menu-item').first()

Given(
  'the GitHub repo is seeded with a file named {string} for new-file creation',
  async ({ page }, path: string) => {
    putBodies = []
    await mockGithubContentsApi(
      page,
      { [path]: SOURCE },
      {
        onPut: (path, body) => putBodies.push({ path, sha: body.sha }),
        // Slow enough for the "New" button's pending spinner to be observable.
        mutationDelayMs: 300,
      },
    )
  },
)

When(
  'the app loads the GitHub-backed file list for new-file creation',
  async ({ page }) => {
    await page.goto('/')

    // The GitHub-backed file list loads asynchronously; wait for the seeded
    // file's tab to appear (the read-only demo files live in their own
    // dropdown now, so they no longer share this list).
    await openFileList(page)
    const originalTab = page.locator('.file-tab-name', {
      hasText: 'original',
    })
    await originalTab.waitFor({ timeout: 15_000 })
  },
)

When(
  'I click the {string} button to create a new file',
  async ({ page }, label: string) => {
    expect(label).toBe('New')
    // Positional locator (not `hasText: 'New'`) since its label is swapped for
    // a spinner while the create is pending.
    await openFileActions(page)
    await newButton({ page }).click()
  },
)

Then('the new-file button shows a pending spinner', async ({ page }) => {
  // The "⋯" dropdown stays open while the create is pending, so its spinner
  // on the "New" button is visible without reopening — user-visible
  // feedback that the op is in flight, given time to render by the mocked
  // PUT's artificial delay (above).
  await expect(
    newButton({ page }).locator('.file-tab-bar-spinner'),
  ).toBeVisible()
})

Then(
  'the active tab becomes the new file {string}',
  async ({ page }, name: string) => {
    // `createFile` names the new file `untitled.jianpu` since that name isn't
    // already taken, and it becomes the active tab.
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then(
  'the new-file button disappears once the create resolves',
  async ({ page }) => {
    // Once the create resolves, the dropdown closes automatically.
    await expect(newButton({ page })).toHaveCount(0)
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then(
  'the new-file create PUT for {string} carries no sha',
  async ({}, path: string) => {
    // Create-only: the PUT that lands the new file must not carry a `sha` —
    // a `sha` would mean the backend fetched the file first, which
    // `createFile` should never do.
    expect(putBodies).toContainEqual({ path, sha: undefined })
  },
)

When('I reload the page after creating', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, so the new tab
  // persisting across a reload proves the backend's create-only `PUT`
  // actually landed in the fake remote, not just in in-memory React state.
  await page.reload()
  await openFileList(page)
})

Then(
  'the file list still shows the new file {string} after reload',
  async ({ page }, name: string) => {
    await page.locator('.file-tab-name', { hasText: name }).waitFor({
      timeout: 15_000,
    })
  },
)

Then(
  'the file list still shows {string} exactly once',
  async ({ page }, name: string) => {
    await expect(page.locator('.file-tab-name', { hasText: name })).toHaveCount(
      1,
    )
  },
)
