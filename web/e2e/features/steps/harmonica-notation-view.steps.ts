import { expect } from '@playwright/test'
import { Given, Then } from './fixtures'

// The last measure's `5,,` sits below a C harmonica's lowest hole.
const SOURCE = [
  '# metadata',
  'title = "Harmonica View Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1 2 3 4',
  '',
  "[M] 5 6 7 1'",
  '',
  "[M] 5,, 1, 3' 1''",
].join('\n')

Given(
  'a score in C major is loaded for the harmonica notation view',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'harmonica.jianpu',
          userFiles: { 'harmonica.jianpu': src },
          bin: {},
          fileIds: { 'harmonica.jianpu': crypto.randomUUID() },
        }),
      )
    }, SOURCE)
    await page.goto('/')
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then(
  'the preview flags the unplayable note heads {string}',
  async ({ page }, expected: string) => {
    await expect(
      page.locator('.preview-page text[data-variant="unplayable-note-head"]'),
    ).toHaveText(expected.split(' '))
  },
)
