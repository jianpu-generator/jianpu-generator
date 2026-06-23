import { expect, test } from '@playwright/test'

/**
 * The default demo source (Twinkle Twinkle Little Star) has the following
 * Monaco line numbers (1-based):
 *
 *   1  # metadata
 *   ...
 *  10  # score
 *  11  (time=4/4 key=C4 bpm=120)
 *  12  1 - - -        ← chord line → measure 1
 *  13  1 1 5 5        ← melody note line → measure 1
 *  14  twin- kle ...  ← lyric line → measure 1
 */
test('renders amber highlight rect when cursor is inside a measure', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  await page.click('.monaco-editor .view-lines')

  // Navigate to line 12 (first note line of measure 1).
  await page.keyboard.press('Control+g')
  await page.keyboard.type('12')
  await page.keyboard.press('Enter')

  // Allow the 300 ms debounce plus the highlight render worker round-trip.
  await page.waitForTimeout(1_000)

  // The highlighted SVG should contain a <rect> with data-testid="measure-highlight".
  const highlightRect = page.locator(
    '.preview-page [data-testid="measure-highlight"]',
  )
  await expect(highlightRect).toBeVisible({ timeout: 5_000 })
})

test('removes highlight rect when cursor moves outside all measures', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  await page.click('.monaco-editor .view-lines')

  // First put cursor inside a measure so the highlight appears.
  await page.keyboard.press('Control+g')
  await page.keyboard.type('12')
  await page.keyboard.press('Enter')
  await page.waitForTimeout(1_000)

  const highlightRect = page.locator(
    '.preview-page [data-testid="measure-highlight"]',
  )
  await expect(highlightRect).toBeVisible({ timeout: 5_000 })

  // Move to line 1 (# metadata section) — outside any measure span.
  await page.keyboard.press('Control+g')
  await page.keyboard.type('1')
  await page.keyboard.press('Enter')
  await page.waitForTimeout(1_000)

  // Highlight should be gone; the plain (non-highlighted) SVGs are shown.
  await expect(highlightRect).not.toBeVisible({ timeout: 3_000 })
})
