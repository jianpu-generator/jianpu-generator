import { expect, type Locator, type Page } from '@playwright/test'
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
    // Seed once: a reload must keep the same file id so its toggles survive.
    if (localStorage.getItem('jianpu:files:v1')) return
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

function lyricsPill(page: Page, abbreviation: string) {
  return partPill(page, `${abbreviation}詞`)
}

function firstLyricLocator(page: Page) {
  return page.locator('.preview-pages text', { hasText: MELODY_LYRICS }).first()
}

let recordedLyricX: number | null = null

async function expectTwoButtons(pill: Locator) {
  await expect(pill.locator('input[type="checkbox"]')).toHaveCount(2)
  await expect(pill.locator('.part-toggle-segment--eye')).toHaveCount(1)
  await expect(pill.locator('.part-toggle-segment--headphones')).toHaveCount(1)
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

function lookup(
  table: Record<string, string>,
  key: string,
  what: string,
): string {
  const value = table[key]
  if (value === undefined) throw new Error(`Unknown ${what}: ${key}`)
  return value
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
    await toggleEye(page, lookup(PART_ABBREVIATIONS, partName, 'part name'))
  },
)

When('I solo the {string} part', async ({ page }, partName: string) => {
  await toggleSolo(page, lookup(PART_ABBREVIATIONS, partName, 'part name'))
})

When(
  'I hide the {string} lyrics row via its eye toggle',
  async ({ page }, partName: string) => {
    await lyricsPill(page, lookup(PART_ABBREVIATIONS, partName, 'part name'))
      .locator('.part-toggle-segment--eye')
      .click()
  },
)

When('I solo the {string} lyrics row', async ({ page }, partName: string) => {
  await lyricsPill(page, lookup(PART_ABBREVIATIONS, partName, 'part name'))
    .locator('.part-toggle-segment--headphones')
    .click()
})

When('I reload the part toggles page', async ({ page }) => {
  await page.reload()
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
})

Given(
  'I note the horizontal position of the first {string} lyric',
  async ({ page }, _partName: string) => {
    const box = await firstLyricLocator(page).boundingBox()
    if (!box) throw new Error('First lyric has no bounding box')
    recordedLyricX = box.x
  },
)

Then(
  'the first {string} lyric is at the same horizontal position',
  async ({ page }, _partName: string) => {
    expect(recordedLyricX).not.toBeNull()
    await expect
      .poll(async () => (await firstLyricLocator(page).boundingBox())?.x)
      // Column widths no longer include the hidden notes, so the layout may
      // shift sub-pixel; the Rust render test uses the same 0.5 tolerance.
      .toBeCloseTo(recordedLyricX as number, 0)
  },
)

Then(
  'the part toggles list contains a {string} part pill',
  async ({ page }, partName: string) => {
    await expect(
      partPill(page, lookup(PART_ABBREVIATIONS, partName, 'part name')),
    ).toHaveCount(1)
  },
)

Then(
  'the part toggles list contains a {string} lyrics pill',
  async ({ page }, partName: string) => {
    await expect(
      lyricsPill(page, lookup(PART_ABBREVIATIONS, partName, 'part name')),
    ).toHaveCount(1)
  },
)

Then(
  'the part toggles list does not contain a {string} lyrics pill',
  async ({ page }, partName: string) => {
    await expect(
      lyricsPill(page, lookup(PART_ABBREVIATIONS, partName, 'part name')),
    ).toHaveCount(0)
  },
)

Then(
  'the {string} part pill has exactly a show\\/hide button and a solo button',
  async ({ page }, partName: string) => {
    await expectTwoButtons(
      partPill(page, lookup(PART_ABBREVIATIONS, partName, 'part name')),
    )
  },
)

Then(
  'the {string} lyrics pill has exactly a show\\/hide button and a solo button',
  async ({ page }, partName: string) => {
    await expectTwoButtons(
      lyricsPill(page, lookup(PART_ABBREVIATIONS, partName, 'part name')),
    )
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
  'the preview contains the {string} legend entry',
  async ({ page }, partName: string) => {
    await expect(page.locator('.preview-pages')).toContainText(
      lookup(LEGEND_BY_LABEL, partName, 'legend label'),
    )
  },
)

Then(
  'the preview does not contain the {string} legend entry',
  async ({ page }, partName: string) => {
    await expect(page.locator('.preview-pages')).not.toContainText(
      lookup(LEGEND_BY_LABEL, partName, 'legend label'),
    )
  },
)
