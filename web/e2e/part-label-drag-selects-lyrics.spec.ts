import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * Regression test: clicking (or drag-selecting vertically across) a part
 * label is supposed to select every note/rest *and* every lyric syllable
 * that part sounds across the whole system the label sits in — mirroring
 * how 'measure' mode's click/drag resolves both `noteCellsInMeasureRange`
 * and `lyricCellsInMeasureRange` together (see
 * `measure-click-selects-lyrics.spec.ts`).
 *
 * `usePreviewDragSelection.ts`'s `'part-label'` mode used to resolve only
 * `noteCellsForPartLabels` and never a lyric-side counterpart, so a
 * part-label drag silently skipped every lyric row underneath the swept
 * parts. `lyricCellsForPartLabels` (in `previewSelection.ts`) is the fix.
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system" so both measures render in one system and both part labels stay
 * within the viewport. Melody carries lyrics; Harmony doesn't, so the test
 * can also assert Harmony's absence stays a no-op rather than an error.
 *
 * Measure 0: Melody "1 2" + lyrics "do re", Harmony "5 6" (2 notes)
 * Measure 1: Melody "3 4" + lyrics "mi fa", Harmony "7 1'" (2 notes)
 */
const source = [
  '# metadata',
  'title = "part label drag lyric test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes+lyrics',
  'Harmony [H] = notes',
  '',
  '# score',
  '[M] 1 2', // measure 0
  '[M] do re', // verse 0
  '[H] 5 6',
  '',
  '[M] 3 4', // measure 1
  '[M] mi fa', // verse 0
  "[H] 7 1'",
].join('\n')

async function loadFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'part-label-drag-lyric-test.jianpu',
        userFiles: { 'part-label-drag-lyric-test.jianpu': source },
        bin: {},
        fileIds: {
          'part-label-drag-lyric-test.jianpu':
            'part-label-drag-lyric-test-id-001',
        },
      }),
    )
  }, source)
}

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

test('clicking a part label also selects the lyric syllables that part sings across the system', async ({
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
  await expect(melodyLabel).toBeVisible({ timeout: 5_000 })
  const box = await melodyLabel.boundingBox()
  if (!box) throw new Error('Could not get bounding box for the Melody label.')

  // A plain click (mousedown + mouseup at the same point, no drag).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()

  // Melody sounds 4 notes total across both measures ("1 2" + "3 4").
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(4)

  // Melody sings 4 syllables total ("do re" + "mi fa") — these must be
  // selected too, not skipped.
  await expect(
    page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
  ).toHaveCount(4)
})

test('dragging vertically across part labels selects both parts notes and the lyrics under them', async ({
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
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(8)

  // Only Melody carries lyrics (4 syllables); Harmony has none, so the total
  // stays 4 rather than erroring or double-counting.
  await expect(
    page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
  ).toHaveCount(4)
})
