import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * A measure's own bar number (drawn in its system's shared directive row,
 * above the musical rows the plain measure click target covers) should be
 * its own hoverable/clickable shortcut for selecting that measure — see
 * `BarNumberClickTarget` in ARCHITECTURE.md.
 *
 * Self-contained source (not a demo file), one measure per system so the
 * first block's bar number ("1") always draws (see
 * `layout_decoration::directive_line_should_emit`).
 *
 * Measure 0 : [M] 1 2 3 4   — 4 notes
 */
const source = [
  '# metadata',
  'title = "bar number click test"',
  'max_measures_per_system = 1',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0 — line 9
].join('\n')

async function loadFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'bar-number-click-test.jianpu',
        userFiles: { 'bar-number-click-test.jianpu': source },
        bin: {},
        fileIds: {
          'bar-number-click-test.jianpu': 'bar-number-click-test-id-001',
        },
      }),
    )
  }, source)
}

/** Waits for measureSpans to be primed (same priming dance other
 * measure-select specs use) so the SVG has settled before hit-testing. */
async function primeMeasureSpans(page: import('@playwright/test').Page) {
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type('9')
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })
  await expect(
    page.locator('.preview-page [data-testid="measure-highlight"]').first(),
  ).toBeVisible({ timeout: 5_000 })
}

test("hovering a measure's bar number shows a highlight background", async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="bar-number"]', { timeout: 10_000 })

  const barNumberRect = page
    .locator(
      'g[data-tag="bar-number"] > rect[data-variant="bar-number-click-target-rect"]',
    )
    .first()
  await expect(barNumberRect).toBeVisible({ timeout: 5_000 })
  const box = await barNumberRect.boundingBox()
  if (!box) throw new Error('Could not get bounding box for the bar number.')

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)

  // The :hover rule's paint can lag the mouse-move event by a frame or two
  // in a headless run, so poll rather than reading getComputedStyle once.
  await expect
    .poll(() => barNumberRect.evaluate((el) => getComputedStyle(el).fill), {
      timeout: 3_000,
    })
    .not.toMatch(/^(none|rgba?\(0, ?0, ?0, ?0\))$/)
})

test("clicking a measure's bar number selects every note in that measure", async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="bar-number"]', { timeout: 10_000 })
  await primeMeasureSpans(page)

  const barNumberRect = page
    .locator(
      'g[data-tag="bar-number"] > rect[data-variant="bar-number-click-target-rect"]',
    )
    .first()
  await expect(barNumberRect).toBeVisible({ timeout: 5_000 })
  const box = await barNumberRect.boundingBox()
  if (!box) throw new Error('Could not get bounding box for the bar number.')

  // A plain click (mousedown + mouseup at the same point, no drag).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()

  // Measure 0 ("1 2 3 4") has exactly 4 notes.
  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )
  await expect(highlightedNotes).toHaveCount(4)

  await expect(page.locator('button.play-measure-btn')).toHaveText(
    /Selection/,
    { timeout: 3_000 },
  )
})
