import { expect, type Page } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

async function getEditorSource(page: Page) {
  return page.evaluate(() => {
    const editors = (
      window as unknown as {
        monaco?: {
          editor?: {
            getEditors?: () => { getValue?: () => string }[]
          }
        }
      }
    ).monaco?.editor?.getEditors?.()
    return editors?.[0]?.getValue?.() ?? ''
  })
}

const SOURCE = [
  '# metadata',
  'title = "Duplicate Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

let putBodies: { path: string; sha?: string }[] = []
let sourceContent = ''
const duplicateButton = ({ page }: { page: Page }) =>
  page.locator('.export-menu-item').nth(1)

Given(
  'the GitHub repo is seeded with a file named {string} for duplication',
  async ({ page }, path: string) => {
    putBodies = []
    await mockGithubContentsApi(
      page,
      { [path]: SOURCE },
      {
        onPut: (path, body) => putBodies.push({ path, sha: body.sha }),
        // Slow enough for the "Duplicate" button's pending spinner to be observable.
        mutationDelayMs: 300,
      },
    )
  },
)

When(
  'the app loads the GitHub-backed file list for duplication',
  async ({ page }) => {
    await page.goto('/')

    // The GitHub-backed file list loads asynchronously; wait for the seeded
    // file's tab to appear (the read-only demo files live in their own
    // dropdown now, so they no longer share this list).
    await openFileList(page)
  },
)

When(
  'I select the {string} tab to duplicate it',
  async ({ page }, name: string) => {
    const tab = page.locator('.file-tab-name', { hasText: name })
    await tab.waitFor({ timeout: 15_000 })

    // Select it (duplicateFile duplicates the active file), then wait for its
    // preview to render before duplicating. Selecting closes the dropdown.
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(name)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Given("I capture the active editor's source content", async ({ page }) => {
  sourceContent = await getEditorSource(page)
})

When(
  'I click the {string} button to duplicate the file',
  async ({ page }, label: string) => {
    expect(label).toBe('Duplicate')
    // Positional locator (not `hasText: 'Duplicate'`) since its label is
    // swapped for a spinner while the duplicate is pending.
    await openFileActions(page)
    await duplicateButton({ page }).click()
  },
)

Then('the duplicate button shows a pending spinner', async ({ page }) => {
  // The "⋯" dropdown stays open while the duplicate is pending, so its
  // spinner is visible without reopening — user-visible feedback that the
  // op is in flight, given time to render by the mocked PUT's artificial
  // delay (above).
  await expect(
    duplicateButton({ page }).locator('.file-tab-bar-spinner'),
  ).toBeVisible()
})

Then(
  'the active tab becomes the duplicate {string}',
  async ({ page }, name: string) => {
    // `duplicateFile` names the copy `original 2.jianpu` since `original.jianpu`
    // is already taken, and it becomes the active tab.
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then(
  'the duplicate button spinner clears and its label resets to {string}',
  async ({ page }, label: string) => {
    // Once the duplicate resolves, the dropdown closes automatically; reopen
    // it and "Duplicate" is usable again.
    await openFileActions(page)
    await expect(
      duplicateButton({ page }).locator('.file-tab-bar-spinner'),
    ).toHaveCount(0)
    await expect(duplicateButton({ page })).toHaveText(label)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then(
  'the duplicated editor content matches the captured source content',
  async ({ page }) => {
    // The duplicate's editor content matches the source's — proves
    // `duplicateFile` actually copied the content rather than starting blank.
    await expect
      .poll(() => getEditorSource(page), { timeout: 5_000 })
      .toBe(sourceContent)
  },
)

Then(
  'the duplicate-create PUT for {string} carries no sha',
  async ({}, path: string) => {
    // Create-only: the PUT that lands the duplicate must not carry a `sha` —
    // a `sha` would mean the backend fetched the file first, which
    // `duplicateFile` should never do.
    expect(putBodies).toContainEqual({ path, sha: undefined })
  },
)

When('I reload the page after duplicating', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, so the duplicate tab
  // persisting across a reload proves the backend's create-only `PUT`
  // actually landed in the fake remote, not just in in-memory React state.
  await page.reload()
  await openFileList(page)
})

Then(
  'the file list still shows both {string} and {string} tabs',
  async ({ page }, originalName: string, duplicateName: string) => {
    const duplicateTab = page.locator('.file-tab-name', {
      hasText: duplicateName,
    })
    await duplicateTab.waitFor({ timeout: 15_000 })
    // Exact-text match, not `hasText: originalName` — since the display name
    // no longer carries the `.jianpu` suffix, the original's name (e.g.
    // "original") is now a substring of the duplicate's (e.g. "original 2"),
    // so a plain substring match would match both tabs.
    await expect(
      page.locator('.file-tab-name', {
        hasText: new RegExp(`^${originalName}$`),
      }),
    ).toHaveCount(1)
    await duplicateTab.click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then(
  "the reloaded duplicate's editor content matches the captured source content",
  async ({ page }) => {
    await expect
      .poll(() => getEditorSource(page), { timeout: 5_000 })
      .toBe(sourceContent)
  },
)
