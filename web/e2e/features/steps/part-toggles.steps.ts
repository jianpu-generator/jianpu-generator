import { expect, type Page } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  'Chords [C] = chords',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1 2 3 4',
  'twin- kle twin- kle',
  '[C] 1 - - -',
].join('\n')

const THREE_PART_SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  'Harmony [H] = notes',
  'Chords [C] = chords',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1 2 3 4',
  'twin- kle twin- kle',
  '[H] 5 6 7 1',
  '[C] 1 - - -',
].join('\n')

// Unique substrings identifying each part's rendered score content.
const MELODY_NOTES = '1234'
const MELODY_LYRICS = 'twin-'
const HARMONY_NOTES = '5671'
const CHORD_CONTENT = '———'

// Part-list legend entries ("abbreviation — display name"), rendered in the
// preview header only for parts whose abbreviation differs from their name.
const MELODY_LEGEND = 'M — Melody'
const HARMONY_LEGEND = 'H — Harmony'
const CHORDS_LEGEND = 'C — Chords'

async function loadSource(page: Page, source: string = SOURCE) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'test.jianpu',
        userFiles: { 'test.jianpu': src },
        bin: {},
        fileIds: { 'test.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)
}

async function waitForPreviewReady(page: Page) {
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await expect(page.locator('.preview-pages')).toContainText(MELODY_NOTES, {
    timeout: 15_000,
  })
}

function partPill(page: Page, abbreviation: string) {
  return page.locator('.part-toggle-pill').filter({
    has: page.locator('.part-toggle-abbr', {
      hasText: new RegExp(`^${abbreviation}$`),
    }),
  })
}

async function toggleEye(page: Page, abbreviation: string) {
  await partPill(page, abbreviation)
    .locator('.part-toggle-segment--eye')
    .click()
}

async function toggleSolo(page: Page, abbreviation: string) {
  await partPill(page, abbreviation)
    .locator('.part-toggle-segment--headphones')
    .click()
}

async function toggleLyrics(page: Page, abbreviation: string) {
  await partPill(page, abbreviation)
    .locator('.part-toggle-segment--mic')
    .click()
}

const PART_ABBREVIATIONS: Record<string, string> = {
  Melody: 'M',
  Harmony: 'H',
  Chords: 'C',
}

const CONTENT_BY_LABEL: Record<string, string> = {
  'Melody notes': MELODY_NOTES,
  'Melody lyrics': MELODY_LYRICS,
  'Harmony notes': HARMONY_NOTES,
  'chord content': CHORD_CONTENT,
}

const LEGEND_BY_LABEL: Record<string, string> = {
  Melody: MELODY_LEGEND,
  Harmony: HARMONY_LEGEND,
  Chords: CHORDS_LEGEND,
}

Given('the two-part melody-chords fixture is loaded', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)
})

Given(
  'the three-part melody-harmony-chords fixture is loaded',
  async ({ page }) => {
    await loadSource(page, THREE_PART_SOURCE)
    await page.goto('/')
    await waitForPreviewReady(page)
  },
)

When(
  'I hide the {string} part via its eye toggle, as seen in part toggles',
  async ({ page }, partName: string) => {
    await toggleEye(page, PART_ABBREVIATIONS[partName])
  },
)

When('I solo the {string} part', async ({ page }, partName: string) => {
  await toggleSolo(page, PART_ABBREVIATIONS[partName])
})

When(
  "I toggle the {string} part's lyrics off",
  async ({ page }, partName: string) => {
    await toggleLyrics(page, PART_ABBREVIATIONS[partName])
  },
)

Then(
  'the preview contains {string} {string}',
  async ({ page }, partName: string, kind: string) => {
    const key = `${partName} ${kind}`
    const content = CONTENT_BY_LABEL[key]
    if (!content) throw new Error(`Unknown preview content label: ${key}`)
    await expect(page.locator('.preview-pages')).toContainText(content)
  },
)

Then(
  'the preview does not contain {string} {string}',
  async ({ page }, partName: string, kind: string) => {
    const key = `${partName} ${kind}`
    const content = CONTENT_BY_LABEL[key]
    if (!content) throw new Error(`Unknown preview content label: ${key}`)
    await expect(page.locator('.preview-pages')).not.toContainText(content)
  },
)

Then('the preview contains the chord content', async ({ page }) => {
  await expect(page.locator('.preview-pages')).toContainText(CHORD_CONTENT)
})

Then('the preview does not contain the chord content', async ({ page }) => {
  await expect(page.locator('.preview-pages')).not.toContainText(CHORD_CONTENT)
})

Then(
  'the {string} part pill has no mic toggle',
  async ({ page }, partName: string) => {
    // The mic (lyrics) toggle only renders for an enabled part with lyrics.
    await expect(
      partPill(page, PART_ABBREVIATIONS[partName]).locator(
        '.part-toggle-segment--mic',
      ),
    ).toHaveCount(0)
  },
)

Then(
  'the {string} part pill has a mic toggle',
  async ({ page }, partName: string) => {
    await expect(
      partPill(page, PART_ABBREVIATIONS[partName]).locator(
        '.part-toggle-segment--mic',
      ),
    ).toHaveCount(1)
  },
)

Then(
  'the preview contains the {string} legend entry',
  async ({ page }, partName: string) => {
    await expect(page.locator('.preview-pages')).toContainText(
      LEGEND_BY_LABEL[partName],
    )
  },
)

Then(
  'the preview does not contain the {string} legend entry',
  async ({ page }, partName: string) => {
    await expect(page.locator('.preview-pages')).not.toContainText(
      LEGEND_BY_LABEL[partName],
    )
  },
)
