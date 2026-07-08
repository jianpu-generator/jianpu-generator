import { expect, test } from '@playwright/test'

// The soundfont is a real ~30 MB asset; some sandboxed environments fail to
// write Chromium's HTTP disk cache for large responses
// (net::ERR_CACHE_WRITE_FAILURE), which otherwise breaks the fetch entirely.
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
  'Melody [M] = notes+lyrics',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 - - -',
  '1 1 5 5',
  'twin- kle',
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

async function waitForEditor(page: import('@playwright/test').Page) {
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 30_000 })
}

async function openEditPartsModal(page: import('@playwright/test').Page) {
  await waitForEditor(page)
  const codeLensLink = page.locator('.codelens-decoration a', {
    hasText: 'Edit Parts',
  })
  await expect(codeLensLink).toBeVisible({ timeout: 15_000 })
  await codeLensLink.click()
  await page.getByTestId('edit-parts-modal').waitFor({ state: 'visible' })
}

async function openSoundfontSearchModal(page: import('@playwright/test').Page) {
  await openEditPartsModal(page)
  await page.getByTestId('soundfont-select-M').click()
  const searchModal = page
    .getByRole('dialog')
    .filter({ hasText: 'Select soundfont' })
  await expect(searchModal).toBeVisible()
  return searchModal
}

test('fuzzy search narrows results to subsequence matches', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  const searchModal = await openSoundfontSearchModal(page)

  await searchModal.getByPlaceholder('Search...').fill('vln')

  await expect(
    searchModal.getByRole('button', { name: '40: Violin', exact: true }),
  ).toBeVisible()
  await expect(
    searchModal.getByRole('button', {
      name: '0: Acoustic Grand Piano',
      exact: true,
    }),
  ).toHaveCount(0)
})

test('tag filter applies an AND-filter across the instrument list', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  const searchModal = await openSoundfontSearchModal(page)

  const violinRow = searchModal
    .getByRole('button', { name: '40: Violin', exact: true })
    .locator('xpath=ancestor::div[1]')
  const stringsTag = violinRow.getByRole('button', {
    name: '#strings',
    exact: true,
  })

  await stringsTag.click()
  await expect(stringsTag).toHaveCSS('color', 'rgb(29, 78, 216)')

  await expect(
    searchModal.getByRole('button', { name: '40: Violin', exact: true }),
  ).toBeVisible()
  await expect(
    searchModal.getByRole('button', { name: '47: Timpani', exact: true }),
  ).toBeVisible()
  await expect(
    searchModal.getByRole('button', {
      name: '0: Acoustic Grand Piano',
      exact: true,
    }),
  ).toHaveCount(0)

  await stringsTag.click()
  await expect(stringsTag).not.toHaveCSS('color', 'rgb(29, 78, 216)')
  await expect(
    searchModal.getByRole('button', {
      name: '0: Acoustic Grand Piano',
      exact: true,
    }),
  ).toBeVisible()
})

test('instrument preview toggles play/pause state', async ({ page }) => {
  test.setTimeout(60_000)

  await loadSource(page)
  await page.goto('/')

  const searchModal = await openSoundfontSearchModal(page)

  const violinRow = searchModal
    .getByRole('button', { name: '40: Violin', exact: true })
    .locator('xpath=ancestor::div[1]')
  const previewButton = violinRow.getByTitle('Preview instrument')

  // The button is a silent no-op until the soundfont finishes loading, so
  // retry clicking until the title actually flips to "Pause preview".
  await expect(async () => {
    await previewButton.click()
    await expect(violinRow.getByTitle('Pause preview')).toBeVisible({
      timeout: 1_000,
    })
  }).toPass({ timeout: 30_000 })

  await violinRow.getByTitle('Pause preview').click()
  await expect(violinRow.getByTitle('Preview instrument')).toBeVisible({
    timeout: 5_000,
  })
})
