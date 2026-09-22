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
  'a fake clock is installed to prevent an autosave race with the tab switch',
  async ({ page }) => {
    // Install the fake clock so the debounce timer never elapses on its
    // own -- the test relies on the tab switch itself forcing the flush,
    // not the debounce interval happening to run out.
    await page.clock.install()
  },
)

When(
  'the app loads the cloud-backed file list for a tab-switch save',
  async ({ page }) => {
    installContentSaveTracking(page)
    await gotoCloudApp(page)
    await openFileList(page)
  },
)

When(
  'I select the {string} tab to test the tab switch',
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
  'I append {string} to the editor to trigger a tab-switch save',
  async ({ page }, text: string) => {
    await typeAtEditorEnd(page, text)
  },
)

Then('no content request has been sent yet before the tab switch', async () => {
  expect(currentContentSaveTracker().records).toHaveLength(0)
})

Then(
  'the tab-switch status badge shows {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toContainText(text)
  },
)

When(
  'I switch to the {string} tab from the file list',
  async ({ page }, name: string) => {
    await openFileList(page)
    await fileTabByExactName(page, name).click()
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then(
  'the reloaded editor still contains the tab-switch-saved edit {string}',
  async ({ page }, text: string) => {
    const tab = fileTabByExactName(page, 'a')
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)
