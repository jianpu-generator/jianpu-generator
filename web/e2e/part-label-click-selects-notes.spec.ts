import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * Clicking (or drag-selecting vertically across) a part label — the
 * abbreviation drawn once per system at the label region's left edge — is a
 * shortcut for selecting every note/rest that part sounds across the whole
 * system the label sits in (see `Preview.tsx`'s `getPartLabelAtPoint`/
 * `noteCellsForPartLabels`). It reuses the same note drag-select highlight
 * (`[data-note-drag-selected]`) and Monaco multicursor pathway
 * (`onNoteRangeSelect`) that dragging a marquee over individual notes uses.
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system" so both measures render in one system and both part labels stay
 * within the viewport.
 *
 * Measure 0: Melody "1 2" (2 notes), Harmony "5 6" (2 notes)
 * Measure 1: Melody "3 4" (2 notes), Harmony "7 1'" (2 notes)
 */
const source = [
  '# metadata',
  'title = "part label click test"',
  'max_measures_per_system = 48',
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
        active: 'part-label-click-test.jianpu',
        userFiles: { 'part-label-click-test.jianpu': source },
        bin: {},
        fileIds: {
          'part-label-click-test.jianpu': 'part-label-click-test-id-001',
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

test('clicking a part label selects every note that part sounds across the system', async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="part-label"][data-part-index="0"]', {
    timeout: 10_000,
  })
  await primeMeasureSpans(page)

  const melodyLabel = page
    .locator('[data-tag="part-label"][data-part-index="0"]')
    .first()
  const harmonyLabel = page
    .locator('[data-tag="part-label"][data-part-index="1"]')
    .first()
  await expect(melodyLabel).toBeVisible({ timeout: 5_000 })
  const box = await melodyLabel.boundingBox()
  if (!box) throw new Error('Could not get bounding box for the Melody label.')

  // A plain click (mousedown + mouseup at the same point, no drag).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()

  // Melody sounds 4 notes total across both measures ("1 2" + "3 4"); none of
  // Harmony's notes should be selected.
  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )
  await expect(highlightedNotes).toHaveCount(4)
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="0"]',
    ),
  ).toHaveCount(4)
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="1"]',
    ),
  ).toHaveCount(0)

  await expect(page.locator('button.play-measure-btn')).toHaveText(
    /Selection/,
    { timeout: 3_000 },
  )

  // The clicked label stays visually selected after mouseup; the untouched
  // one never was.
  await expect(
    melodyLabel.locator('rect[data-variant="part-label-click-target-rect"]'),
  ).toHaveAttribute('data-part-label-drag-active', '')
  await expect(
    harmonyLabel.locator('rect[data-variant="part-label-click-target-rect"]'),
  ).not.toHaveAttribute('data-part-label-drag-active', '')
})

test('a plain click on a notes+lyrics part label does not also select the lyric row', async ({
  page,
}) => {
  // Regression test: 'part-label' drag-selection unions in the lyric row
  // underneath the swept part(s) — a real feature for drags (see
  // `part-label-drag-selects-lyrics.spec.ts`) — but a plain click (zero
  // pointer movement) used to go through that exact same code path and
  // incorrectly pick up the lyric row too.
  const lyricSource = [
    '# metadata',
    'title = "part label click no-lyric test"',
    'max_measures_per_system = 48',
    '',
    '# parts',
    'Melody [M] = notes+lyrics',
    '',
    '# score',
    '[M] 1 2', // measure 0
    '[M] do re', // verse 0
    '',
    '[M] 3 4', // measure 1
    '[M] mi fa', // verse 0
  ].join('\n')

  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'part-label-click-no-lyric-test.jianpu',
        userFiles: { 'part-label-click-no-lyric-test.jianpu': source },
        bin: {},
        fileIds: {
          'part-label-click-no-lyric-test.jianpu':
            'part-label-click-no-lyric-test-id-001',
        },
      }),
    )
  }, lyricSource)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="part-label"][data-part-index="0"]', {
    timeout: 10_000,
  })
  await primeMeasureSpans(page)

  const melodyLabel = page
    .locator('[data-tag="part-label"][data-part-index="0"]')
    .first()
  await expect(melodyLabel).toBeVisible({ timeout: 5_000 })
  const box = await melodyLabel.boundingBox()
  if (!box) throw new Error('Could not get bounding box for the Melody label.')

  // A plain click (mousedown + mouseup at the same point, no drag).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()

  // Melody's 4 notes get selected, same as the notes-only fixture above...
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(4)
  // ...but none of the 4 lyric syllables should be, since this was a click,
  // not a drag.
  await expect(
    page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
  ).toHaveCount(0)
})

