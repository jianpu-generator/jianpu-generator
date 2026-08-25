import { expect } from '@playwright/test'
import { Then, When } from './fixtures'

When('I jump to line {int} via Ctrl+g', async ({ page }, line: number) => {
  await page.keyboard.press('Control+g')
  await page.keyboard.type(String(line))
  await page.keyboard.press('Enter')
})

Then('the play-measure button is disabled', async ({ page }) => {
  const playBtn = page.locator('.play-measure-btn')
  await expect(playBtn).toBeDisabled({ timeout: 5_000 })
})

When('I press Meta+Enter', async ({ page }) => {
  await page.keyboard.press('Meta+Enter')
  await page.waitForTimeout(500)
})

Then(
  'the play-measure button does not enter the playing state',
  async ({ page }) => {
    const playBtn = page.locator('.play-measure-btn')
    await expect(playBtn).not.toHaveClass(/play-measure-btn--playing/)
  },
)
