import { expect, test } from '@playwright/test'

const SOURCE = [
  '# metadata',
  'title = "Test"',
  'subtitle = "Sub"',
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

async function openEditMetadataModal(page: import('@playwright/test').Page) {
  await waitForEditor(page)
  const codeLensLink = page.locator('.codelens-decoration a', {
    hasText: 'Edit Metadata',
  })
  await expect(codeLensLink).toBeVisible({ timeout: 15_000 })
  await codeLensLink.click()
  await page.getByTestId('edit-metadata-modal').waitFor({ state: 'visible' })
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

test('CodeLens Edit Metadata link opens the modal', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  await expect(modal).toContainText('Edit Metadata')

  const titleInput = modal.locator('input[type="text"]').first()
  await expect(titleInput).toHaveValue('Test')
})

test('editing the title field updates the source', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  const titleInput = modal.locator('input[type="text"]').first()
  await titleInput.fill('New Title')

  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })

  const expectedLine = 'title = "New Title"'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('editing a numeric field updates the source', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  const rowHeightInput = modal.locator('input[type="number"]').first()
  await rowHeightInput.fill('30')

  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })

  const expectedLine = 'row height = 30'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('clearing an optional field removes it from the source', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    'subtitle',
  )

  const subtitleInput = modal.locator('input[type="text"]').nth(1)
  await subtitleInput.fill('')

  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })

  await expect.poll(getEditorSource.bind(null, page)).not.toContain('subtitle')
  await expect.poll(getStoredSource.bind(null, page)).not.toContain('subtitle')
})

test('unchecking merge duplicate measures across parts writes = no to the source', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  const mergeCheckbox = modal.locator('input[type="checkbox"]').first()
  await expect(mergeCheckbox).toBeChecked()
  await mergeCheckbox.uncheck()

  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })

  const expectedLine = 'merge duplicate measures across parts = no'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('re-checking merge duplicate measures across parts writes = yes to the source', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  const mergeCheckbox = modal.locator('input[type="checkbox"]').first()
  await mergeCheckbox.uncheck()
  await mergeCheckbox.check()

  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })

  const expectedLine = 'merge duplicate measures across parts = yes'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('unchecking hide resting parts writes = no to the source', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  const hideRestingCheckbox = modal.locator('input[type="checkbox"]').nth(1)
  await expect(hideRestingCheckbox).toBeChecked()
  await hideRestingCheckbox.uncheck()

  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })

  const expectedLine = 'hide resting parts = no'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('re-checking hide resting parts writes = yes to the source', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  const hideRestingCheckbox = modal.locator('input[type="checkbox"]').nth(1)
  await hideRestingCheckbox.uncheck()
  await hideRestingCheckbox.check()

  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })

  const expectedLine = 'hide resting parts = yes'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('checking hide system dividers writes = yes to the source', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  const hideDividersCheckbox = modal.locator('input[type="checkbox"]').last()
  await expect(hideDividersCheckbox).not.toBeChecked()
  await hideDividersCheckbox.check()

  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })

  const expectedLine = 'hide system dividers = yes'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('unchecking hide system dividers writes = no to the source', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  const hideDividersCheckbox = modal.locator('input[type="checkbox"]').last()
  await hideDividersCheckbox.check()
  await hideDividersCheckbox.uncheck()

  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })

  const expectedLine = 'hide system dividers = no'
  await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
  await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
})

test('modal stays within the editor pane and does not cover the preview pane', async ({
  page,
}) => {
  await loadSource(page)
  await page.setViewportSize({ width: 1400, height: 900 })
  await page.goto('/')

  await openEditMetadataModal(page)

  const modal = page.getByTestId('edit-metadata-modal')
  const modalBox = await modal.boundingBox()
  const previewBox = await page.locator('.pane--preview').boundingBox()
  if (!modalBox || !previewBox) {
    throw new Error('expected modal and preview pane to have bounding boxes')
  }

  expect(modalBox.x + modalBox.width).toBeLessThanOrEqual(previewBox.x)
})
