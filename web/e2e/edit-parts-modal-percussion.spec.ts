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
  'Chords [C] = chords',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 - - -',
  '1 1 5 5',
  'twin- kle',
].join('\n')

const PERCUSSION_SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes+lyrics',
  'Drums [D] = percussion',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 - - -',
  '1 1 5 5',
  'twin- kle',
].join('\n')

async function loadSource(
  page: import('@playwright/test').Page,
  source: string = SOURCE,
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

async function getEditorSource(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const editors = (
      window as unknown as {
        monaco?: {
          editor?: {
            getEditors?: () => { getValue?: () => string }[]
          }
        }
      }
    ).monaco?.editor?.getEditors?.()
    return editors?.[0]?.getValue?.() ?? ''
  })
}

async function getStoredSource(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const raw = localStorage.getItem('jianpu:files:v1')
    if (!raw) return ''
    const store = JSON.parse(raw) as {
      active: string
      userFiles: Record<string, string>
    }
    return store.userFiles[store.active] ?? ''
  })
}

test('mode select offers percussion and changing to it updates the source', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await openEditPartsModal(page)

  const modeSelect = page.getByTestId('mode-select-C')
  await expect(modeSelect).toContainText('chords')

  await modeSelect.click()
  await page.getByRole('option', { name: 'percussion', exact: true }).click()

  await expect(modeSelect).toContainText('percussion')

  const expectedLine = 'Chords [C] = percussion'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('soundfont picker shows percussion keys, not GM instruments, for a percussion part', async ({
  page,
}) => {
  await loadSource(page, PERCUSSION_SOURCE)
  await page.goto('/')

  await openEditPartsModal(page)

  await page.getByTestId('soundfont-select-D').click()
  const searchModal = page
    .getByRole('dialog')
    .filter({ hasText: 'Select percussion sound' })
  await expect(searchModal).toBeVisible()

  await expect(
    searchModal.getByRole('button', {
      name: '38: Acoustic Snare',
      exact: true,
    }),
  ).toBeVisible()
  await expect(
    searchModal.getByRole('button', {
      name: '0: Acoustic Grand Piano',
      exact: true,
    }),
  ).toHaveCount(0)
})

test('selecting a percussion key persists to source and updates the button label', async ({
  page,
}) => {
  await loadSource(page, PERCUSSION_SOURCE)
  await page.goto('/')

  await openEditPartsModal(page)

  const soundfontSelect = page.getByTestId('soundfont-select-D')
  await soundfontSelect.click()
  const searchModal = page
    .getByRole('dialog')
    .filter({ hasText: 'Select percussion sound' })
  await expect(searchModal).toBeVisible()

  await searchModal
    .getByRole('button', { name: '38: Acoustic Snare', exact: true })
    .click()

  await expect(soundfontSelect).toContainText('38: Acoustic Snare')

  const expectedLine = 'Drums [D] = percussion "38: Acoustic Snare"'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('percussion preview toggles play/pause state', async ({ page }) => {
  test.setTimeout(60_000)

  await loadSource(page, PERCUSSION_SOURCE)
  await page.goto('/')

  await openEditPartsModal(page)

  await page.getByTestId('soundfont-select-D').click()
  const searchModal = page
    .getByRole('dialog')
    .filter({ hasText: 'Select percussion sound' })
  await expect(searchModal).toBeVisible()

  const snareRow = searchModal
    .getByRole('button', { name: '38: Acoustic Snare', exact: true })
    .locator('xpath=ancestor::div[1]')
  const previewButton = snareRow.getByTitle('Preview instrument')

  // The button is a silent no-op until the soundfont finishes loading, so
  // retry clicking until the title actually flips to "Pause preview".
  await expect(async () => {
    await previewButton.click()
    await expect(snareRow.getByTitle('Pause preview')).toBeVisible({
      timeout: 1_000,
    })
  }).toPass({ timeout: 30_000 })

  await snareRow.getByTitle('Pause preview').click()
  await expect(snareRow.getByTitle('Preview instrument')).toBeVisible({
    timeout: 5_000,
  })
})
