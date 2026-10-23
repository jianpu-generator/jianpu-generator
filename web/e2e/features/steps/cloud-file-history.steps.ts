import { expect, type Page } from '@playwright/test'
import { openFileActions } from '../../fileSwitcherHelpers'
import {
  currentContentSaveTracker,
  lastContentSaveFor,
} from './cloud-content-save-tracking'
import { Then, When } from './fixtures'

const historyPanel = (page: Page) => page.getByTestId('file-history')

/** The text on one side of the History diff: `original` is the chosen
 * version, `modified` the editor's current text. */
function diffSide(page: Page, side: 'original' | 'modified') {
  return page.evaluate((which) => {
    const container = document.querySelector(
      `[data-testid="file-history-diff"] .editor.${which}`,
    )
    const editor = window.monaco?.editor
      .getEditors()
      .find((candidate) => container?.contains(candidate.getContainerDomNode()))
    return editor?.getModel()?.getValue() ?? null
  }, side)
}

const diffEditor = (page: Page) =>
  historyPanel(page).locator('.monaco-diff-editor')

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

Then(
  'the History diff compares the version ending in {string} with the current text ending in {string}',
  async ({ page }, version: string, current: string) => {
    await expect
      .poll(() => diffSide(page, 'original'))
      .toMatch(new RegExp(`${version}\\n?$`))
    await expect
      .poll(() => diffSide(page, 'modified'))
      .toMatch(new RegExp(`${current}\\n?$`))
  },
)

Then('the History diff marks the added characters', async ({ page }) => {
  await expect(diffEditor(page).locator('.char-insert').first()).toBeVisible()
})

When('I switch the History diff to inline', async ({ page }) => {
  await page.getByTestId('file-history-layout-toggle').click()
})

Then('the History diff is inline', async ({ page }) => {
  // Monaco sets `side-by-side` on the diff editor root only in that layout.
  await expect(diffEditor(page)).not.toHaveClass(/\bside-by-side\b/)
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
