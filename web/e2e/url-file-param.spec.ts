import { expect, test } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from './fileSwitcherHelpers'

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

const SOURCE_B = SOURCE_A.replace('File A', 'File B')

async function seedTwoFiles(page: import('@playwright/test').Page) {
  await page.addInitScript(
    ({ a, b }) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'a.jianpu',
          userFiles: { 'a.jianpu': a, 'b.jianpu': b },
          bin: {},
          fileIds: {
            'a.jianpu': crypto.randomUUID(),
            'b.jianpu': crypto.randomUUID(),
          },
        }),
      )
    },
    { a: SOURCE_A, b: SOURCE_B },
  )
}

test('switching the active file updates the ?file= URL param', async ({
  page,
}) => {
  await seedTwoFiles(page)
  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await expect(page).toHaveURL(/[?&]file=a\.jianpu(&|$)/)

  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: 'b' }).click()

  await expect(fileSwitcherTrigger(page)).toContainText('b')
  await expect(page).toHaveURL(/[?&]file=b\.jianpu(&|$)/)
})

test('loading with a ?file= URL param selects that file', async ({ page }) => {
  await seedTwoFiles(page)
  // Stored `active` is a.jianpu, but the URL names b.jianpu — the URL should win.
  await page.goto('/?file=b.jianpu')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await expect(fileSwitcherTrigger(page)).toContainText('b')
  await openFileList(page)
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText('b')
})

test('loading with a non-ASCII ?file= URL param selects that file without reverting', async ({
  page,
}) => {
  const cjkName = '今山古道.jianpu'
  await page.addInitScript(
    ({ a, cjk, source }) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'a.jianpu',
          userFiles: { 'a.jianpu': a, [cjk]: source },
          bin: {},
          fileIds: {
            'a.jianpu': crypto.randomUUID(),
            [cjk]: crypto.randomUUID(),
          },
        }),
      )
    },
    { a: SOURCE_A, cjk: cjkName, source: SOURCE_B },
  )

  await page.goto(`/?file=${encodeURIComponent(cjkName)}`)
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  const displayName = cjkName.replace(/\.jianpu$/, '')
  await expect(fileSwitcherTrigger(page)).toContainText(displayName)
  await openFileList(page)
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    displayName,
  )

  // The URL param must keep naming the selected file, not silently revert
  // to whichever file was `active` in storage before the URL was applied.
  const url = new URL(page.url())
  expect(url.searchParams.get('file')).toBe(cjkName)
})
