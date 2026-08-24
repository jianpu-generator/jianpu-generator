import { expect, test } from '@playwright/test'

/**
 * Regression test: selecting a `# sequence` chain must scroll the SVG
 * preview to wherever the user actually navigated to (the drag's endpoint),
 * the same target `sequence-jump-select-reveal-target.spec.ts` already
 * covers for the Monaco editor.
 *
 * Before this fix, the preview's scroll-to-selection effect always targeted
 * the *envelope*'s earliest measure (`selectedMeasureRange.start`, the
 * min-line entry across the whole chain) rather than the entry the user
 * dragged to — so dragging from an already-visible entry ("Intro", at the
 * top of the source) down to a far-away one ("C") left the Monaco selection
 * correct but never scrolled the preview at all, since the envelope's start
 * line was still "Intro"'s, which was already on screen.
 *
 * Padded with filler measures so "Intro" and "C" can't both fit in one
 * viewport.
 */
const fillerMeasures = Array.from(
  { length: 60 },
  (_, i) => `label="Filler${i}"\n1 2 3 4`,
).join('\n\n')

const source = [
  '# metadata',
  'title = "test"',
  '',
  '# parts',
  'M = notes',
  '',
  '# sequence',
  'Intro, C',
  '',
  '# score',
  'time=4/4 key=C4 bpm=120 label="Intro"',
  '1 2 3 4',
  '',
  fillerMeasures,
  '',
  'label="C"',
  "1' 7 6 5",
].join('\n')

function sequenceToolbarButtons(page: import('@playwright/test').Page) {
  return page
    .locator('[role="toolbar"]')
    .nth(1)
    .locator('button.section-jump-btn')
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'sequence-preview-reveal-target-test.jianpu',
        userFiles: { 'sequence-preview-reveal-target-test.jianpu': src },
        bin: {},
        fileIds: {
          'sequence-preview-reveal-target-test.jianpu': crypto.randomUUID(),
        },
      }),
    )
  }, source)

  await page.goto('/')
  await expect(sequenceToolbarButtons(page)).toHaveCount(2, {
    timeout: 15_000,
  })
})

test('dragging from "Intro" to "C" scrolls the preview to "C", not "Intro"', async ({
  page,
}) => {
  const buttons = sequenceToolbarButtons(page)
  const measureGroups = page.locator('[data-tag="measure"]')
  await expect(measureGroups).toHaveCount(62, { timeout: 15_000 })

  // "C" is the last written measure (index 61): "Intro" (index 0) plus 60
  // filler measures (indices 1-60).
  const lastMeasure = measureGroups.last()
  await expect(lastMeasure).not.toBeInViewport()

  // index 0 = Intro, index 1 = C.
  await buttons.nth(0).hover()
  await page.mouse.down()
  await buttons.nth(1).hover()
  await page.mouse.up()

  await expect(lastMeasure).toBeInViewport({ timeout: 3_000 })
})
