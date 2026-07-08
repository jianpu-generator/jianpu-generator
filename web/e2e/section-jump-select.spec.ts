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

// Read the live Monaco selection off the `monaco` global that
// `@monaco-editor/react`'s loader exposes on `window`, rather than trusting
// only the `selected-measure-range` testid — this confirms the editor's
// actual highlighted text spans the clicked section's lines, not just that
// the app's internal state was updated.
async function getEditorSelection(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const monacoApi = (
      window as unknown as { monaco: typeof import('monaco-editor') }
    ).monaco
    const selection = monacoApi.editor.getEditors()[0]?.getSelection()
    if (!selection) return null
    return {
      startLineNumber: selection.startLineNumber,
      endLineNumber: selection.endLineNumber,
    }
  })
}

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

test('clicking section A button highlights lines 8–11 in the Monaco editor', async ({
  page,
}) => {
  await page.locator('button.section-jump-btn', { hasText: 'A' }).click()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('0-1', {
    timeout: 3_000,
  })

  await expect
    .poll(() => getEditorSelection(page), { timeout: 3_000 })
    .toEqual({ startLineNumber: 8, endLineNumber: 11 })
})

test('clicking section B button highlights lines 13–16 in the Monaco editor', async ({
  page,
}) => {
  await page.locator('button.section-jump-btn', { hasText: 'B' }).click()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('2-3', {
    timeout: 3_000,
  })

  await expect
    .poll(() => getEditorSelection(page), { timeout: 3_000 })
    .toEqual({ startLineNumber: 13, endLineNumber: 16 })
})

// Section labels are also rendered inside the SVG preview itself (as a
// `<g data-tag="section-label" data-section-label="…">` group) and are
// clickable there via the same onMouseDown -> elementFromPoint lookup that
// backs the button toolbar. Cover that path separately, since it goes
// through a different DOM element than `button.section-jump-btn`.
test('clicking section A label in the SVG preview highlights lines 8–11 in the Monaco editor', async ({
  page,
}) => {
  const label = page
    .locator(
      '.preview-pages g[data-tag="section-label"][data-section-label="A"]',
    )
    .first()
  await label.waitFor({ timeout: 15_000 })
  await label.click()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('0-1', {
    timeout: 3_000,
  })

  await expect
    .poll(() => getEditorSelection(page), { timeout: 3_000 })
    .toEqual({ startLineNumber: 8, endLineNumber: 11 })
})

test('clicking section B label in the SVG preview highlights lines 13–16 in the Monaco editor', async ({
  page,
}) => {
  const label = page
    .locator(
      '.preview-pages g[data-tag="section-label"][data-section-label="B"]',
    )
    .first()
  await label.waitFor({ timeout: 15_000 })
  await label.click()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('2-3', {
    timeout: 3_000,
  })

  await expect
    .poll(() => getEditorSelection(page), { timeout: 3_000 })
    .toEqual({ startLineNumber: 13, endLineNumber: 16 })
})
