import { expect, type Page } from '@playwright/test'
import { openFileActions } from '../../fileSwitcherHelpers'
import {
  currentContentSaveTracker,
  lastContentSaveFor,
} from './cloud-content-save-tracking'
import { Then, When } from './fixtures'

const historyPanel = (page: Page) => page.getByTestId('file-history')

const DIFF_MARKS = { removed: '-', added: '+' } as const

const diffLine = (page: Page, kind: 'removed' | 'added', text: string) =>
  historyPanel(page)
    .locator(`.file-history__line--${kind}`)
    .getByText(`${DIFF_MARKS[kind]} ${text}`, { exact: true })

When('I open the History panel', async ({ page }) => {
  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'History' }).click()
  await expect(historyPanel(page)).toBeVisible()
})

Then(
  /^the History panel lists (\d+) earlier versions?$/,
  async ({ page }, count: string) => {
    await expect(
      historyPanel(page)
        .getByRole('list', { name: 'Versions' })
        .getByRole('button'),
    ).toHaveCount(Number(count))
  },
)

Then('the History diff removes {string}', async ({ page }, text: string) => {
  await expect(diffLine(page, 'removed', text)).toHaveCount(1)
})

Then('the History diff adds {string}', async ({ page }, text: string) => {
  await expect(diffLine(page, 'added', text)).toHaveCount(1)
})

When('I click Restore in the History panel', async ({ page }) => {
  await historyPanel(page).getByRole('button', { name: 'Restore' }).click()
  await expect(historyPanel(page)).toBeHidden()
})

Then(
  'the last content save for {string} no longer contains {string}',
  async ({}, name: string, text: string) => {
    await expect
      .poll(() => lastContentSaveFor(currentContentSaveTracker(), name), {
        timeout: 10_000,
      })
      .not.toContain(text)
  },
)
