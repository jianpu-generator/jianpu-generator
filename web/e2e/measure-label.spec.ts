import { expect, test } from '@playwright/test'

/**
 * The default demo source (Twinkle Twinkle Little Star) has the following
 * Monaco line numbers (1-based):
 *
 *   1  [metadata]
 *   2  title = "Twinkle Twinkle Little Star"
 *   ...
 *  10  [score]
 *  11  (time=4/4 key=C4 bpm=120)
 *  12  1 - - -        ← first note line → measure 1
 *  13  1 1 5 5        ← melody track note → measure 1
 *  14  twin- kle ...
 */
test('shows measure number when cursor is placed on a note line', async ({
  page,
}) => {
  await page.goto('/')

  // The PlayMeasureButton toolbar is only rendered once the WASM module reports
  // audioAvailable=true.  Wait up to 15 s for it to appear.
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Focus the Monaco editor.
  await page.click('.monaco-editor .view-lines')

  // Use Monaco's "Go to Line" command (Ctrl+G) to jump to line 12,
  // which is the first note line in the default Twinkle demo source.
  await page.keyboard.press('Control+g')
  await page.keyboard.type('12')
  await page.keyboard.press('Enter')

  // Allow the 300 ms debounce in notifySelection plus worker round-trip.
  await page.waitForTimeout(700)

  // The label should show "measure 1", not "measure null".
  await expect(page.getByText('measure 1')).toBeVisible({ timeout: 3_000 })
})

/**
 * Regression: when the cursor is positioned directly AFTER the last character
 * of a note line, the byte offset equals source_span.end and the measure must
 * still be detected.
 *
 * Line 12 of the default demo is "1 - - -" (chord line for measure 1).
 * Pressing End places the cursor after the trailing "-", which is the
 * exclusive end of the measure span.
 */
test('detects measure when cursor is at end of last character of a note line', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Focus the Monaco editor.
  await page.click('.monaco-editor .view-lines')

  // Navigate to line 12 ("1 - - -") and press End to put the cursor after
  // the trailing "-" — one byte past the last character of the measure span.
  await page.keyboard.press('Control+g')
  await page.keyboard.type('12')
  await page.keyboard.press('Enter')
  await page.keyboard.press('End')

  // Allow the 300 ms debounce in notifySelection plus worker round-trip.
  await page.waitForTimeout(700)

  // Should still detect measure 1, not "measure null".
  await expect(page.getByText('measure 1')).toBeVisible({ timeout: 3_000 })
})
