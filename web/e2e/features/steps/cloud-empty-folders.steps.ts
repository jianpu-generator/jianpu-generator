import { expect } from '@playwright/test'
import { openBin, openFileActions } from '../../fileSwitcherHelpers'
import { Then, When } from './fixtures'

When('I create a new folder from the file actions menu', async ({ page }) => {
  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'New folder', exact: true }).click()
})

Then(
  'the file tree shows the folder {string}',
  async ({ page }, name: string) => {
    await expect(page.getByRole('button', { name, exact: true })).toBeVisible()
  },
)

// The Bin item only exists in the actions menu while something is binned, so
// its absence already means nothing (a placeholder included) is listed.
Then('the bin does not list {string}', async ({ page }, name: string) => {
  await openFileActions(page)
  if ((await page.locator('.file-tab-bar-bin-trigger').count()) === 0) return
  await openBin(page)
  await expect(
    page.locator('.file-tab-bar-bin-name', {
      hasText: new RegExp(`^${name.replace('.', '\\.')}`),
    }),
  ).toHaveCount(0)
})
