import { expect, type Page } from '@playwright/test'
import { Then, When } from './fixtures'

function selectionBar(page: Page) {
  return page.getByTestId('file-selection-bar')
}

function dialog(page: Page, name: string) {
  return page.getByRole('dialog', { name })
}

/** The name label of a user file row; clicking it must tick the row. */
function fileLabel(page: Page, name: string) {
  return page.locator('li.file-tab .file-tab-name', {
    hasText: new RegExp(`^${name}$`),
  })
}

function folderCheckbox(page: Page, name: string) {
  return page.getByRole('checkbox', { name: `Select folder ${name}` })
}

When('I start selecting files', async ({ page }) => {
  await page
    .locator('.file-browser-toolbar')
    .getByRole('button', { name: 'Select', exact: true })
    .click()
})

// The click lands on the label's position; the checkbox's row-wide overlay is
// what receives it, so Playwright's "intercepts pointer events" check is
// skipped on purpose.
When('I tick the file {string}', async ({ page }, name: string) => {
  await fileLabel(page, name).click({ force: true })
})

When('I tick the folder {string}', async ({ page }, name: string) => {
  await page
    .locator('.file-tree-name', { hasText: new RegExp(`^${name}$`) })
    .click({ force: true })
})

When(
  'I click {string} in the selection bar',
  async ({ page }, name: string) => {
    await selectionBar(page).getByRole('button', { name, exact: true }).click()
  },
)

Then('the file browser has no selection bar', async ({ page }) => {
  await expect(page.getByTestId('file-browser')).toBeVisible()
  await expect(selectionBar(page)).toHaveCount(0)
})

Then('the selection bar says {string}', async ({ page }, text: string) => {
  await expect(
    selectionBar(page).locator('.file-browser-selection-count'),
  ).toHaveText(text)
})

Then('the file {string} is ticked', async ({ page }, name: string) => {
  await expect(
    page.getByRole('checkbox', { name: `Select ${name}`, exact: true }),
  ).toBeChecked()
})

Then('the file {string} is not ticked', async ({ page }, name: string) => {
  await expect(
    page.getByRole('checkbox', { name: `Select ${name}`, exact: true }),
  ).not.toBeChecked()
})

Then(
  'the checkbox of the folder {string} is {string}',
  async ({ page }, name: string, state: string) => {
    const aria = { checked: 'true', unchecked: 'false', mixed: 'mixed' }[state]
    await expect(folderCheckbox(page, name)).toHaveAttribute(
      'aria-checked',
      aria ?? state,
    )
  },
)

Then(
  'the selection action {string} is enabled',
  async ({ page }, name: string) => {
    await expect(
      selectionBar(page).getByRole('button', { name, exact: true }),
    ).toBeEnabled()
  },
)

Then(
  'the selection action {string} is disabled',
  async ({ page }, name: string) => {
    await expect(
      selectionBar(page).getByRole('button', { name, exact: true }),
    ).toBeDisabled()
  },
)

Then(
  'the delete selection dialog lists {string} and {string}',
  async ({ page }, first: string, second: string) => {
    await expect(
      dialog(page, 'Delete selected?').getByRole('listitem'),
    ).toHaveText([first, second])
  },
)

When('I cancel the delete selection dialog', async ({ page }) => {
  await dialog(page, 'Delete selected?')
    .getByRole('button', { name: 'Cancel' })
    .click()
})

When('I confirm the delete selection dialog', async ({ page }) => {
  await dialog(page, 'Delete selected?')
    .getByRole('button', { name: 'Move to bin' })
    .click()
})
