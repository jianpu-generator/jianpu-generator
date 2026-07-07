import { expect, test } from '@playwright/test'
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
  await mockGithubContentsApi(page, { 'scores/original.jianpu': SOURCE }, {
    // Slow enough for the delete button's pending spinner to be observable.
    mutationDelayMs: 300,
  })

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
  // file's tab to appear alongside the read-only demo tab.
  const originalTab = page.locator('.file-tab-name', {
    hasText: 'original.jianpu',
  })
  await originalTab.waitFor({ timeout: 15_000 })

  // Select it (it isn't active by default — the backend always loads onto
  // the demo file).
  await originalTab.click()
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'original.jianpu',
  )
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const closeButton = page.locator(
    '.file-tab-close[aria-label="Move original.jianpu to bin"]',
  )
  await closeButton.click()

  // The pending `deleteFile` call (PUT trash/... + DELETE scores/...) shows a
  // spinner on the close button being deleted — this is user-visible
  // feedback that the op is in flight, and the mocked mutation delay (above)
  // gives it time to actually render.
  await expect(closeButton.locator('.file-tab-bar-spinner')).toBeVisible()

  // Once the delete resolves, the tab disappears from the tab list...
  await expect(
    page.locator('.file-tabs .file-tab-name', { hasText: 'original.jianpu' }),
  ).toHaveCount(0, { timeout: 5_000 })

  // ...and the deleted file moves into the bin, listed by name.
  await expect(page.locator('.file-tab-bar-bin-summary')).toHaveText(
    'Bin (1)',
  )
  const binDetails = page.locator('.file-tab-bar-bin')
  await binDetails.evaluate((el) => {
    ;(el as HTMLDetailsElement).open = true
  })
  await expect(page.locator('.file-tab-bar-bin-name')).toHaveText(
    'original.jianpu',
  )

  // With no other user files remaining, the active tab falls back to the
  // read-only demo file.
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'reference.jianpu',
  )

  // Reloading re-fetches from the (mocked) GitHub API, so the deleted file
  // staying gone from the main tab list and present in the bin proves the
  // backend's PUT trash/... + DELETE scores/... pair actually landed in the
  // fake remote, not just in in-memory React state.
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await expect(
    page.locator('.file-tabs .file-tab-name', { hasText: 'original.jianpu' }),
  ).toHaveCount(0)
  await expect(page.locator('.file-tab-bar-bin-summary')).toHaveText(
    'Bin (1)',
  )
  const binDetailsAfterReload = page.locator('.file-tab-bar-bin')
  await binDetailsAfterReload.evaluate((el) => {
    ;(el as HTMLDetailsElement).open = true
  })
  await expect(page.locator('.file-tab-bar-bin-name')).toHaveText(
    'original.jianpu',
  )
})
