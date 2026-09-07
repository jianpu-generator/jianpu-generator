import { expect } from '@playwright/test'
import {
  clickAndClickSelect,
  stableBoundingBox,
} from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

/**
 * Regression fixture for the cross-part `Note ↔ Lyric` click-and-click
 * range gesture when both clicks land in the *same* measure — the
 * `Note ↔ Lyric` counterpart to
 * `note-range-select-crosses-part-within-measure.feature`'s `Note ↔ Note`
 * case. `cross_part` in
 * `crates/jianpu-wasm/src/selection_range/note_lyric.rs` still ranges only
 * by `measure_index`, so two clicks landing in the same measure on
 * different parts (e.g. note 0 of `1 2 3`/"do re mi" on one part, syllable
 * 1 of `4 5 6`/"fa so la" on another) wrongly selects that whole measure's
 * notes and syllables on both sides, regardless of where in the measure
 * either click actually landed.
 *
 * Two parts, three notes and three syllables each, all in measure 0.
 * Anchoring on Melody's first note (position 0) and committing on
 * Harmony's second syllable (position 1) should select positions 0-1 in
 * both parts/rows — Melody's notes 0-1 and syllables "do"/"re", Harmony's
 * notes 0-1 and syllables "fa"/"so" — leaving each part's third note/
 * syllable (position 2) unselected.
 */
const source = [
  '# metadata',
  'title = "cross part within measure note lyric range test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes',
  'Harmony [H] = notes',
  '',
  '# score',
  '[M] 1 2 3', // measure 0 — Melody notes at position 0, 1, 2
  'do re mi', // Melody verse 0
  '[H] 4 5 6', // measure 0 — Harmony notes at position 0, 1, 2
  'fa so la', // Harmony verse 0
].join('\n')

// A note renders as two sibling `[data-tag="note"]` groups sharing the same
// `data-part-index`/`data-note-id` — one wrapping the (pointer-events: none)
// playback-cursor rect, one wrapping the click-target rect (see
// `applyPersistedNoteHighlights`'s doc comment in
// `previewRangeHighlights.ts`) — and only the click-target group ever gets
// marked `data-note-range-selected`. Scope to the one that `:has()` the
// click-target rect so the locator resolves to exactly one element.
function noteAt(
  page: import('@playwright/test').Page,
  partIndex: number,
  noteId: number,
) {
  return page.locator(
    `[data-tag="note"][data-part-index="${partIndex}"][data-note-id="${noteId}"]:has(rect[data-variant="note-click-target-rect"])`,
  )
}

function noteClickTarget(
  page: import('@playwright/test').Page,
  partIndex: number,
  noteId: number,
) {
  return noteAt(page, partIndex, noteId).locator(
    'rect[data-variant="note-click-target-rect"]',
  )
}

function lyricAt(
  page: import('@playwright/test').Page,
  partIndex: number,
  noteId: number,
  verse: number,
) {
  return page.locator(
    `[data-tag="lyric"][data-part-index="${partIndex}"][data-note-id="${noteId}"][data-verse="${verse}"]`,
  )
}

Given(
  'the cross-part-within-measure note-lyric range-selection fixture is loaded and both rows have rendered',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'cross-part-within-measure-note-lyric-range-test.jianpu',
          userFiles: {
            'cross-part-within-measure-note-lyric-range-test.jianpu': src,
          },
          bin: {},
          fileIds: {
            'cross-part-within-measure-note-lyric-range-test.jianpu':
              'cross-part-within-measure-note-lyric-range-test-id-001',
          },
        }),
      )
    }, source)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await page.waitForSelector('[data-tag="measure"][data-measure-index="0"]', {
      timeout: 10_000,
    })
    await expect(
      page.locator('rect[data-variant="note-click-target-rect"]'),
    ).toHaveCount(6, { timeout: 10_000 })
    await expect(page.locator('[data-tag="lyric"]')).toHaveCount(6, {
      timeout: 10_000,
    })
    await page.evaluate(() => document.fonts.ready)
    await page.waitForTimeout(200)
  },
)

When(
  "I click-and-click select Melody's note {int} then Harmony's lyric syllable {int}",
  async ({ page }, noteId: number, lyricNoteId: number) => {
    const noteBox = await stableBoundingBox(noteClickTarget(page, 0, noteId))
    const lyricBox = await stableBoundingBox(lyricAt(page, 1, lyricNoteId, 0))
    if (!noteBox || !lyricBox) {
      throw new Error(
        `Could not get bounding boxes for Melody note ${noteId} and Harmony lyric syllable ${lyricNoteId}.`,
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
  "Melody's and Harmony's notes at position {int} and {int} are range-selected",
  async ({ page }, a: number, b: number) => {
    for (const partIndex of [0, 1]) {
      for (const noteId of [a, b]) {
        await expect(noteAt(page, partIndex, noteId)).toHaveAttribute(
          'data-note-range-selected',
          '',
        )
      }
    }
  },
)

Then(
  "Melody's and Harmony's lyric syllables at position {int} and {int} are range-selected",
  async ({ page }, a: number, b: number) => {
    for (const partIndex of [0, 1]) {
      for (const noteId of [a, b]) {
        await expect(lyricAt(page, partIndex, noteId, 0)).toHaveAttribute(
          'data-lyric-range-selected',
          '',
        )
      }
    }
  },
)

Then(
  "Melody's and Harmony's notes at position {int} are not range-selected",
  async ({ page }, noteId: number) => {
    for (const partIndex of [0, 1]) {
      await expect(noteAt(page, partIndex, noteId)).not.toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)

Then(
  "Melody's and Harmony's lyric syllables at position {int} are not range-selected",
  async ({ page }, noteId: number) => {
    for (const partIndex of [0, 1]) {
      await expect(lyricAt(page, partIndex, noteId, 0)).not.toHaveAttribute(
        'data-lyric-range-selected',
        '',
      )
    }
  },
)
