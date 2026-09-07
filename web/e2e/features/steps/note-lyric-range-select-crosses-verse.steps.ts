import { expect } from '@playwright/test'
import {
  clickAndClickSelect,
  stableBoundingBox,
} from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

/**
 * Regression fixture for the same-part `Note ↔ Lyric` click-and-click range
 * gesture when the second click lands on a syllable in a verse *other than*
 * verse 0 — reported live: `[a] 1 2 3 4` with two lyric lines, clicking note
 * "1" then the second verse's third syllable ("g") selected notes 1-3 and
 * only the second verse's syllables, silently dropping the first verse's
 * syllables it visually swept over on the way there.
 *
 * `note_lyric::same_part` (see `selection_range/note_lyric.rs`) used to
 * restrict `lyric_cells` to the `Lyric` endpoint's own verse alone. But a
 * `Note` endpoint has no verse of its own — the note row renders above every
 * verse row — so a sweep from it down to verse `V` always visually crosses
 * verses `0` through `V`, not just row `V` in isolation. This fixture has
 * two verse rows on one measure of four notes, so verse 0's syllables at the
 * swept note-id range must show up as selected too, while each verse's
 * fourth syllable (note id 3, outside the swept range) stays unselected as a
 * distractor proving the fix ranges by `note_id` alongside `verse`, not just
 * unioning every verse wholesale.
 */
const source = [
  '# metadata',
  'title = "note lyric cross verse range test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0 — note ids 0-3
  'a b c d', // verse 0
  'e f g h', // verse 1
].join('\n')

function noteClickTarget(
  page: import('@playwright/test').Page,
  noteId: number,
) {
  return page
    .locator(
      `[data-tag="note"][data-part-index="0"][data-note-id="${noteId}"]:has(rect[data-variant="note-click-target-rect"])`,
    )
    .locator('rect[data-variant="note-click-target-rect"]')
}

function noteAt(page: import('@playwright/test').Page, noteId: number) {
  return page.locator(
    `[data-tag="note"][data-part-index="0"][data-note-id="${noteId}"]:has(rect[data-variant="note-click-target-rect"])`,
  )
}

function lyricAt(
  page: import('@playwright/test').Page,
  verse: number,
  noteId: number,
) {
  return page.locator(
    `[data-tag="lyric"][data-part-index="0"][data-verse="${verse}"][data-note-id="${noteId}"]`,
  )
}

Given(
  'the note-lyric cross-verse range-selection fixture is loaded and both verses have rendered',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'note-lyric-cross-verse-range-test.jianpu',
          userFiles: { 'note-lyric-cross-verse-range-test.jianpu': src },
          bin: {},
          fileIds: {
            'note-lyric-cross-verse-range-test.jianpu':
              'note-lyric-cross-verse-range-test-id-001',
          },
        }),
      )
    }, source)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await expect(
      page.locator('rect[data-variant="note-click-target-rect"]'),
    ).toHaveCount(4, { timeout: 10_000 })
    await expect(
      page.locator('[data-tag="lyric"][data-part-index="0"]'),
    ).toHaveCount(8, { timeout: 10_000 })
    await page.evaluate(() => document.fonts.ready)
    await page.waitForTimeout(200)
  },
)

When(
  "I click-and-click select Melody's note {int} then verse {int}'s syllable {int}",
  async ({ page }, noteId: number, verse: number, lyricNoteId: number) => {
    const noteBox = await stableBoundingBox(noteClickTarget(page, noteId))
    const lyricBox = await stableBoundingBox(lyricAt(page, verse, lyricNoteId))
    if (!noteBox || !lyricBox) {
      throw new Error(
        `Could not get bounding boxes for Melody note ${noteId} and verse ${verse}'s syllable ${lyricNoteId}.`,
      )
    }

    await clickAndClickSelect(
      page,
      noteBox.x + noteBox.width / 2,
      noteBox.y + noteBox.height / 2,
      lyricBox.x + lyricBox.width / 2,
      lyricBox.y + lyricBox.height / 2,
    )
  },
)

Then(
  "Melody's notes {int} through {int} are range-selected",
  async ({ page }, start: number, end: number) => {
    for (let noteId = start; noteId <= end; noteId++) {
      await expect(noteAt(page, noteId)).toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)

Then(
  "verse {int}'s syllables {int} through {int} are range-selected",
  async ({ page }, verse: number, start: number, end: number) => {
    for (let noteId = start; noteId <= end; noteId++) {
      await expect(lyricAt(page, verse, noteId)).toHaveAttribute(
        'data-lyric-range-selected',
        '',
      )
    }
  },
)

Then(
  "verse {int}'s syllable {int} is not range-selected",
  async ({ page }, verse: number, noteId: number) => {
    await expect(lyricAt(page, verse, noteId)).not.toHaveAttribute(
      'data-lyric-range-selected',
      '',
    )
  },
)
