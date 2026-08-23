import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * The visible bar line (measure divider) between two measures should be a
 * reliable, hoverable drag handle for measure-range selection: a Cmd/Ctrl
 * drag starting exactly on the divider pixel between measure 0 and measure 1
 * must select whole measures, not fall into a per-note marquee drag (see
 * `PreviewSvgRenderer.tsx`'s `renderBarLineDragHandle`). Cmd/Ctrl is required
 * since a plain drag now resolves to note/chord/syllable granularity (see
 * `Preview.tsx`'s `onMouseDown`).
 *
 * Same fixture as `measure-click-selects-notes.spec.ts`:
 * Measure 0 : [M] 1 2 3 4   — 4 notes
 * Measure 1 : [M] 5 6       — 2 notes
 * Measure 2 : [M] 7 1'      — 2 notes
 */
const barLineTestSource = [
  '# metadata',
  'title = "bar line drag test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0
  '',
  '[M] 5 6', // measure 1
  '',
  "[M] 7 1'", // measure 2
].join('\n')

async function loadBarLineTestFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'bar-line-drag-test.jianpu',
        userFiles: { 'bar-line-drag-test.jianpu': source },
        bin: {},
        fileIds: { 'bar-line-drag-test.jianpu': 'bar-line-drag-test-id-001' },
      }),
    )
  }, barLineTestSource)
}

/** Waits for measureSpans to be primed (same priming dance
 * `measure-click-selects-notes.spec.ts` uses) so the SVG has settled before
 * hit-testing. */
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

test('hovering the bar line between two measures shows a drag cursor', async ({
  page,
}) => {
  await loadBarLineTestFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="1"]', {
    timeout: 10_000,
  })
  await primeMeasureSpans(page)

  const measure1 = page
    .locator('[data-tag="measure"][data-measure-index="1"]')
    .first()
  const box = await measure1.boundingBox()
  if (!box) throw new Error('Could not get bounding box for measure 1.')

  // The bar line between measure 0 and measure 1 sits at measure 1's own
  // left edge (see `measure_column_bounds`).
  await page.mouse.move(box.x, box.y + box.height / 2)

  const handle = page.locator('line.bar-line-drag-handle').first()
  await expect(handle).toHaveCSS('cursor', 'col-resize')
})

test('Cmd/Ctrl-dragging from a bar line into a further measure selects every note in the full range', async ({
  page,
}) => {
  await loadBarLineTestFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="2"]', {
    timeout: 10_000,
  })
  await primeMeasureSpans(page)

  const measure1 = page
    .locator('[data-tag="measure"][data-measure-index="1"]')
    .first()
  const measure2 = page
    .locator('[data-tag="measure"][data-measure-index="2"]')
    .first()
  const box1 = await measure1.boundingBox()
  const box2 = await measure2.boundingBox()
  if (!box1 || !box2) {
    throw new Error('Could not get bounding boxes for measures 1 and 2.')
  }

  // Start the drag exactly on the bar line between measure 0 and measure 1
  // (measure 1's own left edge), then drag into measure 2's interior. Held
  // under Cmd/Ctrl (see this file's top-of-file comment).
  await page.mouse.move(box1.x, box1.y + box1.height / 2)
  await page.keyboard.down('Control')
  await page.mouse.down()
  await page.mouse.move(box2.x + box2.width / 2, box2.y + box2.height / 2, {
    steps: 8,
  })
  await page.mouse.up()
  await page.keyboard.up('Control')

  // Measures 1-2 have 2 + 2 = 4 notes in total — the full measure range, not
  // a partial note marquee.
  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )
  await expect(highlightedNotes).toHaveCount(4)

  await expect(page.locator('button.play-measure-btn')).toHaveText(
    /Selection/,
    { timeout: 3_000 },
  )
})
