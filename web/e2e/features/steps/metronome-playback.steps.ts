import { expect } from '@playwright/test'
import { Then, When } from './fixtures'

Then('the metronome toggle is visible', async ({ page }) => {
  await expect(page.getByTestId('metronome-toggle')).toBeVisible({
    timeout: 15_000,
  })
})

Then(
  'the metronome toggle becomes enabled once the soundfont loads',
  async ({ page }) => {
    await expect(page.getByTestId('metronome-toggle')).toBeEnabled({
      timeout: 30_000,
    })
  },
)

Then('the metronome toggle is off', async ({ page }) => {
  await expect(page.getByTestId('metronome-toggle')).toHaveAttribute(
    'aria-pressed',
    'false',
  )
})

Then('the metronome toggle is on', async ({ page }) => {
  await expect(page.getByTestId('metronome-toggle')).toHaveAttribute(
    'aria-pressed',
    'true',
  )
})

Then('the metronome toggle is disabled', async ({ page }) => {
  await expect(page.getByTestId('metronome-toggle')).toBeDisabled()
})

Then('the metronome toggle is enabled again', async ({ page }) => {
  await expect(page.getByTestId('metronome-toggle')).toBeEnabled()
})

When('I click the metronome toggle', async ({ page }) => {
  await page.getByTestId('metronome-toggle').click()
})
