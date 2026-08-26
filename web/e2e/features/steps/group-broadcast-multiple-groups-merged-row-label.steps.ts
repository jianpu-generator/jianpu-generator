import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'
import {
  loadFixture,
  MERGED_ROW_PART_INDEX,
} from './group-broadcast-multiple-groups-merged-row-label.fixture'

Given(
  'parts Soprano 1 [S1], Soprano 2 [S2], Alto 1 [A1], Alto 2 [A2], Tenor [T] are declared in that order',
  async () => {},
)

// The fixture's `# groups` section always declares exactly these two groups;
// this step exists purely to spell the fact out in the scenario text.
Given(
  'groups Soprano [S] = S1 S2 and Alto [A] = A1 A2 are declared',
  async () => {},
)

Given(
  "every measure's [S], [A], and [T] lines give all five parts the same notes",
  async () => {},
)

When('the multiple-groups score is laid out', async ({ page }) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(
    `[data-tag="part-label"][data-part-index="${MERGED_ROW_PART_INDEX}"]`,
    { timeout: 10_000 },
  )
})

Then("the merged row's part label reads {string}", async ({ page }, text: string) => {
  // `[data-tag="part-label"]` only wraps the invisible click-target rect —
  // the visible label glyph is a bare, untagged `<text>` rendered as a
  // geometric sibling (see the same pattern in
  // `group-broadcast-coincidental-match-label.steps.ts`). So "what does this
  // row's label say" is answered by finding the `<text>` whose center falls
  // inside the label click-target's own bounding box. Polled (rather than a
  // single snapshot) since the SVG can still be settling right after
  // `page.goto` even though the click-target element already exists.
  await expect(async () => {
    const labelBox = await page
      .locator(
        `[data-tag="part-label"][data-part-index="${MERGED_ROW_PART_INDEX}"]`,
      )
      .first()
      .boundingBox()
    if (!labelBox) {
      throw new Error("Could not get bounding box for the merged row's part label.")
    }
    const texts = page.locator('text')
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
  }).toPass({ timeout: 10_000 })
})
