import { expect, test } from '@playwright/test'

/**
 * The default demo (Twinkle Twinkle Little Star) declares two parts:
 *
 *   # parts
 *   Chord = chords
 *   Melody = notes+lyrics
 *
 * Line 12 in the editor is the first chord line of measure 1.
 *
 * Regression: when the cursor is inside a measure, `highlightedSvgs` is shown
 * in the Preview. Toggling a part should re-render `highlightedSvgs` with the
 * new part filter, but the effect that fires the re-render only depended on
 * `selectedMeasureRange` — so parts changes were silently ignored.
 */
test('toggling a part rerenders the highlighted SVG while a measure is focused', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Focus the Monaco editor and navigate to the first measure.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('12')
  await page.keyboard.press('Enter')

  // Allow the debounce + highlight render worker round-trip.
  await page.waitForTimeout(1_000)

  // Confirm the measure highlight is visible (i.e. highlightedSvgs is in use).
  const highlightRect = page.locator(
    '.preview-page [data-testid="measure-highlight"]',
  )
  await expect(highlightRect).toBeVisible({ timeout: 5_000 })

  // Capture the rendered SVG content before any part toggle.
  const svgBefore = await page.locator('.preview-pages').innerHTML()

  // Uncheck the first part toggle (the "Chord" part).
  const firstPartCheckbox = page
    .locator('.part-toggles input[type="checkbox"]')
    .first()
  await firstPartCheckbox.uncheck()

  // Give the worker time to re-render with the updated parts filter.
  await page.waitForTimeout(1_500)

  // The SVG should have changed to reflect the disabled part.
  // When the bug is present the highlighted SVG is never re-requested after a
  // parts toggle, so innerHTML will be identical to svgBefore.
  const svgAfter = await page.locator('.preview-pages').innerHTML()
  expect(svgAfter).not.toBe(svgBefore)
})
