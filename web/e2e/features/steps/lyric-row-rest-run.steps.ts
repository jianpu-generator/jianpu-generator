import { expect } from '@playwright/test'
import { tagSelector } from '../../../src/dataAttributes'
import { stableBoundingBox } from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

type State = {
  measureCount: number
  melodyNotes: string
  singingMeasures: number[]
  singing: string
}

let state: State = {
  measureCount: 0,
  melodyNotes: '',
  singingMeasures: [],
  singing: '',
}

Given(
  'parts Melody [M], Verse [V] = lyrics[M] are declared for the lyric-row-rest-run score',
  async () => {
    state = {
      measureCount: 0,
      melodyNotes: '',
      singingMeasures: [],
      singing: '',
    }
  },
)

Given(
  'the lyric-row-rest-run score has {int} measures of Melody notes {string}',
  async ({}, count: number, notes: string) => {
    state.measureCount = count
    state.melodyNotes = notes
  },
)

Given(
  'the Verse line is sung {string} in measures {int} and {int} only',
  async ({}, syllables: string, first: number, second: number) => {
    state.singing = syllables
    state.singingMeasures = [first, second]
  },
)

function buildSource(): string {
  const lines = [
    '# metadata',
    'title = "lyric row rest run test"',
    `max_measures_per_system = ${state.measureCount}`,
    '',
    '# parts',
    'Melody [M] = notes',
    'Verse [V] = lyrics[M]',
    '',
    '# score',
  ]
  for (let i = 0; i < state.measureCount; i++) {
    lines.push(`[M] ${state.melodyNotes}`)
    if (state.singingMeasures.includes(i)) lines.push(`[V] ${state.singing}`)
    lines.push('')
  }
  return lines.join('\n')
}

When('the lyric-row-rest-run score is laid out', async ({ page }) => {
  const source = buildSource()
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'lyric-row-rest-run-test.jianpu',
        userFiles: { 'lyric-row-rest-run-test.jianpu': src },
        bin: {},
        fileIds: {
          'lyric-row-rest-run-test.jianpu': 'lyric-row-rest-run-test-id-001',
        },
      }),
    )
  }, source)

  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(tagSelector('measure'), { timeout: 10_000 })
})

/** The rest bar is identified by shape, as in
 * merged-rest-run-column-width.steps.ts: a thick horizontal `<line>`. */
async function restBarBoxes(page: import('@playwright/test').Page) {
  return page.evaluate(() =>
    Array.from(document.querySelectorAll('svg line'))
      .filter(
        (l) =>
          l.getAttribute('y1') === l.getAttribute('y2') &&
          l.getAttribute('x1') !== l.getAttribute('x2') &&
          Number(l.getAttribute('stroke-width')) > 2,
      )
      .map((l) => {
        const r = l.getBoundingClientRect()
        return { x: r.x, y: r.y, width: r.width, height: r.height }
      }),
  )
}

Then(
  'the lyric row shows one rest bar with the count {string}',
  async ({ page }, count: string) => {
    await expect(page.locator(`svg text:text-is("${count}")`)).toHaveCount(1)
    expect(await restBarBoxes(page)).toHaveLength(1)
  },
)

Then('the lyric row shows no rest bar', async ({ page }) => {
  expect(await restBarBoxes(page)).toHaveLength(0)
})

Then(
  'the rest bar spans measures {int} to {int}',
  async ({ page }, start: number, end: number) => {
    const [bar] = await restBarBoxes(page)
    if (!bar) throw new Error('No rest bar found.')
    const first = await stableBoundingBox(
      page.locator(tagSelector('measure', { index: start })).first(),
    )
    const last = await stableBoundingBox(
      page.locator(tagSelector('measure', { index: end })).first(),
    )
    if (!first || !last) throw new Error('Could not measure the measures.')
    expect(bar.x).toBeGreaterThanOrEqual(first.x)
    expect(bar.x + bar.width).toBeLessThanOrEqual(last.x + last.width)
    // Wider than any single measure: it covers the whole blank run.
    expect(bar.width).toBeGreaterThan(first.width)
  },
)

Then('the rest bar sits below the Melody notes', async ({ page }) => {
  const [bar] = await restBarBoxes(page)
  if (!bar) throw new Error('No rest bar found.')
  const noteBottoms = await page.evaluate(() =>
    Array.from(document.querySelectorAll('svg text'))
      .filter((t) => t.textContent === '1')
      .map((t) => t.getBoundingClientRect().bottom),
  )
  expect(Math.max(...noteBottoms)).toBeLessThanOrEqual(bar.y)
})

Then(
  'the Melody row still shows {int} notes',
  async ({ page }, count: number) => {
    await expect(page.locator('svg text:text-is("1")')).toHaveCount(count)
  },
)
