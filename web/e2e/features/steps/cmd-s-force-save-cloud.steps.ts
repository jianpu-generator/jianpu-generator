import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import {
  currentContentSaveTracker,
  installContentSaveTracking,
} from './cloud-content-save-tracking'
import { Given, Then, When } from './fixtures'

Given(
  'a fake clock is installed to prevent an autosave race with force save',
  async ({ page }) => {
    // Install the fake clock so the debounce timer never elapses on its
    // own -- the test relies on Cmd/Ctrl+S itself forcing the flush, not
    // the debounce interval happening to run out.
    await page.clock.install()
  },
)

When(
  'the app loads the cloud-backed file list for a force save',
  async ({ page }) => {
    installContentSaveTracking(page)
    await gotoCloudApp(page)
    await openFileList(page)
  },
)

When(
  'I select the {string} tab to test the force save',
  async ({ page }, name: string) => {
    const tab = fileTabByExactName(page, name)
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(name)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

When(
  'I append {string} to the editor to trigger a force save',
  async ({ page }, text: string) => {
    await typeAtEditorEnd(page, text)
  },
)

Then('no content request has been sent yet before the force save', async () => {
  // Right after the edit, the debounce hasn't fired yet: no content-save
  // request sent, and the badge shows the pending "Unsaved" countdown.
  expect(currentContentSaveTracker().records).toHaveLength(0)
})

Then(
  'the force-save status badge shows {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toContainText(text)
  },
)

When('I press Cmd\\/Ctrl+S', async ({ page }) => {
  // No `page.clock.fastForward` call anywhere in this test: the save
  // request firing here proves the shortcut itself forced the flush.
  await page.keyboard.press('Meta+s')
})

Then(
  'the reloaded editor still contains the force-saved edit {string}',
  async ({ page }, text: string) => {
    const tab = fileTabByExactName(page, 'save')
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)
