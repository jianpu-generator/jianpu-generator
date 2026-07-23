import { expect, test } from '@playwright/test'
import {
  closeBin,
  fileSwitcherTrigger,
  openBin,
  openFileActions,
  openFileList,
} from './fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

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

test('deleting a file persists via the GitHub storage backend', async ({
  page,
}) => {
  await mockGithubContentsApi(
    page,
    { 'scores/original.jianpu': SOURCE },
    {
      // Slow enough for the delete button's pending spinner to be observable.
      mutationDelayMs: 300,
    },
  )

  await page.addInitScript(
    ({ owner }) => {
      localStorage.setItem(
        'jianpu:storage-backend:v1',
        JSON.stringify({ backend: 'github', github: { owner } }),
      )
      localStorage.setItem(
        'jianpu:github-auth:v1',
        JSON.stringify({ token: 'fake-token', scopes: ['repo'] }),
      )
    },
    { owner: OWNER },
  )

  await page.goto('/')

  // The GitHub-backed file list loads asynchronously; wait for the seeded
  // file's tab to appear (the read-only demo files live in a nested "Demo"
  // submenu, so they don't share this top-level list).
  await openFileList(page)
  const originalTab = page.locator('.file-tab-name', {
    hasText: 'original.jianpu',
  })
  await originalTab.waitFor({ timeout: 15_000 })

  // Select it (it isn't active by default — the backend always loads onto
  // the demo file). Selecting closes the file-switcher dropdown.
  await originalTab.click()
  await expect(fileSwitcherTrigger(page)).toContainText('original.jianpu')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await openFileActions(page)
  const deleteButton = page.getByRole('menuitem', { name: 'Delete' })
  await deleteButton.click()

  // The "⋯" dropdown stays open while the delete is pending, so its spinner
  // (PUT trash/... + DELETE scores/...) is visible without reopening —
  // user-visible feedback that the op is in flight, given time to render by
  // the mocked mutation delay (above).
  await expect(deleteButton.locator('.file-tab-bar-spinner')).toBeVisible()

  // Once the delete resolves, the "⋯" dropdown closes itself (it's now a
  // "Bin" menu item away from the deleted file's now-gone Delete button).
  await expect(deleteButton).toHaveCount(0, { timeout: 5_000 })

  // ...and the deleted file moves into the bin, listed by name.
  await openFileActions(page)
  await expect(page.locator('.file-tab-bar-bin-trigger')).toContainText(
    'Bin (1)',
  )
  await openBin(page)
  await expect(page.locator('.file-tab-bar-bin-name')).toHaveText(
    'original.jianpu',
  )
  await closeBin(page)

  // With no other user files remaining, the active tab falls back to the
  // (first) read-only demo file, and the trigger reflects it directly.
  await expect(fileSwitcherTrigger(page)).toContainText('01-pitches.jianpu')

  // Reloading re-fetches from the (mocked) GitHub API, so the deleted file
  // staying gone from the main tab list and present in the bin proves the
  // backend's PUT trash/... + DELETE scores/... pair actually landed in the
  // fake remote, not just in in-memory React state.
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await openFileList(page)
  await expect(
    page.locator('.file-tabs .file-tab-name', { hasText: 'original.jianpu' }),
  ).toHaveCount(0)
  await openFileActions(page)
  await expect(page.locator('.file-tab-bar-bin-trigger')).toContainText(
    'Bin (1)',
  )
  await openBin(page)
  await expect(page.locator('.file-tab-bar-bin-name')).toHaveText(
    'original.jianpu',
  )
})
