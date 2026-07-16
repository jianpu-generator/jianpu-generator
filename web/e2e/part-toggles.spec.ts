import { expect, type Page, test } from '@playwright/test'

// The "M — Melody" / "C — Chords" legend in the preview header lists only
// parts currently enabled by hide/solo state — a hidden or non-soloed part's
// entry disappears from the legend along with its score content.

const SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes+lyrics',
  'Chords [C] = chords',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1 2 3 4',
  '[M] twin- kle twin- kle',
  '[C] 1 - - -',
].join('\n')

const THREE_PART_SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes+lyrics',
  'Harmony [H] = notes',
  'Chords [C] = chords',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1 2 3 4',
  '[M] twin- kle twin- kle',
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

test('hiding a part removes its content from the preview and hides its lyrics toggle', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(MELODY_LYRICS)
  await expect(preview).toContainText(CHORD_CONTENT)

  await toggleEye(page, 'M')

  await expect(preview).not.toContainText(MELODY_NOTES)
  await expect(preview).not.toContainText(MELODY_LYRICS)
  // The mic (lyrics) toggle only renders for an enabled part with lyrics.
  await expect(
    partPill(page, 'M').locator('.part-toggle-segment--mic'),
  ).toHaveCount(0)

  // The other part is unaffected.
  await expect(preview).toContainText(CHORD_CONTENT)
})

test('unhiding a part restores its content in the preview', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  await toggleEye(page, 'M')
  await expect(page.locator('.preview-pages')).not.toContainText(MELODY_NOTES)

  await toggleEye(page, 'M')

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(MELODY_NOTES)
  await expect(preview).toContainText(MELODY_LYRICS)
  await expect(
    partPill(page, 'M').locator('.part-toggle-segment--mic'),
  ).toHaveCount(1)
})

test('soloing a part hides all other parts in the preview', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(CHORD_CONTENT)

  await toggleSolo(page, 'M')

  await expect(preview).toContainText(MELODY_NOTES)
  await expect(preview).toContainText(MELODY_LYRICS)
  await expect(preview).not.toContainText(CHORD_CONTENT)
})

test('un-soloing restores previously enabled parts', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  await toggleSolo(page, 'M')
  await expect(page.locator('.preview-pages')).not.toContainText(CHORD_CONTENT)

  await toggleSolo(page, 'M')

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(MELODY_NOTES)
  await expect(preview).toContainText(CHORD_CONTENT)
})

test('soloing multiple parts keeps both visible and hides the rest', async ({
  page,
}) => {
  await loadSource(page, THREE_PART_SOURCE)
  await page.goto('/')
  await waitForPreviewReady(page)

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(HARMONY_NOTES)
  await expect(preview).toContainText(CHORD_CONTENT)

  await toggleSolo(page, 'H')
  await toggleSolo(page, 'C')

  await expect(preview).not.toContainText(MELODY_NOTES)
  await expect(preview).not.toContainText(MELODY_LYRICS)
  await expect(preview).toContainText(HARMONY_NOTES)
  await expect(preview).toContainText(CHORD_CONTENT)
})

test('toggling lyrics off removes lyric text but keeps notes rendered', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(MELODY_LYRICS)

  await toggleLyrics(page, 'M')

  await expect(preview).not.toContainText(MELODY_LYRICS)
  // The notes themselves stay rendered — only the lyrics row is filtered.
  await expect(preview).toContainText(MELODY_NOTES)
})

test('toggling lyrics back on restores lyric text', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  await toggleLyrics(page, 'M')
  await expect(page.locator('.preview-pages')).not.toContainText(MELODY_LYRICS)

  await toggleLyrics(page, 'M')

  await expect(page.locator('.preview-pages')).toContainText(MELODY_LYRICS)
})

test('hiding a part removes its entry from the part-list legend', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(MELODY_LEGEND)
  await expect(preview).toContainText(CHORDS_LEGEND)

  await toggleEye(page, 'M')

  await expect(preview).not.toContainText(MELODY_LEGEND)
  // The other part's legend entry is unaffected.
  await expect(preview).toContainText(CHORDS_LEGEND)
})

test('unhiding a part restores its part-list legend entry', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  await toggleEye(page, 'M')
  await expect(page.locator('.preview-pages')).not.toContainText(MELODY_LEGEND)

  await toggleEye(page, 'M')

  await expect(page.locator('.preview-pages')).toContainText(MELODY_LEGEND)
})

test('soloing a part hides other parts legend entries', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(CHORDS_LEGEND)

  await toggleSolo(page, 'M')

  await expect(preview).toContainText(MELODY_LEGEND)
  await expect(preview).not.toContainText(CHORDS_LEGEND)
})

test('un-soloing restores previously enabled parts legend entries', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await waitForPreviewReady(page)

  await toggleSolo(page, 'M')
  await expect(page.locator('.preview-pages')).not.toContainText(CHORDS_LEGEND)

  await toggleSolo(page, 'M')

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(MELODY_LEGEND)
  await expect(preview).toContainText(CHORDS_LEGEND)
})

test('soloing multiple parts keeps both legend entries and hides the rest', async ({
  page,
}) => {
  await loadSource(page, THREE_PART_SOURCE)
  await page.goto('/')
  await waitForPreviewReady(page)

  const preview = page.locator('.preview-pages')
  await expect(preview).toContainText(HARMONY_LEGEND)
  await expect(preview).toContainText(CHORDS_LEGEND)

  await toggleSolo(page, 'H')
  await toggleSolo(page, 'C')

  await expect(preview).not.toContainText(MELODY_LEGEND)
  await expect(preview).toContainText(HARMONY_LEGEND)
  await expect(preview).toContainText(CHORDS_LEGEND)
})
