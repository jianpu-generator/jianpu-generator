import { expect, test } from '@playwright/test'

/**
 * Each lyric syllable gets its own `Tag::Lyric` click target
 * (`data-tag="lyric"`, `data-part-index`/`data-note-id`/`data-verse`), kept
 * independent of the note-selection stack (`Tag::Note`,
 * `[data-tag="note"][data-note-drag-selected]`) for a *syllable-level*
 * click/drag — see `useLyricSelection.ts` and `Preview.tsx`'s
 * `onLyricRangeSelect`/`selectedLyricCells`. This spec covers the
 * cross-cutting independence matrix: a syllable-level lyric click/drag never
 * touches note selection, a lyric drag's Monaco selection matches the
 * dragged source text, and separate verse rows select independently — except
 * a *measure*-level click/drag (on a note or the space around it), which is
 * a shortcut that intentionally selects both notes and every verse's lyrics
 * in that measure at once (see `Preview.tsx`'s `onMeasureRangeSelect`).
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system", one measure of four single-beat notes, and two verses so both
 * single-verse and multi-verse independence can be exercised from one
 * fixture.
 */
const multiVerseSource = [
  '# metadata',
  'title = "lyric independent selection test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes+lyrics',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0 — line 9
  '[M] do re mi fa', // verse 0 — line 10
  '[M] uno dos tres cuatro', // verse 1 — line 11
].join('\n')

async function loadFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'lyric-independent-test.jianpu',
        userFiles: { 'lyric-independent-test.jianpu': source },
        bin: {},
        fileIds: {
          'lyric-independent-test.jianpu': 'lyric-independent-test-id-001',
        },
      }),
    )
  }, multiVerseSource)
}

async function ready(page: import('@playwright/test').Page) {
  await loadFixture(page)
  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="0"]', {
    timeout: 10_000,
  })
  // Wait for both verses' click targets (4 syllables x 2 verses) to render.
  await expect(page.locator('[data-tag="lyric"]')).toHaveCount(8, {
    timeout: 10_000,
  })
  // Let layout fully settle before any bounding-box reads below — under
  // parallel test-suite load, the count above can pass a beat before the
  // two-verse layout has stopped shifting.
  await page.waitForTimeout(200)
}

function lyricRect(
  page: import('@playwright/test').Page,
  noteId: number,
  verse: number,
) {
  return page
    .locator(
      `[data-tag="lyric"][data-note-id="${noteId}"][data-verse="${verse}"]`,
    )
    .locator('rect')
}

test('clicking one syllable selects only that syllable, no notes', async ({
  page,
}) => {
  await ready(page)

  const box = await lyricRect(page, 1, 0).boundingBox()
  if (!box) throw new Error('no box')
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.waitForTimeout(50)
  await page.mouse.up()
  await page.waitForTimeout(100)

  await expect(
    page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
  ).toHaveCount(1)
  await expect(
    page.locator(
      '[data-tag="lyric"][data-lyric-drag-selected][data-note-id="1"][data-verse="0"]',
    ),
  ).toHaveCount(1)
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(0)
})

test('dragging across syllables selects exactly those cells and the matching editor text', async ({
  page,
}) => {
  await ready(page)

  const start = await lyricRect(page, 0, 0).boundingBox()
  const end = await lyricRect(page, 2, 0).boundingBox()
  if (!start || !end) throw new Error('no box')

  await page.mouse.move(start.x + start.width / 2, start.y + start.height / 2)
  await page.mouse.down()
  await page.mouse.move(end.x + end.width / 2, end.y + end.height / 2, {
    steps: 10,
  })
  await page.mouse.up()
  await page.waitForTimeout(200)

  await expect(
    page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
  ).toHaveCount(3)
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(0)

  const selectedText = await page.evaluate(() => {
    const ed = window.monaco.editor.getEditors()[0]
    const model = ed.getModel()
    return model?.getValueInRange(ed.getSelection())
  })
  expect(selectedText).toBe('do re mi')
})

test('clicking a note directly selects the whole measure, notes and every verse of lyrics alike', async ({
  page,
}) => {
  await ready(page)

  // The note's click target row is widened to also cover both verse rows
  // beneath it (see `part_row_ranges`), so its vertical center can land
  // inside a lyric row once there's more than one verse. Click near the top
  // of the rect instead — solidly inside the note glyph's own zone, above
  // where the lyric rows start.
  const noteRect = page
    .locator('[data-tag="note"][data-note-id="1"]')
    .locator('rect[data-variant="note-click-target-rect"]')
  const box = await noteRect.boundingBox()
  if (!box) throw new Error('no box')
  await page.mouse.move(box.x + box.width / 2, box.y + box.height * 0.15)
  await page.mouse.down()
  await page.waitForTimeout(50)
  await page.mouse.up()
  await page.waitForTimeout(100)

  // A plain click on a note selects the whole measure (existing behavior) —
  // and, alongside its 4 notes, every syllable of every verse in that
  // measure (4 syllables x 2 verses), not just the note's own row (see
  // `previewSelection.ts`'s `lyricCellsInMeasureRange`).
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(4)
  await expect(
    page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
  ).toHaveCount(8)
})

test('verses select independently and each syllable maps to its own verse line', async ({
  page,
}) => {
  await ready(page)

  // Click syllable 1 of verse 1 ("dos").
  const box = await lyricRect(page, 1, 1).boundingBox()
  if (!box) throw new Error('no box')
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.waitForTimeout(50)
  await page.mouse.up()
  await page.waitForTimeout(100)

  await expect(
    page.locator(
      '[data-tag="lyric"][data-lyric-drag-selected][data-note-id="1"][data-verse="1"]',
    ),
  ).toHaveCount(1)
  // Verse 0's corresponding syllable must not also be marked selected.
  await expect(
    page.locator(
      '[data-tag="lyric"][data-lyric-drag-selected][data-note-id="1"][data-verse="0"]',
    ),
  ).toHaveCount(0)

  const selectedText = await page.evaluate(() => {
    const ed = window.monaco.editor.getEditors()[0]
    const model = ed.getModel()
    return model?.getValueInRange(ed.getSelection())
  })
  expect(selectedText).toBe('dos')
})
