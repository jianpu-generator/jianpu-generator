import { expect, test } from '@playwright/test'

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

test('Edit Parts button opens the modal', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  const editPartsBtn = page.getByTestId('edit-parts-btn')
  await expect(editPartsBtn).toBeVisible({ timeout: 5_000 })

  await editPartsBtn.click()

  const modal = page.getByTestId('edit-parts-modal')
  await expect(modal).toBeVisible({ timeout: 3_000 })
  await expect(modal).toContainText('Edit Parts')
})

test('mode select changes the part mode', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  await page.getByTestId('edit-parts-btn').click()
  await page.getByTestId('edit-parts-modal').waitFor({ state: 'visible' })

  // The "Chords [C]" part starts as "chords". Change it to "notes".
  const modeSelect = page.getByTestId('mode-select-C')
  await expect(modeSelect).toContainText('chords')

  await modeSelect.click()
  await page.getByRole('option', { name: 'notes', exact: true }).click()

  await expect(modeSelect).toContainText('notes')
})

test('soundfont select changes the instrument for a part', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  await page.getByTestId('edit-parts-btn').click()
  await page.getByTestId('edit-parts-modal').waitFor({ state: 'visible' })

  // The "Melody [M]" part has no soundfont by default (shows "default sound").
  const soundfontSelect = page.getByTestId('soundfont-select-M')
  await expect(soundfontSelect).toContainText('default sound')

  await soundfontSelect.click()
  await page.getByRole('option', { name: '40: Violin' }).click()

  await expect(soundfontSelect).toContainText('40: Violin')
})

test('changing soundfont via modal preserves the editor selection', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Navigate to SOURCE line 10 ("1 - - -") and select to end of line.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('10')
  await page.keyboard.press('Enter')
  await page.keyboard.press('Home')
  await page.keyboard.press('Shift+End')
  await page.waitForTimeout(300)

  type Selection = {
    startLineNumber: number
    startColumn: number
    endLineNumber: number
    endColumn: number
  }

  const getSelection = () =>
    page.evaluate(() => {
      const editors = (
        window as unknown as {
          monaco?: {
            editor?: {
              getEditors?: () => { getSelection?: () => Selection | null }[]
            }
          }
        }
      ).monaco?.editor?.getEditors?.()
      return editors?.[0]?.getSelection?.() ?? null
    })

  const selectionBefore = await getSelection()
  expect(selectionBefore?.startLineNumber).toBe(10)
  expect(selectionBefore?.startColumn).toBe(1)
  expect(selectionBefore?.endLineNumber).toBe(10)
  expect(selectionBefore?.endColumn).toBeGreaterThan(1)

  // Open the modal and change the soundfont.
  await page.getByTestId('edit-parts-btn').click()
  await page.getByTestId('edit-parts-modal').waitFor({ state: 'visible' })
  await page.getByTestId('soundfont-select-M').click()
  await page.getByRole('option', { name: '40: Violin' }).click()

  // Close the modal.
  await page.keyboard.press('Escape')
  await page.getByTestId('edit-parts-modal').waitFor({ state: 'hidden' })

  const selectionAfter = await getSelection()
  expect(selectionAfter).toEqual(selectionBefore)
})
