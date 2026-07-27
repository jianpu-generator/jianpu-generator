import { expect, test } from '@playwright/test'

// The soundfont is a real ~30 MB asset; some sandboxed environments fail to
// write Chromium's HTTP disk cache for large responses
// (net::ERR_CACHE_WRITE_FAILURE), which otherwise breaks the fetch entirely.
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

function exportMenuButton(page: import('@playwright/test').Page) {
  return page.getByRole('button', { name: 'Export', exact: true })
}

test('a toast with a loading indicator is visible while WAV export is in progress, even after the export menu closes', async ({
  page,
}) => {
  test.setTimeout(60_000)

  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const wavItem = page.getByRole('menuitem', { name: 'WAV', exact: true })
  await expect(wavItem).toBeEnabled({ timeout: 30_000 })
  await wavItem.click()

  // Clicking the menu item closes the dropdown immediately (ExportMenuButton
  // sets `open` to false on select), so the toast is the only place a user
  // can observe that export is still running.
  await expect(page.getByRole('menu')).not.toBeVisible()

  const toast = page.getByTestId('wav-export-toast')
  await expect(toast).toBeVisible({ timeout: 5_000 })
  await expect(toast.locator('.file-tab-bar-spinner')).toBeVisible()

  const audioPlayer = page.locator('.preview-audio-player')
  await expect(audioPlayer).toBeVisible({ timeout: 15_000 })

  // Once generation finishes the toast should go away rather than linger.
  await expect(toast).not.toBeVisible({ timeout: 5_000 })
})
