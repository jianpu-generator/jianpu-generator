import { expect } from '@playwright/test'
import { stableBoundingBox } from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

/**
 * Regression fixture for the click-and-click note-range gesture's
 * *hover-preview* step (the live `mouseover` re-resolve `usePreviewClickSelection`
 * does between the anchoring click and the second click — see
 * `handleMouseOver`'s doc comment there), not just its committed result.
 *
 * Two parts, three notes each in one shared measure: Melody has no lyrics
 * at all, Harmony carries the only lyric row (`x y z`, a bare positionally-
 * attached line — see `syntax.md`'s "Positional (unprefixed) lyrics lines").
 * Anchoring on Melody's first note then hovering Harmony's first note
 * resolves via wasm's `Note ↔ Note` cross-part rule
 * (`crates/jianpu-wasm/src/selection_range/note_note.rs`), which always
 * returns `lyric_cells: Vec::new()` — an index/measure range has no notion
 * of a lyric row. So even though the hover's note-range sweeps both parts'
 * note rows, Harmony's `x y z` syllables must stay unhighlighted.
 */
const source = [
  '# metadata',
  'title = "note hover lyric exclusion test"',
  '',
  '# parts',
  'Melody [M] = notes',
  'Harmony [H] = notes',
  '',
  '# score',
  '[M] 1 2 3',
  '[H] 4 5 6',
  'x y z',
].join('\n')

// A note group carries a sibling `Tag::Note` group for its
// (pointer-events: none) playback-cursor rect alongside its own
// click-target rect, so a bare `[data-tag="note"]` query double-counts each
// note — filter to the click-target rect's own group, mirroring
// `note-lyriclabel-range-select.steps.ts`'s `noteInPart` convention.
function noteInPart(page: import('@playwright/test').Page, partIndex: number) {
  return page
    .locator(`[data-tag="note"][data-part-index="${partIndex}"]`)
    .filter({
      has: page.locator('rect[data-variant="note-click-target-rect"]'),
    })
}

function lyricInPart(page: import('@playwright/test').Page, partIndex: number) {
  return page.locator(`[data-tag="lyric"][data-part-index="${partIndex}"]`)
}

Given(
  'the note-hover-lyric-exclusion fixture is loaded and both parts have rendered',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'note-hover-lyric-exclusion-test.jianpu',
          userFiles: { 'note-hover-lyric-exclusion-test.jianpu': src },
          bin: {},
          fileIds: {
            'note-hover-lyric-exclusion-test.jianpu':
              'note-hover-lyric-exclusion-test-id-001',
          },
        }),
      )
    }, source)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await expect(noteInPart(page, 0)).toHaveCount(3, { timeout: 10_000 })
    await expect(noteInPart(page, 1)).toHaveCount(3, { timeout: 10_000 })
    await expect(lyricInPart(page, 1)).toHaveCount(3, { timeout: 10_000 })
  },
)

When(
  "I click Melody's first note then hover over Harmony's first note",
  async ({ page }) => {
    const fromNote = noteInPart(page, 0).nth(0)
    const toNote = noteInPart(page, 1).nth(0)

    const fromBox = await stableBoundingBox(fromNote)
    if (!fromBox)
      throw new Error("Could not get a bounding box for Melody's first note.")
    await page.mouse.move(
      fromBox.x + fromBox.width / 2,
      fromBox.y + fromBox.height / 2,
    )
    await page.mouse.down()
    await page.mouse.up() // click #1 — anchors, does not commit

    const toBox = await stableBoundingBox(toNote)
    if (!toBox)
      throw new Error("Could not get a bounding box for Harmony's first note.")
    await page.mouse.move(
      toBox.x + toBox.width / 2,
      toBox.y + toBox.height / 2,
      { steps: 10 },
    ) // hover preview only — no second click, no commit
  },
)

Then("Harmony's lyric syllables are not range-selected", async ({ page }) => {
  await expect(
    page.locator(
      '[data-tag="lyric"][data-part-index="1"][data-lyric-range-selected]',
    ),
  ).toHaveCount(0)
})
