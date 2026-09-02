import { expect } from '@playwright/test'
import { clickAndClickSelect } from '../../dragSelectHelpers'
import { Given, Then, When } from './fixtures'

/**
 * A lyric syllable has its own `Tag::Lyric` click-target rect
 * (`data-tag="lyric"`, see `render_lyric_click_target` in
 * `src/renderer/new_renderer.rs`), sized to its own column and resolved
 * after — so painted on top of, for `elementFromPoint` hit-testing purposes
 * — the wider `NoteClickTarget` rect that also covers that note's lyric row
 * (see `resolve_click_target_elements` in `src/coordinate_resolver/resolve.rs`).
 * A click landing on or near the lyric glyph's ink therefore resolves to the
 * lyric's own selection, independent of the note it belongs to — see
 * `lyric-syllable-independent-selection.feature` for the fuller independence
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
  'Melody [M] = notes',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0 — line 9
  'do re mi fa', // line 10
].join('\n')

function lyricTexts(page: import('@playwright/test').Page) {
  // Lyric syllables are the only text glyphs tagged with the "lyric" data
  // variant (see `render_lyric`), so this selector picks them out reliably
  // regardless of their actual text content.
  return page.locator('svg text[data-variant="lyric"]')
}

Given(
  'the lyric drag test fixture is loaded and the first measure has rendered',
  async ({ page }) => {
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
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await page.waitForSelector('[data-tag="measure"][data-measure-index="0"]', {
      timeout: 10_000,
    })

    await expect(lyricTexts(page)).toHaveCount(4, { timeout: 10_000 })
  },
)

When(
  'I drag a marquee from lyric syllable {int} to lyric syllable {int}',
  async ({ page }, from: number, to: number) => {
    const boxFrom = await lyricTexts(page).nth(from).boundingBox()
    const boxTo = await lyricTexts(page).nth(to).boundingBox()
    if (!boxFrom || !boxTo) {
      throw new Error(
        `Could not get bounding boxes for lyric syllables ${from} and ${to}.`,
      )
    }

    const startX = boxFrom.x + boxFrom.width / 2
    const startY = boxFrom.y + boxFrom.height / 2
    const endX = boxTo.x + boxTo.width / 2
    const endY = boxTo.y + boxTo.height / 2

    // Click-and-click a marquee across the syllables.
    await clickAndClickSelect(page, startX, startY, endX, endY)
  },
)

When(
  'I click lyric syllable {int} without dragging',
  async ({ page }, index: number) => {
    // A plain click (mousedown + mouseup at the same point, no drag) selects
    // just this syllable.
    const box = await lyricTexts(page).nth(index).boundingBox()
    if (!box) throw new Error('no box')
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
    await page.mouse.down()
    await page.waitForTimeout(50)
    await page.mouse.up()
  },
)

Then(
  'lyric syllables {int}, {int} and {int} are drag-selected',
  async ({ page }, a: number, b: number, c: number) => {
    // The drag resolves through the syllables' own lyric click targets —
    // lyric selection is independent of note selection, so no note cell gets
    // highlighted by this drag at all.
    const highlightedLyrics = page.locator(
      '[data-tag="lyric"][data-lyric-drag-selected]',
    )
    await expect(highlightedLyrics).toHaveCount(3)
    for (const noteId of [a, b, c]) {
      await expect(
        page.locator(
          `[data-tag="lyric"][data-lyric-drag-selected][data-note-id="${noteId}"]`,
        ),
      ).toHaveCount(1)
    }
  },
)

Then(
  'only lyric syllable {int} is drag-selected',
  async ({ page }, noteId: number) => {
    await expect(
      page.locator('[data-tag="lyric"][data-lyric-drag-selected]'),
    ).toHaveCount(1)
    await expect(
      page.locator(
        `[data-tag="lyric"][data-lyric-drag-selected][data-note-id="${noteId}"]`,
      ),
    ).toHaveCount(1)
  },
)

Then('no note is drag-selected', async ({ page }) => {
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(0)
})
