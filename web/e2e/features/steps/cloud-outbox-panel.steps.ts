import { type Download, expect, type Page } from '@playwright/test'
import { saveStatusBadge } from '../../cloudOutboxHelpers'
import { Then, When } from './fixtures'

const syncPanel = (page: Page) => page.getByTestId('sync-panel')

/** The lane `<li>` showing `fileName` (the file name is its first span). */
const laneFor = (page: Page, fileName: string) =>
  syncPanel(page)
    .locator('li.sync-panel__lane')
    .filter({
      has: page.locator('.sync-panel__file-name', { hasText: fileName }),
    })

// Set by "I click ... in the sync panel" so a later Then can inspect the
// download that click triggered.
let pendingDownload: Promise<Download> | undefined

When('I open the sync panel', async ({ page }) => {
  await saveStatusBadge(page).click()
  await expect(syncPanel(page)).toBeVisible()
})

When('I click {string} in the sync panel', async ({ page }, label: string) => {
  pendingDownload = undefined
  if (label === 'Download my copy') {
    pendingDownload = page.waitForEvent('download')
  }
  await syncPanel(page).getByRole('button', { name: label }).click()
})

When('I confirm {string}', async ({ page }, label: string) => {
  await page
    .getByRole('alertdialog')
    .getByRole('button', { name: label, exact: true })
    .click()
})

When(
  'I expand the details for {string}',
  async ({ page }, fileName: string) => {
    await laneFor(page, fileName).locator('summary').click()
  },
)

Then(
  'the sync panel lists {string} as {string}',
  async ({ page }, fileName: string, status: string) => {
    await expect(
      laneFor(page, fileName).locator('.sync-panel__pill'),
    ).toHaveText(new RegExp(`^${status}$`, 'i'), { timeout: 15_000 })
  },
)

Then(
  'a file named {string} containing {string} is downloaded',
  async ({}, fileName: string, content: string) => {
    expect(pendingDownload).toBeDefined()
    const download = await (pendingDownload as Promise<Download>)
    expect(download.suggestedFilename()).toBe(fileName)
    const fs = await import('node:fs')
    const path = await download.path()
    expect(fs.readFileSync(path, 'utf8')).toContain(content)
  },
)

Then('the editor does not contain {string}', async ({ page }, text: string) => {
  await expect(page.locator('.monaco-editor .view-lines')).not.toContainText(
    text,
  )
})

Then(
  'the details show {int} queued {string} message',
  async ({ page }, count: number, kind: string) => {
    const queued = syncPanel(page)
      .locator('details')
      .locator('h4', { hasText: 'Queued messages' })
      .locator('xpath=following-sibling::ul[1]/li')
    await expect(queued).toHaveCount(count)
    await expect(queued.first()).toContainText(kind)
  },
)

Then(
  'the details show an attempt with outcome {string}',
  async ({ page }, outcome: string) => {
    const attempts = syncPanel(page)
      .locator('details')
      .locator('h4', { hasText: 'Attempts' })
      .locator('xpath=following-sibling::ul[1]/li')
    await expect(attempts.first()).toContainText(outcome, { timeout: 15_000 })
  },
)
