import { expect, type Page } from '@playwright/test'
import {
  fileSwitcherTrigger,
  nameNewFile,
  openFileActions,
} from '../../fileSwitcherHelpers'
import { Then, When } from './fixtures'

function dialog(page: Page, name: string) {
  return page.getByRole('dialog', { name })
}

When(
  'I choose {string} from the file actions menu',
  async ({ page }, item: string) => {
    await openFileActions(page)
    await page.getByRole('menuitem', { name: item, exact: true }).click()
  },
)

When(
  'I click {string} in the file tree list',
  async ({ page }, label: string) => {
    await page
      .locator('.file-browser-toolbar')
      .getByRole('button', { name: label, exact: true })
      .click()
  },
)

When('I name the new file {string}', async ({ page }, text: string) => {
  await nameNewFile(page, text)
})

Then('the new file dialog is open', async ({ page }) => {
  await expect(dialog(page, 'New file')).toBeVisible()
})

When(
  'I type {string} in the new file dialog',
  async ({ page }, text: string) => {
    await dialog(page, 'New file').getByRole('textbox').fill(text)
  },
)

Then('the new file dialog cannot be confirmed', async ({ page }) => {
  await expect(
    dialog(page, 'New file').getByRole('button', { name: 'Create' }),
  ).toBeDisabled()
})

When('I open the file actions menu', async ({ page }) => {
  await openFileActions(page)
})

Then('the file action {string} is disabled', async ({ page }, item: string) => {
  await expect(
    page.getByRole('menuitem', { name: item, exact: true }),
  ).toBeDisabled()
})

Then(
  'the file tree shows the folder {string} nested {int} levels deep',
  async ({ page }, name: string, levels: number) => {
    // Top-level rows have depth 0, so "3 levels deep" is --tree-depth 2.
    await expect(
      page
        .locator('li.file-tree-row')
        .filter({ has: page.getByRole('button', { name, exact: true }) }),
    ).toHaveAttribute('style', new RegExp(`--tree-depth:\\s*${levels - 1}`))
  },
)

Then(
  'the move dialog offers the folders {string}, {string}, {string} and {string}',
  async ({ page }, a: string, b: string, c: string, d: string) => {
    const options = dialog(page, 'Move to').locator('.move-target-name')
    await expect(options).toHaveText([a, b, c, d])
  },
)

When(
  'I pick the folder {string} in the move dialog',
  async ({ page }, name: string) => {
    await dialog(page, 'Move to')
      .locator('.move-target', {
        has: page.locator('.move-target-name', {
          hasText: new RegExp(`^${name}$`),
        }),
      })
      .click()
  },
)

When('I confirm the move dialog', async ({ page }) => {
  await dialog(page, 'Move to').getByRole('button', { name: 'Move' }).click()
})

Then('the move dialog says {string}', async ({ page }, text: string) => {
  await expect(dialog(page, 'Move to').getByRole('alert')).toContainText(text)
})

Then('the move dialog shows no message', async ({ page }) => {
  await expect(dialog(page, 'Move to').getByRole('alert')).toHaveCount(0)
})

Then('the move dialog cannot be confirmed', async ({ page }) => {
  await expect(
    dialog(page, 'Move to').getByRole('button', { name: 'Move' }),
  ).toBeDisabled()
})

Then('the move dialog can be confirmed', async ({ page }) => {
  await expect(
    dialog(page, 'Move to').getByRole('button', { name: 'Move' }),
  ).toBeEnabled()
})

When(
  'I rename the active file in the list to {string}',
  async ({ page }, to: string) => {
    await page.locator('.file-tab--active .file-tab-name').dblclick()
    const input = page.locator('input.file-tab-name')
    await input.fill(to)
    await input.press('Enter')
    // The rename lands a beat after Enter; later steps must not race it.
    await expect(fileSwitcherTrigger(page)).toContainText(
      to.split('/').at(-1) ?? to,
    )
  },
)

When(
  "I type {string} in the active file's rename input in the list",
  async ({ page }, text: string) => {
    await page.locator('.file-tab--active .file-tab-name').dblclick()
    await page.locator('input.file-tab-name').fill(text)
  },
)

Then('the list rename input says {string}', async ({ page }, text: string) => {
  await expect(page.locator('.file-tab-rename-error')).toContainText(text)
})

Then('the rename dialog holds {string}', async ({ page }, text: string) => {
  await expect(dialog(page, 'Rename').getByRole('textbox')).toHaveValue(text)
})

When('I type {string} in the rename dialog', async ({ page }, text: string) => {
  await dialog(page, 'Rename').getByRole('textbox').fill(text)
})

When(
  'I type {string} in the rename dialog and confirm',
  async ({ page }, text: string) => {
    const rename = dialog(page, 'Rename')
    await rename.getByRole('textbox').fill(text)
    await rename.getByRole('button', { name: 'Rename' }).click()
  },
)

Then('the rename dialog says {string}', async ({ page }, text: string) => {
  await expect(dialog(page, 'Rename').getByRole('alert')).toContainText(text)
})

Then('the rename dialog shows no message', async ({ page }) => {
  await expect(dialog(page, 'Rename').getByRole('alert')).toHaveCount(0)
})

Then('the rename dialog cannot be confirmed', async ({ page }) => {
  await expect(
    dialog(page, 'Rename').getByRole('button', { name: 'Rename' }),
  ).toBeDisabled()
})

Then('the rename dialog can be confirmed', async ({ page }) => {
  await expect(
    dialog(page, 'Rename').getByRole('button', { name: 'Rename' }),
  ).toBeEnabled()
})

Then('the new folder dialog is open', async ({ page }) => {
  await expect(dialog(page, 'New folder')).toBeVisible()
})

When(
  'I type {string} in the new folder dialog',
  async ({ page }, text: string) => {
    await dialog(page, 'New folder').getByRole('textbox').fill(text)
  },
)

When('I name the new folder {string}', async ({ page }, text: string) => {
  const prompt = dialog(page, 'New folder')
  await prompt.getByRole('textbox').fill(text)
  await prompt.getByRole('button', { name: 'Create' }).click()
})

When('I cancel the new folder dialog', async ({ page }) => {
  await dialog(page, 'New folder')
    .getByRole('button', { name: 'Cancel' })
    .click()
})

Then('the new folder dialog cannot be confirmed', async ({ page }) => {
  await expect(
    dialog(page, 'New folder').getByRole('button', { name: 'Create' }),
  ).toBeDisabled()
})
