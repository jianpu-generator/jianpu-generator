import { expect } from '@playwright/test'
import type { DataTable } from 'playwright-bdd'
import { Then } from './fixtures'

// The pitch drawer's guitar chord boxes, one per chords-db voicing.

Then(
  'the pitch drawer shows a guitar diagram with frets {string}',
  async ({ page }, frets: string) => {
    await expect(
      page.getByTestId('pitch-drawer-guitar-diagram').first().locator('svg'),
    ).toHaveAttribute('data-guitar-frets', frets, { timeout: 10_000 })
  },
)

Then(
  'the pitch drawer shows guitar diagrams with frets:',
  async ({ page }, table: DataTable) => {
    const expected = table.hashes().map((row) => row.frets)
    const diagrams = page.getByTestId('pitch-drawer-guitar-diagram')
    await expect(diagrams).toHaveCount(expected.length, { timeout: 10_000 })
    expect(
      await diagrams
        .locator('svg')
        .evaluateAll((svgs) =>
          svgs.map((svg) => svg.getAttribute('data-guitar-frets')),
        ),
    ).toEqual(expected)
  },
)

Then('the pitch drawer shows no guitar diagram', async ({ page }) => {
  // Wait for the drawer's content to be in place first, so this doesn't
  // pass before the description arrives.
  await expect(page.getByTestId('pitch-drawer-letter-names')).toBeVisible({
    timeout: 10_000,
  })
  await expect(page.getByTestId('pitch-drawer-guitar-diagram')).toHaveCount(0)
})
