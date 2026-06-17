import { expect, test } from '@playwright/test'

/**
 * The default demo (Twinkle Twinkle Little Star) declares two parts:
 *
 *   Chord  = chord          (1 data line per measure)
 *   Melody = notes lyrics   (2 data lines per measure)
 *
 * After WASM initialises and returns score line hints from the pre-desugar
 * score tree, the editor should display inlay hints at each physical data line:
 *
 *   Line 11: (time=4/4 key=C4 bpm=120)   ← directive, no hint
 *   Line 12: [Chord]  1 - - -            ← Chord data line
 *   Line 13: [Melody] 1 1 5 5            ← Melody notes line
 *   Line 14: [Melody] twin- kle ...      ← Melody lyrics line
 */
test('shows part inlay hints on each data line within a measure', async ({
  page,
}) => {
  await page.goto('/')

  // Wait for WASM to initialise (editor toolbar is only shown once WASM is ready).
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Allow the 300 ms debounce for listScoreLineHints, plus Monaco rendering time.
  await page.waitForTimeout(1_000)

  const viewLines = page.locator('.monaco-editor .view-lines')

  await expect(
    viewLines.getByText('[Chord]', { exact: false }).first(),
  ).toBeVisible({
    timeout: 5_000,
  })
  await expect(
    viewLines.getByText('[Melody]', { exact: false }).first(),
  ).toBeVisible({
    timeout: 5_000,
  })
})

test('shows part inlay hint on lyrics line', async ({ page }) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })
  await page.waitForTimeout(1_000)

  // Line 14 is the Melody lyrics line in the default demo.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('14')
  await page.keyboard.press('Enter')
  await page.waitForTimeout(300)

  const lyricsLine = page
    .locator('.monaco-editor .view-line')
    .filter({ hasText: 'twin' })
  await expect(lyricsLine.getByText('[Melody]', { exact: false })).toBeVisible({
    timeout: 3_000,
  })
})

test('does not show a hint on the directive line of a measure', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })
  await page.waitForTimeout(1_000)

  // Line 11 is "(time=4/4 key=C4 bpm=120)" — a directive, not a data line.
  // The view-line for that line should not contain any inlay hint bracket text.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('11')
  await page.keyboard.press('Enter')

  // Give the editor a moment to settle; then verify "[Chord]" is not on line 11.
  // We check that the focused line does not contain the hint text.
  const _cursorLine = page.locator(
    '.monaco-editor .view-line.selected-text, .monaco-editor .current-line',
  )

  // The only way to verify "not on the directive line" is to check that the
  // directive line text (time=4/4…) does not co-exist with "[Chord]" in the
  // same view-line element.
  const directiveLine = page
    .locator('.monaco-editor .view-lines')
    .getByText('time=4/4', { exact: false })
  await expect(directiveLine).toBeVisible({ timeout: 3_000 })

  // The view-line that contains "time=4/4" should not also contain "[Chord]".
  const directiveLineWithHint = page
    .locator('.monaco-editor .view-line')
    .filter({
      hasText: 'time=4/4',
    })
    .filter({ hasText: '[Chord]' })
  await expect(directiveLineWithHint).toHaveCount(0)
})

test('shows hints for every measure, not just the first', async ({ page }) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })
  await page.waitForTimeout(1_000)

  // Scroll to the middle of the score so a different measure is in view.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  // Measure 5 starts around line 28 (4 measures × 4 lines each + overhead).
  await page.keyboard.type('28')
  await page.keyboard.press('Enter')

  await page.waitForTimeout(300)

  // Hints should still be visible for the measures now in the viewport.
  const viewLines = page.locator('.monaco-editor .view-lines')
  await expect(
    viewLines.getByText('[Chord]', { exact: false }).first(),
  ).toBeVisible({
    timeout: 3_000,
  })
  await expect(
    viewLines.getByText('[Melody]', { exact: false }).first(),
  ).toBeVisible({
    timeout: 3_000,
  })
})
