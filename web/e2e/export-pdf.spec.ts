import { expect, test } from '@playwright/test'

// These tests load real font assets for the wasm PDF renderer; some sandboxed
// environments fail to write Chromium's HTTP disk cache for large responses
// (net::ERR_CACHE_WRITE_FAILURE), which otherwise breaks the font fetch.
test.use({
  launchOptions: {
    args: ['--disk-cache-dir=/tmp/chromium-e2e-cache', '--disable-http-cache'],
  },
})

const SINGLE_PART_SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

const MULTI_PART_SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  'Harmony [H] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
  '5 6 7 1',
].join('\n')

async function loadSource(
  page: import('@playwright/test').Page,
  source: string,
) {
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

async function toggleEye(
  page: import('@playwright/test').Page,
  abbreviation: string,
) {
  await page
    .locator('.part-toggle-pill')
    .filter({
      has: page.locator('.part-toggle-abbr', {
        hasText: new RegExp(`^${abbreviation}$`),
      }),
    })
    .locator('.part-toggle-segment--eye')
    .click()
}

function exportMenuButton(page: import('@playwright/test').Page) {
  return page.getByRole('button', { name: 'Export', exact: true })
}

function exportPartsMenuButton(page: import('@playwright/test').Page) {
  return page.getByRole('button', { name: 'Export Parts', exact: true })
}

test('Export > PDF produces a non-empty downloaded file', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const pdfItem = page.getByRole('menuitem', { name: 'PDF', exact: true })
  await expect(pdfItem).toBeEnabled({ timeout: 30_000 })

  const [download] = await Promise.all([
    page.waitForEvent('download'),
    pdfItem.click(),
  ])

  const downloadPath = await download.path()
  expect(downloadPath).toBeTruthy()
  const fs = await import('node:fs')
  const stats = fs.statSync(downloadPath as string)
  expect(stats.size).toBeGreaterThan(1000)
  expect(download.suggestedFilename()).toBe('test.pdf')
})

test('Export Parts > PDF (ZIP) produces a non-empty downloaded zip for a multi-part score', async ({
  page,
}) => {
  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportPartsMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const zipItem = page.getByRole('menuitem', {
    name: 'PDF (ZIP)',
    exact: true,
  })
  await expect(zipItem).toBeEnabled({ timeout: 30_000 })

  const [download] = await Promise.all([
    page.waitForEvent('download'),
    zipItem.click(),
  ])

  const downloadPath = await download.path()
  expect(downloadPath).toBeTruthy()
  const fs = await import('node:fs')
  const stats = fs.statSync(downloadPath as string)
  expect(stats.size).toBeGreaterThan(1000)
  expect(download.suggestedFilename()).toBe('test.zip')
})

test('Export > PDF filename includes only the enabled parts when a part is hidden', async ({
  page,
}) => {
  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await toggleEye(page, 'H')

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const pdfItem = page.getByRole('menuitem', { name: 'PDF', exact: true })
  await expect(pdfItem).toBeEnabled({ timeout: 30_000 })

  const [download] = await Promise.all([
    page.waitForEvent('download'),
    pdfItem.click(),
  ])

  expect(download.suggestedFilename()).toBe('test (Melody).pdf')
})

test('rapid double-click on Export > PDF only triggers a single export', async ({
  page,
}) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const pdfItem = page.getByRole('menuitem', { name: 'PDF', exact: true })
  await expect(pdfItem).toBeEnabled({ timeout: 30_000 })

  let downloadCount = 0
  page.on('download', () => {
    downloadCount += 1
  })

  // Dispatch two click events back-to-back in a single browser task so both
  // reach the handler before React can re-render the menu as closed,
  // exercising the `pdfExporting`/`splitPdfExporting` re-entrancy guard in
  // `exportPdf` (useJianpuWorker.ts) rather than relying on real user timing.
  const [download] = await Promise.all([
    page.waitForEvent('download', { timeout: 30_000 }),
    pdfItem.evaluate((el: HTMLElement) => {
      el.click()
      el.click()
    }),
  ])
  expect(download.suggestedFilename()).toBe('test.pdf')

  // Give a stray second export (if the guard were broken) a chance to fire
  // before asserting only one download ever happened.
  await new Promise((resolve) => setTimeout(resolve, 500))
  expect(downloadCount).toBe(1)
})
