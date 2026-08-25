import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openBin,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

const EXISTING_SOURCE = [
  '# metadata',
  'title = "Existing Active File"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '5 6 7 1',
].join('\n')

const RESTORED_SOURCE = [
  '# metadata',
  'title = "Restored From Bin"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

let putCalls: { path: string; body: { content: string; sha?: string } }[] = []

Given(
  'the GitHub repo has {string} active and {string} binned, for a restore collision',
  async ({ page }, _activePath: string, _binnedPath: string) => {
    putCalls = []
    await mockGithubContentsApi(
      page,
      {
        'scores/original.jianpu': EXISTING_SOURCE,
        'trash/original.jianpu': RESTORED_SOURCE,
      },
      {
        mutationDelayMs: 300,
        onPut: (path, body) => putCalls.push({ path, body }),
      },
    )
  },
)

When(
  'the app loads with the preview ready for the collision test',
  async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

When('I open the file list', async ({ page }) => {
  await openFileList(page)
})

Then(
  'exactly one {string} tab exists and no {string} tab exists',
  async ({ page }, presentName: string, absentName: string) => {
    // Initial state: one active tab (the pre-existing scores/original.jianpu)
    // and one bin entry sharing the same base name.
    await expect(
      page.locator('.file-tabs .file-tab-name', { hasText: presentName }),
    ).toHaveCount(1)
    await expect(
      page.locator('.file-tabs .file-tab-name', { hasText: absentName }),
    ).toHaveCount(0)
  },
)

Then(
  'the file actions bin trigger shows {string} before the collision restore',
  async ({ page }, text: string) => {
    await openFileActions(page)
    await expect(page.locator('.file-tab-bar-bin-trigger')).toContainText(text)
  },
)

When('I open the bin', async ({ page }) => {
  await openBin(page)
})

Then(
  'the bin lists the colliding restorable file {string}',
  async ({ page }, name: string) => {
    await expect(page.locator('.file-tab-bar-bin-name')).toHaveText(name)
  },
)

When(
  'I click the restore button for {string}',
  async ({ page }, name: string) => {
    const restoreButton = page.locator(
      `.file-tab-bar-restore[aria-label="Restore ${name}"]`,
    )
    await restoreButton.click()
  },
)

Then(
  'the colliding restore button shows a pending spinner',
  async ({ page }) => {
    const restoreButton = page.locator(
      '.file-tab-bar-restore[aria-label="Restore original.jianpu"]',
    )
    await expect(restoreButton.locator('.file-tab-bar-spinner')).toBeVisible()
  },
)

Then(
  'the bin modal closes once the colliding restore resolves',
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
  'a {string} tab appears within 5 seconds',
  async ({ page }, name: string) => {
    // The restored file gets renamed to avoid colliding with the existing
    // active `original.jianpu` tab.
    const restoredTab = page.locator('.file-tab-name', { hasText: name })
    await restoredTab.waitFor({ timeout: 5_000 })
  },
)

Then(
  'both {string} and {string} tabs exist exactly once each',
  async ({ page }, nameA: string, nameB: string) => {
    // Both tabs now coexist: the pre-existing one, untouched, and the newly
    // restored one under its renamed identity. Exact-text match, not
    // `hasText: nameA` — since display names drop the `.jianpu` suffix,
    // nameA (e.g. "original") is now a substring of nameB (e.g.
    // "original 2"), so a plain substring match would match both tabs.
    await expect(
      page.locator('.file-tabs .file-tab-name', {
        hasText: new RegExp(`^${nameA}$`),
      }),
    ).toHaveCount(1)
    await expect(
      page.locator('.file-tabs .file-tab-name', { hasText: nameB }),
    ).toHaveCount(1)
  },
)

Then(
  'the active tab is now the renamed restored file {string}',
  async ({ page }, name: string) => {
    // The newly restored file is the active tab.
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then(
  'the file actions bin trigger is gone after the collision restore',
  async ({ page }) => {
    // The bin is now empty.
    await openFileActions(page)
    await expect(page.locator('.file-tab-bar-bin-trigger')).toHaveCount(0)
  },
)

Then(
  'no PUT was ever sent to {string} during the restore',
  async ({}, path: string) => {
    // The restore must never have PUT to the pre-existing file's path — it
    // should only create `scores/original 2.jianpu` and delete
    // `trash/original.jianpu`.
    expect(putCalls.some((call) => call.path === path)).toBe(false)
  },
)

When('I reload the page after the collision restore', async ({ page }) => {
  // Reloading re-fetches from the (mocked) GitHub API, proving both files
  // now genuinely exist as separate entries in the fake remote, and the
  // pre-existing file was never overwritten.
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await openFileList(page)
  await page
    .locator('.file-tab-name', { hasText: /^original$/ })
    .waitFor({ timeout: 15_000 })
})

Then(
  'both {string} and {string} tabs exist exactly once each after reload',
  async ({ page }, nameA: string, nameB: string) => {
    // Exact-text match for nameA — see the non-reload version of this step.
    await expect(
      page.locator('.file-tabs .file-tab-name', {
        hasText: new RegExp(`^${nameA}$`),
      }),
    ).toHaveCount(1)
    await expect(
      page.locator('.file-tabs .file-tab-name', { hasText: nameB }),
    ).toHaveCount(1)
  },
)

Then('the file actions bin trigger is gone after reload', async ({ page }) => {
  await openFileActions(page)
  await expect(page.locator('.file-tab-bar-bin-trigger')).toHaveCount(0)
})

Then(
  "the pre-existing tab's content still shows {string}",
  async ({ page }, text: string) => {
    // The pre-existing tab's content must be untouched by the restore.
    await openFileList(page)
    await page.locator('.file-tab-name', { hasText: /^original$/ }).click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)
