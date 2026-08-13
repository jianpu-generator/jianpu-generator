import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * Clicking (or dragging across) a measure in the SVG preview is a shortcut
 * for selecting every note/rest cell in that measure — there is no separate
 * "measure selected" state anymore (see `Preview.tsx`'s
 * `noteCellsInMeasureRange`). It reuses the same note drag-select highlight
 * (`[data-note-drag-selected]`) and Monaco multicursor pathway
 * (`onNoteRangeSelect`) that dragging a marquee over individual notes uses.
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system" so all measures render in one row and stay within the viewport.
 *
 * Measure 0 : [M] 1 2 3 4   — 4 notes
 * Measure 1 : [M] 5 6       — 2 notes
 * Measure 2 : [M] 7 1'      — 2 notes
 */
const clickTestSource = [
  '# metadata',
  'title = "measure click test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0 — line 9
  '',
  '[M] 5 6', // measure 1 — line 11
  '',
  "[M] 7 1'", // measure 2 — line 13
].join('\n')

async function loadClickTestFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'measure-click-test.jianpu',
        userFiles: { 'measure-click-test.jianpu': source },
        bin: {},
        fileIds: { 'measure-click-test.jianpu': 'measure-click-test-id-001' },
      }),
    )
  }, clickTestSource)
}

/** Waits for measureSpans to be primed (same priming dance the old
 * measure-select specs used) so the SVG has settled before hit-testing. */
async function primeMeasureSpans(page: import('@playwright/test').Page) {
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type('9')
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })
  // Priming the cursor also triggers an async highlight re-render that swaps
  // the SVG DOM and scrolls it into view — wait for that to settle before
  // measuring positions, otherwise bounding boxes captured mid-scroll are
  // inconsistent with each other.
  await expect(
    page.locator('.preview-page [data-testid="measure-highlight"]').first(),
  ).toBeVisible({ timeout: 5_000 })
}

test('clicking a measure selects every note in that measure', async ({
  page,
}) => {
  await loadClickTestFixture(page)
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
  await expect(measure1).toBeVisible({ timeout: 5_000 })
  const box = await measure1.boundingBox()
  if (!box) throw new Error('Could not get bounding box for measure 1.')

  // A plain click (mousedown + mouseup at the same point, no drag).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()

  // Measure 1 ("5 6") has exactly 2 notes.
  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )
  await expect(highlightedNotes).toHaveCount(2)

  // The repurposed play-measure button switching to "▶ Selection" confirms a
  // note range was pushed into Monaco/App state, same as an ordinary note
  // drag-select.
  await expect(page.locator('button.play-measure-btn')).toHaveText(
    /Selection/,
    { timeout: 3_000 },
  )
})

test('clicking right at a measure boundary selects that measure, not its neighbor', async ({
  page,
}) => {
  await loadClickTestFixture(page)
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
  await expect(measure1).toBeVisible({ timeout: 5_000 })
  const box = await measure1.boundingBox()
  if (!box) throw new Error('Could not get bounding box for measure 1.')

  // Click exactly at measure 1's own reported left edge — the pixel where
  // measure 0's and measure 1's click-target rects meet. `getMeasureAtPoint`
  // must resolve this to measure 1 (the measure that pixel is reported as
  // belonging to), not measure 0: at a coincident rect edge,
  // `elementsFromPoint`'s z-order is not a reliable tie-break (see
  // `Preview.tsx`'s `getMeasureAtPoint`), which previously made this click
  // resolve to the wrong (previous) measure.
  await page.mouse.move(box.x, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()

  // Measure 1 ("5 6") has exactly 2 notes; measure 0 ("1 2 3 4") has 4.
  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )
  await expect(highlightedNotes).toHaveCount(2)
  for (const noteId of [4, 5]) {
    await expect(
      page.locator(
        `[data-tag="note"][data-note-drag-selected][data-note-id="${noteId}"]`,
      ),
    ).toHaveCount(1)
  }
})

test('dragging across measures selects every note in the range', async ({
  page,
}) => {
  await loadClickTestFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="2"]', {
    timeout: 10_000,
  })
  await primeMeasureSpans(page)

  const measure0 = page
    .locator('[data-tag="measure"][data-measure-index="0"]')
    .first()
  const measure2 = page
    .locator('[data-tag="measure"][data-measure-index="2"]')
    .first()
  await expect(measure0).toBeVisible({ timeout: 5_000 })
  await expect(measure2).toBeVisible({ timeout: 5_000 })

  const box0 = await measure0.boundingBox()
  const box2 = await measure2.boundingBox()
  if (!box0 || !box2) {
    throw new Error('Could not get bounding boxes for measures 0 and 2.')
  }

  // Notes fully tile their measure's width (no gaps between click-target
  // rects), so a mousedown anywhere inside a measure always lands on some
  // note and this drag follows the raw note-marquee path (see
  // `Preview.tsx`'s 'pending' → 'note' arming) — drag corner-to-corner
  // rather than center-to-center so the marquee's bounding box fully covers
  // every note between measure 0 and measure 2, not just the ones between
  // their two center points.
  await page.mouse.move(box0.x + 1, box0.y + 1)
  await page.mouse.down()
  await page.mouse.move(box2.x + box2.width - 1, box2.y + box2.height - 1, {
    steps: 10,
  })
  await page.mouse.up()

  // Measures 0-2 have 4 + 2 + 2 = 8 notes in total.
  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )
  await expect(highlightedNotes).toHaveCount(8)

  await expect(page.locator('button.play-measure-btn')).toHaveText(
    /Selection/,
    { timeout: 3_000 },
  )

  // Dragging a note-range selection pushes a Monaco multicursor selection,
  // whose cursor-change listener debounces (300 ms) into a worker
  // round-trip that swaps the plain SVG documents for highlighted ones —
  // the highlight must survive that swap.
  await page.waitForTimeout(700)
  await expect(highlightedNotes).toHaveCount(8)
})

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
 * The merged run compiles down to a single rest note/rest cell (one
 * `note_id` spanning the whole run — see `group_elements_by_note_id`), not
 * one cell per source measure, so clicking anywhere on the merged bar must
 * select that one cell, whose source span covers all three measures.
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

test('clicking a merged rest bar selects its one merged-run note cell', async ({
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
  await primeMeasureSpans(page)

  const box = await mergedBar.first().boundingBox()
  if (!box) {
    throw new Error('Could not get bounding box for the merged rest bar.')
  }

  // A plain click (mousedown + mouseup at the same point, no drag).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()

  // The merged run is a single rest cell, not 12 separate ones.
  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )
  await expect(highlightedNotes).toHaveCount(1)

  // Unlike the other two tests, the button does NOT switch to "▶ Selection"
  // here: a rest has no source byte span, so `groupSelectedNotesIntoContiguousRuns`
  // (see `noteSpanSelection.ts`) intentionally drops a selection made up
  // entirely of rest cells rather than pushing an empty/unplayable Monaco
  // selection.
  await expect(page.locator('button.play-measure-btn')).not.toHaveText(
    /Selection/,
    { timeout: 3_000 },
  )
})
