import { expect } from '@playwright/test'
import { CLOUD_WORKER_ORIGIN } from '../../cloudFileHelpers'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import { Given, Then, When } from './fixtures'

let activeTabBeforeCreate: string | null = null
const newButton = ({ page }: { page: import('@playwright/test').Page }) =>
  page.locator('.export-menu-item').first()

Given(
  'the first file-create request will fail with a 500 error',
  async ({ page }) => {
    // Registered before navigating: fails the first `POST /files` (create)
    // with a 500, then `route.continue()`s every request after (including a
    // retried create), so the real worker behaves normally once this
    // one-shot failure has fired. The body content here is never surfaced --
    // unlike the deleted GitHub backend (whose Octokit error carried GitHub's
    // JSON body's own `message` straight through), `cloudBackend.ts`'s
    // `HttpStatusError` always reports a generic
    // "cloud storage request failed with status ${httpStatus}" message
    // regardless of the response body.
    let createCount = 0
    await page.route(`${CLOUD_WORKER_ORIGIN}/files`, async (route) => {
      createCount += 1
      if (createCount === 1) {
        await route.fulfill({ status: 500, body: 'Internal Server Error' })
        return
      }
      await route.continue()
    })
  },
)

When(
  'the app loads the cloud-backed file list for a failing create',
  async ({ page }) => {
    await gotoCloudApp(page)
    await openFileList(page)
    const existingTab = fileTabByExactName(page, 'existing')
    await existingTab.waitFor({ timeout: 15_000 })
  },
)

Given('I remember the currently active tab name', async ({ page }) => {
  activeTabBeforeCreate = await fileSwitcherTrigger(page).textContent()
})

When(
  'I click the {string} button to create a file that will fail',
  async ({ page }, label: string) => {
    expect(label).toBe('New')
    await openFileActions(page)
    await newButton({ page }).click()
  },
)

Then(
  'the error modal is shown with message {string} containing {string}',
  async ({ page }, title: string, detail: string) => {
    const errorModal = page.getByTestId('error-modal')
    await expect(errorModal).toBeVisible()
    await expect(errorModal).toContainText(title)
    await expect(page.getByTestId('error-modal-message')).toContainText(detail)
  },
)

When('I close the error modal', async ({ page }) => {
  // Close the error modal before interacting with anything underneath it --
  // it's a real overlay that blocks pointer events on the rest of the page.
  await page.getByTestId('error-modal').getByRole('button').click()
  await expect(page.getByTestId('error-modal')).toHaveCount(0)
})

Then(
  'the new-file button spinner clears and its label resets to {string}',
  async ({ page }, label: string) => {
    // `finally`'s `setPending(false)` must run on the error path too, not
    // just on success -- otherwise the "New" button would be stuck
    // spinning. The dropdown closes once `onCreate` settles (success or
    // failure), so reopen it to observe the reset state.
    await openFileActions(page)
    await expect(
      newButton({ page }).locator('.file-tab-bar-spinner'),
    ).toHaveCount(0)
    await expect(newButton({ page })).toHaveText(label)
  },
)

Then('no {string} tab exists', async ({ page }, name: string) => {
  // `setStore` is never called on failure, so no phantom file/tab appears
  // and the active tab is unchanged.
  await openFileList(page)
  await expect(fileTabByExactName(page, name)).toHaveCount(0)
})

Then(
  'the active tab is unchanged from before the failed create',
  async ({ page }) => {
    await expect(fileSwitcherTrigger(page)).toHaveText(
      activeTabBeforeCreate ?? '',
    )
  },
)

When(
  'I retry the {string} button in the file actions menu',
  async ({ page }, label: string) => {
    expect(label).toBe('New')
    // The one-shot 500 route has already fired and now falls through to
    // the real worker, so retrying "New" should succeed normally --
    // proving the user can actually recover from the failure.
    await openFileActions(page)
    await newButton({ page }).click()
  },
)

Then(
  'the retried create succeeds and the active tab becomes {string}',
  async ({ page }, name: string) => {
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)
