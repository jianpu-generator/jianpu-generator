import { expect, type Page } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const COMMENTED_HEADERS_SOURCE = [
  '# metadata // song info',
  'title = "Test"',
  '',
  '# parts // note',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1 1 5 5',
].join('\n')

// `  # parts` is not a header (a header's `#` must be at column 0), so only
// the real `# parts` line gets a link.
const INDENTED_HEADER_LIKE_SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1 1 5 5',
  '  # parts',
].join('\n')

async function loadSource(page: Page, source: string) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'code-lens.jianpu',
        userFiles: { 'code-lens.jianpu': src },
        bin: {},
        fileIds: { 'code-lens.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 30_000 })
}

function codeLensLinks(page: Page, title: string) {
  return page.locator('.codelens-decoration a', { hasText: title })
}

Given(
  'the section-header CodeLens fixture with commented headers is loaded',
  async ({ page }) => {
    await loadSource(page, COMMENTED_HEADERS_SOURCE)
  },
)

Given(
  'the section-header CodeLens fixture with an indented header-like line is loaded',
  async ({ page }) => {
    await loadSource(page, INDENTED_HEADER_LIKE_SOURCE)
  },
)

Then(
  'the editor shows {int} {string} CodeLens link(s)',
  async ({ page }, count: number, title: string) => {
    await expect(codeLensLinks(page, title)).toHaveCount(count, {
      timeout: 15_000,
    })
  },
)

When('I click the {string} CodeLens link', async ({ page }, title: string) => {
  await codeLensLinks(page, title).click()
})
