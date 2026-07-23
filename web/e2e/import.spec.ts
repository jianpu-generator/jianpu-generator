import fs from 'node:fs'
import { expect, test } from '@playwright/test'
import { fileSwitcherTrigger, openFileActions } from './fileSwitcherHelpers'

// These tests load real font assets for the wasm PDF renderer; some sandboxed
// environments fail to write Chromium's HTTP disk cache for large responses
// (net::ERR_CACHE_WRITE_FAILURE), which otherwise breaks the font fetch.
test.use({
  launchOptions: {
    args: ['--disk-cache-dir=/tmp/chromium-e2e-cache', '--disable-http-cache'],
  },
})

const SOURCE = [
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

async function loadSource(page: import('@playwright/test').Page) {
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
  }, SOURCE)
}

function importButton(page: import('@playwright/test').Page) {
  return page.getByRole('menuitem', { name: /^Import$/ })
}

test('Import recovers the original source from a previously exported PDF', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const exportButton = page.getByRole('button', { name: 'Export', exact: true })
  await expect(exportButton).toBeEnabled({ timeout: 30_000 })
  await exportButton.click()
  const pdfItem = page.getByRole('menuitem', { name: 'PDF', exact: true })
  await expect(pdfItem).toBeEnabled({ timeout: 30_000 })

  const [download] = await Promise.all([
    page.waitForEvent('download'),
    pdfItem.click(),
  ])
  const pdfPath = await download.path()
  expect(pdfPath).toBeTruthy()
  const pdfBytes = fs.readFileSync(pdfPath as string)

  await openFileActions(page)
  await importButton(page).click()
  await page.setInputFiles('input[type=file]', {
    name: 'test.pdf',
    mimeType: 'application/pdf',
    buffer: pdfBytes,
  })

  // "test.jianpu" is already open, so the recovered file lands under a
  // deduped name rather than overwriting it.
  await expect(fileSwitcherTrigger(page)).toContainText('test 2.jianpu')
  await page.waitForFunction(
    (expected) =>
      window.monaco?.editor.getEditors()[0]?.getModel()?.getValue() ===
      expected,
    SOURCE,
    { timeout: 15_000 },
  )
})

test('Import shows a graceful error for a file with no embedded source', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await openFileActions(page)
  await importButton(page).click()
  await page.setInputFiles('input[type=file]', {
    name: 'plain.svg',
    mimeType: 'image/svg+xml',
    buffer: Buffer.from(
      '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>',
    ),
  })

  await expect(page.getByText('Could not import file')).toBeVisible({
    timeout: 10_000,
  })
  await expect(page.getByTestId('error-modal-message')).toContainText(
    'No embedded source found in this file.',
  )

  // Failing import must not crash the app or replace the active file.
  await expect(page.locator('.preview-page')).toHaveCount(1)
})
