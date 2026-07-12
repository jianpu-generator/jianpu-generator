import { expect, test } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileList,
  typeAtEditorEnd,
} from './fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

const SOURCE = [
  '# metadata',
  'title = "Cmd+S Force Save Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

test('Cmd/Ctrl+S force-flushes a pending debounced GitHub save immediately', async ({
  page,
}) => {
  const putBodies: { path: string; content: string }[] = []
  await mockGithubContentsApi(
    page,
    { 'scores/save.jianpu': SOURCE },
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
  // the test relies on Cmd/Ctrl+S itself forcing the flush, not the
  // debounce interval happening to run out.
  await page.clock.install()

  await page.goto('/')

  await openFileList(page)
  const saveTab = page.locator('.file-tab-name', { hasText: 'save.jianpu' })
  await saveTab.waitFor({ timeout: 15_000 })
  await saveTab.click()
  await expect(fileSwitcherTrigger(page)).toContainText('save.jianpu')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await typeAtEditorEnd(page, ' 5')

  // Right after the edit, the debounce hasn't fired yet: no PUT sent, and
  // the badge shows the pending "Unsaved" countdown.
  expect(putBodies).toHaveLength(0)
  await expect(page.getByTestId('save-status-badge')).toContainText('Unsaved')

  // No `page.clock.fastForward` call anywhere in this test: the PUT firing
  // here proves the shortcut itself forced the flush. (Other specs, e.g.
  // `conflict-resolution-github.spec.ts`, use the same `Meta+s` chord.)
  await page.keyboard.press('Meta+s')

  await expect
    .poll(() => putBodies.find((body) => body.path === 'scores/save.jianpu'))
    .toMatchObject({ content: expect.stringContaining('1 2 3 4 5') })

  // Reloading re-fetches from the (mocked) GitHub API, so the edit surviving
  // a reload proves the flush actually landed in the fake remote, not just
  // in-memory React state.
  await page.reload()
  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: 'save.jianpu' }).waitFor({
    timeout: 15_000,
  })
  await page.locator('.file-tab-name', { hasText: 'save.jianpu' }).click()
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    '1 2 3 4 5',
  )
})
