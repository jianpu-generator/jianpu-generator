import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'
import {
  loadFixture,
  partLabelsFor,
  primeMeasureSpans,
  resetState,
  setDivergentMeasure,
  setMatchingMeasure,
} from './group-broadcast-coincidental-match-label.fixture'

Given(
  'parts Soprano 1 [S1], Soprano 2 [S2], Tenor [T] are declared in that order',
  async () => {
    resetState()
  },
)

// The fixture's `# groups` section always declares exactly this group; this
// step exists purely to spell the fact out in the scenario text.
Given('group Soprano [S] = S1 S2', async () => {})

Given(
  "measure {int}'s [S] broadcast gives S1 and S2 the same notes as Tenor's own line",
  async ({}, index: number) => {
    setMatchingMeasure(index)
  },
)

Given(
  "measure {int}'s [S] broadcast gives S1 and S2 different notes from Tenor's own line",
  async ({}, index: number) => {
    setDivergentMeasure(index)
  },
)

When('the group-broadcast score is laid out', async ({ page }) => {
  const firstMeasureLine = await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="part-label"][data-part-index="0"]', {
    timeout: 10_000,
  })
  await primeMeasureSpans(page, firstMeasureLine)
})

Then(
  "{word}'s part label reads {string}",
  async ({ page }, part: string, text: string) => {
    // `[data-tag="part-label"]` only wraps the invisible click-target rect
    // (see `PartLabelClickTarget`) — the visible label glyph is a bare,
    // untagged `<text>` rendered as a geometric sibling, same as note-digit
    // glyphs (see the note-glyph steps in this file and in
    // `system-part-union-packing-merged-row.steps.ts`). So "what does this
    // row's label say" has to be answered by finding the `<text>` whose
    // center falls inside the label click-target's own bounding box, same
    // as those steps locate note glyphs inside a measure/row intersection.
    const labelBox = await partLabelsFor(page, part).first().boundingBox()
    if (!labelBox) {
      throw new Error(`Could not get bounding box for ${part}'s part label.`)
    }
    const texts = page.locator('text')
    await expect(texts.first()).toBeVisible({ timeout: 10_000 })
    const count = await texts.count()
    let found: string | null = null
    for (let i = 0; i < count; i++) {
      const el = texts.nth(i)
      const box = await el.boundingBox()
      if (!box) continue
      const centerX = box.x + box.width / 2
      const centerY = box.y + box.height / 2
      const inLabel =
        centerX >= labelBox.x &&
        centerX <= labelBox.x + labelBox.width &&
        centerY >= labelBox.y &&
        centerY <= labelBox.y + labelBox.height
      if (inLabel) {
        found = await el.textContent()
        break
      }
    }
    expect(found).toBe(text)
  },
)

Then(
  "{word}'s part label spans measures {int} to {int} across the system",
  async ({ page }, part: string, from: number, to: number) => {
    const label = partLabelsFor(page, part).first()
    await expect(label).toHaveCount(1, { timeout: 10_000 })
    await expect(label).toHaveAttribute(
      'data-measure-index-start',
      String(from),
    )
    await expect(label).toHaveAttribute('data-measure-index-end', String(to))
  },
)

Then(
  "measure {int}'s {word} row shows a real note glyph",
  async ({ page }, index: number, part: string) => {
    // Mirror of the same-named step in
    // `system-part-union-packing-merged-row.steps.ts`: a note digit glyph
    // renders as a bare `<text>` with content `1`-`7`, so "is this part's
    // row genuinely showing its notes here" is answered geometrically —
    // does any digit-1-7 glyph's center fall inside the intersection of
    // this measure's column and this part's row.
    const measureBox = await page
      .locator(`[data-tag="measure"][data-measure-index="${index}"]`)
      .boundingBox()
    const rowBox = await partLabelsFor(page, part).first().boundingBox()
    if (!measureBox || !rowBox) {
      throw new Error(
        `Could not get bounding boxes for measure ${index} / ${part}'s row.`,
      )
    }
    const notes = page.locator('text').getByText(/^[1-7]$/)
    const count = await notes.count()
    let found = false
    for (let i = 0; i < count; i++) {
      const box = await notes.nth(i).boundingBox()
      if (!box) continue
      const centerX = box.x + box.width / 2
      const centerY = box.y + box.height / 2
      const inMeasure =
        centerX >= measureBox.x && centerX <= measureBox.x + measureBox.width
      const inRow = centerY >= rowBox.y && centerY <= rowBox.y + rowBox.height
      if (inMeasure && inRow) {
        found = true
        break
      }
    }
    expect(found).toBe(true)
  },
)
