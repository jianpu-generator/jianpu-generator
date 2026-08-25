import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

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

let putBodies: { path: string; content: string }[] = []
let dialogShown = false

Given(
  'the GitHub repo is seeded with a file named {string} for a pending save',
  async ({ page }, path: string) => {
    await mockGithubContentsApi(page, { [path]: SOURCE })
  },
)

Given(
  'the GitHub repo is seeded with a file named {string} for a landed save',
  async ({ page }, path: string) => {
    putBodies = []
    await mockGithubContentsApi(
      page,
      { [path]: SOURCE },
      {
        onPut: (path, body) =>
          putBodies.push({
            path,
            content: Buffer.from(body.content, 'base64').toString('utf-8'),
          }),
      },
    )
  },
)

Given(
  'I open and edit {string} with a fake clock installed',
  async ({ page }, filename: string) => {
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

    await openFileList(page)
    const tab = page.locator('.file-tab-name', { hasText: filename })
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(filename)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })

    await typeAtEditorEnd(page, ' 5')
  },
)

Then(
  'the beforeunload status badge shows {string}',
  async ({ page }, text: string) => {
    // Right after the edit, the debounce hasn't fired yet: no save has
    // happened, so this is exactly the window `shouldWarnBeforeUnload` should
    // catch via `isPending` — the badge reflects it as "Unsaved".
    await expect(page.getByTestId('save-status-badge')).toContainText(text)
  },
)

Then(
  'the beforeunload status badge shows exactly {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toHaveText(text)
  },
)

When(
  'I fast-forward the clock so the beforeunload save lands',
  async ({ page }) => {
    await page.clock.fastForward(AUTOSAVE_DEBOUNCE_MS)
  },
)

Then(
  'the beforeunload-tested PUT lands for {string} containing {string}',
  async ({}, path: string, content: string) => {
    await expect
      .poll(() => putBodies.find((body) => body.path === path))
      .toMatchObject({ content: expect.stringContaining(content) })
  },
)

When('I close the page without letting the save land', async ({ page }) => {
  dialogShown = false
  page.once('dialog', (dialog) => {
    dialogShown = true
    void dialog.dismiss()
  })
  await page.close({ runBeforeUnload: true })
})

When('I close the page after the save has landed', async ({ page }) => {
  dialogShown = false
  page.once('dialog', (dialog) => {
    dialogShown = true
    void dialog.dismiss()
  })
  await page.close({ runBeforeUnload: true })
})

Then('a beforeunload dialog is shown', async () => {
  await expect.poll(() => dialogShown).toBe(true)
})

Then('no beforeunload dialog is shown', async () => {
  // No dialog is expected; give the (nonexistent) event a moment to fire
  // before asserting its absence. Plain timer, not `page.waitForTimeout`,
  // since `page` is already closed at this point.
  await new Promise((resolve) => setTimeout(resolve, 500))
  expect(dialogShown).toBe(false)
})
