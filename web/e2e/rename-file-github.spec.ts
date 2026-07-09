import { expect, test } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from './fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

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

test('renaming a file persists via the GitHub storage backend', async ({
  page,
}) => {
  await mockGithubContentsApi(
    page,
    { 'scores/original.jianpu': SOURCE },
    {
      // Slow enough for the renaming tab's pending spinner to be observable.
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
  // file's tab to appear alongside the read-only demo tab.
  await openFileList(page)
  const originalTab = page.locator('.file-tab-name', {
    hasText: 'original.jianpu',
  })
  await originalTab.waitFor({ timeout: 15_000 })

  // Select it (it isn't active by default — the backend always loads onto
  // the demo file), then double-click to enter rename mode. Selecting closes
  // the file-switcher dropdown, so reopen it to reach the (now-active) tab.
  await originalTab.click()
  await expect(fileSwitcherTrigger(page)).toContainText('original.jianpu')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await openFileList(page)
  await originalTab.dblclick()
  const input = page.locator('.file-tab--active input.file-tab-name')
  await input.fill('renamed.jianpu')
  await input.press('Enter')

  const activeTabName = page.locator('.file-tab--active .file-tab-name')

  // The pending `renameFile` call shows a spinner on the tab being renamed —
  // this is user-visible feedback that the op is in flight, and the mocked
  // create+delete pair's artificial delay (above) gives it time to render.
  await expect(activeTabName.locator('.file-tab-bar-spinner')).toBeVisible()

  // The tab reflects the new name, and its content/preview survive the
  // rename (i.e. the rename resolved through the mocked create+delete
  // Contents API calls rather than getting stuck or reverting).
  await expect(activeTabName).toHaveText('renamed.jianpu')
  await expect(page.locator('.preview-page').first()).toBeVisible({
    timeout: 5_000,
  })

  // Reloading re-fetches from the (mocked) GitHub API, so the renamed tab
  // persisting across a reload proves the backend's create+delete pair
  // actually landed in the fake remote, not just in in-memory React state.
  await page.reload()
  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: 'renamed.jianpu' }).waitFor({
    timeout: 15_000,
  })
  await expect(
    page.locator('.file-tab-name', { hasText: 'original.jianpu' }),
  ).toHaveCount(0)
})
