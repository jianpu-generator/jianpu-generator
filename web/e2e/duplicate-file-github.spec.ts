import { expect, test } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileActions,
  openFileList,
} from './fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

async function getEditorSource(page: import('@playwright/test').Page) {
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

test('duplicating a file persists via the GitHub storage backend', async ({
  page,
}) => {
  const putBodies: { path: string; sha?: string }[] = []
  await mockGithubContentsApi(
    page,
    { 'scores/original.jianpu': SOURCE },
    {
      onPut: (path, body) => putBodies.push({ path, sha: body.sha }),
      // Slow enough for the "Duplicate" button's pending spinner to be observable.
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
    hasText: /^original$/,
  })
  await originalTab.waitFor({ timeout: 15_000 })

  // Select it (duplicateFile duplicates the active file), then wait for its
  // preview to render before duplicating. Selecting closes the dropdown.
  await originalTab.click()
  await expect(fileSwitcherTrigger(page)).toContainText('original')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  const sourceContent = await getEditorSource(page)

  // Positional locator (not `hasText: 'Duplicate'`) since its label is
  // swapped for a spinner while the duplicate is pending.
  await openFileActions(page)
  const duplicateButton = page.locator('.export-menu-item').nth(1)
  await duplicateButton.click()

  // The "⋯" dropdown stays open while the duplicate is pending, so its
  // spinner is visible without reopening — user-visible feedback that the
  // op is in flight, given time to render by the mocked PUT's artificial
  // delay (above).
  await expect(duplicateButton.locator('.file-tab-bar-spinner')).toBeVisible()

  // `duplicateFile` names the copy `original 2.jianpu` since `original.jianpu`
  // is already taken, and it becomes the active tab.
  await expect(fileSwitcherTrigger(page)).toContainText('original 2')
  const duplicateTab = page.locator('.file-tab-name', {
    hasText: 'original 2',
  })

  // Once the duplicate resolves, the dropdown closes automatically; reopen
  // it and "Duplicate" is usable again.
  await openFileActions(page)
  await expect(duplicateButton.locator('.file-tab-bar-spinner')).toHaveCount(0)
  await expect(duplicateButton).toHaveText('Duplicate')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  // The duplicate's editor content matches the source's — proves
  // `duplicateFile` actually copied the content rather than starting blank.
  await expect
    .poll(() => getEditorSource(page), { timeout: 5_000 })
    .toBe(sourceContent)

  // Create-only: the PUT that lands the duplicate must not carry a `sha` —
  // a `sha` would mean the backend fetched the file first, which
  // `duplicateFile` should never do.
  expect(putBodies).toContainEqual({
    path: 'scores/original 2.jianpu',
    sha: undefined,
  })

  // Reloading re-fetches from the (mocked) GitHub API, so the duplicate tab
  // persisting across a reload proves the backend's create-only `PUT`
  // actually landed in the fake remote, not just in in-memory React state.
  await page.reload()
  await openFileList(page)
  await duplicateTab.waitFor({ timeout: 15_000 })
  await expect(originalTab).toHaveCount(1)
  await duplicateTab.click()
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await expect
    .poll(() => getEditorSource(page), { timeout: 5_000 })
    .toBe(sourceContent)
})
