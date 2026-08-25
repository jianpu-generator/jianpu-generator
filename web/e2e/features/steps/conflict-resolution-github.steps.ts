import { expect, type Page } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileActions,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import {
  type GithubContentsApiController,
  mockGithubContentsApi,
  OWNER,
} from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

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

let putBodies: string[] = []
let controller: GithubContentsApiController

async function setUpConflictingEdit(
  page: Page,
  onPut?: (path: string, body: { content: string; sha?: string }) => void,
): Promise<GithubContentsApiController> {
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
  const tab = page.locator('.file-tab-name', { hasText: 'conflict' })
  await tab.waitFor({ timeout: 15_000 })
  await tab.click()
  await expect(fileSwitcherTrigger(page)).toContainText('conflict')
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

  // The failed force-save left the "Saved" badge showing the conflict's
  // error status — resolving it below should update the badge, not leave it
  // stuck.
  await expect(page.getByTestId('save-status-badge')).toHaveText('Save failed')

  return controller
}

Given(
  'a GitHub save conflict is set up on {string} for overwrite-mine',
  async ({ page }, _path: string) => {
    putBodies = []
    controller = await setUpConflictingEdit(page, (_path, body) =>
      putBodies.push(Buffer.from(body.content, 'base64').toString('utf-8')),
    )
  },
)

Given(
  'a GitHub save conflict is set up on {string} for discard-mine',
  async ({ page }, _path: string) => {
    controller = await setUpConflictingEdit(page)
  },
)

Given(
  'the remote file has since changed to the conflicting content',
  async () => {
    // Simulate the change that raced the user's save actually landing on
    // GitHub, so "discard mine" has different remote content to pull in.
    controller.setRemoteContent('scores/conflict.jianpu', REMOTE_SOURCE)
  },
)

When(
  'I click the conflict-resolution button {string}',
  async ({ page }, buttonName: string) => {
    await page.getByRole('button', { name: buttonName }).click()
  },
)

Then('the conflict banner is gone', async ({ page }) => {
  await expect(page.getByTestId('conflict-banner')).toHaveCount(0)
})

Then(
  'the last PUT for the conflict contains {string}',
  async ({}, text: string) => {
    await expect
      .poll(() => putBodies.at(-1))
      .toEqual(expect.stringContaining(text))
  },
)

Then(
  'the conflict status badge shows exactly {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toHaveText(text)
  },
)

Then('the editor still contains {string}', async ({ page }, text: string) => {
  // The editor still shows the user's edit: overwrite-mine must not have
  // discarded it.
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
})

Then(
  'the editor now shows the remote content {string}',
  async ({ page }, text: string) => {
    // The editor now shows the remote content, not the user's edit.
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)

Then(
  'the editor no longer contains {string}',
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).not.toContainText(
      text,
    )
  },
)
