import { expect, test } from '@playwright/test'

/**
 * Source with two sections (A and B), each containing two measures.
 *
 * Lines (1-based):
 *   8:  time=4/4 key=C4 bpm=120 label="A"       ← view-zone directive
 *   9:  1 2 3 4                                   ← measure 0
 *   10: (blank)
 *   11: 5 6 7 1'                                 ← measure 1
 *   12: (blank)
 *   13: label="B"                                ← view-zone directive
 *   14: 1' 7 6 5                                 ← measure 2
 *   15: (blank)
 *   16: 4 3 2 1                                  ← measure 3
 */
const source = [
  '# metadata',
  'title = "test"',
  '',
  '# parts',
  'M = notes',
  '',
  '# score',
  'time=4/4 key=C4 bpm=120 label="A"',
  '1 2 3 4',
  '',
  "5 6 7 1'",
  '',
  'label="B"',
  "1' 7 6 5",
  '',
  '4 3 2 1',
].join('\n')

test.beforeEach(async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'section-test.jianpu',
        userFiles: { 'section-test.jianpu': src },
        bin: {},
        fileIds: { 'section-test.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)

  await page.goto('/')

  // Section buttons appear once the worker has processed the source and returned measureSpans.
  await page.waitForSelector('button.section-jump-btn', { timeout: 15_000 })
})

test('clicking section A button focuses the editor', async ({ page }) => {
  await page.locator('button.section-jump-btn', { hasText: 'A' }).click()

  // Monaco's focus() is asynchronous in some browsers; wait a tick.
  await page.waitForFunction(
    () => document.activeElement?.closest('.monaco-editor') !== null,
    { timeout: 2_000 },
  )
})

test('clicking section A button selects measures 0–1', async ({ page }) => {
  await page.locator('button.section-jump-btn', { hasText: 'A' }).click()

  // The hidden span reflects selectedMeasureRange after debounce + worker round-trip.
  await expect(page.getByTestId('selected-measure-range')).toHaveText('0-1', {
    timeout: 3_000,
  })
})

test('clicking section B button selects measures 2–3', async ({ page }) => {
  await page.locator('button.section-jump-btn', { hasText: 'B' }).click()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('2-3', {
    timeout: 3_000,
  })
})
