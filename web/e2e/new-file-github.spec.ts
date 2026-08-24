import { expect, test } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileActions,
  openFileList,
} from './fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

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

test('creating a new file persists via the GitHub storage backend', async ({
  page,
}) => {
  const putBodies: { path: string; sha?: string }[] = []
  await mockGithubContentsApi(
    page,
    { 'scores/original.jianpu': SOURCE },
    {
      onPut: (path, body) => putBodies.push({ path, sha: body.sha }),
      // Slow enough for the "New" button's pending spinner to be observable.
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
  // file's tab to appear (the read-only demo files live in their own
  // dropdown now, so they no longer share this list).
  await openFileList(page)
  const originalTab = page.locator('.file-tab-name', {
    hasText: 'original',
  })
  await originalTab.waitFor({ timeout: 15_000 })

  // Positional locator (not `hasText: 'New'`) since its label is swapped for
  // a spinner while the create is pending.
  await openFileActions(page)
  const newButton = page.locator('.export-menu-item').first()
  await newButton.click()

  // The "⋯" dropdown stays open while the create is pending, so its spinner
  // on the "New" button is visible without reopening — user-visible
  // feedback that the op is in flight, given time to render by the mocked
  // PUT's artificial delay (above).
  await expect(newButton.locator('.file-tab-bar-spinner')).toBeVisible()

  // `createFile` names the new file `untitled.jianpu` since that name isn't
  // already taken, and it becomes the active tab.
  const newTab = page.locator('.file-tab-name', { hasText: 'untitled' })
  await expect(fileSwitcherTrigger(page)).toContainText('untitled')

  // Once the create resolves, the dropdown closes automatically.
  await expect(newButton).toHaveCount(0)
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  // Create-only: the PUT that lands the new file must not carry a `sha` —
  // a `sha` would mean the backend fetched the file first, which
  // `createFile` should never do.
  expect(putBodies).toContainEqual({
    path: 'scores/untitled.jianpu',
    sha: undefined,
  })

  // Reloading re-fetches from the (mocked) GitHub API, so the new tab
  // persisting across a reload proves the backend's create-only `PUT`
  // actually landed in the fake remote, not just in in-memory React state.
  await page.reload()
  await openFileList(page)
  await newTab.waitFor({ timeout: 15_000 })
  await expect(
    page.locator('.file-tab-name', { hasText: 'original' }),
  ).toHaveCount(1)
})
