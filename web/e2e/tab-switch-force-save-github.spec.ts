import { expect, test } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileList,
  typeAtEditorEnd,
} from './fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

const SOURCE_A = [
  '# metadata',
  'title = "Tab Switch Test A"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

const SOURCE_B = [
  '# metadata',
  'title = "Tab Switch Test B"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '5 6 7 1',
].join('\n')

test('switching the active file tab force-flushes a pending debounced GitHub save', async ({
  page,
}) => {
  const putBodies: { path: string; content: string }[] = []
  await mockGithubContentsApi(
    page,
    { 'scores/a.jianpu': SOURCE_A, 'scores/b.jianpu': SOURCE_B },
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

  // Install the fake clock so the debounce timer never elapses on its own —
  // the test relies on the tab switch itself forcing the flush, not the
  // debounce interval happening to run out.
  await page.clock.install()

  await page.goto('/')

  await openFileList(page)
  const aTab = page.locator('.file-tab-name', { hasText: 'a.jianpu' })
  await aTab.waitFor({ timeout: 15_000 })
  await aTab.click()
  await expect(fileSwitcherTrigger(page)).toContainText('a.jianpu')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await typeAtEditorEnd(page, ' 5')

  // Right after the edit, the debounce hasn't fired yet: no PUT sent, and
  // the save-status badge shows the pending "Unsaved" countdown.
  expect(putBodies).toHaveLength(0)
  await expect(page.getByTestId('save-status-badge')).toContainText('Unsaved')

  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: 'b.jianpu' }).click()
  await expect(fileSwitcherTrigger(page)).toContainText('b.jianpu')

  // No `page.clock.fastForward` call anywhere in this test: the PUT firing
  // here proves the tab switch itself forced the flush.
  await expect
    .poll(() => putBodies.find((body) => body.path === 'scores/a.jianpu'))
    .toMatchObject({ content: expect.stringContaining('1 2 3 4 5') })

  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: 'a.jianpu' }).click()
  await expect(fileSwitcherTrigger(page)).toContainText('a.jianpu')

  // Reloading re-fetches from the (mocked) GitHub API, so the edit surviving
  // a reload proves the flush actually landed in the fake remote, not just
  // in-memory React state.
  await page.reload()
  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: 'a.jianpu' }).waitFor({
    timeout: 15_000,
  })
  await page.locator('.file-tab-name', { hasText: 'a.jianpu' }).click()
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    '1 2 3 4 5',
  )
})
