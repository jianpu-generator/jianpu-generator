import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

// Padded with filler measures so "Intro" and "C" can't both fit in one
// viewport.
const fillerMeasures = Array.from(
  { length: 60 },
  (_, i) => `label="Filler${i}"\n1 2 3 4`,
).join('\n\n')

const SOURCE = [
  '# metadata',
  'title = "test"',
  '',
  '# parts',
  'M = notes',
  '',
  '# sequence',
  'Intro, C',
  '',
  '# score',
  'time=4/4 key=C4 bpm=120 label="Intro"',
  '1 2 3 4',
  '',
  fillerMeasures,
  '',
  'label="C"',
  "1' 7 6 5",
].join('\n')

function sequenceToolbarButtons(page: import('@playwright/test').Page) {
  return page
    .locator('[role="toolbar"]')
    .nth(1)
    .locator('button.section-jump-btn')
}

function lastMeasure(page: import('@playwright/test').Page) {
  return page.locator('[data-tag="measure"]').last()
}

Given(
  'a sequence chain {string} padded with filler measures is seeded for preview reveal target',
  async ({ page }, _chain: string) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'sequence-preview-reveal-target-test.jianpu',
          userFiles: { 'sequence-preview-reveal-target-test.jianpu': src },
          bin: {},
          fileIds: {
            'sequence-preview-reveal-target-test.jianpu': crypto.randomUUID(),
          },
        }),
      )
    }, SOURCE)
  },
)

When(
  'the app loads with the preview-reveal sequence toolbar ready',
  async ({ page }) => {
    await page.goto('/')
    await expect(sequenceToolbarButtons(page)).toHaveCount(2, {
      timeout: 15_000,
    })
    // "C" is the last written measure (index 61): "Intro" (index 0) plus 60
    // filler measures (indices 1-60).
    await expect(page.locator('[data-tag="measure"]')).toHaveCount(62, {
      timeout: 15_000,
    })
  },
)

Then('the last measure is not in the preview viewport', async ({ page }) => {
  await expect(lastMeasure(page)).not.toBeInViewport()
})

When(
  'I drag from the {string} sequence entry to the {string} sequence entry, as seen in preview reveal target',
  async ({ page }, _from: string, _to: string) => {
    const buttons = sequenceToolbarButtons(page)
    // index 0 = Intro, index 1 = C.
    await buttons.nth(0).hover()
    await page.mouse.down()
    await buttons.nth(1).hover()
    await page.mouse.up()
  },
)

Then('the last measure scrolls into the preview viewport', async ({ page }) => {
  await expect(lastMeasure(page)).toBeInViewport({ timeout: 3_000 })
})
