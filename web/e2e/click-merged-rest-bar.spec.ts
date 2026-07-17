import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * Self-contained source with a run of 3 consecutive all-rest measures
 * (measures 1-3), which the renderer collapses into a single wide
 * multi-measure-rest bar (see "Multi-measure rests" in syntax.md).
 *
 * Measure 0 : [M] 1 1 1 1        — normal, not part of the rest run
 * Measure 1 : [M] 0 0 0 0        — rest, start of the merged run
 * Measure 2 : [M] 0 0 0 0        — rest, middle of the merged run
 * Measure 3 : [M] 0 0 0 0        — rest, end of the merged run
 * Measure 4 : [M] 2 2 2 2        — normal, not part of the rest run
 *
 * Clicking anywhere on the merged bar must select measures 1-3 (not just
 * measure 1, the run's first source measure), i.e. the Monaco selection
 * must span from measure 1's start line to measure 3's end line.
 */
const mergedRestSource = [
  '# metadata',
  'title = "merged rest click test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '[M] 1 1 1 1', // measure 0 — line 9
  '',
  '[M] 0 0 0 0', // measure 1 — line 11
  '',
  '[M] 0 0 0 0', // measure 2 — line 13
  '',
  '[M] 0 0 0 0', // measure 3 — line 15
  '',
  '[M] 2 2 2 2', // measure 4 — line 17
].join('\n')

test('clicking a merged rest bar highlights all its measures in the editor', async ({
  page,
}) => {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'merged-rest-test.jianpu',
        userFiles: { 'merged-rest-test.jianpu': source },
        bin: {},
        fileIds: { 'merged-rest-test.jianpu': 'merged-rest-test-id-001' },
      }),
    )
  }, mergedRestSource)

  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })

  // The merged run (measures 1-3) renders as a single bar whose click target
  // carries measure_index=1 (the run's first source measure) and
  // measure_index_end=3 (the run's last source measure).
  const mergedBar = page.locator(
    '[data-tag="measure"][data-measure-index="1"][data-measure-index-end="3"]',
  )
  await expect(mergedBar.first()).toBeVisible({ timeout: 10_000 })

  // Prime measureSpans by moving the cursor into the editor.
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

  const box = await mergedBar.first().boundingBox()
  if (!box) {
    throw new Error('Could not get bounding box for the merged rest bar.')
  }

  // A plain click (mousedown + mouseup at the same point, no drag).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()

  // Allow the 300 ms debounce in notifySelection plus React re-render.
  await page.waitForTimeout(700)

  // Internal range {start: 1, end: 3} is 1-indexed on display: "Measures 2–4".
  const playBtn = page.locator('button.play-measure-btn')
  await expect(playBtn).toBeVisible({ timeout: 3_000 })
  await expect(playBtn).toHaveText(/Measures 2.4/, { timeout: 3_000 })
})