test('dragging from one part label to another selects both parts notes', async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="part-label"][data-part-index="1"]', {
    timeout: 10_000,
  })
  await primeMeasureSpans(page)

  const melodyLabel = page
    .locator('[data-tag="part-label"][data-part-index="0"]')
    .first()
  const harmonyLabel = page
    .locator('[data-tag="part-label"][data-part-index="1"]')
    .first()
  await expect(melodyLabel).toBeVisible({ timeout: 5_000 })
  await expect(harmonyLabel).toBeVisible({ timeout: 5_000 })

  const melodyBox = await melodyLabel.boundingBox()
  const harmonyBox = await harmonyLabel.boundingBox()
  if (!melodyBox || !harmonyBox) {
    throw new Error('Could not get bounding boxes for the part labels.')
  }

  await page.mouse.move(
    melodyBox.x + melodyBox.width / 2,
    melodyBox.y + melodyBox.height / 2,
  )
  await page.mouse.down()
  await page.mouse.move(
    harmonyBox.x + harmonyBox.width / 2,
    harmonyBox.y + harmonyBox.height / 2,
    { steps: 10 },
  )
  await page.mouse.up()

  // Melody's 4 notes + Harmony's 4 notes = 8.
  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )
  await expect(highlightedNotes).toHaveCount(8)

  // Both dragged-over labels — not just the one the drag started on — stay
  // visually selected once the drag ends.
  const melodyRect = melodyLabel.locator(
    'rect[data-variant="part-label-click-target-rect"]',
  )
  const harmonyRect = harmonyLabel.locator(
    'rect[data-variant="part-label-click-target-rect"]',
  )
  await expect(melodyRect).toHaveAttribute('data-part-label-drag-active', '')
  await expect(harmonyRect).toHaveAttribute('data-part-label-drag-active', '')

  await expect(page.locator('button.play-measure-btn')).toHaveText(
    /Selection/,
    { timeout: 3_000 },
  )

  // Dragging a note-range selection pushes a Monaco multicursor selection,
  // whose cursor-change listener debounces (300 ms) into a worker
  // round-trip that swaps the plain SVG documents for highlighted ones —
  // the highlight (both the notes' and the part labels') must survive that
  // swap.
  await page.waitForTimeout(700)
  await expect(highlightedNotes).toHaveCount(8)
  await expect(melodyRect).toHaveAttribute('data-part-label-drag-active', '')
  await expect(harmonyRect).toHaveAttribute('data-part-label-drag-active', '')
})

test('the part label a drag started on stays visually hovered once the pointer moves onto another label', async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="part-label"][data-part-index="1"]', {
    timeout: 10_000,
  })
  await primeMeasureSpans(page)

  const melodyLabel = page
    .locator('[data-tag="part-label"][data-part-index="0"]')
    .first()
  const harmonyLabel = page
    .locator('[data-tag="part-label"][data-part-index="1"]')
    .first()
  await expect(melodyLabel).toBeVisible({ timeout: 5_000 })
  await expect(harmonyLabel).toBeVisible({ timeout: 5_000 })

  const melodyRect = melodyLabel.locator(
    'rect[data-variant="part-label-click-target-rect"]',
  )

  const melodyBox = await melodyLabel.boundingBox()
  const harmonyBox = await harmonyLabel.boundingBox()
  if (!melodyBox || !harmonyBox) {
    throw new Error('Could not get bounding boxes for the part labels.')
  }

  // Hover the Melody label (no drag yet) and record the fill its `:hover`
  // rule paints — this is the "hovered" look the dragged-from label must
  // keep showing for the rest of the gesture.
  await page.mouse.move(
    melodyBox.x + melodyBox.width / 2,
    melodyBox.y + melodyBox.height / 2,
  )
  const hoveredFill = await melodyRect.evaluate(
    (el) => getComputedStyle(el).fill,
  )
  expect(hoveredFill).not.toBe('none')
  expect(hoveredFill).not.toMatch(/^rgba\(0, ?0, ?0, ?0\)$/)

  // Start the drag on Melody, then move the pointer onto Harmony — Melody
  // is the label the drag started on, and per the vertical-drag part-label
  // shortcut it stays part of the selection while the pointer is anywhere
  // in the column, not just while literally over its own rect.
  await page.mouse.down()
  await page.mouse.move(
    harmonyBox.x + harmonyBox.width / 2,
    harmonyBox.y + harmonyBox.height / 2,
    { steps: 10 },
  )

  const fillWhileDraggingAway = await melodyRect.evaluate(
    (el) => getComputedStyle(el).fill,
  )
  expect(fillWhileDraggingAway).toBe(hoveredFill)

  await page.mouse.up()
})
