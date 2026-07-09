import { expect, test } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileList,
  typeAtEditorEnd,
} from './fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

// Mirrors `useStorageBackend.ts`'s `AUTOSAVE_DEBOUNCE_MS`. Not imported
// directly — that module transitively pulls in `fileStore.ts`'s Vite-only
// `?raw` import, which Playwright's test loader can't resolve.
const AUTOSAVE_DEBOUNCE_MS = 20_000

const SOURCE = [
  '# metadata',
  'title = "Autosave Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

test('editing a file schedules a debounced autosave to the GitHub storage backend', async ({
  page,
}) => {
  const putBodies: { path: string; content: string }[] = []
  await mockGithubContentsApi(
    page,
    { 'scores/auto.jianpu': SOURCE },
    {
      onPut: (path, body) =>
        putBodies.push({
          path,
          content: Buffer.from(body.content, 'base64').toString('utf-8'),
        }),
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

  // Install the fake clock before navigating. It keeps ticking in step with
  // real time until `fastForward` below, which lets us jump straight past
  // the debounce interval instead of waiting it out for real.
  await page.clock.install()

  await page.goto('/')

  await openFileList(page)
  const autoTab = page.locator('.file-tab-name', { hasText: 'auto.jianpu' })
  await autoTab.waitFor({ timeout: 15_000 })
  await autoTab.click()
  await expect(fileSwitcherTrigger(page)).toContainText('auto.jianpu')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await typeAtEditorEnd(page, ' 5')

  // Right after the edit, the debounce hasn't fired yet: no PUT sent, and
  // the save-status badge is still absent (idle renders nothing — see
  // `FileList.tsx`'s `SaveStatusBadge`).
  expect(putBodies).toHaveLength(0)
  await expect(page.getByTestId('save-status-badge')).toHaveCount(0)

  await page.clock.fastForward(AUTOSAVE_DEBOUNCE_MS)

  await expect
    .poll(() => putBodies.find((body) => body.path === 'scores/auto.jianpu'))
    .toMatchObject({ content: expect.stringContaining('1 2 3 4 5') })
  await expect(page.getByTestId('save-status-badge')).toHaveText('Saved')

  // Reloading re-fetches from the (mocked) GitHub API, so the edit surviving
  // a reload proves the autosave actually landed in the fake remote, not
  // just in in-memory React state.
  await page.reload()
  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: 'auto.jianpu' }).waitFor({
    timeout: 15_000,
  })
  await page.locator('.file-tab-name', { hasText: 'auto.jianpu' }).click()
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    '1 2 3 4 5',
  )
})
