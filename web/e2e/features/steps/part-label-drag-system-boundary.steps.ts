import { expect } from '@playwright/test'
import { focusEditor } from '../../fileSwitcherHelpers'
import { Given, Then, When } from './fixtures'

/**
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

function partLabel(
  page: import('@playwright/test').Page,
  partIndex: number,
  measureIndexStart: number,
) {
  return page.locator(
    `[data-tag="part-label"][data-part-index="${partIndex}"][data-measure-index-start="${measureIndexStart}"]`,
  )
}

Given('the part-label system-boundary fixture is loaded', async ({ page }) => {
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
})

When(
  "I drag straight down from system 0's Melody label to system 1's Melody label",
  async ({ page }) => {
    const system0Melody = partLabel(page, 0, 0)
    const system1Melody = partLabel(page, 0, 1)
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
  },
)

Then(
  '{int} drag-selected notes belong to part index {int}, as seen in part label drag system boundary',
  async ({ page }, count: number, partIndex: number) => {
    // Melody has 2 notes per system (4 total across both systems). Only
    // system 0's 2 should be selected — the drag must not reach into system
    // 1's Melody notes just because the pointer ended up over that label.
    // Harmony's system-0 row sits between the two Melody labels on screen, so
    // the vertical drag legitimately sweeps over it too — that's the normal
    // "select more parts within the same system" shortcut, still allowed.
    await expect(
      page.locator(
        `[data-tag="note"][data-note-drag-selected][data-part-index="${partIndex}"]`,
      ),
    ).toHaveCount(count)
  },
)

Then(
  '{int} notes are drag-selected in total, as seen in part label drag system boundary',
  async ({ page }, count: number) => {
    await expect(
      page.locator('[data-tag="note"][data-note-drag-selected]'),
    ).toHaveCount(count)
  },
)

Then(
  "system 0's Melody label's click-target rect is marked drag-active, as seen in part label drag system boundary",
  async ({ page }) => {
    // Only system 0's labels stay visually selected; system 1's Melody label
    // — the one the pointer physically ended the drag on — must not.
    await expect(
      partLabel(page, 0, 0).locator(
        'rect[data-variant="part-label-click-target-rect"]',
      ),
    ).toHaveAttribute('data-part-label-drag-active', '')
  },
)

Then(
  "system 0's Harmony label's click-target rect is marked drag-active, as seen in part label drag system boundary",
  async ({ page }) => {
    await expect(
      partLabel(page, 1, 0).locator(
        'rect[data-variant="part-label-click-target-rect"]',
      ),
    ).toHaveAttribute('data-part-label-drag-active', '')
  },
)

Then(
  "system 1's Melody label's click-target rect is not marked drag-active",
  async ({ page }) => {
    await expect(
      partLabel(page, 0, 1).locator(
        'rect[data-variant="part-label-click-target-rect"]',
      ),
    ).not.toHaveAttribute('data-part-label-drag-active', '')
  },
)
