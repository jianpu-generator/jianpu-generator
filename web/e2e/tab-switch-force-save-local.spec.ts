import { expect, test } from '@playwright/test'
import { openFileList, typeAtEditorEnd } from './fileSwitcherHelpers'

const SOURCE_A = [
  '# metadata',
  'title = "File A"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

const SOURCE_B = [
  '# metadata',
  'title = "File B"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '5 6 7 1',
].join('\n')

async function getStoredFile(
  page: import('@playwright/test').Page,
  name: string,
) {
  return page.evaluate((fileName) => {
    const raw = localStorage.getItem('jianpu:files:v1')
    if (!raw) return ''
    const store = JSON.parse(raw) as {
      active: string
      userFiles: Record<string, string>
    }
    return store.userFiles[fileName] ?? ''
  }, name)
}

test('switching tabs on the local backend never loses an edit that has not been observed via the debounce timer', async ({
  page,
}) => {
  await page.addInitScript(
    ({ sourceA, sourceB }) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'a.jianpu',
          userFiles: { 'a.jianpu': sourceA, 'b.jianpu': sourceB },
          bin: {},
          fileIds: {
            'a.jianpu': crypto.randomUUID(),
            'b.jianpu': crypto.randomUUID(),
          },
        }),
      )
    },
    { sourceA: SOURCE_A, sourceB: SOURCE_B },
  )

  // Install the fake clock and never advance it — `localBackend`'s
  // `saveContent` is a no-op, so the switch must not rely on the autosave
  // debounce timer ever firing to have the latest edit in storage.
  await page.clock.install()

  await page.goto('/')

  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await typeAtEditorEnd(page, ' 5')

  await expect
    .poll(getStoredFile.bind(null, page, 'a.jianpu'))
    .toContain('1 2 3 4 5')

  const editedFileA = await getStoredFile(page, 'a.jianpu')

  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: 'b.jianpu' }).click()
  await openFileList(page)
  await expect(
    page.locator('.file-tab-name', { hasText: 'b.jianpu' }),
  ).toHaveAttribute('aria-current', 'true')
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    '5 6 7 1',
  )

  expect(await getStoredFile(page, 'a.jianpu')).toBe(editedFileA)

  await page.locator('.file-tab-name', { hasText: 'a.jianpu' }).click()
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    '1 2 3 4 5',
  )
})
