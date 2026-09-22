import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import { installContentSaveTracking } from './cloud-content-save-tracking'
import { Given, Then, When } from './fixtures'

// Mirrors `useStorageBackend.ts`'s `AUTOSAVE_DEBOUNCE_MS`. Not imported
// directly -- that module transitively pulls in `fileStore.ts`'s Vite-only
// `?raw` import, which Playwright's test loader can't resolve.
const AUTOSAVE_DEBOUNCE_MS = 20_000

let dialogShown = false

Given(
  'I open and edit {string} with a fake clock installed',
  async ({ page }, name: string) => {
    installContentSaveTracking(page)

    // Install the fake clock before navigating, so the debounce timer only
    // elapses when a test explicitly fast-forwards it.
    await page.clock.install()

    await gotoCloudApp(page)

    await openFileList(page)
    const tab = fileTabByExactName(page, name)
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(name)
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
    // happened, so this is exactly the window `shouldWarnBeforeUnload`
    // should catch via `isPending` -- the badge reflects it as "Unsaved".
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
