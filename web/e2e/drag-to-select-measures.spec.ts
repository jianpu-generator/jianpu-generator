import { expect, test } from '@playwright/test'

/**
 * The default source (reference.jianpu) contains many measures.
 * Measures are 0-indexed internally; the play-button label is 1-indexed.
 *
 * Measure 0 : [M] 0 0 0 0
 * Measure 1 : [M] 1 2 3 0  +  [M] do re mi
 * Measure 2 : [M] 1' 2' 1, 2,  +  [M] _
 * Measure 3 : [M] 1_ 1_ 1= 1= 1= 1= 1 -  +  [M] _
 *
 * Dragging from measure index 1 → measure index 3 should produce
 * selectedMeasureRange = { start: 1, end: 3 } which the play button
 * renders as "▶ Measures 2–4".
 */
test('drag from measure 1 to measure 3 selects measures 1–3', async ({
  page,
}) => {
  await page.goto('/')

  // Wait for the editor toolbar (signals WASM is loaded and app is ready).
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Wait for the SVG preview to render measure groups.
  await page.waitForSelector('[data-tag="measure"][data-measure-index="3"]', {
    timeout: 10_000,
  })

  // The SVG is rendered by the worker before it sends back measureSpans.
  // We prime measureSpans by clicking into the editor at a note line and
  // waiting for the play button to display a measure label — that confirms
  // the worker's measureSpans response has been processed.
  // Line 18 of reference.jianpu is "[M] 1 2 3 0" (measure index 1).
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('18')
  await page.keyboard.press('Enter')
  // Wait for debounce (300 ms) + worker round-trip, then for React to render
  // the play button with the measure label.
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })

  // Grab the bounding boxes for measure 1 and measure 3.
  // The SVG may contain multiple groups with the same index (one per row),
  // so we take the first occurrence for each.
  const measure1 = page
    .locator('[data-tag="measure"][data-measure-index="1"]')
    .first()
  const measure3 = page
    .locator('[data-tag="measure"][data-measure-index="3"]')
    .first()

  const box1 = await measure1.boundingBox()
  const box3 = await measure3.boundingBox()

  if (!box1 || !box3) {
    throw new Error(
      'Could not get bounding boxes for measures 1 and 3. ' +
        'Ensure the SVG preview has rendered.',
    )
  }

  const startX = box1.x + box1.width / 2
  const startY = box1.y + box1.height / 2
  const endX = box3.x + box3.width / 2
  const endY = box3.y + box3.height / 2

  // Perform the drag: mousedown on measure 1, move to measure 3, mouseup.
  await page.mouse.move(startX, startY)
  await page.mouse.down()
  // Interpolate through intermediate points so mousemove events fire on the
  // way, letting the drag-highlight logic track the current measure.
  await page.mouse.move(endX, endY, { steps: 10 })
  await page.mouse.up()

  // Allow the 300 ms debounce in notifySelection plus React re-render.
  await page.waitForTimeout(700)

  // The play button should now label the selected range as "Measures 2–4"
  // (1-indexed: internal start=1 → display 2, internal end=3 → display 4).
  const playBtn = page.locator('button.play-measure-btn')
  await expect(playBtn).toBeVisible({ timeout: 3_000 })

  // The button text shows "▶ Measures 2–4" (en-dash U+2013).
  await expect(playBtn).toHaveText(/Measures 2.4/, { timeout: 3_000 })

  // Also confirm the SVG preview shows at least one highlight rect,
  // meaning the worker re-rendered highlighted documents for the range.
  const highlightRect = page.locator(
    '.preview-page [data-testid="measure-highlight"]',
  )
  await expect(highlightRect.first()).toBeVisible({ timeout: 5_000 })
})

/**
 * Regression test for the off-by-1 bug: dragging N measures must select
 * exactly N measures, not N+1.
 *
 * Dragging from measure index 0 → measure index 3 should produce
 * selectedMeasureRange = { start: 0, end: 3 } which the play button
 * renders as "▶ Measures 1–4".
 */
