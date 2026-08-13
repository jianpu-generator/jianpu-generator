import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * Regression test: a part-label drag (see `Preview.tsx`'s
 * `getPartLabelAtPoint`/`partLabelsInMarquee` in `previewDragHighlights.ts`)
 * is meant to be a vertical shortcut for selecting more *parts within the
 * same system* the drag started in — every part label's click target only
 * ever covers its own system's measure range (`measureIndexStart`/
 * `measureIndexEnd`, one `PartLabelClickTarget` per system, see
 * `grid_layout::click_targets::compute_all_part_label_click_targets`).
 *
 * The marquee test in `partLabelsInMarquee` currently has no awareness of
 * system boundaries though: it just intersects the drag rectangle against
 * every part-label rect in the whole document, so a drag that happens to
 * travel far enough vertically to reach a *different* system's label row
 * picks that label up too — silently splicing together notes from two
 * unrelated systems (undefined/nonsensical as a "part selection"). The drag
 * must clamp to the system it started in instead.
 *
 * `max_measures_per_system = 1` forces each measure onto its own system, so
 * Melody's and Harmony's labels repeat twice, stacked vertically:
 *
 *   System 0 (measure 0): Melody "1 2", Harmony "5 6"
 *   System 1 (measure 1): Melody "3 4", Harmony "7 1'"
 */
const source = [
  '# metadata',
  'title = "part label drag system boundary test"',
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
        active: 'part-label-drag-system-boundary-test.jianpu',
        userFiles: { 'part-label-drag-system-boundary-test.jianpu': source },
        bin: {},
        fileIds: {
          'part-label-drag-system-boundary-test.jianpu':
            'part-label-drag-system-boundary-test-id-001',
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

test('dragging a part label past its own system does not select notes from the next system', async ({
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
  const system1Melody = page.locator(
    '[data-tag="part-label"][data-part-index="0"][data-measure-index-start="1"]',
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

  // Drag straight down from system 0's Melody label to system 1's Melody
  // label — a vertical drag, same gesture the "drag from one part label to
  // another" test uses within a single system, but this one crosses a
  // system boundary.
  await page.mouse.move(
    startBox.x + startBox.width / 2,
    startBox.y + startBox.height / 2,
  )
  await page.mouse.down()
  await page.mouse.move(
    endBox.x + endBox.width / 2,
    endBox.y + endBox.height / 2,
    { steps: 10 },
  )
  await page.mouse.up()

  // Melody has 2 notes per system (4 total across both systems). Only
  // system 0's 2 should be selected — the drag must not reach into system
  // 1's Melody notes just because the pointer ended up over that label.
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="0"]',
    ),
  ).toHaveCount(2)
  // Harmony's system-0 row sits between the two Melody labels on screen, so
  // the vertical drag legitimately sweeps over it too — that's the normal
  // "select more parts within the same system" shortcut, still allowed.
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="1"]',
    ),
  ).toHaveCount(2)
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(4)

  // Only system 0's labels stay visually selected; system 1's Melody label —
  // the one the pointer physically ended the drag on — must not.
  await expect(
    system0Melody.locator('rect[data-variant="part-label-click-target-rect"]'),
  ).toHaveAttribute('data-part-label-drag-active', '')
  await expect(
    system0Harmony.locator('rect[data-variant="part-label-click-target-rect"]'),
  ).toHaveAttribute('data-part-label-drag-active', '')
  await expect(
    system1Melody.locator('rect[data-variant="part-label-click-target-rect"]'),
  ).not.toHaveAttribute('data-part-label-drag-active', '')
})
