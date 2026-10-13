import { expect, type Page } from '@playwright/test'
import { Then, When } from './fixtures'

const history = (page: Page) => page.getByTestId('outbox-history')

const rowFor = (page: Page, fileName: string) =>
  history(page).locator('tr.outbox-history__row', { hasText: fileName })

When(
  'I search the outbox history for {string}',
  async ({ page }, query: string) => {
    await history(page).getByRole('searchbox').fill(query)
  },
)

When(
  'I filter the outbox history by status {string}',
  async ({ page }, status: string) => {
    await history(page).getByLabel('Status').selectOption({ label: status })
  },
)

When(
  'I expand {string} in the outbox history',
  async ({ page }, fileName: string) => {
    await rowFor(page, fileName).getByRole('button', { name: /Show/ }).click()
  },
)

Then(
  'the outbox history lists {string} as {string}',
  async ({ page }, fileName: string, status: string) => {
    await expect(
      rowFor(page, fileName).locator('.outbox-history__pill'),
    ).toHaveText(status, { timeout: 15_000 })
  },
)

Then(
  'the outbox history shows {string} highlighted in the payload of {string}',
  async ({ page }, text: string, fileName: string) => {
    await expect(rowFor(page, fileName)).toBeVisible()
    await expect(
      history(page).locator('.outbox-history__payload mark'),
    ).toHaveText(text)
  },
)

Then(
  'the outbox history row for {string} shows an attempt with outcome {string}',
  async ({ page }, fileName: string, outcome: string) => {
    const detail = rowFor(page, fileName).locator(
      'xpath=following-sibling::tr[1]',
    )
    await expect(detail).toContainText(outcome, { timeout: 15_000 })
  },
)
