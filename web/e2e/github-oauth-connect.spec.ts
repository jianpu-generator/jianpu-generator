import { expect, test } from '@playwright/test'
import {
  focusEditor,
  openFileActions,
  openFileList,
} from './fileSwitcherHelpers'
import {
  mockGithubContentsApi,
  mockGithubRepoExists,
  mockGithubUser,
  OWNER,
  REPO,
} from './github-contents-mock'

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

test('connect via device flow shows the verification code and switches to the github backend', async ({
  page,
}) => {
  const userCode = 'ABCD-1234'

  await page.route('**/device/code', async (route) => {
    await route.fulfill({
      status: 200,
      json: {
        device_code: 'fake-device-code',
        user_code: userCode,
        verification_uri: 'https://github.com/login/device',
        expires_in: 900,
        interval: 5,
      },
    })
  })

  await page.route('**/oauth/token', async (route) => {
    // Delays the response so the device-verification UI is reliably
    // observable before the rest of `handleConnect`'s chain races ahead.
    await delay(250)
    await route.fulfill({
      status: 200,
      json: {
        access_token: 'fake-access-token',
        token_type: 'bearer',
        scope: 'repo',
      },
    })
  })

  await mockGithubUser(page, OWNER)
  await mockGithubRepoExists(page, OWNER, REPO)
  await mockGithubContentsApi(page, {})

  await page.goto('/')

  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Storage…' }).click()
  await page.getByTestId('storage-settings-modal').waitFor()

  await page.getByLabel('GitHub repository').check()
  await expect(page.getByTestId('github-connect')).toBeVisible()

  await page.getByRole('button', { name: 'Connect GitHub' }).click()

  await expect(page.getByTestId('device-verification')).toBeVisible()
  await expect(page.getByTestId('device-verification')).toContainText(userCode)

  await expect(page.getByTestId('github-connected')).toBeVisible({
    timeout: 10_000,
  })
  await expect(page.getByTestId('github-connected')).toContainText(
    `Connected as @${OWNER}`,
  )

  const preference = await page.evaluate(() =>
    localStorage.getItem('jianpu:storage-backend:v1'),
  )
  expect(JSON.parse(preference ?? '{}')).toEqual({
    backend: 'github',
    github: { owner: OWNER },
  })
})

test('disconnect reverts to the local backend and stops saving to github', async ({
  page,
}) => {
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

  await mockGithubUser(page, OWNER)
  const onPut = () => {
    throw new Error('unexpected PUT to github after disconnect')
  }
  await mockGithubContentsApi(
    page,
    { 'scores/song.jianpu': '# metadata\ntitle = "Song"\n' },
    { onPut },
  )

  await page.goto('/')

  await openFileList(page)
  const tab = page.locator('.file-tab-name', { hasText: 'song' })
  await tab.waitFor({ timeout: 15_000 })
  await tab.click()
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })

  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Storage…' }).click()
  await page.getByTestId('storage-settings-modal').waitFor()

  await expect(page.getByTestId('github-connected')).toBeVisible()
  await expect(page.getByTestId('github-connected')).toContainText(`@${OWNER}`)

  await page.getByRole('button', { name: 'Disconnect' }).click()

  const storedAuth = await page.evaluate(() =>
    localStorage.getItem('jianpu:github-auth:v1'),
  )
  expect(storedAuth).toBeNull()

  await expect(page.getByTestId('github-connected')).toHaveCount(0)
  await expect(page.getByLabel('This browser')).toBeChecked()

  await page.keyboard.press('Escape')
  // Disconnecting switches the active file to the read-only reference/demo
  // file (`isReadOnlyFile` in `fileStore.ts`), so this attempted edit is a
  // no-op in the editor itself — `typeAtEditorEnd` isn't used here since its
  // landed-text verification would never succeed against a read-only model.
  // The assertion below only cares that force-saving afterwards doesn't hit
  // GitHub, regardless of whether the keystrokes changed anything.
  await focusEditor(page)
  await page.keyboard.press('Control+End')
  await page.keyboard.type(' edited')
  await page.keyboard.press('Meta+s')

  // Give any (incorrect) autosave to GitHub a chance to fire; the `onPut`
  // spy above throws if it does.
  await delay(500)
})
