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

function exportMenuButton(page: import('@playwright/test').Page) {
  return page.getByRole('button', { name: 'Export', exact: true })
}

function exportPartsMenuButton(page: import('@playwright/test').Page) {
  return page.getByRole('button', { name: 'Export Parts', exact: true })
}

test('Export Parts button only appears when the score has more than one part', async ({
  page,
}) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await expect(exportMenuButton(page)).toBeVisible({ timeout: 30_000 })
  await expect(exportPartsMenuButton(page)).toHaveCount(0)
})

test('Export menu lists PDF, WAV, and MIDI', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const menu = page.getByRole('menu')
  await expect(menu).toBeVisible()
  const itemLabels = await menu.getByRole('menuitem').allTextContents()
  expect(itemLabels).toEqual(['PDF', 'WAV', 'MIDI'])
})

test('Export Parts menu lists PDF (ZIP), WAV (ZIP), and MIDI (ZIP) for a multi-part score', async ({
  page,
}) => {
  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportPartsMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()

  const menu = page.getByRole('menu')
  await expect(menu).toBeVisible()
  const itemLabels = await menu.getByRole('menuitem').allTextContents()
  expect(itemLabels).toEqual(['PDF (ZIP)', 'WAV (ZIP)', 'MIDI (ZIP)'])
})

test('pressing Escape closes an open export menu', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()
  await expect(page.getByRole('menu')).toBeVisible()

  await page.keyboard.press('Escape')
  await expect(page.getByRole('menu')).toHaveCount(0)
  await expect(menuButton).toHaveAttribute('aria-expanded', 'false')
})

test('clicking outside an open export menu closes it', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()
  await expect(page.getByRole('menu')).toBeVisible()

  await page.locator('.preview-pages').click({ position: { x: 5, y: 5 } })
  await expect(page.getByRole('menu')).toHaveCount(0)
  await expect(menuButton).toHaveAttribute('aria-expanded', 'false')
})

test('opening the Export Parts menu closes an already-open Export menu, since the click lands outside it', async ({
  page,
}) => {
  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const menuButton = exportMenuButton(page)
  const partsMenuButton = exportPartsMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await expect(partsMenuButton).toBeEnabled({ timeout: 30_000 })

  await menuButton.click()
  await expect(page.getByRole('menu')).toHaveCount(1)

  await partsMenuButton.click()
  await expect(page.getByRole('menu')).toHaveCount(1)
  await expect(menuButton).toHaveAttribute('aria-expanded', 'false')
  await expect(partsMenuButton).toHaveAttribute('aria-expanded', 'true')
})
