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

// Drags a mouse (already pressed down) from one section button to another,
// interpolating through intermediate points so the cursor never technically
// leaves the toolbar's bounding box mid-transition. A single-jump hover
// (locator.hover()) can land the cursor exactly on the toolbar's edge for a
// frame, spuriously firing the container's onMouseLeave and cancelling the
// drag (App.tsx's toolbar `onMouseLeave` resets dragStartLabel) — this
// mirrors the multi-step pattern already used in drag-to-select-measures.spec.ts.
async function dragBetweenSectionButtons(
  page: import('@playwright/test').Page,
  from: import('@playwright/test').Locator,
  to: import('@playwright/test').Locator,
) {
  const fromBox = await from.boundingBox()
  const toBox = await to.boundingBox()
  if (!fromBox || !toBox) {
    throw new Error('Could not get bounding boxes for section buttons.')
  }

  await page.mouse.move(
    fromBox.x + fromBox.width / 2,
    fromBox.y + fromBox.height / 2,
  )
  await page.mouse.down()
  // Wait for the mousedown's setDragStartLabel state update to commit
  // before moving on — otherwise the target button's onMouseEnter can fire
  // fast enough (under synthetic/automated input) to read the pre-drag
  // (null) dragStartLabel and skip handleSectionRangeSelect entirely.
  await expect(from).toHaveClass(/section-jump-btn--dragging/, {
    timeout: 3_000,
  })
  await page.mouse.move(toBox.x + toBox.width / 2, toBox.y + toBox.height / 2, {
    steps: 10,
  })
  // Wait for the target's mouseenter (and the resulting
  // handleSectionRangeSelect call) to actually land before the caller
  // releases the mouse — under parallel test load the final synthetic
  // mousemove can otherwise be processed after mouseup, making the drag a
  // no-op that only registers the initial mousedown click.
  await expect(to).toHaveClass(/section-jump-btn--dragging/, {
    timeout: 3_000,
  })
}

// Dragging from one section button to another should select the merged
// line/measure range spanning both sections, via
// useSectionNavigation's handleSectionRangeSelect.
test('dragging from section A button to section B button selects the merged range', async ({
  page,
}) => {
  const buttonA = page.locator('button.section-jump-btn', { hasText: 'A' })
  const buttonB = page.locator('button.section-jump-btn', { hasText: 'B' })

  await dragBetweenSectionButtons(page, buttonA, buttonB)
  await page.mouse.up()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('0-3', {
    timeout: 3_000,
  })

  await expect
    .poll(() => getEditorSelection(page), { timeout: 3_000 })
    .toEqual({ startLineNumber: 8, endLineNumber: 16 })
})

// Reverse-direction drag confirms the order-independent lookup in
// handleSectionRangeSelect (labels are matched start/end in either order).
test('dragging from section B button to section A button selects the merged range', async ({
  page,
}) => {
  const buttonA = page.locator('button.section-jump-btn', { hasText: 'A' })
  const buttonB = page.locator('button.section-jump-btn', { hasText: 'B' })

  await dragBetweenSectionButtons(page, buttonB, buttonA)
  await page.mouse.up()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('0-3', {
    timeout: 3_000,
  })

  await expect
    .poll(() => getEditorSelection(page), { timeout: 3_000 })
    .toEqual({ startLineNumber: 8, endLineNumber: 16 })
})

// While a drag is in progress, every section button between the drag start
// and the currently-hovered button should get the `--dragging` highlight
// class (dragHighlightedLabels). The class doubles as the "active
// selection" indicator (activeHighlightedLabels), so it stays applied after
// mouseup too — the completed drag leaves the merged A-B range selected,
// it doesn't merely mark an in-progress drag.
test('dragging between section buttons highlights them while dragging', async ({
  page,
}) => {
  const buttonA = page.locator('button.section-jump-btn', { hasText: 'A' })
  const buttonB = page.locator('button.section-jump-btn', { hasText: 'B' })

  await dragBetweenSectionButtons(page, buttonA, buttonB)

  await expect(buttonA).toHaveClass(/section-jump-btn--dragging/)
  await expect(buttonB).toHaveClass(/section-jump-btn--dragging/)

  await page.mouse.up()

  // The A-B range is now the active selection, so both buttons remain
  // highlighted (not cleared) — confirms activeHighlightedLabels falls back
  // to selectedLineRange once dragStartLabel resets to null.
  await expect(buttonA).toHaveClass(/section-jump-btn--dragging/)
  await expect(buttonB).toHaveClass(/section-jump-btn--dragging/)
})

// Touch equivalent of dragBetweenSectionButtons above. Real touchmove events
// (unlike mouse) keep their `target` pinned to whatever element the touch
// started on, so SectionJumpToolbar resolves the button under the finger via
// `document.elementFromPoint` instead of relying on onMouseEnter — this drives
// that path with actual dispatched touch events (via CDP) rather than mouse
// events, so a regression to the elementFromPoint lookup would still be
// caught even though mouse-based tests keep passing.
async function dragBetweenSectionButtonsWithTouch(
  page: import('@playwright/test').Page,
  from: import('@playwright/test').Locator,
  to: import('@playwright/test').Locator,
) {
  const fromBox = await from.boundingBox()
  const toBox = await to.boundingBox()
  if (!fromBox || !toBox) {
    throw new Error('Could not get bounding boxes for section buttons.')
  }

  const fromPoint = {
    x: fromBox.x + fromBox.width / 2,
    y: fromBox.y + fromBox.height / 2,
  }
  const toPoint = {
    x: toBox.x + toBox.width / 2,
    y: toBox.y + toBox.height / 2,
  }

  const client = await page.context().newCDPSession(page)

  await client.send('Input.dispatchTouchEvent', {
    type: 'touchStart',
    touchPoints: [{ x: fromPoint.x, y: fromPoint.y }],
  })
  await expect(from).toHaveClass(/section-jump-btn--dragging/, {
    timeout: 3_000,
  })

  const steps = 10
  for (let step = 1; step <= steps; step += 1) {
    const x = fromPoint.x + ((toPoint.x - fromPoint.x) * step) / steps
    const y = fromPoint.y + ((toPoint.y - fromPoint.y) * step) / steps
    await client.send('Input.dispatchTouchEvent', {
      type: 'touchMove',
      touchPoints: [{ x, y }],
    })
  }
  await expect(to).toHaveClass(/section-jump-btn--dragging/, {
    timeout: 3_000,
  })

  await client.send('Input.dispatchTouchEvent', {
    type: 'touchEnd',
    touchPoints: [],
  })
}

test('touch-dragging from section A button to section B button selects the merged range', async ({
  page,
}) => {
  const buttonA = page.locator('button.section-jump-btn', { hasText: 'A' })
  const buttonB = page.locator('button.section-jump-btn', { hasText: 'B' })

  await dragBetweenSectionButtonsWithTouch(page, buttonA, buttonB)

  await expect(page.getByTestId('selected-measure-range')).toHaveText('0-3', {
    timeout: 3_000,
  })

  await expect
    .poll(() => getEditorSelection(page), { timeout: 3_000 })
    .toEqual({ startLineNumber: 8, endLineNumber: 16 })
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
