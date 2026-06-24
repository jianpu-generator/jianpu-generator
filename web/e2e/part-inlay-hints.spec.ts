import { expect, test } from '@playwright/test'

/**
 * The default demo (reference.jianpu) declares two parts:
 *
 *   Melody [M] = notes lyrics   (2 data lines per measure)
 *   Chords [C] = chord          (1 data line per measure)
 *
 * After WASM initialises and returns score line hints from the pre-desugar
 * score tree, the editor should display inlay hints at each positional data line:
 *
 *   Line 17: label="Scale degrees 1-7 & rest 0"  ← directive, no hint
 *   Line 18: [M] 1 2 3 0                          ← Melody notes line
 *   Line 19: [M] do re mi                         ← Melody lyrics line
 *
 * Lines already prefixed with a key (e.g. "[C] 1 - - -") receive no hint.
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
    viewLines.getByText('[C]', { exact: false }).first(),
  ).toBeVisible({
    timeout: 5_000,
  })
  await expect(
    viewLines.getByText('[M]', { exact: false }).first(),
  ).toBeVisible({
    timeout: 5_000,
  })
})

test('shows part inlay hint on lyrics line', async ({ page }) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })
  await page.waitForTimeout(1_000)

  // Line 19 is the Melody lyrics line ("do re mi") in reference.jianpu.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('19')
  await page.keyboard.press('Enter')
  await page.waitForTimeout(300)

  const lyricsLine = page
    .locator('.monaco-editor .view-line')
    .filter({ hasText: 'do re mi' })
  await expect(lyricsLine.getByText('[M]', { exact: false })).toBeVisible({
    timeout: 3_000,
  })
})

test('does not show a hint on the directive line of a measure', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })
  await page.waitForTimeout(1_000)

  // Line 17 is 'label="Scale degrees 1-7 & rest 0"' — a directive, not a data line.
  // The view-line for that line should not contain any inlay hint bracket text.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('17')
  await page.keyboard.press('Enter')

  // Verify the directive line is visible.
  const directiveLine = page
    .locator('.monaco-editor .view-lines')
    .getByText('Scale degrees', { exact: false })
  await expect(directiveLine).toBeVisible({ timeout: 3_000 })

  // The view-line that contains "Scale degrees" should not also contain "[M]".
  const directiveLineWithHint = page
    .locator('.monaco-editor .view-line')
    .filter({ hasText: 'Scale degrees' })
    .filter({ hasText: '[M]' })
  await expect(directiveLineWithHint).toHaveCount(0)
})

test('shows hints for every measure, not just the first', async ({ page }) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })
  await page.waitForTimeout(1_000)

  // Scroll to a later measure so we verify hints are not restricted to measure 1.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  // Line 38 is the Melody notes line of the "Slur group" measure.
  await page.keyboard.type('38')
  await page.keyboard.press('Enter')

  await page.waitForTimeout(300)

  // Hints should still be visible for the measures now in the viewport.
  const viewLines = page.locator('.monaco-editor .view-lines')
  await expect(
    viewLines.getByText('[C]', { exact: false }).first(),
  ).toBeVisible({
    timeout: 3_000,
  })
  await expect(
    viewLines.getByText('[M]', { exact: false }).first(),
  ).toBeVisible({
    timeout: 3_000,
  })
})

test('keyed lines do not receive an inlay hint', async ({ page }) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })
  await page.waitForTimeout(1_000)

  // Line 100 is "[C] 1 - - -" in the "Explicit chord key" fixture measure.
  // Because it already carries the [C] key prefix, no inlay hint should be prepended.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('100')
  await page.keyboard.press('Enter')
  await page.waitForTimeout(300)

  // Find the view-line whose source text is the keyed chord line.
  const keyedChordLine = page
    .locator('.monaco-editor .view-line')
    .filter({ hasText: '[C] 1 - - -' })

  await expect(keyedChordLine).toHaveCount(1)

  // If an inlay hint were added, the rendered text would contain "[C]" twice
  // (once from the source prefix, once from the hint). Assert exactly one "[C]".
  const lineText = await keyedChordLine.textContent()
  const occurrences = (lineText ?? '').split('[C]').length - 1
  expect(occurrences).toBe(1)
})
