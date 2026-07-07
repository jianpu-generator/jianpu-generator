import { expect, test } from '@playwright/test'

test('Meta+Enter does nothing when cursor is outside all measures', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('1')
  await page.keyboard.press('Enter')

  const playBtn = page.locator('.play-measure-btn')
  await expect(playBtn).toBeDisabled({ timeout: 5_000 })

  await page.keyboard.press('Meta+Enter')
  await page.waitForTimeout(500)

  await expect(playBtn).not.toHaveClass(/play-measure-btn--playing/)
})
