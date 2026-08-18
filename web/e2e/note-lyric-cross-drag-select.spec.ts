import { expect, test } from '@playwright/test'

/**
 * Regression test for a marquee drag that starts on one cell type (note or
 * lyric) and is dragged across the other type never selecting that other
 * type at all.
 *
 * `PreviewDragState` (see `usePreviewDragSelection.ts`) is a discriminated
 * union that commits to exactly one mode on mousedown — a mousedown that
 * lands on a note arms `'note'` mode (via `'pending'`, see `Preview.tsx`),
 * and a mousedown that lands on a lyric syllable arms `'lyric'` mode. For
 * the rest of the gesture, `usePreviewDragSelection.ts`'s
 * `handleMouseMove`/`handleMouseUp` call only that one mode's highlighter —
 * `applyNoteDragHighlights` for `'note'` mode, `applyLyricDragHighlights`
 * for `'lyric'` mode — never both.
 *
 * That's a real behavioral gap: dragging a marquee that starts on a NOTE
 * downward, so it visually covers the lyric syllables underneath, does NOT
 * select those syllables. The symmetric case — starting on a LYRIC syllable
 * and dragging up over the notes above it — does not select those notes
 * either.
 *
 * Contrast with `'measure'` mode (a drag starting on empty space or a bare
 * bar line), which unions both: its move/up handlers call
 * `noteCellsInMeasureRange` and `lyricCellsInMeasureRange` together and
 * apply both highlight sets (see `usePreviewDragSelection.ts`).
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system" and four single-beat notes with one syllable each, so all four
 * note/lyric pairs render side by side in one row and stay within the
 * viewport during the drag — same fixture shape as
 * `lyric-drag-select-highlight.spec.ts`.
 */
const dragTestSource = [
  '# metadata',
  'title = "note lyric cross drag test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes+lyrics',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0 — line 9
  '[M] do re mi fa', // verse 0 — line 10
].join('\n')

async function loadDragTestFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'note-lyric-cross-drag-test.jianpu',
        userFiles: { 'note-lyric-cross-drag-test.jianpu': source },
        bin: {},
        fileIds: {
          'note-lyric-cross-drag-test.jianpu':
            'note-lyric-cross-drag-test-id-001',
        },
      }),
    )
  }, dragTestSource)
}

test('dragging a marquee that starts on a note down across lyric syllables also selects those syllables', async ({
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

  const noteRects = page.locator('rect[data-variant="note-click-target-rect"]')
  await expect(noteRects).toHaveCount(4, { timeout: 10_000 })
  const lyricTexts = page.locator('svg text[dominant-baseline="hanging"]')
  await expect(lyricTexts).toHaveCount(4, { timeout: 10_000 })
  // Let layout fully settle (e.g. web-font metrics finishing load can still
  // reflow the note/lyric rows' vertical position after the counts above
  // are already satisfied) before reading any bounding boxes below — this
  // drag crosses the note/lyric row boundary, so it's sensitive to exactly
  // where that boundary lands, unlike a same-row drag. Same fixture-settle
  // pattern as `lyric-syllable-independent-selection.spec.ts`'s `ready()`.
  await page.waitForTimeout(200)

  // Anchor the drag on note 0's own click-target rect, and drag down/across
  // to lyric syllable 2 ("mi") — a marquee that visually spans notes 0-2 and
  // their lyric row underneath.
  const noteBox0 = await noteRects.nth(0).boundingBox()
  const lyricBox2 = await lyricTexts.nth(2).boundingBox()
  if (!noteBox0 || !lyricBox2) {
    throw new Error(
      'Could not get bounding boxes for note 0 and lyric syllable 2.',
    )
  }

  const startX = noteBox0.x + noteBox0.width / 2
  const startY = noteBox0.y + noteBox0.height / 2
  const endX = lyricBox2.x + lyricBox2.width / 2
  const endY = lyricBox2.y + lyricBox2.height / 2

  await page.mouse.move(startX, startY)
  await page.mouse.down()
  // Past the note-drag arm threshold and down into the lyric row, in
  // several steps so the marquee's bounding box genuinely sweeps over both
  // rows rather than jumping straight to the end point.
  await page.mouse.move(startX, endY, { steps: 5 })
  await page.mouse.move(endX, endY, { steps: 5 })
  await page.mouse.up()

  // The drag started on a note, so note mode is armed and notes 0-2 get
  // selected as expected.
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(3)

  // Bug: the marquee also visually covers lyric syllables 0-2 ("do", "re",
  // "mi"), but since the drag is locked into 'note' mode, no lyric cell
  // ever gets marked as drag-selected.
  await expect(
    page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
  ).toHaveCount(3)
})

test('dragging a marquee that starts on a lyric syllable up across notes also selects those notes', async ({
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

  const noteRects = page.locator('rect[data-variant="note-click-target-rect"]')
  await expect(noteRects).toHaveCount(4, { timeout: 10_000 })
  const lyricTexts = page.locator('svg text[dominant-baseline="hanging"]')
  await expect(lyricTexts).toHaveCount(4, { timeout: 10_000 })
  // Let layout fully settle (e.g. web-font metrics finishing load can still
  // reflow the note/lyric rows' vertical position after the counts above
  // are already satisfied) before reading any bounding boxes below — this
  // drag crosses the note/lyric row boundary, so it's sensitive to exactly
  // where that boundary lands, unlike a same-row drag. Same fixture-settle
  // pattern as `lyric-syllable-independent-selection.spec.ts`'s `ready()`.
  await page.waitForTimeout(200)

  // Anchor the drag on lyric syllable 0 ("do"), and drag up/across to note
  // 2's click-target rect — the symmetric case, a marquee that visually
  // spans the lyric row and notes 0-2 above it.
  const lyricBox0 = await lyricTexts.nth(0).boundingBox()
  const noteBox2 = await noteRects.nth(2).boundingBox()
  if (!lyricBox0 || !noteBox2) {
    throw new Error(
      'Could not get bounding boxes for lyric syllable 0 and note 2.',
    )
  }

  const startX = lyricBox0.x + lyricBox0.width / 2
  const startY = lyricBox0.y + lyricBox0.height / 2
  const endX = noteBox2.x + noteBox2.width / 2
  const endY = noteBox2.y + noteBox2.height / 2

  await page.mouse.move(startX, startY)
  await page.mouse.down()
  await page.mouse.move(startX, endY, { steps: 5 })
  await page.mouse.move(endX, endY, { steps: 5 })
  await page.mouse.up()

  // The drag started on a lyric syllable, so lyric mode is armed and
  // syllables 0-2 get selected as expected.
  await expect(
    page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
  ).toHaveCount(3)

  // Bug: the marquee also visually covers notes 0-2 above the lyric row,
  // but since the drag is locked into 'lyric' mode, no note cell ever gets
  // marked as drag-selected.
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(3)
})
