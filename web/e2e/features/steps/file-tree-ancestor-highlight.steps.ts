import { expect, type Page } from '@playwright/test'
import { Then, When } from './fixtures'

function treeRows(page: Page) {
  return page.locator('ul.file-tabs[aria-label="Files"] > li.file-tree-row')
}

When('I hover the file tree file {string}', async ({ page }, name: string) => {
  await page
    .locator('.file-tab-name', { hasText: new RegExp(`^${name}$`) })
    .hover()
})

When(
  'I hover the file tree folder {string}',
  async ({ page }, name: string) => {
    await treeRows(page).getByRole('button', { name, exact: true }).hover()
  },
)

When('I move the pointer off the file tree', async ({ page }) => {
  await page.mouse.move(0, 0)
})

Then(
  'the file tree highlights the ancestor folders {string}',
  async ({ page }, names: string) => {
    await expect(
      page.locator('li.file-tree-row--ancestor .file-tree-name'),
    ).toHaveText(names.split(', '))
  },
)

Then('the file tree highlights no ancestor folders', async ({ page }) => {
  await expect(page.locator('li.file-tree-row--ancestor')).toHaveCount(0)
})
