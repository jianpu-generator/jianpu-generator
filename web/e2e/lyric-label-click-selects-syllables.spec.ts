import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * Clicking (or drag-selecting vertically across) a lyric-verse label — the
 * abbreviation drawn at the label region's left edge on each verse's own
 * row, e.g. "M:v1" for Melody's first verse — is a shortcut for selecting
 * every syllable that verse sings across the whole system the label sits
 * in (see `Preview.tsx`'s `getLyricLabelAtPoint`/`lyricCellsForLyricLabels`).
 * The lyric-side mirror of `part-label-click-selects-notes.spec.ts`.
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system" so both measures render in one system and every label stays
 * within the viewport.
 *
 * Measure 0: Melody "1 2", verse 1 "do re", verse 2 "fa sol"
 * Measure 1: Melody "3 4", verse 1 "la ti", verse 2 "da di"
 */
const source = [
  '# metadata',
  'title = "lyric label click test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes+lyrics',
  '',
  '# score',
  '[M] 1 2', // measure 0
  '[M] do re', // verse 0
  '[M] fa sol', // verse 1
  '',
  '[M] 3 4', // measure 1
  '[M] la ti', // verse 0
  '[M] da di', // verse 1
].join('\n')

async function loadFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'lyric-label-click-test.jianpu',
        userFiles: { 'lyric-label-click-test.jianpu': source },
        bin: {},
        fileIds: {
          'lyric-label-click-test.jianpu': 'lyric-label-click-test-id-001',
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

test('clicking a verse label selects every syllable that verse sings across the system', async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(
    '[data-tag="lyric-label"][data-part-index="0"][data-verse="0"]',
    { timeout: 10_000 },
  )
  await primeMeasureSpans(page)

  const verse1Label = page
    .locator('[data-tag="lyric-label"][data-part-index="0"][data-verse="0"]')
    .first()
  const verse2Label = page
    .locator('[data-tag="lyric-label"][data-part-index="0"][data-verse="1"]')
    .first()
  await expect(verse1Label).toBeVisible({ timeout: 5_000 })
  const box = await verse1Label.boundingBox()
  if (!box) throw new Error('Could not get bounding box for the verse 1 label.')

  // A plain click (mousedown + mouseup at the same point, no drag).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()

  // Verse 1 sounds 4 syllables total across both measures ("do re" + "la
  // ti"); none of verse 2's syllables should be selected.
  const highlightedLyrics = page.locator(
    '[data-tag="lyric"][data-lyric-drag-selected]',
  )
  await expect(highlightedLyrics).toHaveCount(4)
  await expect(
    page.locator(
      '[data-tag="lyric"][data-lyric-drag-selected][data-verse="0"]',
    ),
  ).toHaveCount(4)
  await expect(
    page.locator(
      '[data-tag="lyric"][data-lyric-drag-selected][data-verse="1"]',
    ),
  ).toHaveCount(0)

  // The clicked label stays visually selected after mouseup; the untouched
  // one never was.
  await expect(
    verse1Label.locator('rect[data-variant="lyric-label-click-target-rect"]'),
  ).toHaveAttribute('data-lyric-label-drag-active', '')
  await expect(
    verse2Label.locator('rect[data-variant="lyric-label-click-target-rect"]'),
  ).not.toHaveAttribute('data-lyric-label-drag-active', '')
})

test('dragging from one verse label to another selects both verses syllables', async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(
    '[data-tag="lyric-label"][data-part-index="0"][data-verse="1"]',
    { timeout: 10_000 },
  )
  await primeMeasureSpans(page)

  const verse1Label = page
    .locator('[data-tag="lyric-label"][data-part-index="0"][data-verse="0"]')
    .first()
  const verse2Label = page
    .locator('[data-tag="lyric-label"][data-part-index="0"][data-verse="1"]')
    .first()
  await expect(verse1Label).toBeVisible({ timeout: 5_000 })
  await expect(verse2Label).toBeVisible({ timeout: 5_000 })

  const verse1Box = await verse1Label.boundingBox()
  const verse2Box = await verse2Label.boundingBox()
  if (!verse1Box || !verse2Box) {
    throw new Error('Could not get bounding boxes for the verse labels.')
  }

  await page.mouse.move(
    verse1Box.x + verse1Box.width / 2,
    verse1Box.y + verse1Box.height / 2,
  )
  await page.mouse.down()
  await page.mouse.move(
    verse2Box.x + verse2Box.width / 2,
    verse2Box.y + verse2Box.height / 2,
    { steps: 10 },
  )
  await page.mouse.up()

  // Verse 1's 4 syllables + verse 2's 4 syllables = 8.
  const highlightedLyrics = page.locator(
    '[data-tag="lyric"][data-lyric-drag-selected]',
  )
  await expect(highlightedLyrics).toHaveCount(8)

  const verse1Rect = verse1Label.locator(
    'rect[data-variant="lyric-label-click-target-rect"]',
  )
  const verse2Rect = verse2Label.locator(
    'rect[data-variant="lyric-label-click-target-rect"]',
  )
  await expect(verse1Rect).toHaveAttribute('data-lyric-label-drag-active', '')
  await expect(verse2Rect).toHaveAttribute('data-lyric-label-drag-active', '')
})
