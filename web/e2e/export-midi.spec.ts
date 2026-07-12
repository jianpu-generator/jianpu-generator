import { expect, test } from '@playwright/test'

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

test('Export > MIDI produces a non-empty downloaded file', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const midiItem = page.getByRole('menuitem', { name: 'MIDI', exact: true })
  await expect(midiItem).toBeEnabled({ timeout: 30_000 })

  const [download] = await Promise.all([
    page.waitForEvent('download'),
    midiItem.click(),
  ])

  const downloadPath = await download.path()
  expect(downloadPath).toBeTruthy()
  const fs = await import('node:fs')
  const stats = fs.statSync(downloadPath as string)
  expect(stats.size).toBeGreaterThan(0)
  expect(download.suggestedFilename()).toBe('test.mid')
})

test('Export Parts > MIDI (ZIP) produces a non-empty downloaded zip for a multi-part score', async ({
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
    name: 'MIDI (ZIP)',
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
  expect(stats.size).toBeGreaterThan(0)
  expect(download.suggestedFilename()).toBe('test (MIDI parts).zip')
})

test('Export > MIDI filename includes only the enabled parts when a part is hidden', async ({
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

  const midiItem = page.getByRole('menuitem', { name: 'MIDI', exact: true })
  await expect(midiItem).toBeEnabled({ timeout: 30_000 })

  const [download] = await Promise.all([
    page.waitForEvent('download'),
    midiItem.click(),
  ])

  expect(download.suggestedFilename()).toBe('test (Melody).mid')
})
