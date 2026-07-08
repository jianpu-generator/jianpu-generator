import { expect, test } from '@playwright/test'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

// Mirrors `useStorageBackend.ts`'s `AUTOSAVE_DEBOUNCE_MS`. Not imported
// directly — that module transitively pulls in `fileStore.ts`'s Vite-only
// `?raw` import, which Playwright's test loader can't resolve.
const AUTOSAVE_DEBOUNCE_MS = 20_000

const SOURCE = [
  '# metadata',
  'title = "Beforeunload Warning Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

async function openEditedFile(
  page: import('@playwright/test').Page,
  filename: string,
): Promise<void> {
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

  // Install the fake clock before navigating, so the debounce timer only
  // elapses when a test explicitly fast-forwards it.
  await page.clock.install()

  await page.goto('/')

  const tab = page.locator('.file-tab-name', { hasText: filename })
  await tab.waitFor({ timeout: 15_000 })
  await tab.click()
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    filename,
  )
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+End')
  await page.keyboard.type(' 5')
}

test('closing the tab warns while a GitHub save is still pending', async ({
  page,
}) => {
  await mockGithubContentsApi(page, { 'scores/pending.jianpu': SOURCE })
  await openEditedFile(page, 'pending.jianpu')

  // Right after the edit, the debounce hasn't fired yet: no save has
  // happened, so this is exactly the window `shouldWarnBeforeUnload` should
  // catch via `isPending`.
  await expect(page.getByTestId('save-status-badge')).toHaveCount(0)

  let dialogShown = false
  page.once('dialog', (dialog) => {
    dialogShown = true
    void dialog.dismiss()
  })
  await page.close({ runBeforeUnload: true })

  await expect.poll(() => dialogShown).toBe(true)
})

test('closing the tab does not warn once the GitHub save has landed', async ({
  page,
}) => {
  const putBodies: { path: string; content: string }[] = []
  await mockGithubContentsApi(
    page,
    { 'scores/saved.jianpu': SOURCE },
    {
      onPut: (path, body) =>
        putBodies.push({
          path,
          content: Buffer.from(body.content, 'base64').toString('utf-8'),
        }),
    },
  )
  await openEditedFile(page, 'saved.jianpu')

  await page.clock.fastForward(AUTOSAVE_DEBOUNCE_MS)
  await expect
    .poll(() => putBodies.find((body) => body.path === 'scores/saved.jianpu'))
    .toMatchObject({ content: expect.stringContaining('1 2 3 4 5') })
  await expect(page.getByTestId('save-status-badge')).toHaveText('Saved')

  let dialogShown = false
  page.once('dialog', (dialog) => {
    dialogShown = true
    void dialog.dismiss()
  })
  await page.close({ runBeforeUnload: true })

  // No dialog is expected; give the (nonexistent) event a moment to fire
  // before asserting its absence. Plain timer, not `page.waitForTimeout`,
  // since `page` is already closed at this point.
  await new Promise((resolve) => setTimeout(resolve, 500))
  expect(dialogShown).toBe(false)
})
