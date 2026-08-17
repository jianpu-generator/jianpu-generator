import { expect, test } from '@playwright/test'
import { typeAtEditorEnd } from './fileSwitcherHelpers'

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

test('Export > WAV produces a playable inline audio player and Export > WAV (regenerate) replaces it', async ({
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

  const audioPlayer = page.locator('.preview-audio-player')
  await expect(audioPlayer).toBeVisible({ timeout: 15_000 })

  const firstSrc = await audioPlayer.getAttribute('src')
  expect(firstSrc).toMatch(/^blob:/)

  // A silently empty/corrupt WAV would still produce a `blob:` src but have
  // zero duration — assert the browser actually decoded playable audio.
  const firstDuration = await audioPlayer.evaluate(
    (el: HTMLAudioElement) =>
      new Promise<number>((resolve) => {
        if (el.readyState >= 1 && Number.isFinite(el.duration)) {
          resolve(el.duration)
          return
        }
        el.addEventListener('loadedmetadata', () => resolve(el.duration), {
          once: true,
        })
      }),
  )
  expect(firstDuration).toBeGreaterThan(0)

  await menuButton.click()
  const regenerateItem = page.getByRole('menuitem', {
    name: 'WAV (regenerate)',
    exact: true,
  })
  await expect(regenerateItem).toBeVisible()
  await regenerateItem.click()

  // The previous blob URL is revoked and a new one set (useJianpuWorker.ts's
  // setNextWavUrl) — assert the src actually changed rather than being reused.
  await expect
    .poll(() => audioPlayer.getAttribute('src'), { timeout: 15_000 })
    .not.toBe(firstSrc)
  const secondSrc = await audioPlayer.getAttribute('src')
  expect(secondSrc).toMatch(/^blob:/)

  // Editing the source in place (no file switch) must not clear the
  // existing audio or reset the item back to "WAV" — wavUrl is only cleared
  // on `activeFile` change, not on `source` change.
  await typeAtEditorEnd(page, ' 5')

  await expect(audioPlayer).toBeVisible()
  await expect(audioPlayer).toHaveAttribute('src', secondSrc as string)

  await menuButton.click()
  await expect(
    page.getByRole('menuitem', { name: 'WAV (regenerate)', exact: true }),
  ).toBeVisible()
})

test('audio download link filename includes only the enabled parts when a part is hidden', async ({
  page,
}) => {
  test.setTimeout(60_000)

  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await toggleEye(page, 'H')

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const wavItem = page.getByRole('menuitem', { name: 'WAV', exact: true })
  await expect(wavItem).toBeEnabled({ timeout: 30_000 })
  await wavItem.click()

  const downloadLink = page.locator('.preview-audio-download')
  await expect(downloadLink).toBeVisible({ timeout: 15_000 })
  await expect(downloadLink).toHaveAttribute('download', 'test (Melody).wav')
})

test('Export Parts > WAV (ZIP) produces a non-empty downloaded zip for a multi-part score', async ({
  page,
}) => {
  test.setTimeout(60_000)

  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const zipItem = page.getByRole('menuitem', {
    name: 'WAV (ZIP)',
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
  expect(download.suggestedFilename()).toBe('test (WAV parts).zip')
})
