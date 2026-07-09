import { expect, test } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileActions,
  openFileList,
  typeAtEditorEnd,
} from './fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from './github-contents-mock'

const SOURCE = [
  '# metadata',
  'title = "Conflict Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

const REMOTE_SOURCE = [
  '# metadata',
  'title = "Conflict Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '5 6 7 1',
].join('\n')

async function setUpConflictingEdit(
  page: import('@playwright/test').Page,
  onPut?: (path: string, body: { content: string; sha?: string }) => void,
) {
  const controller = await mockGithubContentsApi(
    page,
    { 'scores/conflict.jianpu': SOURCE },
    { onPut },
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

  await openFileList(page)
  const tab = page.locator('.file-tab-name', { hasText: 'conflict.jianpu' })
  await tab.waitFor({ timeout: 15_000 })
  await tab.click()
  await expect(fileSwitcherTrigger(page)).toContainText('conflict.jianpu')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await typeAtEditorEnd(page, ' 5')

  // Force-save immediately rather than waiting out the debounce; the next
  // PUT this triggers is the one `failNextPutWith409` targets below.
  controller.failNextPutWith409('scores/conflict.jianpu')
  await page.keyboard.press('Meta+s')

  // Opening the modal after the failed save (rather than before) avoids its
  // overlay intercepting the editor click above.
  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Storage…' }).click()
  await page.getByTestId('storage-settings-modal').waitFor()

  await expect(page.getByTestId('conflict-banner')).toBeVisible({
    timeout: 10_000,
  })
  await expect(page.getByTestId('conflict-banner')).toContainText(
    'scores/conflict.jianpu',
  )

  return controller
}

test('overwriting mine re-pushes the in-memory edit and clears the conflict banner', async ({
  page,
}) => {
  const putBodies: string[] = []
  await setUpConflictingEdit(page, (_path, body) =>
    putBodies.push(Buffer.from(body.content, 'base64').toString('utf-8')),
  )

  await page.getByRole('button', { name: 'Overwrite mine' }).click()

  await expect(page.getByTestId('conflict-banner')).toHaveCount(0)
  await expect
    .poll(() => putBodies.at(-1))
    .toEqual(expect.stringContaining('1 2 3 4 5'))

  // The editor still shows the user's edit: overwrite-mine must not have
  // discarded it.
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    '1 2 3 4 5',
  )
})

test('discarding mine reloads the remote content and clears the conflict banner', async ({
  page,
}) => {
  const controller = await setUpConflictingEdit(page)

  // Simulate the change that raced the user's save actually landing on
  // GitHub, so "discard mine" has different remote content to pull in.
  controller.setRemoteContent('scores/conflict.jianpu', REMOTE_SOURCE)

  await page.getByRole('button', { name: 'Discard mine' }).click()

  await expect(page.getByTestId('conflict-banner')).toHaveCount(0)

  // The editor now shows the remote content, not the user's edit.
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    '5 6 7 1',
  )
  await expect(page.locator('.monaco-editor .view-lines')).not.toContainText(
    '1 2 3 4 5',
  )
})
