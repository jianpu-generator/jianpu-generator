import { expect, type Page } from '@playwright/test'
import { Then } from './fixtures'

function previewTexts(page: Page) {
  return page.evaluate(() =>
    Array.from(
      document.querySelectorAll('.preview-page svg text'),
      (text) => text.textContent?.trim() ?? '',
    ),
  )
}

Then('the preview shows the text {string}', async ({ page }, text: string) => {
  await expect.poll(() => previewTexts(page)).toContain(text)
})

Then(
  'the preview does not show the text {string}',
  async ({ page }, text: string) => {
    await expect.poll(() => previewTexts(page)).not.toContain(text)
  },
)
