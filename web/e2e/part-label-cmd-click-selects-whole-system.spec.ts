import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * Cmd/Ctrl-click(-drag) on a part label elevates the selection from "this
 * one part's system" (a plain click/drag, see
 * `part-label-drag-system-boundary.spec.ts`) to "every part in every system
 * the gesture touches" — see `PreviewDragState`'s 'part-label-system' doc
 * comment and `partLabelsInMarqueeAcrossSystems`.
 *
 * Same fixture as `part-label-drag-system-boundary.spec.ts`:
 * `max_measures_per_system = 1` forces each measure onto its own system, so
 * Melody's and Harmony's labels repeat twice, stacked vertically:
 *
 *   System 0 (measure 0): Melody "1 2", Harmony "5 6"
 *   System 1 (measure 1): Melody "3 4", Harmony "7 1'"
 */
const source = [
  '# metadata',
  'title = "part label cmd click test"',
  'max_measures_per_system = 1',
  '',
  '# parts',
  'Melody [M] = notes',
  'Harmony [H] = notes',
  '',
  '# score',
  '[M] 1 2', // measure 0
  '[H] 5 6',
  '',
  '[M] 3 4', // measure 1
  "[H] 7 1'",
].join('\n')

async function loadFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'part-label-cmd-click-test.jianpu',
        userFiles: { 'part-label-cmd-click-test.jianpu': source },
        bin: {},
        fileIds: {
          'part-label-cmd-click-test.jianpu':
            'part-label-cmd-click-test-id-001',
        },
      }),
    )
  }, source)
}

/** Waits for measureSpans to be primed (same priming dance the measure-select
 * specs use) so the SVG has settled before hit-testing. */
async function primeMeasureSpans(page: import('@playwright/test').Page) {
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type('10')
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })
  await expect(
    page.locator('.preview-page [data-testid="measure-highlight"]').first(),
  ).toBeVisible({ timeout: 5_000 })
}

test("Cmd/Ctrl-clicking one part label selects every part in that label's system", async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(
    '[data-tag="part-label"][data-part-index="0"][data-measure-index-start="1"]',
    { timeout: 10_000 },
  )
  await primeMeasureSpans(page)

  const system0Melody = page.locator(
    '[data-tag="part-label"][data-part-index="0"][data-measure-index-start="0"]',
  )
  const system0Harmony = page.locator(
    '[data-tag="part-label"][data-part-index="1"][data-measure-index-start="0"]',
  )
  await expect(system0Melody).toBeVisible({ timeout: 5_000 })

  const box = await system0Melody.boundingBox()
  if (!box) throw new Error('Could not get bounding box for system 0 Melody.')

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.keyboard.down('Control')
  await page.mouse.down()
  await page.mouse.up()
  await page.keyboard.up('Control')

  // Both parts' notes in system 0 (2 + 2 = 4), none from system 1.
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="0"]',
    ),
  ).toHaveCount(2)
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="1"]',
    ),
  ).toHaveCount(2)
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(4)

  await expect(
    system0Melody.locator('rect[data-variant="part-label-click-target-rect"]'),
  ).toHaveAttribute('data-part-label-drag-active', '')
  await expect(
    system0Harmony.locator('rect[data-variant="part-label-click-target-rect"]'),
  ).toHaveAttribute('data-part-label-drag-active', '')
})

test("Cmd/Ctrl-dragging from one system's part label into another system selects every part across both systems", async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(
    '[data-tag="part-label"][data-part-index="0"][data-measure-index-start="1"]',
    { timeout: 10_000 },
  )
  await primeMeasureSpans(page)

  const system0Melody = page.locator(
    '[data-tag="part-label"][data-part-index="0"][data-measure-index-start="0"]',
  )
  const system1Melody = page.locator(
    '[data-tag="part-label"][data-part-index="0"][data-measure-index-start="1"]',
  )
  const system1Harmony = page.locator(
    '[data-tag="part-label"][data-part-index="1"][data-measure-index-start="1"]',
  )
  await expect(system0Melody).toBeVisible({ timeout: 5_000 })
  await expect(system1Melody).toBeVisible({ timeout: 5_000 })

  const startBox = await system0Melody.boundingBox()
  const endBox = await system1Melody.boundingBox()
  if (!startBox || !endBox) {
    throw new Error(
      'Could not get bounding boxes for the system 0/1 Melody labels.',
    )
  }

  await page.mouse.move(
    startBox.x + startBox.width / 2,
    startBox.y + startBox.height / 2,
  )
  await page.keyboard.down('Control')
  await page.mouse.down()
  await page.mouse.move(
    endBox.x + endBox.width / 2,
    endBox.y + endBox.height / 2,
    { steps: 10 },
  )
  await page.mouse.up()
  await page.keyboard.up('Control')

  // Every part in both systems: Melody 2+2, Harmony 2+2 = 8 total.
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="0"]',
    ),
  ).toHaveCount(4)
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="1"]',
    ),
  ).toHaveCount(4)
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(8)

  await expect(
    system1Melody.locator('rect[data-variant="part-label-click-target-rect"]'),
  ).toHaveAttribute('data-part-label-drag-active', '')
  await expect(
    system1Harmony.locator('rect[data-variant="part-label-click-target-rect"]'),
  ).toHaveAttribute('data-part-label-drag-active', '')
})
