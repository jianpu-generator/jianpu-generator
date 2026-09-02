import { expect } from '@playwright/test'
import {
  focusEditor,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import {
  mockGithubContentsApi,
  mockGithubRepoExists,
  mockGithubUser,
  OWNER,
  REPO,
} from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

Given(
  'the GitHub device-flow OAuth endpoints are mocked with user code {string}',
  async ({ page }, userCode: string) => {
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
  },
)

Given(
  'the mocked GitHub user and repo exist for the mocked owner',
  async ({ page }) => {
    await mockGithubUser(page, OWNER)
    await mockGithubRepoExists(page, OWNER, REPO)
  },
)

Given('the mocked GitHub user exists for disconnect', async ({ page }) => {
  await mockGithubUser(page, OWNER)
})

Given(
  'the GitHub Contents API is mocked with no seeded files',
  async ({ page }) => {
    await mockGithubContentsApi(page, {})
  },
)

Given(
  'the GitHub Contents API is mocked with a seeded file {string} and PUT forbidden',
  async ({ page }, path: string) => {
    const onPut = () => {
      throw new Error('unexpected PUT to github after disconnect')
    }
    await mockGithubContentsApi(
      page,
      { [path]: '# metadata\ntitle = "Song"\n' },
      { onPut },
    )
  },
)

When('the app loads for the OAuth connect flow', async ({ page }) => {
  await page.goto('/')
})

When(
  'the app loads the GitHub-backed file list for disconnect',
  async ({ page }) => {
    await page.goto('/')
    await openFileList(page)
    const tab = page.locator('.file-tab-name', { hasText: 'song' })
    await tab.waitFor({ timeout: 15_000 })
  },
)

When(
  'I select the {string} tab before disconnecting',
  async ({ page }, name: string) => {
    const tab = page.locator('.file-tab-name', { hasText: name })
    await tab.click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
  },
)

When('I open the storage settings modal for OAuth', async ({ page }) => {
  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Storage…' }).click()
  await page.getByTestId('storage-settings-modal').waitFor()
})

When(
  'I select the {string} storage option',
  async ({ page }, label: string) => {
    await page.getByRole('button', { name: label }).click()
    await expect(page.getByTestId('github-connect')).toBeVisible()
  },
)

When(
  'I click the GitHub OAuth {string} button',
  async ({ page }, buttonName: string) => {
    await page.getByRole('button', { name: buttonName }).click()
  },
)

Then(
  'the device verification UI shows the code {string}',
  async ({ page }, userCode: string) => {
    await expect(page.getByTestId('device-verification')).toBeVisible()
    await expect(page.getByTestId('device-verification')).toContainText(
      userCode,
    )
  },
)

Then('the app shows connected as the mocked owner', async ({ page }) => {
  await expect(page.getByTestId('github-connected')).toBeVisible({
    timeout: 10_000,
  })
  await expect(page.getByTestId('github-connected')).toContainText(
    `Connected as @${OWNER}`,
  )
})

Then(
  'the stored storage-backend preference is set to github for the mocked owner',
  async ({ page }) => {
    const preference = await page.evaluate(() =>
      localStorage.getItem('jianpu:storage-backend:v1'),
    )
    expect(JSON.parse(preference ?? '{}')).toEqual({
      backend: 'github',
      github: { owner: OWNER },
    })
  },
)

Then('the stored github-auth is cleared', async ({ page }) => {
  const storedAuth = await page.evaluate(() =>
    localStorage.getItem('jianpu:github-auth:v1'),
  )
  expect(storedAuth).toBeNull()
})

Then('the app no longer shows as connected', async ({ page }) => {
  await expect(page.getByTestId('github-connected')).toHaveCount(0)
})

Then(
  'the {string} storage option is checked',
  async ({ page }, label: string) => {
    await expect(page.getByRole('button', { name: label })).toHaveAttribute(
      'aria-pressed',
      'true',
    )
  },
)

When(
  'I close the storage settings modal and attempt to edit and force-save',
  async ({ page }) => {
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
  },
)

Then('no PUT to GitHub occurs after disconnecting', async () => {
  // Give any (incorrect) autosave to GitHub a chance to fire; the `onPut`
  // spy above throws if it does.
  await delay(500)
})
