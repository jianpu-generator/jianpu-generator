import { expect, test } from '@playwright/test'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

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

test('restoring a file persists via the GitHub storage backend', async ({
  page,
}) => {
  // Seed only `trash/`, so the file loads straight into the bin without a
  // prior delete step in this test.
  await mockGithubContentsApi(
    page,
    { 'trash/original.jianpu': SOURCE },
    {
      // Slow enough for the restore button's pending spinner to be observable.
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
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  // The seeded file loads straight into the bin, not the main tab list.
  await expect(page.locator('.file-tab-bar-bin-summary')).toHaveText('Bin (1)')
  await expect(
    page.locator('.file-tabs .file-tab-name', { hasText: 'original.jianpu' }),
  ).toHaveCount(0)

  const binDetails = page.locator('.file-tab-bar-bin')
  await binDetails.evaluate((el) => {
    ;(el as HTMLDetailsElement).open = true
  })
  await expect(page.locator('.file-tab-bar-bin-name')).toHaveText(
    'original.jianpu',
  )

  const restoreButton = page.locator(
    '.file-tab-bar-restore[aria-label="Restore original.jianpu"]',
  )
  await restoreButton.click()

  // The pending `restoreFile` call (PUT scores/... + DELETE trash/...) shows
  // a spinner on the restore button — user-visible feedback that the op is
  // in flight, made observable by the mocked mutation delay (above).
  await expect(restoreButton.locator('.file-tab-bar-spinner')).toBeVisible()

  // Once the restore resolves, the file reappears as an active tab...
  const originalTab = page.locator('.file-tab-name', {
    hasText: 'original.jianpu',
  })
  await originalTab.waitFor({ timeout: 5_000 })
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'original.jianpu',
  )

  // ...and the bin empties out, so its `<details>` no longer renders.
  await expect(page.locator('.file-tab-bar-bin')).toHaveCount(0)

  // Reloading re-fetches from the (mocked) GitHub API, so the restored file
  // staying in the main tab list and gone from the bin proves the backend's
  // PUT scores/... + DELETE trash/... pair actually landed in the fake
  // remote, not just in-memory React state.
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await page.locator('.file-tab-name', { hasText: 'original.jianpu' }).waitFor({
    timeout: 15_000,
  })
  await expect(page.locator('.file-tab-bar-bin')).toHaveCount(0)
})