test('drag from measure 0 to measure 3 selects exactly 4 measures (not 5)', async ({
  page,
}) => {
  await page.goto('/')

  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="3"]', {
    timeout: 10_000,
  })

  // Prime measureSpans: click into the editor, navigate to a measure line,
  // and wait for the play button to confirm measureSpans are loaded.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('18')
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })

  const measure0 = page
    .locator('[data-tag="measure"][data-measure-index="0"]')
    .first()
  const measure3 = page
    .locator('[data-tag="measure"][data-measure-index="3"]')
    .first()

  const box0 = await measure0.boundingBox()
  const box3 = await measure3.boundingBox()

  if (!box0 || !box3) {
    throw new Error(
      'Could not get bounding boxes for measures 0 and 3. ' +
        'Ensure the SVG preview has rendered.',
    )
  }

  const startX = box0.x + box0.width / 2
  const startY = box0.y + box0.height / 2
  const endX = box3.x + box3.width / 2
  const endY = box3.y + box3.height / 2

  await page.mouse.move(startX, startY)
  await page.mouse.down()
  await page.mouse.move(endX, endY, { steps: 10 })
  await page.mouse.up()

  // Brief wait for React state update and re-render.
  await page.waitForTimeout(400)

  // Must show exactly "Measures 1–4" (4 measures), NOT "Measures 1–5".
  const playBtn = page.locator('button.play-measure-btn')
  await expect(playBtn).toBeVisible({ timeout: 3_000 })
  await expect(playBtn).toHaveText(/Measures 1.4/, { timeout: 3_000 })

  const highlightRect = page.locator(
    '.preview-page [data-testid="measure-highlight"]',
  )
  await expect(highlightRect.first()).toBeVisible({ timeout: 5_000 })
})

/**
 * Regression test: CJK characters in the source (metadata lines before the
 * measures) must not corrupt the measure-range selection.
 *
 * The old byte-offset approach broke with CJK because each CJK character is
 * 3 UTF-8 bytes but only 1 UTF-16 code-unit, causing Monaco's getPositionAt
 * to land on the wrong line.  The fix uses start_line/end_line from
 * MeasureSpan directly so no byte→char conversion happens.
 *
 * After dragging from measure 0 → measure 3 the play button must show
 * "Measures 1–4" (not 5).
 */
test('drag from measure 0 to measure 3 with CJK source selects exactly 4 measures', async ({
  page,
  context,
}) => {
  // Source with CJK characters in every metadata line before the measures.
  // Uses a plain "notes" part (no lyrics) so that each [M] line is exactly
  // one measure — no blank-line separators needed between them.
  // Measures are separated by blank lines in jianpu syntax.
  const cjkSource = [
    '# metadata',
    'title = "彌勒淨土鄉"',
    'subtitle = "測試副標題"',
    'author = "天然師尊"',
    'max columns = 48',
    '',
    '# parts',
    'Melody [M] = notes',
    '',
    '# score',
    '[M] 0 0 0 0', // measure 0 — line 11
    '',
    '[M] 1 2 3 4', // measure 1 — line 13
    '',
    "[M] 5 6 7 1'", // measure 2 — line 15
    '',
    '[M] 1_ 2_ 3_ 4_', // measure 3 — line 17
  ].join('\n')

  // Pre-populate localStorage before the page loads so the app opens the CJK
  // file instead of the read-only demo file.
  await context.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'cjk-test.jianpu',
        userFiles: { 'cjk-test.jianpu': source },
        bin: {},
        fileIds: { 'cjk-test.jianpu': 'cjk-test-id-001' },
      }),
    )
  }, cjkSource)

  await page.goto('/')

  // Wait for the editor toolbar (signals WASM is loaded and app is ready).
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Wait for the SVG preview to render all four measure groups.
  await page.waitForSelector('[data-tag="measure"][data-measure-index="3"]', {
    timeout: 15_000,
  })

  // Prime measureSpans: navigate to line 11 ("[M] 0 0 0 0", measure 0) and
  // wait for the play button to confirm the worker has sent measureSpans back.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('11')
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })

  const measure0 = page
    .locator('[data-tag="measure"][data-measure-index="0"]')
    .first()
  const measure3 = page
    .locator('[data-tag="measure"][data-measure-index="3"]')
    .first()

  const box0 = await measure0.boundingBox()
  const box3 = await measure3.boundingBox()

  if (!box0 || !box3) {
    throw new Error(
      'Could not get bounding boxes for measures 0 and 3. ' +
        'Ensure the SVG preview has rendered.',
    )
  }

  // Drag from measure 0 to measure 3.
  await page.mouse.move(box0.x + box0.width / 2, box0.y + box0.height / 2)
  await page.mouse.down()
  await page.mouse.move(box3.x + box3.width / 2, box3.y + box3.height / 2, {
    steps: 10,
  })
  await page.mouse.up()

  // Allow the debounce (300 ms) + React re-render.
  await page.waitForTimeout(700)

  // Must show exactly "Measures 1–4" (4 measures), NOT "Measures 1–5".
  const playBtn = page.locator('button.play-measure-btn')
  await expect(playBtn).toBeVisible({ timeout: 3_000 })
  await expect(playBtn).toHaveText(/Measures 1.4/, { timeout: 3_000 })

  // At least one highlight rect must be visible, confirming the worker
  // re-rendered with the correct highlighted range.
  const highlightRect2 = page.locator(
    '.preview-page [data-testid="measure-highlight"]',
  )
  await expect(highlightRect2.first()).toBeVisible({ timeout: 5_000 })
})
