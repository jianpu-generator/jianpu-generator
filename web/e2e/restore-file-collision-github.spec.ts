import { expect, test } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openBin,
  openFileActions,
  openFileList,
} from './fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

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

test('restoring a file that collides with an active file renames it via the GitHub storage backend', async ({
  page,
}) => {
  const putCalls: { path: string; body: { content: string; sha?: string } }[] =
    []

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

  await openFileList(page)
  // Initial state: one active tab (the pre-existing scores/original.jianpu)
  // and one bin entry sharing the same base name.
  await expect(
    page.locator('.file-tabs .file-tab-name', { hasText: /^original$/ }),
  ).toHaveCount(1)
  await expect(
    page.locator('.file-tabs .file-tab-name', {
      hasText: 'original 2',
    }),
  ).toHaveCount(0)

  await openFileActions(page)
  await expect(page.locator('.file-tab-bar-bin-trigger')).toContainText(
    'Bin (1)',
  )

  await openBin(page)
  await expect(page.locator('.file-tab-bar-bin-name')).toHaveText('original')

  const restoreButton = page.locator(
    '.file-tab-bar-restore[aria-label="Restore original.jianpu"]',
  )
  await restoreButton.click()

  await expect(restoreButton.locator('.file-tab-bar-spinner')).toBeVisible()

  // Once the restore resolves, the bin empties out and the modal — which
  // blocks interaction with the rest of the page while it's open — closes
  // itself automatically.
  await expect(page.locator('[data-testid="bin-modal"]')).toHaveCount(0, {
    timeout: 5_000,
  })

  // The restored file gets renamed to avoid colliding with the existing
  // active `original.jianpu` tab.
  await openFileList(page)
  const restoredTab = page.locator('.file-tab-name', {
    hasText: 'original 2',
  })
  await restoredTab.waitFor({ timeout: 5_000 })

  // Both tabs now coexist: the pre-existing one, untouched, and the newly
  // restored one under its renamed identity.
  await expect(
    page.locator('.file-tabs .file-tab-name', { hasText: /^original$/ }),
  ).toHaveCount(1)
  await expect(
    page.locator('.file-tabs .file-tab-name', {
      hasText: 'original 2',
    }),
  ).toHaveCount(1)

  // The newly restored file is the active tab.
  await expect(fileSwitcherTrigger(page)).toContainText('original 2')

  // The bin is now empty.
  await openFileActions(page)
  await expect(page.locator('.file-tab-bar-bin-trigger')).toHaveCount(0)

  // The restore must never have PUT to the pre-existing file's path — it
  // should only create `scores/original 2.jianpu` and delete
  // `trash/original.jianpu`.
  expect(putCalls.some((call) => call.path === 'scores/original.jianpu')).toBe(
    false,
  )

  // Reloading re-fetches from the (mocked) GitHub API, proving both files
  // now genuinely exist as separate entries in the fake remote, and the
  // pre-existing file was never overwritten.
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await openFileList(page)
  await page
    .locator('.file-tab-name', { hasText: /^original$/ })
    .waitFor({ timeout: 15_000 })
  await expect(
    page.locator('.file-tabs .file-tab-name', { hasText: /^original$/ }),
  ).toHaveCount(1)
  await expect(
    page.locator('.file-tabs .file-tab-name', {
      hasText: 'original 2',
    }),
  ).toHaveCount(1)
  await openFileActions(page)
  await expect(page.locator('.file-tab-bar-bin-trigger')).toHaveCount(0)

  // The pre-existing tab's content must be untouched by the restore.
  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: /^original$/ }).click()
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    'Existing Active File',
  )
})
