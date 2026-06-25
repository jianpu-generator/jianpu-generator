import { expect, test } from '@playwright/test'

const SOURCE_WITH_UNDERFLOW_IN_MEASURE_1 = [
  '# metadata',
  'title="t"',
  'author="a"',
  '',
  '# parts',
  'Melody = notes+lyrics',
  '',
  '# score',
  '(time=4/4 key=C4 bpm=120)',
  '1 2 3 4',
  'a b',
  '5 6 7 1',
  'do re mi fa',
].join('\n')

const FILE_STORE_KEY = 'jianpu:files:v1'

test('lyric underflow in measure 1 shows error overlay and still renders measure 2', async ({
  page,
}) => {
  // Pre-seed localStorage with a user file that has lyric underflow in measure 1
  // but valid lyrics in measure 2. The demo file is read-only, so we use a user file.
  await page.goto('/')
  await page.evaluate(
    ({ key, source }: { key: string; source: string }) => {
      const store = {
        active: 'test.jianpu',
        userFiles: { 'test.jianpu': source },
        bin: {},
        fileIds: { 'test.jianpu': 'test-file-id' },
      }
      localStorage.setItem(key, JSON.stringify(store))
    },
    { key: FILE_STORE_KEY, source: SOURCE_WITH_UNDERFLOW_IN_MEASURE_1 },
  )

  // Reload so the app initialises with our user file active.
  await page.reload()
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Wait for the debounce + render worker round-trip.
  await page.waitForTimeout(2_000)

  // The error overlay rect for the erroneous measure must appear in the SVG.
  const errorHighlight = page.locator(
    '.preview-page [data-testid="error-highlight"]',
  )
  await expect(errorHighlight).toBeVisible({ timeout: 5_000 })

  // Measure 2 lyrics ("do") must also appear — confirming best-effort render.
  const previewContent = await page.locator('.preview-page').first().innerHTML()
  expect(previewContent).toContain('do')
})
