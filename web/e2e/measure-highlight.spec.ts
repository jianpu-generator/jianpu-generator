import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * The default demo source (demo/01-pitches.jianpu) has the following Monaco
 * line numbers (1-based):
 *
 *   1  # metadata
 *   ...
 *   8  # score
 *   9  label="Scale degrees & rest"
 *  10  [M] 1 2 3 0   ← melody note line → measure 1
 *
 * Line 1 (`# metadata`) sits outside any measure span.
 */
test('renders amber highlight rect when cursor is inside a measure', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })

  await focusEditor(page)

  // Navigate to line 10 (first note line of measure 1).
  await page.keyboard.press('Control+g')
  await page.keyboard.type('10')
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
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })

  await focusEditor(page)

  // First put cursor inside a measure so the highlight appears.
  await page.keyboard.press('Control+g')
  await page.keyboard.type('10')
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
