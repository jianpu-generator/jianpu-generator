import { expect, test } from '@playwright/test'

/**
 * A lyric syllable has its own `Tag::Lyric` click-target rect
 * (`data-tag="lyric"`, see `render_lyric_click_target` in
 * `src/renderer/new_renderer.rs`), sized to its own column and resolved
 * after — so painted on top of, for `elementFromPoint` hit-testing purposes
 * — the wider `NoteClickTarget` rect that also covers that note's lyric row
 * (see `resolve_click_target_elements` in `src/coordinate_resolver/resolve.rs`).
 * A click landing on or near the lyric glyph's ink therefore resolves to the
 * lyric's own selection, independent of the note it belongs to — see
 * `lyric-syllable-independent-selection.spec.ts` for the fuller independence
 * matrix (note-click vs. lyric-click, multi-verse, Monaco sync).
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system" and four single-beat notes with one syllable each, so all four
 * note/lyric pairs render side by side in one row and stay within the
 * viewport during the drag.
 */
const dragTestSource = [
  '# metadata',
  'title = "lyric drag test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes+lyrics',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0 — line 9
  '[M] do re mi fa', // line 10
].join('\n')

async function loadDragTestFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'lyric-drag-test.jianpu',
        userFiles: { 'lyric-drag-test.jianpu': source },
        bin: {},
        fileIds: { 'lyric-drag-test.jianpu': 'lyric-drag-test-id-001' },
      }),
    )
  }, dragTestSource)
}

test('dragging a marquee across lyric syllables selects the syllables, not their underlying notes', async ({
  page,
}) => {
  await loadDragTestFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="0"]', {
    timeout: 10_000,
  })

  // Lyric syllables are the only text glyphs rendered with a "hanging"
  // baseline (see `render_lyric`), so this selector picks them out reliably
  // regardless of their actual text content.
  const lyricTexts = page.locator('svg text[dominant-baseline="hanging"]')
  await expect(lyricTexts).toHaveCount(4, { timeout: 10_000 })

  const box0 = await lyricTexts.nth(0).boundingBox() // "do", under note 0
  const box2 = await lyricTexts.nth(2).boundingBox() // "mi", under note 2
  if (!box0 || !box2) {
    throw new Error('Could not get bounding boxes for lyric syllables 0 and 2.')
  }

  const startX = box0.x + box0.width / 2
  const startY = box0.y + box0.height / 2
  const endX = box2.x + box2.width / 2
  const endY = box2.y + box2.height / 2

  // Drag a marquee across the first three syllables ("do", "re", "mi").
  await page.mouse.move(startX, startY)
  await page.mouse.down()
  await page.mouse.move(endX, endY, { steps: 10 })
  await page.mouse.up()

  // The drag resolves through the syllables' own lyric click targets —
  // lyric selection is independent of note selection, so no note cell gets
  // highlighted by this drag at all.
  const highlightedLyrics = page.locator(
    '[data-tag="lyric"][data-lyric-drag-selected]',
  )
  await expect(highlightedLyrics).toHaveCount(3)
  for (const noteId of [0, 1, 2]) {
    await expect(
      page.locator(
        `[data-tag="lyric"][data-lyric-drag-selected][data-note-id="${noteId}"]`,
      ),
    ).toHaveCount(1)
  }
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(0)
})

test('clicking a single lyric syllable selects only that syllable, not the note', async ({
  page,
}) => {
  await loadDragTestFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="0"]', {
    timeout: 10_000,
  })

  const lyricTexts = page.locator('svg text[dominant-baseline="hanging"]')
  await expect(lyricTexts).toHaveCount(4, { timeout: 10_000 })

  // A plain click (mousedown + mouseup at the same point, no drag) selects
  // just this syllable — unlike a plain click on the note glyph itself,
  // which selects the whole measure (see `measure-click-selects-notes.spec.ts`).
  const box = await lyricTexts.nth(1).boundingBox() // "re", under note 1
  if (!box) throw new Error('no box')
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.waitForTimeout(50)
  await page.mouse.up()

  await expect(
    page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
  ).toHaveCount(1)
  await expect(
    page.locator(
      '[data-tag="lyric"][data-lyric-drag-selected][data-note-id="1"]',
    ),
  ).toHaveCount(1)
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(0)
})
