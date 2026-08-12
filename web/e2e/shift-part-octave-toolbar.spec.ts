import { expect, test } from '@playwright/test'

const SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  'Bass [B] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1 2 3 4',
  '[B] 5 6 7 1',
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

test('clicking notation octave up shifts only the target part in the editor text', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await openEditPartsModal(page)

  await page.getByTestId('notation-octave-up-M').click()

  await expect
    .poll(getEditorSource.bind(null, page))
    .toContain("[M] 1' 2' 3' 4'")
  await expect
    .poll(getStoredSource.bind(null, page))
    .toContain("[M] 1' 2' 3' 4'")

  const source = await getEditorSource(page)
  expect(source).toContain('[B] 5 6 7 1')
})

test('notation octave down control is hidden for a follow part', async ({
  page,
}) => {
  const followSource = [
    '# metadata',
    'title = "Test"',
    '',
    '# parts',
    'Melody [M] = notes',
    'Chords [C] = follow[M]',
    '',
    '# score',
    '(bpm=120 key=C4 time=4/4)',
    '[M] 1 2 3 4',
  ].join('\n')

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
  }, followSource)
  await page.goto('/')

  await openEditPartsModal(page)

  await expect(page.getByTestId('notation-octave-up-M')).toBeVisible()
  await expect(page.getByTestId('notation-octave-up-C')).toHaveCount(0)
})
